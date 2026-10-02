//! Property-based tests: invariants that must hold for every input, not just for the
//! examples in the unit tests.

use data_encoding::BASE32_NOPAD;
use iroh::{EndpointId, SecretKey};
use papo::{
    proto::{self, Envelope, Frame, MAX_FRAME_BYTES, MAX_NAME_LEN, PeerKind, Presence, validate_name},
    room::{Invite, RoomSecret},
};
use proptest::prelude::*;

fn room_from(bytes: [u8; 32]) -> RoomSecret {
    RoomSecret::from_base32(&BASE32_NOPAD.encode(&bytes)).unwrap()
}

prop_compose! {
    fn arb_room()(bytes in any::<[u8; 32]>()) -> RoomSecret { room_from(bytes) }
}

prop_compose! {
    fn arb_peer()(seed in any::<[u8; 32]>()) -> EndpointId { SecretKey::from_bytes(&seed).public() }
}

fn arb_kind() -> impl Strategy<Value = PeerKind> {
    prop_oneof![Just(PeerKind::Agent), Just(PeerKind::Human)]
}

prop_compose! {
    fn arb_envelope()(
        id in "[0-9a-f]{10}",
        from in "[a-zA-Z0-9_.-]{1,32}",
        node in "[0-9a-f]{64}",
        kind in arb_kind(),
        to in proptest::option::of("[a-zA-Z0-9_.-]{1,32}"),
        reply_to in proptest::option::of("[0-9a-f]{10}"),
        ts in any::<u64>(),
        body in any::<String>(),
    ) -> Envelope {
        Envelope { id, from, node, kind, to, reply_to, ts, body }
    }
}

fn arb_frame() -> impl Strategy<Value = Frame> {
    prop_oneof![
        arb_envelope().prop_map(Frame::Msg),
        ("[0-9a-f]{10}", "[a-z]{1,32}").prop_map(|(id, by)| Frame::Ack { id, by }),
        ("[0-9a-f]{64}", "[a-z]{1,32}", proptest::option::of(any::<String>()), arb_kind(), any::<bool>()).prop_map(
            |(node, name, about, kind, ephemeral)| Frame::Hello(Presence { node, name, about, kind, ephemeral })
        ),
    ]
}

proptest! {
    #[test]
    fn invites_roundtrip(room in arb_room(), peers in prop::collection::vec(arb_peer(), 0..=4)) {
        let invite = Invite { secret: room, peers };
        let code = invite.encode();
        prop_assert!(code.starts_with("papo1"));
        prop_assert!(code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        prop_assert_eq!(Invite::decode(&code).unwrap(), invite.clone());
        // Chat apps and people retype codes in other cases and with stray whitespace.
        prop_assert_eq!(Invite::decode(&format!("  {}\n", code.to_ascii_uppercase().replacen("PAPO1", "papo1", 1))).unwrap(), invite);
    }

    #[test]
    fn invites_carry_at_most_four_entry_points(room in arb_room(), peers in prop::collection::vec(arb_peer(), 5..9)) {
        let decoded = Invite::decode(&Invite { secret: room, peers: peers.clone() }.encode()).unwrap();
        prop_assert_eq!(decoded.peers, peers[..4].to_vec());
    }

    #[test]
    fn truncated_invites_are_rejected(
        room in arb_room(),
        peers in prop::collection::vec(arb_peer(), 0..=4),
        cut in any::<prop::sample::Index>(),
    ) {
        let code = Invite { secret: room, peers }.encode();
        // Any strict prefix that still has the scheme: what a bad copy-paste produces.
        let len = "papo1".len() + cut.index(code.len() - "papo1".len());
        prop_assert!(Invite::decode(&code[..len]).is_err(), "accepted truncated invite {}", &code[..len]);
    }

    #[test]
    fn sealed_frames_roundtrip_and_have_fixed_overhead(room in arb_room(), plaintext in prop::collection::vec(any::<u8>(), 0..4096)) {
        let sealed = room.seal(&plaintext);
        // 24-byte nonce + 16-byte Poly1305 tag.
        prop_assert_eq!(sealed.len(), plaintext.len() + 40);
        prop_assert_eq!(room.open(&sealed).unwrap(), plaintext);
    }

    #[test]
    fn any_single_bit_flip_is_detected(
        room in arb_room(),
        plaintext in prop::collection::vec(any::<u8>(), 0..512),
        at in any::<prop::sample::Index>(),
        bit in 0u8..8,
    ) {
        let mut sealed = room.seal(&plaintext);
        let i = at.index(sealed.len());
        sealed[i] ^= 1 << bit;
        prop_assert!(room.open(&sealed).is_err());
    }

    #[test]
    fn other_rooms_cannot_open_frames(a in arb_room(), b in arb_room(), plaintext in prop::collection::vec(any::<u8>(), 0..256)) {
        prop_assume!(a != b);
        prop_assert!(b.open(&a.seal(&plaintext)).is_err());
        prop_assert_ne!(a.topic(), b.topic());
    }

    #[test]
    fn frames_roundtrip_through_the_wire(room in arb_room(), frame in arb_frame()) {
        match proto::encode(&room, &frame) {
            Ok(bytes) => {
                prop_assert!(bytes.len() <= MAX_FRAME_BYTES);
                prop_assert_eq!(proto::decode(&room, &bytes).unwrap(), frame);
            }
            // The only acceptable refusal is size.
            Err(e) => prop_assert!(e.to_string().contains("frame too large"), "{e}"),
        }
    }

    #[test]
    fn room_derivations_are_deterministic(bytes in any::<[u8; 32]>()) {
        let a = room_from(bytes);
        let b = RoomSecret::from_base32(&a.to_base32()).unwrap();
        prop_assert_eq!(a.topic(), b.topic());
        prop_assert_eq!(a.room_id(), b.room_id());
        let id = a.room_id();
        prop_assert!(id.len() == 8 && id.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn well_formed_names_are_accepted(name in "[a-zA-Z0-9_.-]{1,32}") {
        prop_assert!(validate_name(&name).is_ok());
    }

    #[test]
    fn accepted_names_are_short_and_safe(name in any::<String>()) {
        if validate_name(&name).is_ok() {
            prop_assert!(!name.is_empty() && name.len() <= MAX_NAME_LEN);
            prop_assert!(name.chars().all(|c| c.is_alphanumeric() || "-_.".contains(c)));
        }
    }

    #[test]
    fn names_with_separators_or_controls_are_rejected(prefix in "[a-z]{0,10}", bad in "[ /:@\\t\\n<>\"']", suffix in "[a-z]{0,10}") {
        let name = [prefix, bad, suffix].concat();
        prop_assert!(validate_name(&name).is_err(), "accepted {:?}", name);
    }

    #[test]
    fn addressing_matches_names_case_insensitively(to in "[a-zA-Z]{1,8}", name in "[a-zA-Z]{1,8}") {
        let mut env = Envelope {
            id: "x".into(), from: "f".into(), node: "n".into(), kind: PeerKind::Agent,
            to: None, reply_to: None, ts: 0, body: String::new(),
        };
        prop_assert!(env.is_for(&name), "broadcast reaches everyone");
        env.to = Some(to.clone());
        prop_assert_eq!(env.is_for(&name), to.eq_ignore_ascii_case(&name));
    }
}

#[test]
fn message_ids_are_short_hex_and_do_not_collide() {
    let ids: std::collections::HashSet<String> = (0..20_000).map(|_| proto::new_msg_id()).collect();
    assert_eq!(ids.len(), 20_000);
    assert!(ids.iter().all(|id| id.len() == 10 && id.chars().all(|c| c.is_ascii_hexdigit())));
}
