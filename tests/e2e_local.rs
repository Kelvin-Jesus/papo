//! Hermetic end to end: two real `papo mcp` processes and the human CLI talking through
//! an in-process relay, with no internet. Needs the `test-network` feature:
//! `cargo test --features test-network --test e2e_local`.
#![cfg(feature = "test-network")]

mod common;

use std::{path::Path, process::Command, time::Duration};

use common::{BIN, McpClient, endpoint_id, invite_in, papo};
use serde_json::json;

/// A local relay that lives as long as the test.
struct Relay {
    // Field order matters: the server must go before the runtime that drives it.
    _server: Box<dyn std::any::Any>,
    _rt: tokio::runtime::Runtime,
    url: String,
}

fn relay() -> Relay {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let (_map, url, server) = rt.block_on(iroh::test_utils::run_relay_server()).unwrap();
    Relay { _server: Box::new(server), _rt: rt, url: url.to_string() }
}

struct Room {
    relay: Relay,
    ana: tempfile::TempDir,
    bob: tempfile::TempDir,
}

impl Room {
    fn new() -> Self {
        let ana = tempfile::tempdir().unwrap();
        let bob = tempfile::tempdir().unwrap();
        let invite = invite_in(&papo(ana.path(), &["new", "--name", "ana", "--about", "api-checkout"]));
        papo(bob.path(), &["join", &invite, "--name", "bob"]);
        Self { relay: relay(), ana, bob }
    }

    /// Network environment for a process of `home`, which can reach `peers`.
    fn env(&self, peers: &[&Path]) -> Vec<(&'static str, String)> {
        let ids: Vec<String> = peers.iter().map(|h| endpoint_id(h, "default").to_string()).collect();
        vec![("PAPO_TEST_RELAY", self.relay.url.clone()), ("PAPO_TEST_PEERS", ids.join(","))]
    }

    fn agent(&self, home: &Path, peer: &Path) -> McpClient {
        let env = self.env(&[peer]);
        let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
        McpClient::spawn_with(home, &env, true)
    }

    /// Runs the human CLI from `home` and returns (success, stdout, stderr).
    fn cli(&self, home: &Path, peer: &Path, args: &[&str]) -> (bool, String, String) {
        let out = Command::new(BIN)
            .args(args)
            .env("PAPO_HOME", home)
            .envs(self.env(&[peer]))
            .output()
            .unwrap();
        (out.status.success(), String::from_utf8_lossy(&out.stdout).into(), String::from_utf8_lossy(&out.stderr).into())
    }
}

#[test]
fn two_agents_and_a_human_converse_without_internet() {
    let room = Room::new();
    let mut ana = room.agent(room.ana.path(), room.bob.path());
    let mut bob = room.agent(room.bob.path(), room.ana.path());
    bob.wait_status_contains("ana (agent): online, working on: api-checkout", Duration::from_secs(30));

    let (sent, is_error) = ana.call("send", json!({"message": "Bob, o webhook vai assinado com HMAC?"}));
    assert!(!is_error && sent.starts_with("Delivered to bob"), "{sent}");
    let event = bob.wait_channel_event(Duration::from_secs(20));
    assert_eq!(event["params"]["meta"]["from"], "ana");
    let msg_id = event["params"]["meta"]["msg_id"].as_str().unwrap().to_string();

    let (reply, _) = bob.call("send", json!({"message": "Sim, X-Hub-Signature-256.", "reply_to": msg_id}));
    assert!(reply.starts_with("Delivered to ana"), "{reply}");
    let (got, _) = ana.call("wait", json!({"timeout_seconds": 20}));
    assert!(got.contains("X-Hub-Signature-256") && got.contains(&format!("reply_to={msg_id}")), "{got}");

    // Bob, the person, chimes in from his terminal.
    let (ok, out, err) =
        room.cli(room.bob.path(), room.ana.path(), &["say", "--to", "ana", "pode", "seguir", "com", "o", "HMAC"]);
    assert!(ok, "say failed: {err}");
    assert!(out.starts_with("entregue a ana"), "{out}");
    assert!(!err.contains("subscription"), "a clean exit must not warn about the subscription: {err}");
    let human = ana.wait_channel_message("pode seguir com o HMAC", Duration::from_secs(20));
    assert_eq!(human["params"]["meta"]["sender_kind"], "human");

    let (ok, out, err) = room.cli(room.bob.path(), room.ana.path(), &["status", "--timeout", "15"]);
    assert!(ok, "status failed: {err}");
    assert!(out.contains("ana [") && out.contains("] online — api-checkout"), "{out}");
    assert!(!err.contains("subscription"), "{err}");

    let (history, _) = ana.call("history", json!({}));
    assert!(history.contains("delivered") && history.contains("bob (human) -> ana"), "{history}");
    assert!(ana.shutdown().success() && bob.shutdown().success());
}

#[test]
fn a_message_queued_while_the_peer_is_away_arrives_when_it_starts() {
    let room = Room::new();
    let mut ana = room.agent(room.ana.path(), room.bob.path());
    let (queued, is_error) = ana.call("send", json!({"message": "Quando abrir, revisa o PR 42?"}));
    assert!(!is_error && queued.starts_with("Not acknowledged yet"), "{queued}");
    assert!(ana.call("status", json!({})).0.contains("Queued for delivery: 1."));

    let mut bob = room.agent(room.bob.path(), room.ana.path());
    let event = bob.wait_channel_event(Duration::from_secs(30));
    assert_eq!(event["params"]["content"], "Quando abrir, revisa o PR 42?");
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let (status, _) = ana.call("status", json!({}));
        if status.contains("Queued for delivery: 0.") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "outbox never drained:\n{status}");
        std::thread::sleep(Duration::from_millis(300));
    }
}
