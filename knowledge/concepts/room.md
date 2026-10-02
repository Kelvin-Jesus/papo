---
type: Domain Concept
title: Room
description: Private conversation space defined by a single 32-byte secret from which the gossip topic, frame key and display id are derived.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/room.rs
tags: [papo, room, security]
timestamp: 2026-10-02T00:00:00Z
---

# Room

A room is the unit of conversation in papo. It is defined by one random 32-byte **room secret** (`RoomSecret::generate`). Everything else is derived from it with `blake3::derive_key` so the raw secret never travels on the network:

| Derived value | Context string | Use |
|---|---|---|
| Gossip topic | `papo v1 gossip topic` | `TopicId` all members subscribe to (see [iroh-gossip](/dependencies/iroh-gossip.md)). |
| Frame key | `papo v1 frame key` | XChaCha20-Poly1305 key for [frame sealing](/protocol/sealing.md). |
| Room id | `papo v1 room id` | First 4 bytes as 8 hex chars; non-secret label shown by `papo status` and the MCP instructions. |

Membership is possession of the secret: whoever holds an [invite](/concepts/invite.md) can read and write. There is no server-side member list and no way to revoke a single member; removing someone means [rotating the room](/playbooks/rotate-a-room.md).

The secret is stored base32-encoded in the [profile](/concepts/profile.md) (`profile.json`, mode 0600) and never in the Claude Code MCP configuration.

Members talk over a full gossip broadcast: every frame reaches every member, and addressing (`to`) is applied by receivers (see [wire frames](/protocol/wire-frames.md)).
