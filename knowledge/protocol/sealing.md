---
type: Cryptographic Scheme
title: Frame sealing
description: Each frame is encrypted and authenticated with XChaCha20-Poly1305 under a key derived from the room secret, independent of transport encryption.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/room.rs
tags: [papo, security, protocol]
timestamp: 2026-10-02T00:00:00Z
---

# Frame sealing

Gossip forwards frames through any member, so papo seals every frame itself rather than relying only on iroh's QUIC/TLS. Only holders of the [room](/concepts/room.md) secret can read or inject frames.

# Schema

| Element | Value |
|---|---|
| Cipher | XChaCha20-Poly1305 (`chacha20poly1305` 0.11) |
| Key | `blake3::derive_key("papo v1 frame key", room_secret)` |
| Nonce | 24 random bytes per frame, prepended to the ciphertext |
| AAD | `papo/v1` |
| Sealed layout | `nonce (24) || ciphertext || tag (16)` |

Opening fails for frames shorter than the nonce, for the wrong room, or for any tampering; the node drops such frames and logs `dropped frame relayed by <peer>` to stderr.

Other derive_key contexts in use: `papo v1 gossip topic` (topic id) and `papo v1 room id` (display id). Changing any context or the AAD is a breaking protocol change (see [wire frames](/protocol/wire-frames.md)).
