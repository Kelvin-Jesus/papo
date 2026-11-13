//! `papo update`: finds the latest release and upgrades papo the way it was installed.
//!
//! The latest tag comes from the redirect of `/releases/latest`, like `scripts/install.sh`
//! (no API call, so no rate limit). How to upgrade follows from where the executable
//! lives: a package manager's directory hands the job to that manager, anything else is
//! an `install.sh` install and gets the new release's script run into the same directory.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

pub const REPO: &str = "Kelvin-Jesus/papo";

/// How this copy of papo was installed, read off the executable's path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Channel {
    Homebrew,
    Scoop,
    /// `cargo install`, from crates.io or a checkout.
    Cargo,
    /// `/usr/bin` and friends: a `.deb`, `.rpm` or the AUR package, owned by the system.
    System,
    /// `scripts/install.sh` (or a copy by hand) into this directory.
    Script {
        dir: PathBuf,
    },
    /// The Windows `.zip` unpacked by hand into this directory.
    WindowsZip {
        dir: PathBuf,
    },
}

/// What `papo update` does for a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Runs this command, printing `label` first.
    Run { program: String, args: Vec<String>, label: String },
    /// Nothing papo should do by itself (it would need sudo or a manual download): tells how.
    Manual(String),
}

pub fn channel_for(exe: &Path, windows: bool) -> Channel {
    // Read as text so a Windows path parses the same on any host (and in tests).
    let text = exe.to_string_lossy();
    let sep = if windows { '\\' } else { '/' };
    let parts: Vec<String> = text.split(['/', '\\']).map(str::to_ascii_lowercase).collect();
    let dir = text.rsplit_once(sep).map(|(d, _)| d).unwrap_or("").to_string();
    let has = |name: &str| parts.iter().any(|p| p == name);
    if parts.windows(2).any(|w| w[0] == ".cargo" && w[1] == "bin") {
        return Channel::Cargo;
    }
    if windows {
        return if has("scoop") { Channel::Scoop } else { Channel::WindowsZip { dir: dir.into() } };
    }
    if has("cellar") || has(".linuxbrew") || has("homebrew") {
        return Channel::Homebrew;
    }
    if matches!(dir.as_str(), "/usr/bin" | "/bin" | "/usr/sbin" | "/sbin") {
        return Channel::System;
    }
    Channel::Script { dir: dir.into() }
}

/// The newest release's tag. `PAPO_TEST_LATEST_TAG` replaces the lookup in tests.
pub fn latest_tag() -> Result<String> {
    let url = match std::env::var("PAPO_TEST_LATEST_TAG") {
        Ok(tag) => format!("https://github.com/{REPO}/releases/tag/{tag}"),
        Err(_) => {
            // Windows 10+ ships curl.exe too. Only the headers: the redirect is the answer.
            let out = Command::new("curl")
                .args(["-fsSLI", "--max-time", "20", "-o", if cfg!(windows) { "NUL" } else { "/dev/null" }])
                .args(["-w", "%{url_effective}", &format!("https://github.com/{REPO}/releases/latest")])
                .output()
                .context("preciso do curl para procurar a versão nova")?;
            if !out.status.success() {
                bail!("não consegui falar com o GitHub ({}). Veja https://github.com/{REPO}/releases", out.status);
            }
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
    };
    tag_from_url(&url)
        .with_context(|| format!("não consegui descobrir a última versão. Veja https://github.com/{REPO}/releases"))
}

/// `https://github.com/o/r/releases/tag/v0.3.1` → `v0.3.1`.
pub fn tag_from_url(url: &str) -> Option<String> {
    let (rest, tag) = url.trim().rsplit_once('/')?;
    (rest.ends_with("/releases/tag") && parse_version(tag).is_some()).then(|| tag.to_string())
}

/// `v0.3.1` or `0.3.1` → `(0, 3, 1)`. Pre-release suffixes are not published, so refused.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.strip_prefix('v').unwrap_or(s).split('.').map(|p| p.parse::<u64>().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(version)
}

/// Whether `latest` (a tag) is newer than `current` (the crate version).
pub fn is_newer(current: &str, latest: &str) -> bool {
    matches!((parse_version(current), parse_version(latest)), (Some(c), Some(l)) if l > c)
}

pub fn upgrade_step(channel: &Channel, tag: &str) -> Step {
    let run = |program: &str, args: &[&str], label: &str| Step::Run {
        program: program.into(),
        args: args.iter().map(|a| a.to_string()).collect(),
        label: label.into(),
    };
    match channel {
        Channel::Homebrew => run("brew", &["upgrade", "papo"], "brew upgrade papo"),
        // scoop is a .cmd/.ps1 shim, which `Command` does not resolve on its own.
        Channel::Scoop => run("cmd", &["/C", "scoop", "update", "papo"], "scoop update papo"),
        Channel::Cargo => {
            run("cargo", &["install", "papo", "--locked", "--force"], "cargo install papo --locked --force")
        }
        // The script installs to a temporary name and renames, so a running `papo mcp`
        // keeps its old inode. Values go in as $1..$3, never spliced into the script.
        Channel::Script { dir } => Step::Run {
            program: "sh".into(),
            args: vec![
                "-c".into(),
                // Downloaded first: in `curl | sh` a failed download still exits 0.
                r#"script="$(curl -fsSL "$1")" || exit 1; printf '%s\n' "$script" | sh -s -- --dir "$2" --version "$3""#
                    .into(),
                "sh".into(),
                format!("https://raw.githubusercontent.com/{REPO}/{tag}/scripts/install.sh"),
                dir.to_string_lossy().into_owned(),
                tag.into(),
            ],
            label: format!("o instalador da {tag} em {}", dir.display()),
        },
        Channel::System => Step::Manual(format!(
            "Este papo foi instalado pelo sistema. Atualize pelo mesmo caminho:\n\n  \
             AUR:       yay -Syu papo-bin (ou paru -Syu papo-bin)\n  \
             .deb/.rpm: baixe o pacote da {tag} em https://github.com/{REPO}/releases/tag/{tag}\n             \
             e instale com sudo apt install ./<arquivo>.deb ou sudo dnf install ./<arquivo>.rpm"
        )),
        Channel::WindowsZip { dir } => Step::Manual(format!(
            "Baixe https://github.com/{REPO}/releases/download/{tag}/papo-{tag}-x86_64-pc-windows-msvc.zip\n\
             e troque o papo.exe em {}. Feche antes o Claude Code: o Windows não deixa\n\
             substituir um executável em uso. Com o Scoop (scoop install papo) isto vira papo update.",
            dir.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tag_is_the_last_segment_of_the_latest_redirect() {
        assert_eq!(tag_from_url("https://github.com/Kelvin-Jesus/papo/releases/tag/v0.3.1").as_deref(), Some("v0.3.1"));
        assert_eq!(
            tag_from_url("https://github.com/Kelvin-Jesus/papo/releases/tag/v0.3.1\r\n").as_deref(),
            Some("v0.3.1")
        );
        // No release yet: GitHub leaves you on /releases.
        assert_eq!(tag_from_url("https://github.com/Kelvin-Jesus/papo/releases"), None);
        assert_eq!(tag_from_url(""), None);
    }

    #[test]
    fn versions_compare_numerically_not_as_text() {
        assert_eq!(parse_version("v0.10.2"), Some((0, 10, 2)));
        assert_eq!(parse_version("0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("v0.3"), None);
        assert_eq!(parse_version("v0.3.1-rc1"), None);
        assert_eq!(parse_version("latest"), None);
        assert!(is_newer("0.3.0", "v0.3.1"));
        assert!(is_newer("0.9.0", "v0.10.0"));
        assert!(!is_newer("0.3.1", "v0.3.1"));
        assert!(!is_newer("0.4.0", "v0.3.9"));
        // A tag that does not parse never triggers an upgrade.
        assert!(!is_newer("0.3.0", "v1"));
    }

    #[test]
    fn the_channel_comes_from_where_the_executable_lives() {
        let unix = |p: &str| channel_for(Path::new(p), false);
        assert_eq!(unix("/opt/homebrew/Cellar/papo/0.3.0/bin/papo"), Channel::Homebrew);
        assert_eq!(unix("/home/linuxbrew/.linuxbrew/Cellar/papo/0.3.0/bin/papo"), Channel::Homebrew);
        assert_eq!(unix("/usr/local/Cellar/papo/0.3.0/bin/papo"), Channel::Homebrew);
        assert_eq!(unix("/home/voce/.cargo/bin/papo"), Channel::Cargo);
        assert_eq!(unix("/usr/bin/papo"), Channel::System);
        assert_eq!(unix("/bin/papo"), Channel::System);
        assert_eq!(unix("/home/voce/.local/bin/papo"), Channel::Script { dir: "/home/voce/.local/bin".into() });
        assert_eq!(unix("/usr/local/bin/papo"), Channel::Script { dir: "/usr/local/bin".into() });

        let win = |p: &str| channel_for(Path::new(p), true);
        assert_eq!(win(r"C:\Users\voce\scoop\apps\papo\current\papo.exe"), Channel::Scoop);
        assert_eq!(win(r"C:\Users\voce\.cargo\bin\papo.exe"), Channel::Cargo);
        assert_eq!(win(r"C:\Tools\papo\papo.exe"), Channel::WindowsZip { dir: r"C:\Tools\papo".into() });
    }

    #[test]
    fn package_managers_upgrade_themselves() {
        let run = |c: Channel| match upgrade_step(&c, "v0.3.1") {
            Step::Run { program, args, .. } => format!("{program} {}", args.join(" ")),
            Step::Manual(m) => panic!("expected a command, got: {m}"),
        };
        assert_eq!(run(Channel::Homebrew), "brew upgrade papo");
        assert!(run(Channel::Scoop).ends_with("scoop update papo"));
        assert_eq!(run(Channel::Cargo), "cargo install papo --locked --force");
    }

    #[test]
    fn a_script_install_runs_the_new_release_installer_into_the_same_directory() {
        let Step::Run { program, args, .. } =
            upgrade_step(&Channel::Script { dir: "/home/voce/my bin".into() }, "v0.3.1")
        else {
            panic!("expected a command")
        };
        assert_eq!(program, "sh");
        // The directory travels as an argument, never inside the script text.
        assert!(!args[1].contains("my bin"), "{}", args[1]);
        assert_eq!(
            &args[3..],
            [
                "https://raw.githubusercontent.com/Kelvin-Jesus/papo/v0.3.1/scripts/install.sh",
                "/home/voce/my bin",
                "v0.3.1"
            ]
        );
    }

    /// Runs the script install's shell snippet with `url` in place of the release's script.
    #[cfg(unix)]
    fn run_script_step(url: &str, dir: &Path) -> std::process::ExitStatus {
        let Step::Run { program, mut args, .. } = upgrade_step(&Channel::Script { dir: dir.into() }, "v0.3.1") else {
            panic!("expected a command")
        };
        args[3] = url.into();
        Command::new(program).args(args).status().unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_download_fails_the_update_instead_of_running_nothing() {
        // The snippet runs curl; the test-stage container has none.
        if Command::new("curl").arg("--version").output().is_err() {
            eprintln!("skipped: no curl");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        assert!(!run_script_step("file:///nonexistent/install.sh", dir.path()).success());

        // A script that downloads fine runs with the directory and the tag.
        let script = dir.path().join("install.sh");
        std::fs::write(&script, "printf '%s %s %s %s' \"$1\" \"$2\" \"$3\" \"$4\" > \"$2/args\"\n").unwrap();
        assert!(run_script_step(&format!("file://{}", script.display()), dir.path()).success());
        let args = std::fs::read_to_string(dir.path().join("args")).unwrap();
        assert_eq!(args, format!("--dir {} --version v0.3.1", dir.path().display()));
    }

    #[test]
    fn system_packages_and_the_windows_zip_are_explained_not_touched() {
        let Step::Manual(text) = upgrade_step(&Channel::System, "v0.3.1") else { panic!("expected manual") };
        assert!(text.contains("papo-bin") && text.contains("v0.3.1"), "{text}");
        let Step::Manual(text) = upgrade_step(&Channel::WindowsZip { dir: r"C:\Tools\papo".into() }, "v0.3.1") else {
            panic!("expected manual")
        };
        assert!(text.contains("papo-v0.3.1-x86_64-pc-windows-msvc.zip") && text.contains(r"C:\Tools\papo"), "{text}");
    }
}
