//! Integration tests for room membership and delivery. Every node runs a real iroh
//! endpoint and real gossip; only the network is local (in-process relay + in-memory
//! address lookup), so these tests never reach the internet.

mod common;

use std::time::Duration;

use common::localnet::{LocalNet, STEP, eventually};
use papo::{
    node::{NodeEvent, SendOutcome},
    proto::PeerKind,
    store::LogEntry,
};

#[tokio::test]
async fn message_is_delivered_acked_and_logged() {
    let net = LocalNet::new().await;
    let voce = net.agent("voce", vec![]).await;
    let colega = net.agent("colega", vec![voce.id()]).await;
    assert!(colega.wait_for_neighbor(STEP).await, "colega never joined voce");

    let outcome = voce.send("qual o schema do webhook?", None, None, STEP).await.unwrap();
    let SendOutcome::Delivered { id, by } = outcome else { panic!("expected delivery, got {outcome:?}") };
    assert_eq!(by, "colega");

    let got = colega.wait(STEP, None).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, id);
    assert_eq!(got[0].from, "voce");
    assert_eq!(got[0].body, "qual o schema do webhook?");
    assert!(colega.unread().is_empty(), "wait must consume what it returns");
    assert!(voce.pending().is_empty(), "acked message must leave the outbox");

    let log = net.store("voce").read_log().unwrap();
    assert!(log.iter().any(|e| matches!(e, LogEntry::Out { msg } if msg.id == id)));
    assert!(log.iter().any(|e| matches!(e, LogEntry::Delivered { id: d, by, .. } if *d == id && by == "colega")));

    // Presence travelled too: colega knows who voce is and what that agent works on.
    let peers = colega.peers();
    let voce_view = peers.iter().find(|p| p.node == voce.id()).expect("colega should know voce");
    assert_eq!(voce_view.name.as_deref(), Some("voce"));
    assert_eq!(voce_view.about.as_deref(), Some("voce-repo"));
    assert!(voce_view.online);
}

#[tokio::test]
async fn queued_message_survives_sender_restart_and_reaches_late_peer() {
    let net = LocalNet::new().await;
    let voce = net.agent("voce", vec![]).await;
    let voce_id = voce.id();

    let outcome = voce.send("deploy amanhã às 9h?", None, None, Duration::from_millis(300)).await.unwrap();
    let SendOutcome::Queued { id } = outcome else { panic!("nobody is online, expected queue, got {outcome:?}") };
    voce.shutdown().await;
    drop(voce);

    // Same profile, same identity: the outbox must come back from disk.
    let voce = net.agent("voce", vec![]).await;
    assert_eq!(voce.id(), voce_id, "identity must be stable across restarts");
    assert_eq!(voce.pending().len(), 1);
    let mut voce_events = voce.subscribe();

    let colega = net.agent("colega", vec![voce_id]).await;
    let got = colega.wait(STEP, None).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, id);

    let delivered = tokio::time::timeout(STEP, async {
        loop {
            if let Ok(NodeEvent::Delivered { id: d, by }) = voce_events.recv().await
                && d == id
            {
                return by;
            }
        }
    })
    .await
    .expect("voce never saw the ack");
    assert_eq!(delivered, "colega");
    assert!(voce.pending().is_empty());
}

#[tokio::test]
async fn addressed_message_only_reaches_the_addressee() {
    let net = LocalNet::new().await;
    let voce = net.agent("voce", vec![]).await;
    let colega = net.agent("colega", vec![voce.id()]).await;
    let terceiro = net.agent("terceiro", vec![voce.id()]).await;
    assert!(colega.wait_for_neighbor(STEP).await);
    assert!(terceiro.wait_for_neighbor(STEP).await);

    let outcome = voce.send("só pra terceiro", Some("Terceiro".into()), None, STEP).await.unwrap();
    assert!(matches!(&outcome, SendOutcome::Delivered { by, .. } if by == "terceiro"), "{outcome:?}");
    assert_eq!(terceiro.wait(STEP, None).await.len(), 1);
    assert!(colega.wait(Duration::from_secs(1), None).await.is_empty(), "colega must not get terceiro's message");
}

#[tokio::test]
async fn replying_marks_that_peers_earlier_messages_as_read() {
    let net = LocalNet::new().await;
    let voce = net.agent("voce", vec![]).await;
    let colega = net.agent("colega", vec![voce.id()]).await;
    let quarto = net.agent("quarto", vec![voce.id()]).await;
    assert!(colega.wait_for_neighbor(STEP).await);
    assert!(quarto.wait_for_neighbor(STEP).await);

    colega.send("primeira", Some("voce".into()), None, STEP).await.unwrap();
    let SendOutcome::Delivered { id: second, .. } =
        colega.send("segunda", Some("voce".into()), None, STEP).await.unwrap()
    else {
        panic!("expected delivery")
    };
    quarto.send("da quarto", Some("voce".into()), None, STEP).await.unwrap();
    eventually("voce has 3 unread", || voce.unread().len() == 3).await;

    voce.send("respondendo a segunda", Some("colega".into()), Some(second), STEP).await.unwrap();
    let left: Vec<String> = voce.unread().into_iter().map(|m| m.body).collect();
    assert_eq!(left, vec!["da quarto".to_string()], "only colega's messages up to the reply are read");
}

#[tokio::test]
async fn human_cli_can_talk_but_is_not_remembered_as_member() {
    let net = LocalNet::new().await;
    let colega = net.agent("colega", vec![]).await;
    let voce = net.cli("voce", vec![colega.id()]).await;
    assert!(voce.wait_for_neighbor(STEP).await);

    let outcome = voce.send("Claude, pergunta pro terceiro sobre o deploy", None, None, STEP).await.unwrap();
    assert!(matches!(&outcome, SendOutcome::Delivered { by, .. } if by == "colega"), "{outcome:?}");
    let got = colega.wait(STEP, None).await;
    assert_eq!(got[0].kind, PeerKind::Human);

    let known = net.store("colega").known_peers().unwrap();
    assert!(!known.contains_key(&voce.id()), "ephemeral CLI identities must not be persisted");

    // And a CLI node never consumes messages meant for the agent.
    colega.send("oi voce", None, None, Duration::from_millis(500)).await.unwrap();
    assert!(voce.unread().is_empty());
}
