//! Integration tests for room membership and delivery. Every node runs a real iroh
//! endpoint and real gossip; only the network is local (in-process relay + in-memory
//! address lookup), so these tests never reach the internet.

use std::time::Duration;

use iroh::{
    Endpoint, EndpointAddr, RelayMap, RelayMode, RelayUrl, SecretKey, address_lookup::memory::MemoryLookup,
    endpoint::presets, tls::CaTlsConfig,
};
use papo::{
    node::{Node, NodeEvent, NodeOptions, SendOutcome},
    proto::{PeerKind, now_ms},
    room::RoomSecret,
    store::{LogEntry, Profile, Store},
};
use tempfile::TempDir;

const STEP: Duration = Duration::from_secs(20);

struct LocalNet {
    relay_map: RelayMap,
    relay_url: RelayUrl,
    lookup: MemoryLookup,
    room: RoomSecret,
    dir: TempDir,
    // Keeps the in-process relay alive for the test's lifetime.
    _relay: Box<dyn std::any::Any + Send>,
}

impl LocalNet {
    async fn new() -> Self {
        let (relay_map, relay_url, server) = iroh::test_utils::run_relay_server().await.unwrap();
        Self {
            relay_map,
            relay_url,
            lookup: MemoryLookup::new(),
            room: RoomSecret::generate(),
            dir: tempfile::tempdir().unwrap(),
            _relay: Box::new(server),
        }
    }

    /// Opens the profile if it exists, so a restarted node sees its previous state.
    fn store(&self, name: &str) -> Store {
        let dir = self.dir.path().join(name);
        if let Ok(store) = Store::open_at(dir.clone()) {
            return store;
        }
        let profile = Profile { name: name.into(), about: None, room: self.room.to_base32(), created_ms: now_ms() };
        Store::create_at(dir, &profile).unwrap()
    }

    async fn endpoint(&self, key: SecretKey) -> Endpoint {
        let ep = Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Custom(self.relay_map.clone()))
            .secret_key(key)
            .ca_tls_config(CaTlsConfig::insecure_skip_verify())
            .bind()
            .await
            .unwrap();
        ep.address_lookup().unwrap().add(self.lookup.clone());
        self.lookup.add_endpoint_info(EndpointAddr::new(ep.id()).with_relay_url(self.relay_url.clone()));
        ep.online().await;
        ep
    }

    /// A long-running agent node with persistent state, like the MCP server.
    async fn agent(&self, name: &str, bootstrap: Vec<iroh::EndpointId>) -> Node {
        let store = self.store(name);
        let ep = self.endpoint(store.secret_key().unwrap()).await;
        let opts = NodeOptions {
            name: name.into(),
            about: Some(format!("{name}-repo")),
            kind: PeerKind::Agent,
            ephemeral: false,
            bootstrap,
        };
        Node::spawn(ep, self.room.clone(), opts, Some(store)).await.unwrap()
    }

    /// A one-shot CLI identity, like `papo say`.
    async fn cli(&self, name: &str, bootstrap: Vec<iroh::EndpointId>) -> Node {
        let ep = self.endpoint(SecretKey::generate()).await;
        let opts = NodeOptions { name: name.into(), about: None, kind: PeerKind::Human, ephemeral: true, bootstrap };
        Node::spawn(ep, self.room.clone(), opts, None).await.unwrap()
    }
}

async fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + STEP;
    while !check() {
        assert!(tokio::time::Instant::now() < deadline, "timed out waiting for: {what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn message_is_delivered_acked_and_logged() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await, "bob never joined ana");

    let outcome = ana.send("qual o schema do webhook?", None, None, STEP).await.unwrap();
    let SendOutcome::Delivered { id, by } = outcome else { panic!("expected delivery, got {outcome:?}") };
    assert_eq!(by, "bob");

    let got = bob.wait(STEP, None).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, id);
    assert_eq!(got[0].from, "ana");
    assert_eq!(got[0].body, "qual o schema do webhook?");
    assert!(bob.unread().is_empty(), "wait must consume what it returns");
    assert!(ana.pending().is_empty(), "acked message must leave the outbox");

    let log = net.store("ana").read_log().unwrap();
    assert!(log.iter().any(|e| matches!(e, LogEntry::Out { msg } if msg.id == id)));
    assert!(log.iter().any(|e| matches!(e, LogEntry::Delivered { id: d, by, .. } if *d == id && by == "bob")));

    // Presence travelled too: bob knows who ana is and what she works on.
    let peers = bob.peers();
    let ana_view = peers.iter().find(|p| p.node == ana.id()).expect("bob should know ana");
    assert_eq!(ana_view.name.as_deref(), Some("ana"));
    assert_eq!(ana_view.about.as_deref(), Some("ana-repo"));
    assert!(ana_view.online);
}

#[tokio::test]
async fn queued_message_survives_sender_restart_and_reaches_late_peer() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let ana_id = ana.id();

    let outcome = ana.send("deploy amanhã às 9h?", None, None, Duration::from_millis(300)).await.unwrap();
    let SendOutcome::Queued { id } = outcome else { panic!("nobody is online, expected queue, got {outcome:?}") };
    ana.shutdown().await;
    drop(ana);

    // Same profile, same identity: the outbox must come back from disk.
    let ana = net.agent("ana", vec![]).await;
    assert_eq!(ana.id(), ana_id, "identity must be stable across restarts");
    assert_eq!(ana.pending().len(), 1);
    let mut ana_events = ana.subscribe();

    let bob = net.agent("bob", vec![ana_id]).await;
    let got = bob.wait(STEP, None).await;
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, id);

    let delivered = tokio::time::timeout(STEP, async {
        loop {
            if let Ok(NodeEvent::Delivered { id: d, by }) = ana_events.recv().await
                && d == id
            {
                return by;
            }
        }
    })
    .await
    .expect("ana never saw the ack");
    assert_eq!(delivered, "bob");
    assert!(ana.pending().is_empty());
}

#[tokio::test]
async fn addressed_message_only_reaches_the_addressee() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    let carol = net.agent("carol", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);
    assert!(carol.wait_for_neighbor(STEP).await);

    let outcome = ana.send("só pra carol", Some("Carol".into()), None, STEP).await.unwrap();
    assert!(matches!(&outcome, SendOutcome::Delivered { by, .. } if by == "carol"), "{outcome:?}");
    assert_eq!(carol.wait(STEP, None).await.len(), 1);
    assert!(bob.wait(Duration::from_secs(1), None).await.is_empty(), "bob must not get carol's message");
}

#[tokio::test]
async fn replying_marks_that_peers_earlier_messages_as_read() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let bob = net.agent("bob", vec![ana.id()]).await;
    let dave = net.agent("dave", vec![ana.id()]).await;
    assert!(bob.wait_for_neighbor(STEP).await);
    assert!(dave.wait_for_neighbor(STEP).await);

    bob.send("primeira", Some("ana".into()), None, STEP).await.unwrap();
    let SendOutcome::Delivered { id: second, .. } = bob.send("segunda", Some("ana".into()), None, STEP).await.unwrap()
    else {
        panic!("expected delivery")
    };
    dave.send("da dave", Some("ana".into()), None, STEP).await.unwrap();
    eventually("ana has 3 unread", || ana.unread().len() == 3).await;

    ana.send("respondendo a segunda", Some("bob".into()), Some(second), STEP).await.unwrap();
    let left: Vec<String> = ana.unread().into_iter().map(|m| m.body).collect();
    assert_eq!(left, vec!["da dave".to_string()], "only bob's messages up to the reply are read");
}

#[tokio::test]
async fn human_cli_can_talk_but_is_not_remembered_as_member() {
    let net = LocalNet::new().await;
    let ana = net.agent("ana", vec![]).await;
    let kj = net.cli("kj", vec![ana.id()]).await;
    assert!(kj.wait_for_neighbor(STEP).await);

    let outcome = kj.send("Claude, pergunta pro bob sobre o deploy", None, None, STEP).await.unwrap();
    assert!(matches!(&outcome, SendOutcome::Delivered { by, .. } if by == "ana"), "{outcome:?}");
    let got = ana.wait(STEP, None).await;
    assert_eq!(got[0].kind, PeerKind::Human);

    let known = net.store("ana").known_peers().unwrap();
    assert!(!known.contains_key(&kj.id()), "ephemeral CLI identities must not be persisted");

    // And a CLI node never consumes messages meant for the agent.
    ana.send("oi kj", None, None, Duration::from_millis(500)).await.unwrap();
    assert!(kj.unread().is_empty());
}
