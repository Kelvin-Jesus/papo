---
type: Domain Concept
title: Delivery semantics
description: At-least-once store-and-forward delivery with a persistent outbox, receiver acks, dedupe by message id and reply-based read tracking.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/node.rs
tags: [papo, delivery, reliability]
timestamp: 2026-10-02T00:00:00Z
---

# Delivery semantics

Gossip broadcast is best effort, so papo adds its own guarantees on top (see [iroh-gossip](/dependencies/iroh-gossip.md)).

## Sending

1. `send` validates the body (non-empty after trim, at most 48 KiB) and the optional `to` name, then encodes the frame first so an oversized frame fails before anything is queued.
2. The envelope is appended to the **outbox** and persisted (`outbox.json`) *before* it is broadcast, then logged as `out`.
3. The sender waits for an `ack` up to a timeout: 8 s in the MCP `send` tool, 15 s in `papo say`. No ack in time is not an error: the result is `Queued` and the message stays in the outbox.

## Receiving

1. Receivers ignore frames from themselves, messages whose `to` names someone else, and everything when they are ephemeral.
2. A new message (id not in the `seen` set) is appended to the **inbox**, persisted, logged as `in`, and only then acked. Persist-before-ack means an ack promises the message is safe on disk.
3. Duplicates are re-acked but not re-delivered, so a lost ack heals itself.
4. The `seen` set is rebuilt on start from `in` entries of `log.jsonl` plus the ids still in `inbox.json`.

## Acks and retries

* The first `ack` for an id removes it from the sender's outbox (logged as `delivered`). Only the addressee acks an addressed message; any receiver acks a broadcast.
* The outbox is re-broadcast when a gossip neighbor comes up and every 30 s while connected.
* Limitation: in rooms with more than two members, a broadcast leaves the outbox on the first ack, so members offline at that moment may never get it. Use `to` for guaranteed delivery to one member.

## Read tracking

Unread = in the inbox. Messages leave the inbox when `wait` or `inbox` return them, or when the agent sends a reply with `reply_to=<id>`: that removes the replied message and every earlier inbox message from the same sender node. Pushed channel notifications do not mark anything read (see [push vs pull](/concepts/push-vs-pull.md)).

## Rate limit

The MCP `send` tool refuses more than 40 sends per sliding 10-minute window (configurable with `PAPO_MAX_SENDS_PER_10MIN`), returning an error that tells the agent it is probably in a loop and should check with its user.
