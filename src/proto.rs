//! Wire frames exchanged inside a room.
//!
//! Frames are JSON (easy to debug, schema-tolerant across versions) sealed with the room
//! key before they hit the gossip layer. Every frame is broadcast to the whole room;
//! addressing (`to`) is applied by receivers.
//!
//! ```
//! use papo::{proto::{self, Envelope, Frame, PeerKind}, room::RoomSecret};
//!
//! let room = RoomSecret::generate();
//! let msg = Envelope {
//!     id: proto::new_msg_id(),
//!     from: "voce".into(),
//!     node: "endpoint-id".into(),
//!     kind: PeerKind::Agent,
//!     to: Some("colega".into()),
//!     reply_to: None,
//!     ts: proto::now_ms(),
//!     body: "o handler aceita X-Signature?".into(),
//! };
//! assert!(msg.is_for("Colega") && !msg.is_for("terceiro"));
//!
//! let bytes = proto::encode(&room, &Frame::Msg(msg.clone())).unwrap();
//! assert_eq!(proto::decode(&room, &bytes).unwrap(), Frame::Msg(msg));
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::room::RoomSecret;

/// Upper bound for a message body. Agents share code snippets and logs, so the gossip
/// default (4 KiB) is far too small, but whole files should be summarized, not pasted.
pub const MAX_BODY_BYTES: usize = 48 * 1024;
/// Gossip-level limit: body + JSON envelope + nonce/tag overhead, with headroom.
pub const MAX_FRAME_BYTES: usize = 64 * 1024;
pub const MAX_NAME_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerKind {
    /// An AI agent behind the MCP server.
    Agent,
    /// A person typing through the CLI (`papo say`).
    Human,
}

impl PeerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PeerKind::Agent => "agent",
            PeerKind::Human => "human",
        }
    }
}

/// Announces who sits behind an endpoint. Sent when a neighbor connects and as a
/// periodic heartbeat, so `online` can be derived from recency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub node: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    pub kind: PeerKind,
    /// Short-lived CLI identities: never persisted as known peers and never redialed.
    #[serde(default)]
    pub ephemeral: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub id: String,
    pub from: String,
    pub node: String,
    pub kind: PeerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    /// Sender's wall clock in unix milliseconds; informational only (clocks drift).
    pub ts: u64,
    pub body: String,
}

impl Envelope {
    pub fn is_for(&self, name: &str) -> bool {
        self.to.as_deref().is_none_or(|to| to.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Frame {
    Hello(Presence),
    Msg(Envelope),
    /// Receipt so the sender can drop the message from its outbox.
    Ack {
        id: String,
        by: String,
    },
}

pub fn encode(room: &RoomSecret, frame: &Frame) -> Result<Bytes> {
    let json = serde_json::to_vec(frame).context("serialize frame")?;
    let sealed = room.seal(&json);
    ensure!(sealed.len() <= MAX_FRAME_BYTES, "frame too large ({} bytes)", sealed.len());
    Ok(Bytes::from(sealed))
}

pub fn decode(room: &RoomSecret, bytes: &[u8]) -> Result<Frame> {
    let json = room.open(bytes)?;
    serde_json::from_slice(&json).context("frame is not valid papo JSON")
}

/// 10 hex chars: short enough for an agent to copy into `reply_to` without mistakes,
/// long enough that collisions inside one conversation are negligible.
pub fn new_msg_id() -> String {
    let bytes: [u8; 5] = rand::random();
    data_encoding::HEXLOWER.encode(&bytes)
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or_default()
}

/// Names travel in every frame and are used for addressing, so keep them simple.
///
/// ```
/// use papo::proto::validate_name;
///
/// assert!(validate_name("joão_2").is_ok());
/// assert!(validate_name("com espaço").is_err());
/// assert!(validate_name("").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "name cannot be empty");
    ensure!(name.len() <= MAX_NAME_LEN, "name must be at most {MAX_NAME_LEN} characters");
    ensure!(
        name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.'),
        "name may only contain letters, digits, '-', '_' and '.'"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Envelope {
        Envelope {
            id: new_msg_id(),
            from: "kj".into(),
            node: "abc".into(),
            kind: PeerKind::Agent,
            to: None,
            reply_to: None,
            ts: now_ms(),
            body: "qual o formato do payload do webhook?".into(),
        }
    }

    #[test]
    fn frames_roundtrip_through_seal() {
        let room = RoomSecret::generate();
        let frame = Frame::Msg(sample());
        let bytes = encode(&room, &frame).unwrap();
        assert_eq!(decode(&room, &bytes).unwrap(), frame);
    }

    #[test]
    fn size_limits_match_what_agents_and_docs_promise() {
        // The send tool description and the docs promise about 48 KB per message.
        assert_eq!(MAX_BODY_BYTES, 48 * 1024);
        assert_eq!(MAX_FRAME_BYTES, 64 * 1024);
        // A plain-text body at the limit still fits in one frame with its envelope.
        let mut env = sample();
        env.body = "a".repeat(MAX_BODY_BYTES);
        assert!(encode(&RoomSecret::generate(), &Frame::Msg(env)).is_ok());
    }

    #[test]
    fn oversized_frames_are_refused_before_sending() {
        let room = RoomSecret::generate();
        let mut env = sample();
        env.body = "x".repeat(MAX_FRAME_BYTES);
        assert!(encode(&room, &Frame::Msg(env)).is_err());
    }

    #[test]
    fn addressing_is_case_insensitive_and_broadcast_reaches_everyone() {
        let mut env = sample();
        assert!(env.is_for("anyone"));
        env.to = Some("Voce".into());
        assert!(env.is_for("voce"));
        assert!(!env.is_for("colega"));
    }

    #[test]
    fn names_are_validated() {
        assert!(validate_name("kj").is_ok());
        assert!(validate_name("joão_2").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name(&"a".repeat(40)).is_err());
    }
}
