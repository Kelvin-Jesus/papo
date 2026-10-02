//! Drives the real `papo mcp` binary over stdio the way Claude Code does.

mod common;

use std::time::{Duration, Instant};

use common::{McpClient, invite_in, papo};
use serde_json::json;

#[test]
fn speaks_mcp_and_advertises_the_channel_capability() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "ana", "--about", "api-repo"]);
    let mut client = McpClient::spawn(home.path());

    let init = client.request("initialize", json!({"protocolVersion": "2026-07-28", "capabilities": {}}));
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25", "must not negotiate revisions that disable channels");
    assert_eq!(init["result"]["capabilities"]["experimental"]["claude/channel"], json!({}));
    assert!(init["result"]["instructions"].as_str().unwrap().contains("You are \"ana\""));

    let tools = client.request("tools/list", json!({}));
    let names: Vec<&str> =
        tools["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["send", "wait", "inbox", "history", "status"]);

    let (status, is_error) = client.call("status", json!({}));
    assert!(!is_error, "{status}");
    assert!(status.contains("You are \"ana\""), "{status}");

    let (inbox, _) = client.call("inbox", json!({}));
    assert_eq!(inbox, "No unread messages.");

    let (empty, is_error) = client.call("send", json!({"message": "   "}));
    assert!(is_error, "blank messages must be rejected: {empty}");

    assert_eq!(client.request("bogus/method", json!({}))["error"]["code"], -32601);
}

#[test]
fn unconfigured_profile_still_starts_and_explains_setup() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    let (text, is_error) = client.call("status", json!({}));
    assert!(is_error);
    assert!(text.contains("papo new"), "{text}");
}

#[test]
fn second_server_on_the_same_profile_is_refused() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "ana"]);
    let mut first = McpClient::spawn(home.path());
    assert!(!first.call("status", json!({})).1);
    let mut second = McpClient::spawn(home.path());
    let (text, is_error) = second.call("status", json!({}));
    assert!(is_error);
    assert!(text.contains("already running"), "{text}");
}

#[test]
fn wait_times_out_and_can_be_cancelled() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "ana"]);
    let mut client = McpClient::spawn(home.path());

    let started = Instant::now();
    let (text, is_error) = client.call("wait", json!({"timeout_seconds": 1}));
    assert!(!is_error && text.starts_with("No new messages after 1s"), "{text}");
    assert!(started.elapsed() < Duration::from_secs(10));

    // A cancelled long poll must not answer; the server keeps serving other requests.
    client.write(json!({"jsonrpc": "2.0", "id": 900, "method": "tools/call",
        "params": {"name": "wait", "arguments": {"timeout_seconds": 600}}}));
    std::thread::sleep(Duration::from_millis(300));
    client.notify("notifications/cancelled", json!({"requestId": 900, "reason": "user interrupted"}));
    let pong = client.request("ping", json!({}));
    assert_eq!(pong["result"], json!({}));
    std::thread::sleep(Duration::from_millis(300));
    assert!(client.next_message(Duration::from_millis(100)).is_none(), "cancelled request must not get a response");
}

#[test]
fn closing_stdin_ends_the_server_cleanly() {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", "ana"]);
    let mut client = McpClient::spawn(home.path());
    assert!(!client.call("status", json!({})).1);
    let status = client.shutdown();
    assert!(status.success(), "{status}");

    // The profile lock is released with the process, so the next session can start.
    let mut next = McpClient::spawn(home.path());
    assert!(!next.call("status", json!({})).1);
}

/// Two agents on the real internet (n0 DNS + relays), exactly as two colleagues would
/// run it. Ignored by default because it needs network access:
/// `cargo test --test mcp -- --ignored`
#[test]
#[ignore = "needs internet access"]
fn two_agents_talk_over_the_public_network() {
    let ana_home = tempfile::tempdir().unwrap();
    let bob_home = tempfile::tempdir().unwrap();
    let invite = invite_in(&papo(ana_home.path(), &["new", "--name", "ana"]));
    papo(bob_home.path(), &["join", &invite, "--name", "bob"]);

    let mut ana = McpClient::spawn(ana_home.path());
    let mut bob = McpClient::spawn(bob_home.path());

    // Bob dials Ana from the invite; first contact can take a few seconds while the
    // address is resolved and holes are punched.
    bob.wait_status_contains("ana (agent): online", Duration::from_secs(90));

    let (sent, is_error) = ana.call("send", json!({"message": "Bob, qual porta o serviço de auth usa?"}));
    assert!(!is_error && sent.starts_with("Delivered to bob"), "{sent}");

    // Pushed into Bob's session as a channel event, with the metadata Claude needs to reply.
    let event = bob.wait_channel_event(Duration::from_secs(30));
    assert_eq!(event["params"]["content"], "Bob, qual porta o serviço de auth usa?");
    assert_eq!(event["params"]["meta"]["from"], "ana");
    let msg_id = event["params"]["meta"]["msg_id"].as_str().unwrap().to_string();

    let (reply, is_error) = bob.call("send", json!({"message": "8443, com TLS.", "reply_to": msg_id}));
    assert!(!is_error && reply.starts_with("Delivered to ana"), "{reply}");
    let (bob_inbox, _) = bob.call("inbox", json!({}));
    assert_eq!(bob_inbox, "No unread messages.", "replying must mark the message read");

    let (got, _) = ana.call("wait", json!({"timeout_seconds": 30}));
    assert!(
        got.contains("from=bob") && got.contains("8443, com TLS.") && got.contains(&format!("reply_to={msg_id}")),
        "{got}"
    );

    let (history, _) = ana.call("history", json!({}));
    assert!(history.contains("delivered"), "{history}");
}
