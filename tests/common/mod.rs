//! Helpers shared by the integration tests that drive the real `papo` binary.
//!
//! Every helper isolates state with its own `PAPO_HOME`, so tests can run in parallel
//! without touching the developer's `~/.papo`.

// Each test crate includes this module and uses a different subset of it.
#![allow(dead_code)]

use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

use iroh::{EndpointId, SecretKey};
use serde_json::{Value, json};

pub mod localnet;

pub const BIN: &str = env!("CARGO_BIN_EXE_papo");

/// Runs `papo` against an isolated home and returns stdout; fails the test (showing
/// stderr) when the command fails.
pub fn papo(home: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN).args(args).env("PAPO_HOME", home).output().expect("spawn papo");
    assert!(out.status.success(), "papo {args:?} failed:\n{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).expect("utf-8 stdout")
}

/// First `papo1...` token in a command's output.
pub fn invite_in(output: &str) -> String {
    output.split_whitespace().find(|w| w.starts_with("papo1")).expect("invite in output").to_string()
}

pub fn profile_dir(home: &Path, profile: &str) -> PathBuf {
    home.join("profiles").join(profile)
}

/// The endpoint id a profile's MCP server will use, read from its `secret.key`.
pub fn endpoint_id(home: &Path, profile: &str) -> EndpointId {
    let hex = std::fs::read_to_string(profile_dir(home, profile).join("secret.key")).expect("secret.key");
    let bytes = data_encoding::HEXLOWER.decode(hex.trim().as_bytes()).expect("hex key");
    SecretKey::from_bytes(&bytes.try_into().expect("32 bytes")).public()
}

/// Speaks JSON-RPC over the stdio of a `papo mcp` child, the way Claude Code does.
pub struct McpClient {
    child: Child,
    /// `None` once closed, which is how Claude Code ends a session.
    stdin: Option<ChildStdin>,
    rx: mpsc::Receiver<Value>,
    next_id: u64,
    /// Notifications that arrived while waiting for a response.
    pub notifications: Vec<Value>,
}

impl McpClient {
    /// Spawns and completes the initialize handshake.
    pub fn spawn(home: &Path) -> Self {
        Self::spawn_with(home, &[], true)
    }

    pub fn spawn_with(home: &Path, envs: &[(&str, &str)], initialize: bool) -> Self {
        let mut cmd = Command::new(BIN);
        cmd.arg("mcp").env("PAPO_HOME", home).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("spawn papo mcp");
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                // Anything that is not JSON on stdout corrupts the protocol for Claude Code.
                let value: Value =
                    serde_json::from_str(&line).unwrap_or_else(|_| panic!("server wrote non-JSON to stdout: {line}"));
                if tx.send(value).is_err() {
                    break;
                }
            }
        });
        let mut client = Self { child, stdin: Some(stdin), rx, next_id: 1, notifications: vec![] };
        if initialize {
            let init = client.request(
                "initialize",
                json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}),
            );
            assert_eq!(init["result"]["protocolVersion"], "2025-06-18", "{init}");
            client.notify("notifications/initialized", json!({}));
        }
        client
    }

    pub fn write_raw(&mut self, line: &str) {
        let stdin = self.stdin.as_mut().expect("stdin already closed");
        stdin.write_all(line.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
    }

    pub fn write(&mut self, value: Value) {
        self.write_raw(&serde_json::to_string(&value).unwrap());
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        self.write(json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    pub fn fresh_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn request(&mut self, method: &str, params: Value) -> Value {
        let id = json!(self.fresh_id());
        self.request_with_id(id, method, params)
    }

    /// Sends a request with an arbitrary id and returns the response carrying that id.
    pub fn request_with_id(&mut self, id: Value, method: &str, params: Value) -> Value {
        self.write(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        self.response_for(&id, Duration::from_secs(120))
    }

    pub fn response_for(&mut self, id: &Value, timeout: Duration) -> Value {
        let deadline = Instant::now() + timeout;
        loop {
            let msg = self.next_message(deadline.saturating_duration_since(Instant::now())).expect("no response");
            if msg.get("id") == Some(id) && msg.get("method").is_none() {
                return msg;
            }
            self.notifications.push(msg);
        }
    }

    pub fn next_message(&mut self, timeout: Duration) -> Option<Value> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// Calls a tool and returns (text, isError).
    pub fn call(&mut self, tool: &str, args: Value) -> (String, bool) {
        let resp = self.request("tools/call", json!({"name": tool, "arguments": args}));
        let result = &resp["result"];
        let text = result["content"][0]["text"].as_str().unwrap_or_default().to_string();
        (text, result["isError"].as_bool().unwrap_or(false))
    }

    /// Waits for a notification with `method`, consuming earlier buffered ones first.
    pub fn wait_notification(&mut self, method: &str, timeout: Duration) -> Value {
        if let Some(pos) = self.notifications.iter().position(|m| m["method"] == method) {
            return self.notifications.remove(pos);
        }
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let msg = self.next_message(left).unwrap_or_else(|| panic!("no {method} notification arrived"));
            if msg["method"] == method {
                return msg;
            }
            self.notifications.push(msg);
        }
    }

    pub fn wait_channel_event(&mut self, timeout: Duration) -> Value {
        self.wait_notification("notifications/claude/channel", timeout)
    }

    /// Waits for the channel event carrying `content`, skipping others (a message read
    /// through `wait` was usually also pushed, so earlier events can be pending).
    pub fn wait_channel_message(&mut self, content: &str, timeout: Duration) -> Value {
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let event = self.wait_channel_event(left);
            if event["params"]["content"] == content {
                return event;
            }
        }
    }

    /// Polls `status` until `needle` shows up (peers take a moment to find each other).
    pub fn wait_status_contains(&mut self, needle: &str, timeout: Duration) -> String {
        let deadline = Instant::now() + timeout;
        loop {
            let (status, _) = self.call("status", json!({}));
            if status.contains(needle) {
                return status;
            }
            assert!(Instant::now() < deadline, "status never contained {needle:?}:\n{status}");
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Closes stdin like Claude Code does at the end of a session and waits for exit.
    pub fn shutdown(mut self) -> std::process::ExitStatus {
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "papo mcp did not exit after stdin closed");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for McpClient {
    /// Ends the session the way Claude Code does (EOF on stdin) so every test also
    /// exercises a clean shutdown, and so instrumented builds get to write their
    /// coverage profile; a SIGKILL would lose it. Kills only as a last resort.
    fn drop(&mut self) {
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = self.child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn is_channel_event(msg: &Value) -> bool {
    msg["method"] == "notifications/claude/channel"
}
