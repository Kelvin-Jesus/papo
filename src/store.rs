//! On-disk state for a profile.
//!
//! Layout under `$PAPO_HOME` (default `~/.papo`):
//!
//! ```text
//! profiles/<profile>/
//!   profile.json   name, about, room secret            (secret: 0600)
//!   secret.key     iroh endpoint identity               (secret: 0600)
//!   peers.json     endpoint ids of room members we have met, used to rejoin
//!   inbox.json     received messages the agent has not consumed yet
//!   outbox.json    sent messages still waiting for an ack
//!   log.jsonl      append-only conversation log (what `papo log` shows)
//!   lock           held by the running MCP server for this profile
//! ```
//!
//! A stable identity matters: peers remember our endpoint id and redial it when either
//! side restarts, so the room survives without a server.

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use iroh::{EndpointId, SecretKey};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    proto::{Envelope, validate_name},
    room::RoomSecret,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Base32 room secret. Stored here (not in the Claude MCP config) so it never ends up
    /// in a committed `.mcp.json`.
    pub room: String,
    pub created_ms: u64,
}

impl Profile {
    pub fn room_secret(&self) -> Result<RoomSecret> {
        RoomSecret::from_base32(&self.room).context("profile.json has a corrupt room secret")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnownPeer {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub last_seen_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "ev", rename_all = "snake_case")]
pub enum LogEntry {
    In { msg: Envelope },
    Out { msg: Envelope },
    Delivered { id: String, by: String, ts: u64 },
}

#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

pub fn home() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("PAPO_HOME") {
        return Ok(PathBuf::from(dir));
    }
    let home = dirs::home_dir().context("could not determine the home directory; set PAPO_HOME")?;
    Ok(home.join(".papo"))
}

pub fn validate_profile_name(profile: &str) -> Result<()> {
    ensure!(
        !profile.is_empty()
            && profile.len() <= 32
            && profile.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "profile must be 1-32 chars of [a-zA-Z0-9_-]"
    );
    Ok(())
}

impl Store {
    pub fn dir_for(profile: &str) -> Result<PathBuf> {
        validate_profile_name(profile)?;
        Ok(home()?.join("profiles").join(profile))
    }

    pub fn exists(profile: &str) -> Result<bool> {
        Ok(Self::dir_for(profile)?.join("profile.json").exists())
    }

    /// Opens an already configured profile.
    pub fn open(profile: &str) -> Result<Self> {
        let dir = Self::dir_for(profile)?;
        if !dir.join("profile.json").exists() {
            bail!(
                "profile '{profile}' is not configured. Run `papo new --name <you>` to create a room \
                 or `papo join <invite> --name <you>` to enter one."
            );
        }
        Ok(Self { dir })
    }

    /// Opens a profile stored in an explicit directory (see [`Store::create_at`]).
    pub fn open_at(dir: PathBuf) -> Result<Self> {
        ensure!(dir.join("profile.json").exists(), "no profile in {}", dir.display());
        Ok(Self { dir })
    }

    /// Creates (or overwrites, when `force`) a profile. The endpoint identity is kept
    /// across overwrites so peers that already know us keep recognizing us.
    pub fn create(profile: &str, data: &Profile, force: bool) -> Result<Self> {
        let dir = Self::dir_for(profile)?;
        if dir.join("profile.json").exists() && !force {
            bail!("profile '{profile}' already exists (use --force to replace it, or --profile to pick another name)");
        }
        Self::create_at(dir, data)
    }

    /// Creates a profile in an explicit directory, bypassing `$PAPO_HOME`. Tests use
    /// this because the environment is shared by every test thread.
    pub fn create_at(dir: PathBuf, data: &Profile) -> Result<Self> {
        validate_name(&data.name)?;
        create_private_dir(&dir)?;
        let store = Self { dir };
        write_private(&store.path("profile.json"), &serde_json::to_vec_pretty(data)?)?;
        // A different room means different peers and a different conversation.
        for stale in ["peers.json", "inbox.json", "outbox.json"] {
            let _ = fs::remove_file(store.path(stale));
        }
        Ok(store)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    pub fn profile(&self) -> Result<Profile> {
        read_json(&self.path("profile.json"))?.context("profile.json disappeared")
    }

    pub fn save_profile(&self, profile: &Profile) -> Result<()> {
        write_private(&self.path("profile.json"), &serde_json::to_vec_pretty(profile)?)
    }

    /// Loads the endpoint identity, generating it on first use.
    pub fn secret_key(&self) -> Result<SecretKey> {
        let path = self.path("secret.key");
        match fs::read_to_string(&path) {
            Ok(hex) => {
                let bytes = data_encoding::HEXLOWER.decode(hex.trim().as_bytes()).context("secret.key is corrupt")?;
                let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("secret.key has the wrong length"))?;
                Ok(SecretKey::from_bytes(&arr))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let key = SecretKey::generate();
                write_private(&path, data_encoding::HEXLOWER.encode(&key.to_bytes()).as_bytes())?;
                Ok(key)
            }
            Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
        }
    }

    pub fn known_peers(&self) -> Result<BTreeMap<EndpointId, KnownPeer>> {
        let raw: BTreeMap<String, KnownPeer> = read_json(&self.path("peers.json"))?.unwrap_or_default();
        // Skip entries we cannot parse instead of failing: a bad entry should not lock
        // the user out of the room.
        Ok(raw.into_iter().filter_map(|(id, peer)| id.parse().ok().map(|id| (id, peer))).collect())
    }

    pub fn save_known_peers(&self, peers: &BTreeMap<EndpointId, KnownPeer>) -> Result<()> {
        let raw: BTreeMap<String, &KnownPeer> = peers.iter().map(|(id, p)| (id.to_string(), p)).collect();
        write_atomic(&self.path("peers.json"), &serde_json::to_vec_pretty(&raw)?)
    }

    pub fn inbox(&self) -> Result<Vec<Envelope>> {
        Ok(read_json(&self.path("inbox.json"))?.unwrap_or_default())
    }

    pub fn save_inbox(&self, inbox: &[Envelope]) -> Result<()> {
        write_atomic(&self.path("inbox.json"), &serde_json::to_vec_pretty(inbox)?)
    }

    pub fn outbox(&self) -> Result<Vec<Envelope>> {
        Ok(read_json(&self.path("outbox.json"))?.unwrap_or_default())
    }

    pub fn save_outbox(&self, outbox: &[Envelope]) -> Result<()> {
        write_atomic(&self.path("outbox.json"), &serde_json::to_vec_pretty(outbox)?)
    }

    pub fn log_path(&self) -> PathBuf {
        self.path("log.jsonl")
    }

    pub fn append_log(&self, entry: &LogEntry) -> Result<()> {
        let mut line = serde_json::to_vec(entry)?;
        line.push(b'\n');
        // Single write of a full line with O_APPEND, so concurrent writers (MCP server and
        // `papo say`) never interleave partial lines.
        let mut file = OpenOptions::new().create(true).append(true).open(self.log_path())?;
        file.write_all(&line)?;
        Ok(())
    }

    pub fn read_log(&self) -> Result<Vec<LogEntry>> {
        let file = match File::open(self.log_path()) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e).context("open log.jsonl"),
        };
        // Tolerate a torn last line (crash mid-write) rather than hiding the whole history.
        Ok(BufReader::new(file).lines().map_while(|l| l.ok()).filter_map(|l| serde_json::from_str(&l).ok()).collect())
    }

    /// Exclusive lock for the lifetime of the returned file. Two processes sharing one
    /// endpoint identity would fight over the same id on the network and corrupt the
    /// inbox, so only one MCP server per profile may run.
    pub fn lock(&self) -> Result<File> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.path("lock"))
            .context("open lock file")?;
        match file.try_lock() {
            Ok(()) => Ok(file),
            Err(fs::TryLockError::WouldBlock) => bail!(
                "another papo server is already running with this profile (another Claude Code session?). \
                 Close it or use a different --profile."
            ),
            Err(fs::TryLockError::Error(e)) => Err(e).context("lock profile"),
        }
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).with_context(|| format!("parse {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

/// Write-then-rename so a crash never leaves a half-written inbox/outbox behind.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic(path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn create_private_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
