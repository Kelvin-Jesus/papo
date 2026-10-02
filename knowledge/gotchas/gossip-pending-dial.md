---
type: Gotcha
title: iroh-gossip stays stuck after a failed first dial
description: In iroh-gossip 0.101 a peer whose first dial fails stays Pending forever and later join_peers calls never redial; papo dials peers itself.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/node.rs
tags: [papo, iroh-gossip, networking, gotcha]
timestamp: 2026-10-02T00:00:00Z
---

# iroh-gossip stays stuck after a failed first dial

## Symptom

The MCP `status` tool lists the other member as `unknown (...)`, offline, forever (the CLI `papo status` shows `(nome desconhecido)`), even though both sides are running and the network is fine. With `PAPO_LOG=iroh_gossip=debug,iroh=info` the joining side shows one `start to dial`, then `dial failed: No addressing information available`, and afterwards only `handle out_event SendMessage(...)` lines without new dials.

## Cause

In the gossip actor, `OutEvent::SendMessage` to a peer in `PeerState::Pending { queue }` only starts a dial when the queue is empty. A failed dial leaves the queue non-empty, so no later message (including new joins) dials again. The common trigger is dialing a peer in the ~1 s after it starts, before it has published its address to pkarr (see [iroh](/dependencies/iroh.md)). An *incoming* connection for that peer clears the state (`accept_conn` flushes the queue).

## Rule

Never pass peers to `Gossip::subscribe` as bootstrap and never call `join_peers` for a peer without a live connection. papo subscribes with no bootstrap; its maintenance loop calls `Endpoint::connect(peer, GOSSIP_ALPN)`, hands the connection to `Gossip::handle_connection` as if it were incoming, and only then calls `join_peers([peer])`. Retries use 1 s doubling to 10 s while the node has no neighbors.

Measured on the public network: first contact went from never (gossip bootstrap) to 12 s (fixed 10 s retries) to about 6 s end-to-end in the public e2e test (backoff).

If a future iroh-gossip redials on its own, the workaround still works; simplify only after verifying.

# Citations

[1] iroh-gossip 0.101.0 crate source, `src/net.rs`: `OutEvent::SendMessage` handling, `Dialer::queue_dial`, `handle_connection`.
[2] [iroh-gossip](/dependencies/iroh-gossip.md)
[3] ADR `docs/adr/0003-o-papo-disca-os-pares-antes-do-gossip.md` in the repository.
