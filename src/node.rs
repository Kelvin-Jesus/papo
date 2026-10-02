//! A room member: joins the gossip topic, delivers messages with acknowledgements and
//! keeps an inbox/outbox so nothing is lost while a peer is offline.
//!
//! Delivery model (store-and-forward, at-least-once):
//! - `send` puts the message in the outbox *before* broadcasting, so a crash or an
//!   offline peer never loses it.
//! - Receivers persist to their inbox, then broadcast an `Ack`; the sender drops the
//!   message from its outbox on the first ack (from the addressee when `to` is set).
//! - The outbox is re-broadcast whenever a neighbor connects and periodically while
//!   connected. Receivers dedupe by message id and re-ack duplicates, so a lost ack
//!   heals itself.
//!
//! Connectivity: we dial room members ourselves instead of handing them to gossip as
//! bootstrap peers. iroh-gossip (0.101) dials a bootstrap peer once; if that fails (for
//! instance because the peer has not published its address yet, a ~1s window after it
//! starts) the peer stays "pending" forever and later joins never redial. A connection
//! that gossip *receives* clears that state, and the gossip wire protocol is symmetric,
//! so we dial with the gossip ALPN, give the connection to gossip as if it were
//! incoming, and then ask it to join that peer. Retried every tick while alone.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use iroh::{Endpoint, EndpointId, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender},
    net::{GOSSIP_ALPN, Gossip},
};
use n0_future::{StreamExt, join_all, task::AbortOnDropHandle};
use serde::Serialize;
use tokio::sync::{Notify, broadcast, oneshot};

use crate::{
    proto::{self, Envelope, Frame, MAX_BODY_BYTES, MAX_FRAME_BYTES, PeerKind, Presence, now_ms, validate_name},
    room::RoomSecret,
    store::{KnownPeer, LogEntry, Store},
};

/// Granularity of the maintenance loop.
const TICK: Duration = Duration::from_secs(1);
/// Reconnect backoff while alone. Starts short because the most common failure is
/// dialing a peer in the second after it starts, before its address is published.
const MIN_REDIAL: Duration = Duration::from_secs(1);
const MAX_REDIAL: Duration = Duration::from_secs(10);
/// Covers address lookup plus hole punching; an offline peer usually fails much faster.
const DIAL_TIMEOUT: Duration = Duration::from_secs(15);
/// Presence heartbeat and outbox re-broadcast cadence while connected.
const HEARTBEAT: Duration = Duration::from_secs(30);
/// A peer counts as online if we heard from it within this window (> 2 heartbeats).
const ONLINE_WINDOW_MS: u64 = 75_000;

#[derive(Debug, Clone)]
pub struct NodeOptions {
    pub name: String,
    pub about: Option<String>,
    pub kind: PeerKind,
    /// Send-only identity for one-shot CLI commands: it neither stores nor acks incoming
    /// messages (that is the job of the long-running agent node) and peers do not
    /// remember it.
    pub ephemeral: bool,
    /// Extra endpoints to dial on startup (e.g. the ones inside an invite).
    pub bootstrap: Vec<EndpointId>,
}

#[derive(Debug, Clone)]
pub enum NodeEvent {
    Message(Envelope),
    Delivered { id: String, by: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOutcome {
    Delivered {
        id: String,
        by: String,
    },
    /// Nobody acked in time; the message stays in the outbox and is retried.
    Queued {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct PeerView {
    pub node: EndpointId,
    pub name: Option<String>,
    pub about: Option<String>,
    pub kind: Option<PeerKind>,
    pub ephemeral: bool,
    /// Directly connected in the gossip mesh right now.
    pub neighbor: bool,
    pub online: bool,
    pub last_seen_ms: u64,
}

struct PeerState {
    presence: Presence,
    last_seen_ms: u64,
}

#[derive(Default)]
struct State {
    peers: HashMap<EndpointId, PeerState>,
    neighbors: HashSet<EndpointId>,
    known: BTreeMap<EndpointId, KnownPeer>,
    inbox: Vec<Envelope>,
    outbox: Vec<Envelope>,
    seen: HashSet<String>,
    ack_waiters: HashMap<String, oneshot::Sender<String>>,
}

struct Inner {
    room: RoomSecret,
    me: EndpointId,
    endpoint: Endpoint,
    gossip: Gossip,
    opts: NodeOptions,
    store: Option<Store>,
    sender: GossipSender,
    bootstrap: Vec<EndpointId>,
    state: Mutex<State>,
    events: broadcast::Sender<NodeEvent>,
    inbox_changed: Notify,
    neighbors_changed: Notify,
    /// Cleared if the gossip subscription dies; nothing arrives after that.
    subscribed: AtomicBool,
    /// Set by a graceful `shutdown`: the subscription ending and broadcasts failing are
    /// then expected, not something to warn the user about.
    closing: AtomicBool,
}

pub struct Node {
    inner: Arc<Inner>,
    router: Router,
    _tasks: Vec<AbortOnDropHandle<()>>,
}

impl Node {
    /// Joins the room on an already bound endpoint. The caller picks how the endpoint
    /// reaches the network (public n0 infrastructure in production, a local relay in tests).
    pub async fn spawn(endpoint: Endpoint, room: RoomSecret, opts: NodeOptions, store: Option<Store>) -> Result<Self> {
        validate_name(&opts.name)?;
        let me = endpoint.id();
        let gossip = Gossip::builder().max_message_size(MAX_FRAME_BYTES).spawn(endpoint.clone());
        let router = Router::builder(endpoint.clone()).accept(GOSSIP_ALPN, gossip.clone()).spawn();

        let mut state = State::default();
        if let Some(store) = &store {
            state.known = store.known_peers()?;
            state.inbox = store.inbox()?;
            state.outbox = store.outbox()?;
            // Remember ids across restarts so a peer re-sending its outbox does not
            // resurrect messages the agent already consumed.
            state.seen = store
                .read_log()?
                .into_iter()
                .filter_map(|e| match e {
                    LogEntry::In { msg } => Some(msg.id),
                    _ => None,
                })
                .collect();
            // The inbox is written before the log; a crash in between must not turn a
            // redelivery into a duplicate.
            state.seen.extend(state.inbox.iter().map(|m| m.id.clone()));
        }

        let mut bootstrap: Vec<EndpointId> = opts.bootstrap.clone();
        bootstrap.extend(state.known.keys().copied());
        bootstrap.retain(|id| *id != me);
        bootstrap.sort();
        bootstrap.dedup();

        // No bootstrap peers here on purpose: the maintenance loop dials them (see the
        // module docs for why gossip must not dial them itself).
        let topic = gossip.subscribe(room.topic(), vec![]).await.context("join gossip topic")?;
        let (sender, receiver) = topic.split();

        let (events, _) = broadcast::channel(256);
        let inner = Arc::new(Inner {
            room,
            me,
            endpoint,
            gossip,
            opts,
            store,
            sender,
            bootstrap,
            state: Mutex::new(state),
            events,
            inbox_changed: Notify::new(),
            neighbors_changed: Notify::new(),
            subscribed: AtomicBool::new(true),
            closing: AtomicBool::new(false),
        });

        let tasks = vec![
            AbortOnDropHandle::new(tokio::spawn(event_loop(inner.clone(), receiver))),
            AbortOnDropHandle::new(tokio::spawn(maintenance_loop(inner.clone()))),
        ];
        Ok(Self { inner, router, _tasks: tasks })
    }

    pub fn id(&self) -> EndpointId {
        self.inner.me
    }

    pub fn name(&self) -> &str {
        &self.inner.opts.name
    }

    pub fn room_id(&self) -> String {
        self.inner.room.room_id()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.inner.events.subscribe()
    }

    /// Broadcasts a message and waits up to `ack_timeout` for a receipt. A message that
    /// is not acked in time is *not* an error: it stays queued and is retried until a
    /// peer comes online.
    pub async fn send(
        &self,
        body: &str,
        to: Option<String>,
        reply_to: Option<String>,
        ack_timeout: Duration,
    ) -> Result<SendOutcome> {
        let body = body.trim();
        ensure!(!body.is_empty(), "message is empty");
        ensure!(
            body.len() <= MAX_BODY_BYTES,
            "message is {} bytes; the limit is {MAX_BODY_BYTES}. Summarize or split it.",
            body.len()
        );
        if let Some(to) = &to {
            validate_name(to)?;
        }
        let env = Envelope {
            id: proto::new_msg_id(),
            from: self.inner.opts.name.clone(),
            node: self.inner.me.to_string(),
            kind: self.inner.opts.kind,
            to,
            reply_to: reply_to.clone(),
            ts: now_ms(),
            body: body.to_string(),
        };
        // Encode first so an oversized frame fails before touching the outbox.
        let bytes = proto::encode(&self.inner.room, &Frame::Msg(env.clone()))?;

        let (tx, rx) = oneshot::channel();
        {
            let mut state = self.inner.lock();
            state.outbox.push(env.clone());
            self.inner.persist_outbox(&state);
            state.ack_waiters.insert(env.id.clone(), tx);
            if let Some(reply_to) = &reply_to {
                self.inner.mark_read_through(&mut state, reply_to);
            }
        }
        self.inner.log(LogEntry::Out { msg: env.clone() });

        if let Err(e) = self.inner.sender.broadcast(bytes).await {
            // Still queued: the outbox flush will retry once the topic is healthy.
            eprintln!("papo: broadcast failed, message stays queued: {e}");
        }

        match tokio::time::timeout(ack_timeout, rx).await {
            Ok(Ok(by)) => Ok(SendOutcome::Delivered { id: env.id, by }),
            _ => {
                self.inner.lock().ack_waiters.remove(&env.id);
                Ok(SendOutcome::Queued { id: env.id })
            }
        }
    }

    /// Removes and returns unread messages, optionally only those from one sender.
    pub fn take_unread(&self, from: Option<&str>) -> Vec<Envelope> {
        let mut state = self.inner.lock();
        let (taken, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut state.inbox)
            .into_iter()
            .partition(|m| from.is_none_or(|f| m.from.eq_ignore_ascii_case(f)));
        state.inbox = kept;
        if !taken.is_empty() {
            self.inner.persist_inbox(&state);
        }
        taken
    }

    pub fn unread(&self) -> Vec<Envelope> {
        self.inner.lock().inbox.clone()
    }

    pub fn pending(&self) -> Vec<Envelope> {
        self.inner.lock().outbox.clone()
    }

    /// Blocks until at least one unread message (matching `from`) exists or the timeout
    /// elapses. Returns the messages and marks them read.
    pub async fn wait(&self, timeout: Duration, from: Option<&str>) -> Vec<Envelope> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            // Register interest before checking, otherwise a message landing between the
            // check and the await would be missed until the next one arrives.
            let notified = self.inner.inbox_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let msgs = self.take_unread(from);
            if !msgs.is_empty() {
                return msgs;
            }
            tokio::select! {
                _ = &mut notified => continue,
                _ = tokio::time::sleep_until(deadline) => return vec![],
            }
        }
    }

    /// Waits until the gossip mesh has at least one direct neighbor.
    pub async fn wait_for_neighbor(&self, timeout: Duration) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let notified = self.inner.neighbors_changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !self.inner.lock().neighbors.is_empty() {
                return true;
            }
            tokio::select! {
                _ = &mut notified => continue,
                _ = tokio::time::sleep_until(deadline) => return false,
            }
        }
    }

    /// Everyone we know about: remembered room members plus whoever announced
    /// themselves during this session.
    pub fn peers(&self) -> Vec<PeerView> {
        let state = self.inner.lock();
        let now = now_ms();
        let mut views: BTreeMap<EndpointId, PeerView> = state
            .known
            .iter()
            .map(|(id, k)| {
                (
                    *id,
                    PeerView {
                        node: *id,
                        name: k.name.clone(),
                        about: None,
                        kind: None,
                        ephemeral: false,
                        neighbor: false,
                        online: false,
                        last_seen_ms: k.last_seen_ms,
                    },
                )
            })
            .collect();
        for (id, p) in &state.peers {
            let view = views.entry(*id).or_insert_with(|| PeerView {
                node: *id,
                name: None,
                about: None,
                kind: None,
                ephemeral: false,
                neighbor: false,
                online: false,
                last_seen_ms: 0,
            });
            view.name = Some(p.presence.name.clone());
            view.about = p.presence.about.clone();
            view.kind = Some(p.presence.kind);
            view.ephemeral = p.presence.ephemeral;
            view.last_seen_ms = view.last_seen_ms.max(p.last_seen_ms);
        }
        for view in views.values_mut() {
            view.neighbor = state.neighbors.contains(&view.node);
            view.online = view.neighbor || now.saturating_sub(view.last_seen_ms) < ONLINE_WINDOW_MS;
        }
        views.into_values().collect()
    }

    /// False once the gossip subscription has ended: the node can no longer receive and
    /// the process must be restarted.
    pub fn is_healthy(&self) -> bool {
        self.inner.subscribed.load(Ordering::Relaxed)
    }

    pub fn neighbor_count(&self) -> usize {
        self.inner.lock().neighbors.len()
    }

    pub async fn shutdown(&self) {
        // Router shutdown closes the endpoint, which lets peers see us leave promptly
        // instead of waiting for a QUIC idle timeout.
        self.inner.closing.store(true, Ordering::Relaxed);
        let _ = self.router.shutdown().await;
    }
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        // A panic while holding the lock leaves plain data behind; keep serving.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn presence(&self) -> Presence {
        Presence {
            node: self.me.to_string(),
            name: self.opts.name.clone(),
            about: self.opts.about.clone(),
            kind: self.opts.kind,
            ephemeral: self.opts.ephemeral,
        }
    }

    async fn broadcast(&self, frame: &Frame) {
        let bytes = match proto::encode(&self.room, frame) {
            Ok(b) => b,
            Err(e) => return eprintln!("papo: could not encode frame: {e:#}"),
        };
        if let Err(e) = self.sender.broadcast(bytes).await
            && !self.closing.load(Ordering::Relaxed)
        {
            eprintln!("papo: broadcast failed: {e}");
        }
    }

    async fn flush_outbox(&self) {
        let pending = self.lock().outbox.clone();
        for env in pending {
            self.broadcast(&Frame::Msg(env)).await;
        }
    }

    /// Replying to a message implies the agent saw it and everything that peer sent
    /// before it (channel pushes put them in context without going through `wait`).
    fn mark_read_through(&self, state: &mut State, reply_to: &str) {
        let Some(pos) = state.inbox.iter().position(|m| m.id == reply_to) else {
            return;
        };
        let sender = state.inbox[pos].node.clone();
        let mut idx = 0;
        state.inbox.retain(|m| {
            let keep = !(idx <= pos && m.node == sender);
            idx += 1;
            keep
        });
        self.persist_inbox(state);
    }

    async fn handle_event(&self, event: Event) {
        match event {
            Event::NeighborUp(id) => {
                self.lock().neighbors.insert(id);
                self.neighbors_changed.notify_waiters();
                // Introduce ourselves and hand over anything that waited for this peer.
                self.broadcast(&Frame::Hello(self.presence())).await;
                self.flush_outbox().await;
            }
            Event::NeighborDown(id) => {
                self.lock().neighbors.remove(&id);
                self.neighbors_changed.notify_waiters();
            }
            Event::Received(msg) => match proto::decode(&self.room, &msg.content) {
                Ok(frame) => self.handle_frame(frame).await,
                Err(e) => eprintln!("papo: dropped frame relayed by {}: {e:#}", msg.delivered_from.fmt_short()),
            },
            Event::Lagged => eprintln!("papo: gossip receiver lagged; some frames were skipped"),
        }
    }

    async fn handle_frame(&self, frame: Frame) {
        match frame {
            Frame::Hello(presence) => self.handle_hello(presence).await,
            Frame::Msg(env) => self.handle_msg(env).await,
            Frame::Ack { id, by } => self.handle_ack(id, by),
        }
    }

    async fn handle_hello(&self, presence: Presence) {
        let Ok(node) = presence.node.parse::<EndpointId>() else {
            return;
        };
        if node == self.me || validate_name(&presence.name).is_err() {
            return;
        }
        let now = now_ms();
        let first_time = {
            let mut state = self.lock();
            let first_time = !state.peers.contains_key(&node);
            if !presence.ephemeral && !self.opts.ephemeral {
                let entry = state.known.entry(node).or_default();
                let changed = entry.name.as_deref() != Some(presence.name.as_str());
                entry.name = Some(presence.name.clone());
                entry.last_seen_ms = now;
                // Persist only on change; heartbeats would otherwise rewrite the file
                // every few seconds.
                if changed || first_time {
                    self.persist_known(&state);
                }
            }
            state.peers.insert(node, PeerState { presence, last_seen_ms: now });
            first_time
        };
        if first_time {
            // Members reached through other members (not direct neighbors) never get our
            // NeighborUp hello, so answer the first hello we see from anyone.
            self.broadcast(&Frame::Hello(self.presence())).await;
        }
    }

    async fn handle_msg(&self, env: Envelope) {
        if self.opts.ephemeral || env.node == self.me.to_string() || !env.is_for(&self.opts.name) {
            return;
        }
        let is_new = {
            let mut state = self.lock();
            if let Ok(node) = env.node.parse::<EndpointId>()
                && let Some(peer) = state.peers.get_mut(&node)
            {
                peer.last_seen_ms = now_ms();
            }
            let is_new = state.seen.insert(env.id.clone());
            if is_new {
                state.inbox.push(env.clone());
                self.persist_inbox(&state);
            }
            is_new
        };
        // Persist before acking: an ack promises the message is safe on our side.
        if is_new {
            self.log(LogEntry::In { msg: env.clone() });
        }
        self.broadcast(&Frame::Ack { id: env.id.clone(), by: self.opts.name.clone() }).await;
        if is_new {
            self.inbox_changed.notify_waiters();
            let _ = self.events.send(NodeEvent::Message(env));
        }
    }

    fn handle_ack(&self, id: String, by: String) {
        let (removed, waiter) = {
            let mut state = self.lock();
            let before = state.outbox.len();
            state.outbox.retain(|m| m.id != id);
            let removed = state.outbox.len() != before;
            if removed {
                self.persist_outbox(&state);
            }
            (removed, state.ack_waiters.remove(&id))
        };
        if let Some(waiter) = waiter {
            let _ = waiter.send(by.clone());
        }
        if removed {
            self.log(LogEntry::Delivered { id: id.clone(), by: by.clone(), ts: now_ms() });
            let _ = self.events.send(NodeEvent::Delivered { id, by });
        }
    }

    fn rejoin_candidates(&self) -> Vec<EndpointId> {
        let state = self.lock();
        let mut ids: Vec<EndpointId> = self.bootstrap.iter().chain(state.known.keys()).copied().collect();
        ids.retain(|id| *id != self.me);
        ids.sort();
        ids.dedup();
        ids
    }

    fn persist_inbox(&self, state: &State) {
        if let Some(store) = &self.store
            && let Err(e) = store.save_inbox(&state.inbox)
        {
            eprintln!("papo: could not save inbox: {e:#}");
        }
    }

    fn persist_outbox(&self, state: &State) {
        if let Some(store) = &self.store
            && let Err(e) = store.save_outbox(&state.outbox)
        {
            eprintln!("papo: could not save outbox: {e:#}");
        }
    }

    fn persist_known(&self, state: &State) {
        if let Some(store) = &self.store
            && let Err(e) = store.save_known_peers(&state.known)
        {
            eprintln!("papo: could not save peers: {e:#}");
        }
    }

    fn log(&self, entry: LogEntry) {
        if let Some(store) = &self.store
            && let Err(e) = store.append_log(&entry)
        {
            eprintln!("papo: could not append to log: {e:#}");
        }
    }
}

async fn event_loop(inner: Arc<Inner>, mut receiver: GossipReceiver) {
    loop {
        match receiver.try_next().await {
            Ok(Some(event)) => inner.handle_event(event).await,
            Ok(None) => {
                if !inner.closing.load(Ordering::Relaxed) {
                    eprintln!("papo: gossip subscription closed");
                }
                break;
            }
            Err(e) => {
                if !inner.closing.load(Ordering::Relaxed) {
                    eprintln!("papo: gossip subscription ended: {e}");
                }
                break;
            }
        }
    }
    inner.subscribed.store(false, Ordering::Relaxed);
}

async fn maintenance_loop(inner: Arc<Inner>) {
    let mut interval = tokio::time::interval(TICK);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut redial = MIN_REDIAL;
    let mut next_dial = tokio::time::Instant::now();
    let mut next_heartbeat = tokio::time::Instant::now() + HEARTBEAT;
    loop {
        interval.tick().await;
        if inner.closing.load(Ordering::Relaxed) {
            return;
        }
        let now = tokio::time::Instant::now();
        let alone = inner.lock().neighbors.is_empty();
        if alone {
            if now >= next_dial {
                // Members that were offline (or not yet resolvable) at startup only
                // become reachable if we keep trying.
                let candidates = inner.rejoin_candidates();
                join_all(candidates.into_iter().map(|peer| connect_peer(&inner, peer))).await;
                next_dial = tokio::time::Instant::now() + redial;
                redial = (redial * 2).min(MAX_REDIAL);
            }
        } else {
            redial = MIN_REDIAL;
            if now >= next_heartbeat {
                next_heartbeat = now + HEARTBEAT;
                inner.broadcast(&Frame::Hello(inner.presence())).await;
                inner.flush_outbox().await;
            }
        }
    }
}

async fn connect_peer(inner: &Inner, peer: EndpointId) {
    let conn = match tokio::time::timeout(DIAL_TIMEOUT, inner.endpoint.connect(peer, GOSSIP_ALPN)).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => return tracing::debug!(peer = %peer.fmt_short(), "dial failed: {e}"),
        Err(_) => return tracing::debug!(peer = %peer.fmt_short(), "dial timed out"),
    };
    tracing::debug!(peer = %peer.fmt_short(), "connected; handing connection to gossip");
    if let Err(e) = inner.gossip.handle_connection(conn).await {
        return eprintln!("papo: gossip refused connection to {}: {e}", peer.fmt_short());
    }
    if let Err(e) = inner.sender.join_peers(vec![peer]).await {
        eprintln!("papo: could not join {}: {e}", peer.fmt_short());
    }
}
