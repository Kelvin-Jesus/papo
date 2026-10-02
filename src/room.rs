//! Room secret, invite codes and frame sealing.
//!
//! A room is defined by a single 32-byte secret. Everything else is derived from it:
//! the gossip topic peers meet on, the symmetric key every frame is sealed with, and a
//! short non-secret id used for display. Knowing the secret is what makes someone a
//! member, so the invite code that carries it must be shared privately.

use std::fmt;

use anyhow::{Context, Result, bail, ensure};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use data_encoding::BASE32_NOPAD;
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;

const INVITE_PREFIX: &str = "papo1";
/// Bound to every sealed frame so ciphertext from another protocol version never parses.
const AAD: &[u8] = b"papo/v1";
const NONCE_LEN: usize = 24;
/// Invites stay pasteable in a chat message; a handful of entry points is plenty.
const MAX_INVITE_PEERS: usize = 4;
/// Trailing integrity check. Without it, a copy-paste cut at a 32-byte boundary decoded
/// as a valid invite with fewer entry points, silently.
const INVITE_CHECK_LEN: usize = 4;

#[derive(Clone, PartialEq, Eq)]
pub struct RoomSecret([u8; 32]);

impl RoomSecret {
    pub fn generate() -> Self {
        Self(rand::random())
    }

    pub fn to_base32(&self) -> String {
        BASE32_NOPAD.encode(&self.0).to_ascii_lowercase()
    }

    pub fn from_base32(s: &str) -> Result<Self> {
        let bytes =
            BASE32_NOPAD.decode(s.trim().to_ascii_uppercase().as_bytes()).context("room secret is not valid base32")?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("room secret must be 32 bytes"))?;
        Ok(Self(arr))
    }

    /// Gossip topic. Derived (not the raw secret) because peers exchange the topic id
    /// in cleartext inside the QUIC session during the join handshake.
    pub fn topic(&self) -> TopicId {
        TopicId::from_bytes(blake3::derive_key("papo v1 gossip topic", &self.0))
    }

    /// Short, non-secret label for humans and directory names.
    pub fn room_id(&self) -> String {
        let id = blake3::derive_key("papo v1 room id", &self.0);
        data_encoding::HEXLOWER.encode(&id[..4])
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        let key = blake3::derive_key("papo v1 frame key", &self.0);
        XChaCha20Poly1305::new(&key.into())
    }

    /// Encrypts and authenticates a frame. Gossip forwards frames through any room member,
    /// and this is what guarantees that only holders of the room secret can read or
    /// inject messages, independent of transport-level encryption.
    pub fn seal(&self, plaintext: &[u8]) -> Vec<u8> {
        let nonce_bytes: [u8; NONCE_LEN] = rand::random();
        let nonce = XNonce::from(nonce_bytes);
        let ciphertext = self
            .cipher()
            .encrypt(&nonce, Payload { msg: plaintext, aad: AAD })
            .expect("XChaCha20Poly1305 encryption cannot fail for in-memory buffers");
        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        out
    }

    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>> {
        ensure!(sealed.len() > NONCE_LEN, "sealed frame too short");
        let (nonce_bytes, ciphertext) = sealed.split_at(NONCE_LEN);
        let nonce_arr: [u8; NONCE_LEN] = nonce_bytes.try_into().expect("split_at guarantees length");
        self.cipher()
            .decrypt(&XNonce::from(nonce_arr), Payload { msg: ciphertext, aad: AAD })
            .map_err(|_| anyhow::anyhow!("frame failed authentication (wrong room or tampered)"))
    }
}

// Never print the secret by accident (logs, panics, debug output).
impl fmt::Debug for RoomSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RoomSecret({})", self.room_id())
    }
}

/// What a member hands to a newcomer: the room secret plus a few endpoint ids that can
/// be dialed to enter the gossip swarm. Endpoint ids are enough because iroh resolves
/// them to current addresses through its address lookup (DNS/pkarr) and relays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
    pub secret: RoomSecret,
    pub peers: Vec<EndpointId>,
}

impl Invite {
    pub fn encode(&self) -> String {
        let mut bytes = Vec::with_capacity(32 + 32 * self.peers.len() + INVITE_CHECK_LEN);
        bytes.extend_from_slice(&self.secret.0);
        for peer in self.peers.iter().take(MAX_INVITE_PEERS) {
            bytes.extend_from_slice(peer.as_bytes());
        }
        let check = invite_check(&bytes);
        bytes.extend_from_slice(&check);
        format!("{INVITE_PREFIX}{}", BASE32_NOPAD.encode(&bytes).to_ascii_lowercase())
    }

    pub fn decode(code: &str) -> Result<Self> {
        let code = code.trim();
        let Some(body) = code.strip_prefix(INVITE_PREFIX) else {
            bail!("invite must start with '{INVITE_PREFIX}'");
        };
        let bytes = BASE32_NOPAD
            .decode(body.to_ascii_uppercase().as_bytes())
            .context("invite is not valid base32 (was it truncated when copying?)")?;
        ensure!(
            bytes.len() >= 32 + INVITE_CHECK_LEN && (bytes.len() - INVITE_CHECK_LEN) % 32 == 0,
            "invite has an invalid length (was it truncated when copying?)"
        );
        let (payload, check) = bytes.split_at(bytes.len() - INVITE_CHECK_LEN);
        ensure!(
            invite_check(payload) == check,
            "invite is damaged or incomplete (was it truncated or mistyped when copying?)"
        );
        let secret = RoomSecret(payload[..32].try_into().expect("checked length"));
        let peers = payload[32..]
            .as_chunks::<32>()
            .0
            .iter()
            .map(|arr| EndpointId::from_bytes(arr).context("invite contains an invalid endpoint id"))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { secret, peers })
    }
}

fn invite_check(payload: &[u8]) -> [u8; INVITE_CHECK_LEN] {
    let digest = blake3::derive_key("papo v1 invite check", payload);
    digest[..INVITE_CHECK_LEN].try_into().expect("digest is 32 bytes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh::SecretKey;

    #[test]
    fn invite_roundtrip_preserves_secret_and_peers() {
        let peers = vec![SecretKey::generate().public(), SecretKey::generate().public()];
        let invite = Invite { secret: RoomSecret::generate(), peers };
        let decoded = Invite::decode(&invite.encode()).unwrap();
        assert_eq!(decoded, invite);
    }

    #[test]
    fn invite_tolerates_whitespace_and_case_from_chat_apps() {
        let invite = Invite { secret: RoomSecret::generate(), peers: vec![SecretKey::generate().public()] };
        let mangled = format!("  {}\n", invite.encode().to_ascii_uppercase().replacen("PAPO1", "papo1", 1));
        assert_eq!(Invite::decode(&mangled).unwrap(), invite);
    }

    #[test]
    fn truncated_invite_is_rejected() {
        let invite = Invite { secret: RoomSecret::generate(), peers: vec![SecretKey::generate().public()] };
        let code = invite.encode();
        assert!(Invite::decode(&code[..code.len() - 5]).is_err());
        assert!(Invite::decode("hello").is_err());
    }

    #[test]
    fn invite_cut_at_an_entry_point_boundary_is_rejected() {
        let peers: Vec<_> = (0..3).map(|_| SecretKey::generate().public()).collect();
        let full = Invite { secret: RoomSecret::generate(), peers: peers.clone() };
        // Exactly what one fewer peer encodes to, minus its checksum: base32 of 96 bytes.
        let shorter = Invite { secret: full.secret.clone(), peers: peers[..2].to_vec() }.encode();
        let code = full.encode();
        for len in [shorter.len(), shorter.len() - 7, INVITE_PREFIX.len() + 52, INVITE_PREFIX.len() + 154] {
            assert!(Invite::decode(&code[..len]).is_err(), "accepted {len}-char prefix");
        }
    }

    #[test]
    fn mistyped_invite_is_rejected() {
        let code = Invite { secret: RoomSecret::generate(), peers: vec![SecretKey::generate().public()] }.encode();
        let mut chars: Vec<char> = code.chars().collect();
        let i = INVITE_PREFIX.len() + 10;
        chars[i] = if chars[i] == 'a' { 'b' } else { 'a' };
        let err = Invite::decode(&chars.into_iter().collect::<String>()).unwrap_err();
        assert!(err.to_string().contains("damaged"), "{err}");
    }

    #[test]
    fn sealed_frames_only_open_with_the_same_room() {
        let room = RoomSecret::generate();
        let other = RoomSecret::generate();
        let sealed = room.seal(b"hi peer");
        assert_eq!(room.open(&sealed).unwrap(), b"hi peer");
        assert!(other.open(&sealed).is_err());
    }

    #[test]
    fn tampered_frames_are_rejected() {
        let room = RoomSecret::generate();
        let mut sealed = room.seal(b"hi peer");
        let last = sealed.len() - 1;
        sealed[last] ^= 0x01;
        assert!(room.open(&sealed).is_err());
        assert!(room.open(&[0u8; 10]).is_err());
    }

    #[test]
    fn derived_values_are_stable_and_distinct_per_room() {
        let room = RoomSecret::generate();
        let same = RoomSecret::from_base32(&room.to_base32()).unwrap();
        assert_eq!(room.topic(), same.topic());
        assert_eq!(room.room_id(), same.room_id());
        assert_ne!(room.topic(), RoomSecret::generate().topic());
        assert_eq!(room.room_id().len(), 8);
    }
}
