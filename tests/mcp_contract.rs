//! Contract tests for the MCP surface Claude Code depends on: JSON-RPC behavior, tool
//! schemas, notifications and the text agents read. Snapshots (tests/snapshots/) make
//! any change to `initialize` or `tools/list` a deliberate, reviewed diff.

mod common;

use std::{collections::BTreeMap, path::Path, time::Duration};

use common::{McpClient, endpoint_id, papo, profile_dir};
use iroh::SecretKey;
use papo::{
    proto::{Envelope, PeerKind, now_ms},
    store::{KnownPeer, LogEntry, Store},
};
use serde_json::{Value, json};

fn configured_home(name: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    papo(home.path(), &["new", "--name", name, "--about", "api-repo"]);
    home
}

fn store(home: &Path) -> Store {
    Store::open_at(profile_dir(home, "default")).unwrap()
}

fn envelope(id: &str, from: &str, body: &str) -> Envelope {
    Envelope {
        id: id.into(),
        from: from.into(),
        node: SecretKey::generate().public().to_string(),
        kind: PeerKind::Agent,
        to: None,
        reply_to: None,
        ts: now_ms(),
        body: body.into(),
    }
}

fn tool_schemas(client: &mut McpClient) -> BTreeMap<String, Value> {
    let tools = client.request("tools/list", json!({}));
    tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t["name"].as_str().unwrap().to_string(), t["inputSchema"].clone()))
        .collect()
}

#[test]
fn parse_errors_answer_with_null_id_and_the_server_keeps_serving() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    client.write_raw("{not json");
    let err = client.response_for(&Value::Null, Duration::from_secs(10));
    assert_eq!(err["error"]["code"], -32700);
    assert_eq!(client.request("ping", json!({}))["result"], json!({}));
}

#[test]
fn request_ids_are_echoed_verbatim() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    for id in [json!("abc-1"), json!(0), json!(9_007_199_254_740_991u64), json!("")] {
        let resp = client.request_with_id(id.clone(), "ping", json!({}));
        assert_eq!(resp["id"], id);
        assert_eq!(resp["jsonrpc"], "2.0");
    }
}

#[test]
fn notifications_and_client_responses_never_get_an_answer() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    client.notify("notifications/whatever", json!({"x": 1}));
    client.notify("notifications/initialized", json!({}));
    // A response to a server->client request we never sent.
    client.write(json!({"jsonrpc": "2.0", "id": 77, "result": {}}));
    client.write_raw("");
    client.write_raw("   ");
    let pong = client.request_with_id(json!("after"), "ping", json!({}));
    assert_eq!(pong["id"], "after");
    assert!(client.notifications.is_empty(), "unexpected messages: {:?}", client.notifications);
}

#[test]
fn unknown_methods_are_method_not_found() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    let resp = client.request("resources/list", json!({}));
    assert_eq!(resp["error"]["code"], -32601);
    assert!(resp["error"]["message"].as_str().unwrap().contains("resources/list"));
}

#[test]
fn initialize_without_a_version_gets_the_latest_supported() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn_with(home.path(), &[], false);
    let init = client.request("initialize", json!({}));
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(init["result"]["serverInfo"]["name"], "papo");
    assert!(init["result"]["instructions"].as_str().unwrap().contains("not configured yet"));
    for v in ["2025-06-18", "2025-03-26", "2024-11-05"] {
        let init = client.request("initialize", json!({"protocolVersion": v}));
        assert_eq!(init["result"]["protocolVersion"], v);
    }
}

#[test]
fn every_tool_schema_is_valid_json_schema_and_accepts_only_sane_arguments() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    let schemas = tool_schemas(&mut client);
    assert_eq!(schemas.len(), 5);

    let cases: &[(&str, Value, bool)] = &[
        ("send", json!({"message": "oi"}), true),
        ("send", json!({"message": "oi", "to": "colega", "reply_to": "abc"}), true),
        ("send", json!({}), false),
        ("send", json!({"message": 5}), false),
        ("send", json!({"message": "oi", "cc": "x"}), false),
        ("wait", json!({}), true),
        ("wait", json!({"timeout_seconds": 30, "from": "colega"}), true),
        ("wait", json!({"timeout_seconds": 0}), false),
        ("wait", json!({"timeout_seconds": 1201}), false),
        ("wait", json!({"timeout_seconds": "30"}), false),
        ("inbox", json!({}), true),
        ("inbox", json!({"all": true}), false),
        ("history", json!({"limit": 200}), true),
        ("history", json!({"limit": 0}), false),
        ("history", json!({"limit": 201}), false),
        ("status", json!({}), true),
        ("status", json!({"verbose": true}), false),
    ];
    for (name, schema) in &schemas {
        assert!(jsonschema::meta::is_valid(schema), "{name}: inputSchema is not valid JSON Schema");
        assert_eq!(schema["type"], "object", "{name}");
    }
    for (tool, args, ok) in cases {
        let validator = jsonschema::validator_for(&schemas[*tool]).unwrap();
        assert_eq!(validator.is_valid(args), *ok, "{tool} {args}");
    }
}

#[test]
fn unknown_tool_and_missing_arguments_are_handled() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn(home.path());
    let (text, is_error) = client.call("teleport", json!({}));
    assert!(is_error && text.contains("unknown tool: teleport"), "{text}");

    // `arguments` is optional in tools/call.
    let resp = client.request("tools/call", json!({"name": "status"}));
    assert_eq!(resp["result"]["isError"], Value::Null, "{resp}");
}

#[test]
fn invalid_send_arguments_are_explained() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn(home.path());
    let (text, is_error) = client.call("send", json!({"message": "oi", "to": "nome com espaço"}));
    assert!(is_error && text.contains("name may only contain"), "{text}");
    let (text, is_error) = client.call("send", json!({"message": "x".repeat(60 * 1024)}));
    assert!(is_error && text.contains("the limit is"), "{text}");
    let (text, is_error) = client.call("send", json!({}));
    assert!(is_error && text.contains("`message` is required"), "{text}");
    // Nothing was queued by the refused sends.
    assert!(store(home.path()).outbox().unwrap().is_empty());
}

#[test]
fn wait_sends_progress_only_when_a_token_is_given() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn_with(home.path(), &[("PAPO_TEST_PROGRESS_MS", "150")], true);

    let resp = client.request(
        "tools/call",
        json!({"name": "wait", "arguments": {"timeout_seconds": 1}, "_meta": {"progressToken": "tok-1"}}),
    );
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().starts_with("No new messages after 1s"));
    let progress: Vec<Value> =
        client.notifications.drain(..).filter(|m| m["method"] == "notifications/progress").collect();
    assert!(progress.len() >= 3, "expected periodic progress, got {progress:?}");
    let mut last = -1.0;
    for p in &progress {
        assert_eq!(p["params"]["progressToken"], "tok-1");
        assert_eq!(p["params"]["total"], 1);
        let value = p["params"]["progress"].as_f64().unwrap();
        assert!(value >= last, "progress went backwards: {progress:?}");
        last = value;
    }

    client.call("wait", json!({"timeout_seconds": 1}));
    assert!(
        !client.notifications.iter().any(|m| m["method"] == "notifications/progress"),
        "no token, no progress: {:?}",
        client.notifications
    );
}

#[test]
fn wait_clamps_out_of_range_timeouts() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn(home.path());
    let (text, _) = client.call("wait", json!({"timeout_seconds": 0}));
    assert!(text.starts_with("No new messages after 1s"), "{text}");
}

#[test]
fn the_send_rate_limit_turns_loops_into_errors() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn_with(home.path(), &[("PAPO_MAX_SENDS_PER_10MIN", "0")], true);
    let (text, is_error) = client.call("send", json!({"message": "obrigado!"}));
    assert!(is_error && text.contains("rate limit") && text.contains("stuck in a loop"), "{text}");
}

#[test]
fn unread_backlog_is_pushed_at_startup_and_consumed_by_inbox() {
    let home = configured_home("voce");
    let mut first = envelope("aaa111", "colega", "primeira pergunta");
    first.to = Some("voce".into());
    let mut second = envelope("bbb222", "colega", "segunda");
    second.reply_to = Some("zzz999".into());
    second.kind = PeerKind::Human;
    store(home.path()).save_inbox(&[first, second]).unwrap();

    let mut client = McpClient::spawn(home.path());
    let a = client.wait_channel_event(Duration::from_secs(20));
    let b = client.wait_channel_event(Duration::from_secs(20));
    assert_eq!(a["params"]["content"], "primeira pergunta");
    assert_eq!(
        a["params"]["meta"],
        json!({"from": "colega", "msg_id": "aaa111", "sender_kind": "agent", "to": "voce"})
    );
    assert_eq!(
        b["params"]["meta"],
        json!({"from": "colega", "msg_id": "bbb222", "sender_kind": "human", "reply_to": "zzz999"})
    );

    let (status, _) = client.call("status", json!({}));
    assert!(status.contains("Unread: 2."), "{status}");
    let (inbox, _) = client.call("inbox", json!({}));
    assert!(inbox.contains("[msg_id=aaa111 from=colega (agent)") && inbox.contains(" to=voce]\nprimeira pergunta"));
    assert!(inbox.contains("[msg_id=bbb222 from=colega (human)") && inbox.contains("reply_to=zzz999"));
    let (status, _) = client.call("status", json!({}));
    assert!(status.contains("Unread: 0."), "{status}");
    drop(client);

    // Consumed means gone for good: a new session has nothing to push.
    assert!(store(home.path()).inbox().unwrap().is_empty());
    let mut client = McpClient::spawn(home.path());
    client.call("status", json!({}));
    assert!(!client.notifications.iter().any(common::is_channel_event));
}

#[test]
fn wait_with_from_only_takes_that_senders_messages() {
    let home = configured_home("voce");
    store(home.path())
        .save_inbox(&[envelope("b1", "colega", "do colega"), envelope("c1", "terceiro", "da terceiro")])
        .unwrap();
    let mut client = McpClient::spawn(home.path());
    let (got, _) = client.call("wait", json!({"timeout_seconds": 5, "from": "Terceiro"}));
    assert!(got.contains("da terceiro") && !got.contains("do colega"), "{got}");
    let (rest, _) = client.call("inbox", json!({}));
    assert!(rest.contains("do colega") && !rest.contains("da terceiro"), "{rest}");
}

#[test]
fn history_reads_the_log_and_honors_the_limit() {
    let home = configured_home("voce");
    let s = store(home.path());
    let mut out = envelope("q1", "voce", "qual porta?");
    out.to = Some("colega".into());
    let mut inc = envelope("r1", "colega", "8443");
    inc.reply_to = Some("q1".into());
    s.append_log(&LogEntry::Out { msg: out }).unwrap();
    s.append_log(&LogEntry::Delivered { id: "q1".into(), by: "colega".into(), ts: now_ms() }).unwrap();
    s.append_log(&LogEntry::In { msg: inc }).unwrap();

    let mut client = McpClient::spawn(home.path());
    let (history, _) = client.call("history", json!({}));
    let lines: Vec<&str> = history.lines().collect();
    assert_eq!(lines.len(), 3, "{history}");
    assert!(lines[0].ends_with("voce -> colega (msg q1): qual porta?"));
    assert!(lines[1].ends_with("delivered q1 to colega"));
    assert!(lines[2].ends_with("colega (agent) -> voce (msg r1, reply to q1): 8443"));

    let (last, _) = client.call("history", json!({"limit": 1}));
    assert_eq!(last.lines().count(), 1);
    assert!(last.ends_with("8443"));
}

#[test]
fn empty_history_says_so() {
    let home = configured_home("kj");
    let mut client = McpClient::spawn(home.path());
    assert_eq!(client.call("history", json!({})).0, "No conversation yet.");
}

#[test]
fn status_lists_known_members_and_the_queue() {
    let home = configured_home("kj");
    let s = store(home.path());
    let colega = SecretKey::generate().public();
    let stranger = SecretKey::generate().public();
    let peers = [
        (colega, KnownPeer { name: Some("colega".into()), last_seen_ms: now_ms() - 3 * 3_600_000 }),
        (stranger, KnownPeer::default()),
    ]
    .into_iter()
    .collect();
    s.save_known_peers(&peers).unwrap();
    s.save_outbox(&[envelope("out1", "kj", "pendente")]).unwrap();

    let mut client = McpClient::spawn(home.path());
    let (status, _) = client.call("status", json!({}));
    assert!(status.contains(&format!("(endpoint {})", endpoint_id(home.path(), "default").fmt_short())), "{status}");
    assert!(status.contains("- colega: offline, last seen 3h ago"), "{status}");
    assert!(status.contains(&format!("- unknown ({}): offline, last seen never", stranger.fmt_short())), "{status}");
    assert!(status.contains("Queued for delivery: 1."), "{status}");
}

#[test]
fn status_without_members_points_to_the_invite() {
    let home = configured_home("kj");
    let mut client = McpClient::spawn(home.path());
    let (status, _) = client.call("status", json!({}));
    assert!(status.contains("No room members known yet") && status.contains("papo invite"), "{status}");
}

#[test]
fn snapshot_of_initialize_result() {
    let home = configured_home("voce");
    let mut client = McpClient::spawn_with(home.path(), &[], false);
    let init = client.request("initialize", json!({"protocolVersion": "2025-11-25"}));
    insta::with_settings!({filters => vec![(r"room [0-9a-f]{8}", "room [room-id]")]}, {
        insta::assert_json_snapshot!("initialize_result", init["result"], {".serverInfo.version" => "[version]"});
    });
}

#[test]
fn snapshot_of_tools_list() {
    let home = tempfile::tempdir().unwrap();
    let mut client = McpClient::spawn(home.path());
    let tools = client.request("tools/list", json!({}));
    insta::assert_json_snapshot!("tools_list", tools["result"]);
}

#[test]
fn an_empty_relay_variable_means_the_default_relays() {
    // Shells and container env files often export empty variables.
    for value in ["", "   "] {
        let home = configured_home("voce");
        let mut client = McpClient::spawn_with(home.path(), &[("PAPO_RELAY", value)], true);
        let (status, is_error) = client.call("status", json!({}));
        assert!(!is_error, "PAPO_RELAY={value:?} broke startup: {status}");
    }
    let home = configured_home("voce");
    let mut client = McpClient::spawn_with(home.path(), &[("PAPO_RELAY", "not a url")], true);
    let (status, is_error) = client.call("status", json!({}));
    assert!(is_error && status.contains("PAPO_RELAY is not a valid URL"), "{status}");
}
