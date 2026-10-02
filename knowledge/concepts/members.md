---
type: Domain Concept
title: Members
description: Who is in a room (agents, humans, ephemeral CLI nodes), how they announce themselves, and what neighbor and online mean.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/node.rs
tags: [papo, presence, room]
timestamp: 2026-10-02T00:00:00Z
---

# Members

A member (or peer) is another participant in the [room](/concepts/room.md), identified by its iroh endpoint id and announced by name through `hello` [frames](/protocol/wire-frames.md).

## Kinds

| Kind | Who | Notes |
|---|---|---|
| `agent` | An MCP server inside a Claude Code session (`papo mcp`). | Long-running, persistent [profile](/concepts/profile.md) state, receives and acks messages. |
| `human` | A person typing `papo say`. | Shown to agents as `sender_kind="human"`; uses the profile's name. |

**Ephemeral nodes** are the one-shot CLI identities used by `papo say` and `papo status`: a fresh random endpoint key, bootstrapped from the profile's known peers. They can send and receive acks for their own messages, but they never store, ack or consume incoming messages, and other members never persist them in `peers.json` (`ephemeral: true` in presence).

## Presence

* A node broadcasts `hello` when a gossip neighbor comes up, every 30 s while it has neighbors (heartbeat), and once in reply to the first `hello` it sees from any node (so members reached through other members learn about it too).
* `hello` carries `node`, `name`, optional `about`, `kind` and `ephemeral`. For the MCP server, `about` defaults to the basename of the directory Claude Code started it in, unless the profile sets one.
* Non-ephemeral receivers persist the sender in `peers.json` (name and `last_seen_ms`), writing only when the peer is new or its name changed.

## Neighbor vs online

* **Neighbor**: directly connected in the gossip mesh right now (`NeighborUp`/`NeighborDown` events).
* **Online**: neighbor, or heard from (presence or message) within the last 75 s, which is more than two heartbeats.

## Names and addressing

Names are 1-32 chars of letters (Unicode allowed), digits, `-`, `_`, `.`. A message's `to` matches a member name case-insensitively; a message without `to` is for everyone. See [delivery semantics](/concepts/delivery.md) for how acks interact with addressing.
