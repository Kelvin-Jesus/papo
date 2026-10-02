---
name: papo-network-debug
description: Diagnose papo connectivity - members not seeing each other, messages stuck in the queue, "ninguém da sala está online", relay or UDP blocks - using papo status, PAPO_LOG filters and iroh log lines. Use when a papo room does not connect, delivery stalls, or the user reports papo is offline.
---

# Debug papo connectivity

Full playbook: `knowledge/playbooks/debug-connectivity.md`. Background: `knowledge/dependencies/iroh.md`, `knowledge/gotchas/gossip-pending-dial.md`.

## Triage

1. `papo status` on both sides: same room id on the first line? Peers listed? `--timeout 30` to wait longer.
2. Is the other member actually running (Claude Code session with papo installed, or `papo status`)?
3. Empty `peers.json` ("ainda não conheço ninguém nesta sala"): the creator learns members only after a joiner connects.
4. "another papo server is already running": a second session holds the profile lock.
5. `WARNING: papo lost its network subscription`: restart the Claude Code session.

## Logs

```sh
PAPO_LOG=info papo status
PAPO_LOG=iroh_gossip=debug,iroh=info,iroh::address_lookup=debug papo status --timeout 30
```

| Log line | Meaning |
|---|---|
| `home is now relay https://...relay.n0.iroh.link./` | Endpoint online through a relay. |
| `Publishing endpoint info to pkarr` | Dialable by id shortly after. |
| `dial failed: No addressing information available` | Peer offline or not yet published; papo retries (1 s to 10 s backoff). |
| `start to dial` then only `SendMessage` lines, never connecting | The gossip pending-dial bug; check that nobody reintroduced gossip bootstrap. |
| `dropped frame relayed by <peer>` | Different room secret or corrupted frame. |

## Network

- `curl -sI https://euc1-1.relay.n0.iroh.link/` and `curl -sI https://dns.iroh.link/` must succeed; UDP is optional (relay fallback over HTTPS).
- Blocked public relays: run `iroh-relay` and set `PAPO_RELAY=<url>` for every member.
- Sanity-check the whole stack on this machine: `cargo test --test mcp -- --ignored`.

Never ask the user to paste invites, `profile.json` or `secret.key`.
