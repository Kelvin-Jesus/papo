//! Moving one running `papo mcp` between rooms (`rooms`, `switch_room`, `join_room`,
//! `leave_room`) without restarting Claude Code or touching its MCP config.

mod common;

use std::time::Duration;

use common::{McpClient, invite_in, papo, profile_dir};
use iroh::SecretKey;
use papo::{
    proto::{Envelope, PeerKind, now_ms},
    store::Store,
};
use serde_json::json;

/// `default` (you are "voce") and `outra` (you are "voce2"), each in its own room.
fn two_rooms() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "voce"]);
    papo(home.path(), &["--profile", "outra", "new", "--name", "voce2"]);
    home
}

#[test]
fn switch_room_moves_the_session_and_the_next_one_starts_there() {
    let home = two_rooms();
    let mut client = McpClient::spawn(home.path());

    let (rooms, is_error) = client.call("rooms", json!({}));
    assert!(!is_error, "{rooms}");
    assert!(rooms.contains("- default: you are \"voce\" in room ") && rooms.contains("(active in this session)"));
    assert!(rooms.contains("- outra: you are \"voce2\" in room "), "{rooms}");

    let (switched, is_error) = client.call("switch_room", json!({"profile": "outra"}));
    assert!(!is_error, "{switched}");
    assert!(switched.contains("Switched to profile outra") && switched.contains("You are \"voce2\""), "{switched}");
    let (status, _) = client.call("status", json!({}));
    assert!(status.contains("You are \"voce2\""), "{status}");

    // The room we left is unlocked at once: the CLI can leave it while the session runs.
    let left = papo(home.path(), &["--profile", "default", "leave", "--yes"]);
    assert!(left.contains("Perfil default apagado"), "{left}");

    let (again, is_error) = client.call("switch_room", json!({"profile": "outra"}));
    assert!(!is_error && again.starts_with("Already in this room."), "{again}");
    assert!(client.shutdown().success());

    let mut next = McpClient::spawn(home.path());
    let (status, _) = next.call("status", json!({}));
    assert!(status.contains("You are \"voce2\""), "the switch must survive a restart: {status}");
    // The CLI without --profile follows the agent too.
    assert_eq!(papo(home.path(), &["invite"]), papo(home.path(), &["--profile", "outra", "invite"]));
}

#[test]
fn switching_to_a_profile_another_session_uses_keeps_the_current_room() {
    let home = two_rooms();
    let mut other = McpClient::spawn_with(home.path(), &[("PAPO_PROFILE", "outra")], true);
    assert!(!other.call("status", json!({})).1);

    let mut client = McpClient::spawn(home.path());
    let (text, is_error) = client.call("switch_room", json!({"profile": "outra"}));
    assert!(is_error && text.contains("already running"), "{text}");
    let (status, is_error) = client.call("status", json!({}));
    assert!(!is_error && status.contains("You are \"voce\""), "{status}");
}

#[test]
fn switching_to_an_unknown_profile_names_the_existing_ones() {
    let home = two_rooms();
    let mut client = McpClient::spawn(home.path());
    let (text, is_error) = client.call("switch_room", json!({"profile": "nada"}));
    assert!(is_error && text.contains("existing profiles: default, outra"), "{text}");
    let (text, is_error) = client.call("switch_room", json!({"profile": "../x"}));
    assert!(is_error && text.contains("profile must be"), "{text}");
}

#[test]
fn join_room_enters_a_new_room_and_reuses_the_current_name() {
    let colega = tempfile::tempdir().unwrap();
    let invite = invite_in(&papo(colega.path(), &["new", "--name", "colega"]));
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "voce"]);
    let mut client = McpClient::spawn(home.path());

    let (joined, is_error) = client.call("join_room", json!({"invite": invite}));
    assert!(!is_error, "{joined}");
    assert!(joined.contains("as \"voce\" (profile sala-") && joined.contains("Switched to profile sala-"), "{joined}");
    let (rooms, _) = client.call("rooms", json!({}));
    assert!(rooms.contains("- default: you are \"voce\""), "{rooms}");
    assert!(rooms.lines().any(|l| l.starts_with("- sala-") && l.ends_with("(active in this session)")), "{rooms}");

    // The same invite again is a switch, not a second profile for the same room.
    let (again, is_error) = client.call("join_room", json!({"invite": invite}));
    assert!(!is_error && again.contains("You already have this room in profile sala-"), "{again}");

    let (taken, is_error) = client.call("join_room", json!({"invite": invite_in(&papo(colega.path(), &["--profile", "b", "new", "--name", "c"])), "profile": "default"}));
    assert!(is_error && taken.contains("profile 'default' already exists"), "{taken}");
}

#[test]
fn join_room_from_no_room_needs_a_name() {
    let colega = tempfile::tempdir().unwrap();
    let invite = invite_in(&papo(colega.path(), &["new", "--name", "colega"]));
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());

    let (rooms, _) = client.call("rooms", json!({}));
    assert!(rooms.starts_with("No rooms on this machine yet"), "{rooms}");
    let (text, is_error) = client.call("join_room", json!({"invite": invite}));
    assert!(is_error && text.contains("`name` is required"), "{text}");
    let (text, is_error) = client.call("join_room", json!({"invite": "papo1naoeconvite"}));
    assert!(is_error && text.contains("invite"), "{text}");

    let (joined, is_error) = client.call("join_room", json!({"invite": invite, "name": "voce", "profile": "trabalho"}));
    assert!(!is_error && joined.contains("(profile trabalho)"), "{joined}");
    let (status, is_error) = client.call("status", json!({}));
    assert!(!is_error && status.contains("You are \"voce\""), "{status}");
}

#[test]
fn leave_room_deletes_the_profile_and_the_session_can_join_another() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "voce"]);
    let mut client = McpClient::spawn(home.path());
    assert!(!client.call("status", json!({})).1);

    let (left, is_error) = client.call("leave_room", json!({}));
    assert!(!is_error, "{left}");
    assert!(left.contains("Nobody to tell. Profile default deleted."), "{left}");
    assert!(!profile_dir(home.path(), "default").exists());

    let (status, is_error) = client.call("status", json!({}));
    assert!(is_error && status.contains("not in a room") && status.contains("you left room"), "{status}");
    let (again, is_error) = client.call("leave_room", json!({}));
    assert!(is_error && again.contains("not in a room"), "{again}");

    let colega = tempfile::tempdir().unwrap();
    let invite = invite_in(&papo(colega.path(), &["new", "--name", "colega"]));
    let (joined, is_error) = client.call("join_room", json!({"invite": invite, "name": "voce"}));
    assert!(!is_error, "{joined}");
}

#[test]
fn messages_waiting_in_the_new_room_are_pushed_after_the_switch() {
    let home = two_rooms();
    let waiting = Envelope {
        id: "ccc333".into(),
        from: "colega".into(),
        node: SecretKey::generate().public().to_string(),
        kind: PeerKind::Agent,
        to: None,
        reply_to: None,
        ts: now_ms(),
        body: "chegou enquanto você estava na outra sala".into(),
    };
    Store::open_at(profile_dir(home.path(), "outra")).unwrap().save_inbox(&[waiting]).unwrap();
    let mut client = McpClient::spawn(home.path());

    let (rooms, _) = client.call("rooms", json!({}));
    assert!(rooms.contains("- outra: you are \"voce2\" in room ") && rooms.contains(", 1 unread"), "{rooms}");
    assert!(!client.call("switch_room", json!({"profile": "outra"})).1);
    let event = client.wait_channel_message("chegou enquanto você estava na outra sala", Duration::from_secs(20));
    assert_eq!(event["params"]["meta"]["msg_id"], "ccc333");
}
