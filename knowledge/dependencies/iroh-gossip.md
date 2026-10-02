---
type: External Library
title: iroh-gossip 0.101
description: Gossip broadcast (HyParView + PlumTree) over iroh that carries every papo frame; papo dials peers itself because of a stuck-pending dial quirk.
resource: https://docs.rs/iroh-gossip/0.101.0
tags: [papo, networking, dependency]
timestamp: 2026-10-02T00:00:00Z
---

# iroh-gossip 0.101

## How papo uses it

* `Gossip::builder().max_message_size(64 KiB).spawn(endpoint)`; the library default of 4096 bytes is too small for messages with code snippets.
* `gossip.subscribe(room.topic(), vec![])` with **no bootstrap peers**, then `split()` into `GossipSender` (`broadcast`, `join_peers`) and `GossipReceiver` (events `NeighborUp`, `NeighborDown`, `Received`, `Lagged`).
* Peers are dialed by papo's maintenance loop: `Endpoint::connect(peer, GOSSIP_ALPN)`, then `Gossip::handle_connection(conn)`, then `join_peers([peer])`. Retried with exponential backoff (1 s doubling to 10 s, 15 s dial timeout) while the node has no neighbors. Rationale: [pending dial gotcha](/gotchas/gossip-pending-dial.md).
* The wire protocol uses unidirectional streams in both directions, so a connection dialed by either side works for both.
* `Received` messages carry `delivered_from` (the forwarding neighbor), not the author; papo identifies authors through the `node` field of its own frames.
* If the receiver stream ends, papo marks the node unhealthy and the MCP `status`/`send` results tell the agent to have the session restarted.

# Citations

[1] [iroh-gossip 0.101.0 API docs](https://docs.rs/iroh-gossip/0.101.0)
[2] `src/net.rs` in the iroh-gossip 0.101.0 crate source (`Dialer::queue_dial`, `PeerState::Pending`, `handle_connection`).
