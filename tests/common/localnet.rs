//! A private network for node tests: an in-process relay, an in-memory address table
//! and real iroh endpoints, so nothing reaches the internet.

use std::time::Duration;

use bytes::Bytes;
use iroh::{
    Endpoint, EndpointAddr, EndpointId, RelayMap, RelayMode, RelayUrl, SecretKey, address_lookup::memory::MemoryLookup,
    endpoint::presets, protocol::Router, tls::CaTlsConfig,
};
use iroh_gossip::{
    api::{Event, GossipSender},
    net::{GOSSIP_ALPN, Gossip},
};
use n0_future::{StreamExt, task::AbortOnDropHandle};
use papo::{
    node::{Node, NodeOptions},
    proto::{self, Frame, MAX_FRAME_BYTES, PeerKind, now_ms},
    room::RoomSecret,
    store::{Profile, Store},
};
use tempfile::TempDir;
use tokio::sync::mpsc;

pub const STEP: Duration = Duration::from_secs(20);

pub struct LocalNet {
    relay_map: RelayMap,
    relay_url: RelayUrl,
    lookup: MemoryLookup,
    pub room: RoomSecret,
    pub dir: TempDir,
    // Keeps the in-process relay alive for the test's lifetime.
    _relay: Box<dyn std::any::Any + Send>,
}

impl LocalNet {
    pub async fn new() -> Self {
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
    pub fn store(&self, name: &str) -> Store {
        let dir = self.dir.path().join(name);
        if let Ok(store) = Store::open_at(dir.clone()) {
            return store;
        }
        let profile =
            Profile { name: name.into(), about: None, room: self.room.to_base32(), created_ms: now_ms(), owner: false };
        Store::create_at(dir, &profile).unwrap()
    }

    pub async fn endpoint(&self, key: SecretKey) -> Endpoint {
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
    pub async fn agent(&self, name: &str, bootstrap: Vec<EndpointId>) -> Node {
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
    pub async fn cli(&self, name: &str, bootstrap: Vec<EndpointId>) -> Node {
        let ep = self.endpoint(SecretKey::generate()).await;
        let opts = NodeOptions { name: name.into(), about: None, kind: PeerKind::Human, ephemeral: true, bootstrap };
        Node::spawn(ep, self.room.clone(), opts, None).await.unwrap()
    }
}

pub async fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + STEP;
    while !check() {
        assert!(tokio::time::Instant::now() < deadline, "timed out waiting for: {what}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

pub enum RawEvent {
    NeighborUp,
    Frame(Frame),
}

/// A bare gossip participant that speaks the wire format directly, used to send what a
/// well-behaved node never would (duplicates, echoes, foreign or garbage frames) and to
/// observe exactly what a node puts on the wire.
pub struct RawPeer {
    pub id: EndpointId,
    room: RoomSecret,
    sender: GossipSender,
    events: mpsc::UnboundedReceiver<RawEvent>,
    _router: Router,
    _task: AbortOnDropHandle<()>,
}

impl RawPeer {
    pub async fn spawn(net: &LocalNet) -> Self {
        let endpoint = net.endpoint(SecretKey::generate()).await;
        let id = endpoint.id();
        let gossip = Gossip::builder().max_message_size(MAX_FRAME_BYTES).spawn(endpoint.clone());
        let router = Router::builder(endpoint).accept(GOSSIP_ALPN, gossip.clone()).spawn();
        let (sender, mut receiver) = gossip.subscribe(net.room.topic(), vec![]).await.unwrap().split();
        let (tx, events) = mpsc::unbounded_channel();
        let room = net.room.clone();
        let task = tokio::spawn(async move {
            while let Ok(Some(event)) = receiver.try_next().await {
                let raw = match event {
                    Event::NeighborUp(_) => RawEvent::NeighborUp,
                    Event::Received(msg) => match proto::decode(&room, &msg.content) {
                        Ok(frame) => RawEvent::Frame(frame),
                        Err(_) => continue,
                    },
                    _ => continue,
                };
                if tx.send(raw).is_err() {
                    break;
                }
            }
        });
        Self { id, room: net.room.clone(), sender, events, _router: router, _task: AbortOnDropHandle::new(task) }
    }

    /// Waits until a node (which has this peer as bootstrap) has connected.
    pub async fn wait_neighbor(&mut self) {
        tokio::time::timeout(STEP, async {
            loop {
                if let Some(RawEvent::NeighborUp) = self.events.recv().await {
                    return;
                }
            }
        })
        .await
        .expect("no node connected to the raw peer");
    }

    pub async fn send(&self, frame: &Frame) {
        self.send_bytes(proto::encode(&self.room, frame).unwrap()).await;
    }

    pub async fn send_bytes(&self, bytes: Bytes) {
        self.sender.broadcast(bytes).await.unwrap();
    }

    /// Every frame received within `window`.
    pub async fn frames_within(&mut self, window: Duration) -> Vec<Frame> {
        let mut frames = vec![];
        let deadline = tokio::time::Instant::now() + window;
        while let Ok(Some(event)) = tokio::time::timeout_at(deadline, self.events.recv()).await {
            if let RawEvent::Frame(frame) = event {
                frames.push(frame);
            }
        }
        frames
    }

    /// Waits for the first ack of `id`, returning who sent it.
    pub async fn ack_for(&mut self, id: &str) -> Option<String> {
        tokio::time::timeout(STEP, async {
            loop {
                match self.events.recv().await {
                    Some(RawEvent::Frame(Frame::Ack { id: acked, by })) if acked == id => return Some(by),
                    Some(_) => continue,
                    None => return None,
                }
            }
        })
        .await
        .ok()
        .flatten()
    }
}
