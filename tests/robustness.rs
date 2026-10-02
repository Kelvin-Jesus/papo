//! Fuzz-style robustness on stable Rust: whatever a peer, a chat app or a client throws
//! at papo must produce an error, never a panic or a dead server. The coverage-guided
//! fuzzers in fuzz/ explore the same entry points deeper (nightly only).

mod common;

use std::time::Duration;

use common::McpClient;
use papo::{
    proto,
    room::{Invite, RoomSecret},
};
use proptest::prelude::*;
use serde_json::{Value, json};

fn arb_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::from),
        any::<i64>().prop_map(Value::from),
        any::<f64>().prop_filter("finite", |f| f.is_finite()).prop_map(Value::from),
        ".{0,20}".prop_map(Value::from),
    ];
    leaf.prop_recursive(3, 24, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
            prop::collection::btree_map("[a-z_]{1,8}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// Lines a confused or hostile client could send: raw garbage, valid JSON of the wrong
/// shape, and requests with random ids, methods and arguments.
fn arb_line() -> impl Strategy<Value = String> {
    let methods = prop_oneof![
        Just("tools/call".to_string()),
        Just("tools/list".to_string()),
        Just("initialize".to_string()),
        Just("ping".to_string()),
        Just("notifications/cancelled".to_string()),
        "[a-z/]{0,16}",
    ];
    let tools = prop_oneof![Just("inbox"), Just("status"), Just("history"), Just("wait"), Just("send"), Just("nope")];
    prop_oneof![
        ".{0,80}".prop_map(|s| s.replace('\n', " ")),
        arb_json().prop_map(|v| v.to_string()),
        (arb_json(), methods, arb_json()).prop_map(|(id, method, params)| {
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
        }),
        (any::<u32>(), tools, arb_json()).prop_map(|(id, tool, args)| {
            // Short waits only, so a random `wait` cannot stall the test.
            let args = if tool == "wait" { json!({"timeout_seconds": 1}) } else { args };
            json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": tool, "arguments": args}})
                .to_string()
        }),
    ]
}

proptest! {
    #[test]
    fn invite_decode_never_panics(input in ".*") {
        let _ = Invite::decode(&input);
        let _ = Invite::decode(&format!("papo1{input}"));
    }

    #[test]
    fn room_secret_parse_never_panics(input in ".*") {
        let _ = RoomSecret::from_base32(&input);
    }

    #[test]
    fn opening_arbitrary_bytes_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        let room = RoomSecret::generate();
        prop_assert!(room.open(&bytes).is_err());
        prop_assert!(proto::decode(&room, &bytes).is_err());
    }

    /// A member of the room (who has the key) sends well-sealed but malformed JSON.
    #[test]
    fn decoding_sealed_garbage_never_panics(json in arb_json(), raw in prop::collection::vec(any::<u8>(), 0..512)) {
        let room = RoomSecret::generate();
        let _ = proto::decode(&room, &room.seal(json.to_string().as_bytes()));
        let _ = proto::decode(&room, &room.seal(&raw));
    }
}

proptest! {
    // Each case starts a real process, so fewer cases with many lines each.
    #![proptest_config(ProptestConfig { cases: 6, ..ProptestConfig::default() })]

    #[test]
    fn the_mcp_server_survives_any_input(lines in prop::collection::vec(arb_line(), 20..60)) {
        let home = tempfile::tempdir().unwrap();
        let mut client = McpClient::spawn(home.path());
        for line in &lines {
            client.write_raw(line);
        }
        // Still alive and in sync: the next request gets its own answer. The reader
        // thread in McpClient panics if anything but JSON ever reaches stdout.
        let pong = client.request_with_id(json!("still-alive"), "ping", json!({}));
        prop_assert_eq!(&pong["result"], &json!({}));
        let tools = client.request("tools/list", json!({}));
        prop_assert_eq!(tools["result"]["tools"].as_array().map(Vec::len), Some(5));
        // Give in-flight random calls a moment to finish; none may crash the server.
        std::thread::sleep(Duration::from_millis(1500));
        let pong = client.request_with_id(json!("still-alive-2"), "ping", json!({}));
        prop_assert_eq!(&pong["result"], &json!({}));
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 3, ..ProptestConfig::default() })]

    /// Same barrage against a configured profile, so random arguments reach the real
    /// tool handlers (send, history, inbox, status) instead of the "not running" path.
    #[test]
    fn configured_tools_survive_random_arguments(lines in prop::collection::vec(arb_line(), 20..40)) {
        let home = tempfile::tempdir().unwrap();
        common::papo(home.path(), &["new", "--name", "ana"]);
        let mut client = McpClient::spawn(home.path());
        for line in &lines {
            client.write_raw(line);
        }
        let (status, is_error) = client.call("status", json!({}));
        prop_assert!(!is_error, "{}", status);
        prop_assert!(status.contains("You are \"ana\""));
        let pong = client.request_with_id(json!("still-alive"), "ping", json!({}));
        prop_assert_eq!(&pong["result"], &json!({}));
    }
}
