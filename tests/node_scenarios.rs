//! Delivery scenarios beyond the happy path: duplicates, echoes, hostile frames,
//! restarts, crashes, ordering, concurrency and bigger rooms. Real endpoints and real
//! gossip on a private network (see tests/common/localnet.rs).

mod common;

use std::time::Duration;

use bytes::Bytes;
use common::localnet::{LocalNet, RawPeer, STEP, eventually};
use papo::{
    node::SendOutcome,
    proto::{Envelope, Frame, MAX_BODY_BYTES, PeerKind, new_msg_id, now_ms},
    room::RoomSecret,
    store::LogEntry,
};

const SHORT: Duration = Duration::from_millis(400);

fn msg_from_raw(raw: &RawPeer, to: Option<&str>, body: &str) -> Envelope {
    Envelope {
        id: new_msg_id(),
        from: "raw".into(),
        node: raw.id.to_string(),
        kind: PeerKind::Agent,
        to: to.map(Into::into),
        reply_to: None,
        ts: now_ms(),
        body: body.into(),
    }
}

fn acks_for(frames: &[Frame], id: &str) -> usize {
    frames.iter().filter(|f| matches!(f, Frame::Ack { id: acked, .. } if acked == id)).count()
}

#[tokio::test]
async fn duplicate_frames_are_acked_again_but_delivered_once() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;

    let msg = msg_from_raw(&raw, None, "pergunta repetida");
    raw.send(&Frame::Msg(msg.clone())).await;
    assert_eq!(raw.ack_for(&msg.id).await.as_deref(), Some("ana"));
    // The sender never saw our ack (from its point of view) and retries.
    raw.send(&Frame::Msg(msg.clone())).await;
    assert_eq!(raw.ack_for(&msg.id).await.as_deref(), Some("ana"), "duplicates must be re-acked");

    let got = ana.wait(STEP, None).await;
    assert_eq!(got.len(), 1);
    assert!(ana.wait(SHORT, None).await.is_empty(), "delivered only once");
}

#[tokio::test]
async fn a_nodes_own_messages_echoed_back_are_ignored() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;

    let mut echo = msg_from_raw(&raw, None, "eco");
    echo.node = ana.id().to_string();
    raw.send(&Frame::Msg(echo.clone())).await;
    let frames = raw.frames_within(Duration::from_secs(1)).await;
    assert_eq!(acks_for(&frames, &echo.id), 0);
    assert!(ana.unread().is_empty());
}

#[tokio::test]
async fn messages_for_someone_else_are_neither_kept_nor_acked() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;

    let other = msg_from_raw(&raw, Some("carol"), "só pra carol");
    raw.send(&Frame::Msg(other.clone())).await;
    let mine = msg_from_raw(&raw, Some("ANA"), "pra ana");
    raw.send(&Frame::Msg(mine.clone())).await;
    assert_eq!(raw.ack_for(&mine.id).await.as_deref(), Some("ana"));
    let frames = raw.frames_within(SHORT).await;
    assert_eq!(acks_for(&frames, &other.id), 0);
    let got = ana.wait(STEP, None).await;
    assert_eq!(got.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), ["pra ana"]);
}

#[tokio::test]
async fn foreign_and_garbage_frames_are_dropped_and_the_node_keeps_working() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;

    // Sealed for another room (someone who learned the topic but not the secret).
    let foreign = RoomSecret::generate();
    let forged = papo::proto::encode(&foreign, &Frame::Msg(msg_from_raw(&raw, None, "forjada"))).unwrap();
    raw.send_bytes(forged).await;
    raw.send_bytes(Bytes::from_static(b"\x00\x01 not a frame at all")).await;
    // Well sealed, but not a papo frame.
    raw.send_bytes(Bytes::from(net.room.seal(br#"{"t":"teleport","x":1}"#))).await;
    // A hello with a bogus node id and name.
    raw.send(&Frame::Hello(papo::proto::Presence {
        node: "nonsense".into(),
        name: "bad name!".into(),
        about: None,
        kind: PeerKind::Agent,
        ephemeral: false,
    }))
    .await;

    let good = msg_from_raw(&raw, None, "válida");
    raw.send(&Frame::Msg(good.clone())).await;
    assert_eq!(raw.ack_for(&good.id).await.as_deref(), Some("ana"));
    let got = ana.wait(STEP, None).await;
    assert_eq!(got.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), ["válida"]);
    assert!(!ana.peers().iter().any(|p| p.name.as_deref() == Some("bad name!")));
}

#[tokio::test]
async fn consumed_messages_are_not_resurrected_after_a_restart() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;
    let msg = msg_from_raw(&raw, None, "uma vez só");
    raw.send(&Frame::Msg(msg.clone())).await;
    assert_eq!(ana.wait(STEP, None).await.len(), 1);
    ana.shutdown().await;
    drop(ana);

    let ana = net.agent("ana", vec![raw.id]).await;
    raw.wait_neighbor().await;
    raw.send(&Frame::Msg(msg.clone())).await;
    assert_eq!(raw.ack_for(&msg.id).await.as_deref(), Some("ana"), "the restarted node still acks it");
    assert!(ana.wait(Duration::from_secs(1), None).await.is_empty(), "but does not deliver it again");
}

#[tokio::test]
async fn invalid_sends_fail_without_touching_the_outbox_or_log() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let too_big = "x".repeat(MAX_BODY_BYTES + 1);
    assert!(ana.send(&too_big, None, None, SHORT).await.unwrap_err().to_string().contains("limit"));
    assert!(ana.send("   \n ", None, None, SHORT).await.unwrap_err().to_string().contains("empty"));
    assert!(ana.send("oi", Some("nome ruim".into()), None, SHORT).await.is_err());
    assert!(ana.pending().is_empty());
    let store = net.store("ana");
    assert!(store.outbox().unwrap().is_empty());
    assert!(store.read_log().unwrap().is_empty());

    // The largest allowed body still goes out (and is queued, since nobody is here).
    let max = "y".repeat(MAX_BODY_BYTES);
    assert!(matches!(ana.send(&max, None, None, SHORT).await.unwrap(), SendOutcome::Queued { .. }));
}

#[tokio::test]
async fn many_messages_arrive_complete_and_in_order() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);

    let sent: Vec<String> = (0..25).map(|i| format!("mensagem {i:02}")).collect();
    for body in &sent {
        ana.send(body, None, None, STEP).await.unwrap();
    }
    let mut got = vec![];
    while got.len() < sent.len() {
        let batch = bob.wait(STEP, None).await;
        assert!(!batch.is_empty(), "stalled after {} messages", got.len());
        got.extend(batch.into_iter().map(|m| m.body));
    }
    assert_eq!(got, sent);
    eventually("ana's outbox drains", || ana.pending().is_empty()).await;
}

#[tokio::test]
async fn concurrent_sends_are_all_delivered() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);

    let sends = (0..24).map(|i| {
        let ana = &ana;
        async move { ana.send(&format!("paralela {i}"), None, None, STEP).await.unwrap() }
    });
    let outcomes = n0_future::join_all(sends).await;
    assert!(outcomes.iter().all(|o| matches!(o, SendOutcome::Delivered { .. })), "{outcomes:?}");

    let mut got = std::collections::BTreeSet::new();
    while got.len() < 24 {
        let batch = bob.wait(STEP, None).await;
        assert!(!batch.is_empty());
        got.extend(batch.into_iter().map(|m| m.body));
    }
    assert_eq!(got.len(), 24);
    assert!(ana.pending().is_empty());
}

#[tokio::test]
async fn a_four_member_chain_reaches_everyone() {
    let net = LocalNet::new().await;
    let a = net.agent("a", vec![]).await;
    let b = net.agent("b", vec![a.id()]).await;
    let c = net.agent("c", vec![b.id()]).await;
    let d = net.agent("d", vec![c.id()]).await;
    for n in [&b, &c, &d] {
        assert!(n.wait_for_neighbor(STEP).await);
    }

    a.send("reunião às 15h", None, None, STEP).await.unwrap();
    for n in [&b, &c, &d] {
        let got = n.wait(STEP, None).await;
        assert_eq!(got[0].body, "reunião às 15h", "{} missed it", n.name());
    }
    // An addressed message reaches only its addressee, however far it is.
    let outcome = d.send("só pro a", Some("a".into()), None, STEP).await.unwrap();
    assert!(matches!(&outcome, SendOutcome::Delivered { by, .. } if by == "a"), "{outcome:?}");
    assert_eq!(a.wait(STEP, None).await[0].body, "só pro a");
    assert!(b.wait(SHORT, None).await.is_empty() && c.wait(SHORT, None).await.is_empty());
}

#[tokio::test]
async fn a_restarted_peer_reconnects_and_receives_what_was_queued() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);
    bob.shutdown().await;
    drop(bob);
    eventually("ana notices bob left", || ana.neighbor_count() == 0).await;

    let SendOutcome::Queued { id } = ana.send("você voltou?", None, None, SHORT).await.unwrap() else {
        panic!("bob is offline, the message must be queued")
    };
    // Bob comes back with no bootstrap: he remembers ana from his own peers.json.
    let bob = net.agent("bob", vec![]).await;
    let got = bob.wait(STEP, None).await;
    assert_eq!(got[0].id, id);
    eventually("ana's outbox drains", || ana.pending().is_empty()).await;
    assert_eq!(ana.neighbor_count(), 1);
}

#[tokio::test]
async fn messages_to_an_absent_member_wait_for_that_member() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);

    // Bob is online but is not the addressee, so nobody acks.
    let outcome = ana.send("carol, cadê o PR?", Some("carol".into()), None, Duration::from_secs(1)).await.unwrap();
    assert!(matches!(outcome, SendOutcome::Queued { .. }));
    assert_eq!(ana.pending().len(), 1);

    let carol = net.agent("carol", vec![ana.id()]).await;
    assert_eq!(carol.wait(STEP, None).await[0].body, "carol, cadê o PR?");
    eventually("delivered to carol", || ana.pending().is_empty()).await;
    assert!(bob.unread().is_empty());
}

#[tokio::test]
async fn crash_without_shutdown_loses_nothing() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let ana_id = ana.id();
    let SendOutcome::Queued { id } = ana.send("antes do crash", None, None, SHORT).await.unwrap() else {
        panic!("nobody online, expected queue")
    };
    // Simulated crash: no graceful shutdown, the process state is simply gone.
    drop(ana);
    tokio::time::sleep(Duration::from_millis(300)).await;

    let ana = net.agent("ana", vec![]).await;
    assert_eq!(ana.id(), ana_id);
    assert_eq!(ana.pending().len(), 1, "the outbox was on disk before the send returned");
    let bob = net.agent("bob", vec![ana_id]).await;
    assert_eq!(bob.wait(STEP, None).await[0].id, id);
    eventually("delivered after the crash", || ana.pending().is_empty()).await;
}

#[tokio::test]
async fn wait_times_out_quickly_and_filters_by_sender() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let started = tokio::time::Instant::now();
    assert!(ana.wait(Duration::from_millis(200), None).await.is_empty());
    assert!(started.elapsed() < Duration::from_secs(2));

    let bob = net.agent("bob", vec![ana.id()]).await;
    let carol = net.agent("carol", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await && carol.wait_for_neighbor(STEP).await);
    bob.send("do bob", Some("ana".into()), None, STEP).await.unwrap();
    carol.send("da carol", Some("ana".into()), None, STEP).await.unwrap();
    eventually("both arrived", || ana.unread().len() == 2).await;

    let from_carol = ana.wait(STEP, Some("CAROL")).await;
    assert_eq!(from_carol.iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), ["da carol"]);
    assert_eq!(ana.unread().iter().map(|m| m.body.as_str()).collect::<Vec<_>>(), ["do bob"]);
    // `from` waits for that sender even if others keep talking.
    assert!(ana.wait(SHORT, Some("carol")).await.is_empty());
}

#[tokio::test]
async fn unicode_and_large_bodies_arrive_byte_for_byte() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);

    let snippet = format!("```rust\nfn main() {{ println!(\"olá\"); }}\n```\n{}", "linha\n".repeat(3000));
    let bodies = [
        "ação, coração, pão — ñ, ü, ß".to_string(),
        "日本語のテキスト 🐦🔥 עברית مرحبا".to_string(),
        "e\u{301} combinado, \u{200d} zero-width, \t tabs".to_string(),
        snippet,
        "z".repeat(MAX_BODY_BYTES),
    ];
    for body in &bodies {
        ana.send(body, None, None, STEP).await.unwrap();
    }
    let mut got = vec![];
    while got.len() < bodies.len() {
        got.extend(bob.wait(STEP, None).await.into_iter().map(|m| m.body));
    }
    // Leading/trailing whitespace is trimmed on send; everything else is preserved.
    assert_eq!(got, bodies.iter().map(|b| b.trim().to_string()).collect::<Vec<_>>());
}

#[tokio::test]
async fn members_are_remembered_with_names_and_shown_with_context() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);
    eventually("ana learns bob's name", || ana.peers().iter().any(|p| p.name.as_deref() == Some("bob"))).await;

    let view = ana.peers().into_iter().find(|p| p.node == bob.id()).unwrap();
    assert_eq!(view.about.as_deref(), Some("bob-repo"));
    assert_eq!(view.kind, Some(PeerKind::Agent));
    assert!(view.online && view.neighbor && !view.ephemeral);

    let known = net.store("ana").known_peers().unwrap();
    assert_eq!(known[&bob.id()].name.as_deref(), Some("bob"));
    let log = net.store("ana").read_log().unwrap();
    assert!(log.iter().all(|e| !matches!(e, LogEntry::In { .. })), "presence is not conversation");
    assert_eq!(ana.room_id(), net.room.room_id());
    assert!(ana.is_healthy());
}

#[tokio::test]
async fn ephemeral_nodes_ignore_messages_and_are_not_remembered() {
    let net = LocalNet::new().await;
    let mut raw = RawPeer::spawn(&net).await;
    let kj = net.cli("kj", vec![raw.id]).await;
    raw.wait_neighbor().await;
    let msg = msg_from_raw(&raw, Some("kj"), "pro agente, não pro CLI");
    raw.send(&Frame::Msg(msg.clone())).await;
    let frames = raw.frames_within(Duration::from_secs(1)).await;
    assert_eq!(acks_for(&frames, &msg.id), 0, "a CLI node must not consume messages for the agent");
    assert!(kj.unread().is_empty());
    // It announced itself as ephemeral and human.
    assert!(frames.iter().any(|f| matches!(f, Frame::Hello(p) if p.ephemeral && p.kind == PeerKind::Human)));
}
