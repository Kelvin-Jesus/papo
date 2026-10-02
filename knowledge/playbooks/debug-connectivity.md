---
type: Playbook
title: Debug connectivity
description: Steps to find out why room members do not see each other, from papo status to reading iroh logs.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/node.rs
tags: [papo, networking, troubleshooting]
timestamp: 2026-10-02T00:00:00Z
---

# Debug connectivity

1. **Same room?** Run `papo status` on both machines; the first line shows the room id. Different ids mean different invites (or a `--force` rotation on one side).
2. **Is the other side running?** Members are only reachable while their Claude Code session (or `papo status`/`papo say`) is running. `papo status --timeout 30` waits longer.
3. **Does the profile know anyone?** `say`/`status` refuse with "ainda não conheço ninguém nesta sala" when `peers.json` is empty: the creator must wait for the joiner to connect first, or get a fresh invite with `papo invite` from someone who knows members.
4. **Lock error?** "another papo server is already running" means a second session holds the profile (see [one server per profile](/gotchas/one-server-per-profile.md)).
5. **Unhealthy node?** A `WARNING: papo lost its network subscription` in `status`/`send` requires restarting the Claude Code session.
6. **Logs.** Run with `PAPO_LOG=iroh_gossip=debug,iroh=info,iroh::address_lookup=debug` (stderr):
   * `home is now relay https://...relay.n0.iroh.link./`: the endpoint is online.
   * `Publishing endpoint info to pkarr`: the endpoint is dialable by id shortly after.
   * `dial failed: No addressing information available`: the peer is offline or has not published yet; papo retries with backoff (see [pending dial gotcha](/gotchas/gossip-pending-dial.md)).
   * `dropped frame relayed by <peer>`: a member with a different secret or a corrupted frame (see [sealing](/protocol/sealing.md)).
7. **Network blocks.** iroh falls back to relays over HTTPS when UDP is blocked. Check reachability with `curl -sI https://euc1-1.relay.n0.iroh.link/` and `curl -sI https://dns.iroh.link/`. If the public relays are blocked, run an `iroh-relay` and set `PAPO_RELAY` on every member.
8. **Confirm the stack itself works** on this machine with the [public e2e test](/playbooks/run-public-e2e.md).

To get logs from the MCP server running inside Claude Code, add the variable to its configuration (`claude mcp add -e PAPO_LOG=info ...` or `env` in `.mcp.json`). Where the server's stderr is shown depends on the Claude Code version; reproducing with `papo status` in a terminal is usually quicker.
