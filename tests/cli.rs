//! The human CLI: setup commands, their error messages and what they leave on disk.

mod common;

use std::{path::Path, time::Duration};

use assert_cmd::Command;
use common::{BIN, endpoint_id, invite_in, profile_dir};
use papo::{
    proto::{Envelope, PeerKind},
    room::Invite,
    store::{LogEntry, Store},
};
use predicates::prelude::*;

fn papo(home: &Path) -> Command {
    let mut cmd = Command::new(BIN);
    cmd.env("PAPO_HOME", home).env_remove("PAPO_PROFILE").timeout(Duration::from_secs(60));
    cmd
}

fn stdout(cmd: &mut Command) -> String {
    String::from_utf8(cmd.assert().success().get_output().stdout.clone()).unwrap()
}

fn new_room(home: &Path, name: &str) -> String {
    invite_in(&stdout(papo(home).args(["new", "--name", name])))
}

#[test]
fn help_lists_every_command() {
    let help = stdout(papo(Path::new("/nonexistent")).arg("--help"));
    insta::assert_snapshot!("help", help);
}

#[test]
fn version_matches_the_crate() {
    papo(Path::new("/nonexistent"))
        .arg("--version")
        .assert()
        .success()
        .stdout(format!("papo {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn new_creates_a_room_and_prints_a_working_invite() {
    let home = tempfile::tempdir().unwrap();
    let out = stdout(papo(home.path()).args(["new", "--name", "kj", "--about", "api"]));
    insta::with_settings!({filters => vec![
        (r"papo1[a-z0-9]+", "[invite]"),
        (r"Sala [0-9a-f]{8}", "Sala [room-id]"),
    ]}, {
        insta::assert_snapshot!("new_output", out);
    });

    let invite = Invite::decode(&invite_in(&out)).unwrap();
    assert_eq!(invite.peers, vec![endpoint_id(home.path(), "default")], "the creator is the entry point");
    let store = Store::open_at(profile_dir(home.path(), "default")).unwrap();
    let profile = store.profile().unwrap();
    assert_eq!((profile.name.as_str(), profile.about.as_deref()), ("kj", Some("api")));
    assert_eq!(profile.room_secret().unwrap(), invite.secret);
}

#[test]
fn new_refuses_to_overwrite_unless_forced_and_keeps_the_identity() {
    let home = tempfile::tempdir().unwrap();
    let first = new_room(home.path(), "kj");
    let id = endpoint_id(home.path(), "default");
    papo(home.path())
        .args(["new", "--name", "kj"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists").and(predicate::str::contains("--force")));

    let second = invite_in(&stdout(papo(home.path()).args(["new", "--name", "kj", "--force"])));
    assert_ne!(Invite::decode(&first).unwrap().secret, Invite::decode(&second).unwrap().secret, "a new room");
    assert_eq!(endpoint_id(home.path(), "default"), id, "but the same identity");
}

#[test]
fn names_and_profiles_are_validated() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path())
        .args(["new", "--name", "com espaço"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("erro: name may only contain"));
    papo(home.path())
        .args(["new", "--name", "kj", "--profile", "../fora"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("profile must be"));
    assert!(!home.path().join("profiles").exists(), "nothing written on invalid input");
}

#[test]
fn join_enters_the_same_room_and_remembers_the_inviter() {
    let voce = tempfile::tempdir().unwrap();
    let colega = tempfile::tempdir().unwrap();
    let invite = new_room(voce.path(), "voce");
    papo(colega.path())
        .args(["join", &invite, "--name", "colega"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Você entrou na sala").and(predicate::str::contains("\"colega\"")));

    let colega_store = Store::open_at(profile_dir(colega.path(), "default")).unwrap();
    let voce_store = Store::open_at(profile_dir(voce.path(), "default")).unwrap();
    assert_eq!(
        colega_store.profile().unwrap().room_secret().unwrap(),
        voce_store.profile().unwrap().room_secret().unwrap()
    );
    let known = colega_store.known_peers().unwrap();
    assert_eq!(known.keys().copied().collect::<Vec<_>>(), vec![endpoint_id(voce.path(), "default")]);
}

#[test]
fn joining_with_your_own_invite_does_not_list_yourself() {
    let home = tempfile::tempdir().unwrap();
    let invite = new_room(home.path(), "kj");
    papo(home.path()).args(["join", &invite, "--name", "kj", "--force"]).assert().success();
    let store = Store::open_at(profile_dir(home.path(), "default")).unwrap();
    assert!(store.known_peers().unwrap().is_empty());
}

#[test]
fn bad_invites_are_explained() {
    let home = tempfile::tempdir().unwrap();
    let good = new_room(tempfile::tempdir().unwrap().path(), "voce");
    let mut mistyped: Vec<char> = good.chars().collect();
    mistyped[20] = if mistyped[20] == 'a' { 'b' } else { 'a' };
    let cases = [
        ("hello".to_string(), "invite must start with 'papo1'"),
        (good[..good.len() - 9].to_string(), "truncated"),
        (mistyped.into_iter().collect(), "damaged"),
    ];
    for (invite, why) in cases {
        papo(home.path())
            .args(["join", &invite, "--name", "colega"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(why));
    }
}

#[test]
fn invite_puts_me_first_and_includes_known_members() {
    let voce = tempfile::tempdir().unwrap();
    let colega = tempfile::tempdir().unwrap();
    let invite = new_room(voce.path(), "voce");
    papo(colega.path()).args(["join", &invite, "--name", "colega"]).assert().success();

    let from_bob = Invite::decode(stdout(papo(colega.path()).arg("invite")).trim()).unwrap();
    assert_eq!(from_bob.peers, vec![endpoint_id(colega.path(), "default"), endpoint_id(voce.path(), "default")]);
    assert_eq!(from_bob.secret, Invite::decode(&invite).unwrap().secret);
}

#[test]
fn install_print_shows_the_mcp_config() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "kj");
    let json: serde_json::Value =
        serde_json::from_str(&stdout(papo(home.path()).args(["install", "--print"]))).unwrap();
    let server = &json["mcpServers"]["papo"];
    assert_eq!(server["args"], serde_json::json!(["mcp"]));
    assert!(server["command"].as_str().unwrap().ends_with(&format!("papo{}", std::env::consts::EXE_SUFFIX)));

    papo(home.path()).args(["new", "--name", "kj", "--profile", "time-b"]).assert().success();
    let json: serde_json::Value =
        serde_json::from_str(&stdout(papo(home.path()).args(["install", "--print", "--profile", "time-b"]))).unwrap();
    assert_eq!(json["mcpServers"]["papo"]["args"], serde_json::json!(["mcp", "--profile", "time-b"]));
    // The room secret never ends up in Claude's config.
    let secret = Store::open_at(profile_dir(home.path(), "time-b")).unwrap().profile().unwrap().room;
    assert!(!json.to_string().contains(&secret));
}

#[test]
fn install_command_can_be_overridden_for_containers() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "kj");
    let out = stdout(
        papo(home.path()).env("PAPO_INSTALL_COMMAND", "docker exec -i papo-kj papo").args(["install", "--print"]),
    );
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["mcpServers"]["papo"]["command"], "docker");
    assert_eq!(json["mcpServers"]["papo"]["args"], serde_json::json!(["exec", "-i", "papo-kj", "papo", "mcp"]));
}

#[test]
fn commands_on_an_unconfigured_profile_explain_the_setup() {
    let home = tempfile::tempdir().unwrap();
    for args in [&["install", "--print"][..], &["invite"], &["log"], &["say", "oi"], &["status"]] {
        papo(home.path())
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains("is not configured").and(predicate::str::contains("papo new")));
    }
}

#[test]
fn say_and_status_fail_fast_when_nobody_is_known() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "kj");
    for args in [&["say", "oi"][..], &["status"]] {
        papo(home.path())
            .args(args)
            .timeout(Duration::from_secs(10))
            .assert()
            .failure()
            .stderr(predicate::str::contains("ainda não conheço ninguém"));
    }
}

#[test]
fn log_prints_the_conversation_and_honors_n() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "voce");
    papo(home.path()).arg("log").assert().success().stdout("");

    let store = Store::open_at(profile_dir(home.path(), "default")).unwrap();
    let msg = |id: &str, from: &str, body: &str| Envelope {
        id: id.into(),
        from: from.into(),
        node: "n".into(),
        kind: PeerKind::Agent,
        to: None,
        reply_to: None,
        ts: 1_700_000_000_000,
        body: body.into(),
    };
    store.append_log(&LogEntry::Out { msg: msg("q1", "voce", "qual porta?") }).unwrap();
    store.append_log(&LogEntry::In { msg: msg("r1", "colega", "8443") }).unwrap();

    let all = stdout(papo(home.path()).arg("log"));
    let lines: Vec<&str> = all.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("voce -> room (msg q1): qual porta?"), "{all}");
    assert!(lines[1].ends_with("colega (agent) -> voce (msg r1): 8443"), "{all}");
    papo(home.path()).args(["log", "-n", "1"]).assert().success().stdout(predicate::str::ends_with("8443\n"));
}

#[test]
fn log_follow_streams_new_entries_as_they_are_written() {
    use std::io::{BufRead, BufReader};
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "kj");
    let store = Store::open_at(profile_dir(home.path(), "default")).unwrap();
    store.append_log(&LogEntry::Delivered { id: "old".into(), by: "voce".into(), ts: 1 }).unwrap();

    let mut child = std::process::Command::new(BIN)
        .args(["log", "-f", "-n", "0"])
        .env("PAPO_HOME", home.path())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
    std::thread::sleep(Duration::from_millis(700));
    store.append_log(&LogEntry::Delivered { id: "new1".into(), by: "voce".into(), ts: 2 }).unwrap();
    store.append_log(&LogEntry::Delivered { id: "new2".into(), by: "colega".into(), ts: 3 }).unwrap();
    let first = rx.recv_timeout(Duration::from_secs(10)).expect("follow printed nothing");
    let second = rx.recv_timeout(Duration::from_secs(10)).expect("follow stopped after one line");
    let _ = child.kill();
    let _ = child.wait();
    assert!(first.ends_with("delivered new1 to voce"), "{first}");
    assert!(second.ends_with("delivered new2 to colega"), "{second}");
    assert!(rx.try_recv().is_err(), "-n 0 must not replay old entries");
}

#[test]
fn the_profile_can_come_from_the_environment() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path()).env("PAPO_PROFILE", "trabalho").args(["new", "--name", "kj"]).assert().success();
    assert!(profile_dir(home.path(), "trabalho").join("profile.json").exists());
    papo(home.path()).env("PAPO_PROFILE", "trabalho").arg("invite").assert().success();
}

#[cfg(unix)]
mod install_with_claude {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    /// A stand-in for the `claude` CLI that records its arguments.
    fn fake_claude(dir: &Path, exit: i32) -> std::path::PathBuf {
        let log = dir.join("args.txt");
        let script = dir.join("claude");
        std::fs::write(&script, format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit {exit}\n", log.display()))
            .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        log
    }

    #[test]
    fn registers_papo_with_claude_code() {
        let home = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        new_room(home.path(), "kj");
        let log = fake_claude(bin.path(), 0);
        papo(home.path())
            .env("PATH", bin.path())
            .args(["install", "--scope", "user"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--dangerously-load-development-channels server:papo"));
        let args: Vec<String> = std::fs::read_to_string(log).unwrap().lines().map(String::from).collect();
        assert_eq!(&args[..6], ["mcp", "add", "--scope", "user", "papo", "--"]);
        assert!(args[6].ends_with("/papo"));
        assert_eq!(args[7..], ["mcp"]);
    }

    #[test]
    fn explains_how_to_recover_when_claude_refuses() {
        let home = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        new_room(home.path(), "kj");
        fake_claude(bin.path(), 1);
        papo(home.path())
            .env("PATH", bin.path())
            .arg("install")
            .assert()
            .failure()
            .stderr(predicate::str::contains("claude mcp remove papo --scope local"));
    }

    #[test]
    fn prints_the_manual_command_without_claude_on_path() {
        let home = tempfile::tempdir().unwrap();
        let empty = tempfile::tempdir().unwrap();
        new_room(home.path(), "kj");
        papo(home.path()).env("PATH", empty.path()).arg("install").assert().success().stdout(
            predicate::str::contains("Não achei o comando `claude`")
                .and(predicate::str::contains("claude mcp add --scope local papo --")),
        );
    }
}

#[test]
fn leave_with_nobody_to_tell_deletes_the_profile() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "voce");
    papo(home.path()).args(["leave", "--yes"]).assert().success().stdout(predicate::str::contains("apagado"));
    assert!(!profile_dir(home.path(), "default").exists());
    papo(home.path()).arg("invite").assert().failure();
}

#[test]
fn a_closed_profile_is_removed_by_close_too() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "voce");
    Store::open_at(profile_dir(home.path(), "default")).unwrap().mark_closed().unwrap();
    papo(home.path()).arg("mcp").write_stdin("").assert().success(); // a closed profile still starts: it serves the tombstone
    papo(home.path()).args(["close", "--yes"]).assert().success();
    assert!(!profile_dir(home.path(), "default").exists());
}

#[test]
fn the_owner_leaving_with_nobody_online_keeps_only_a_tombstone() {
    let home = tempfile::tempdir().unwrap();
    new_room(home.path(), "voce");
    let dir = profile_dir(home.path(), "default");
    let store = Store::open_at(dir.clone()).unwrap();
    let colega = iroh::SecretKey::generate().public();
    store.save_known_peers(&[(colega, Default::default())].into()).unwrap();

    papo(home.path()).args(["leave", "--yes"]).assert().success().stdout(predicate::str::contains("lápide"));
    assert!(store.is_closed());
    assert!(dir.join("profile.json").exists() && dir.join("secret.key").exists());
    assert!(!dir.join("peers.json").exists(), "members are forgotten");

    papo(home.path()).args(["leave", "--yes"]).assert().success();
    assert!(!dir.exists(), "leaving again removes the tombstone");
}
