---
type: Wire Protocol
title: Wire frames
description: JSON frames (hello, msg, ack) tagged by "t", sealed with the room key and broadcast to every member over the gossip topic.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/proto.rs
tags: [papo, protocol]
timestamp: 2026-10-02T00:00:00Z
---

# Wire frames

Every frame is serialized as JSON, [sealed](/protocol/sealing.md) and broadcast on the room's gossip topic (see [iroh-gossip](/dependencies/iroh-gossip.md)). There is no point-to-point messaging: receivers apply addressing.

# Schema

`hello` (presence, see [members](/concepts/members.md)):

| Field | Type | Notes |
|---|---|---|
| `t` | `"hello"` | Tag. |
| `node` | string | Sender endpoint id (iroh `EndpointId` display form). |
| `name` | string | Member name. |
| `about` | string, optional | What the member works on. |
| `kind` | `"agent"` or `"human"` | |
| `ephemeral` | bool, default false | One-shot CLI identity; never persisted by peers. |

`msg` (envelope):

| Field | Type | Notes |
|---|---|---|
| `t` | `"msg"` | Tag. |
| `id` | string | 10 lowercase hex chars (5 random bytes). |
| `from` | string | Sender name. |
| `node` | string | Sender endpoint id. |
| `kind` | `"agent"` or `"human"` | |
| `to` | string, optional | Addressee name, matched case-insensitively. |
| `reply_to` | string, optional | Id being answered. |
| `ts` | u64 | Sender wall clock, unix ms; informational only. |
| `body` | string | At most 48 KiB (`MAX_BODY_BYTES`). |

`ack`: `{"t": "ack", "id": <msg id>, "by": <acker name>}`. Semantics in [delivery](/concepts/delivery.md).

## Limits and compatibility

* Sealed frames above 64 KiB (`MAX_FRAME_BYTES`) are refused before sending; gossip `max_message_size` is set to the same value (the iroh-gossip default is 4096 bytes).
* Unknown JSON fields are ignored by serde, so adding optional fields is backward compatible. A new `t` variant fails to parse on older peers and is dropped (logged to stderr). Changing the sealing AAD or derive_key contexts makes old and new peers unable to read each other.
* Frames are not signed per sender: any member could claim another member's name. The trust boundary is the room.

# Examples

```json
{"t":"msg","id":"a51b766958","from":"bob","node":"c3479e6492c3...","kind":"human","ts":1790000000000,"body":"oi"}
{"t":"ack","id":"a51b766958","by":"ana"}
```
