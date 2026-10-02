//! Drives the real `papo` binary over stdio the way Claude Code does.

use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_papo");

fn papo(home: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN).args(args).env("PAPO_HOME", home).output().unwrap();
    assert!(out.status.success(), "papo {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    rx: mpsc::Receiver<Value>,
    next_id: u64,
    /// Notifications received while waiting for responses.
    notifications: Vec<Value>,
}

impl McpClient {
    fn spawn(home: &Path) -> Self {
        let mut child = Command::new(BIN)
            .arg("mcp")
            .env("PAPO_HOME", home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let value: Value = serde_json::from_str(&line).expect("server wrote non-JSON to stdout");
                if tx.send(value).is_err() {
                    break;
                }
            }
        });
        let mut client = Self { child, stdin, rx, next_id: 1, notifications: vec![] };
        let init = client.request(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}),
        );
        assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
        client.notify("notifications/initialized", json!({}));
        client
    }

    fn write(&mut self, value: Value) {
        let mut line = serde_json::to_vec(&value).unwrap();
        line.push(b'\n');
        self.stdin.write_all(&line).unwrap();
        self.stdin.flush().unwrap();
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.write(json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.write(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let msg = self.rx.recv_timeout(left).expect("no response from server");
            if msg.get("id") == Some(&json!(id)) {
                return msg;
            }
            self.notifications.push(msg);
        }
    }

    fn call(&mut self, tool: &str, args: Value) -> (String, bool) {
        let resp = self.request("tools/call", json!({"name": tool, "arguments": args}));
        let result = &resp["result"];
        let text = result["content"][0]["text"].as_str().unwrap_or_default().to_string();
        (text, result["isError"].as_bool().unwrap_or(false))
    }

    fn wait_channel_event(&mut self, timeout: Duration) -> Value {
        if let Some(pos) = self.notifications.iter().position(is_channel_event) {
            return self.notifications.remove(pos);
        }
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let msg = self.rx.recv_timeout(left).expect("no channel notification arrived");
            if is_channel_event(&msg) {
                return msg;
            }
            self.notifications.push(msg);
        }
    }
}

fn is_channel_event(msg: &Value) -> bool {
    msg["method"] == "notifications/claude/channel"
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

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

/// Two agents on the real internet (n0 DNS + relays), exactly as two colleagues would
/// run it. Ignored by default because it needs network access:
/// `cargo test --test mcp -- --ignored`
#[test]
#[ignore = "needs internet access"]
fn two_agents_talk_over_the_public_network() {
    let ana_home = tempfile::tempdir().unwrap();
    let bob_home = tempfile::tempdir().unwrap();
    let created = papo(ana_home.path(), &["new", "--name", "ana"]);
    let invite = created.split_whitespace().find(|w| w.starts_with("papo1")).expect("invite in output");
    papo(bob_home.path(), &["join", invite, "--name", "bob"]);

    let mut ana = McpClient::spawn(ana_home.path());
    let mut bob = McpClient::spawn(bob_home.path());

    // Bob dials Ana from the invite; first contact can take a few seconds while the
    // address is resolved and holes are punched.
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let (status, _) = bob.call("status", json!({}));
        if status.contains("ana (agent): online") {
            break;
        }
        assert!(Instant::now() < deadline, "bob never saw ana online:\n{status}");
        std::thread::sleep(Duration::from_secs(2));
    }

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
    assert!(client.rx.try_recv().is_err(), "cancelled request must not get a response");
}
