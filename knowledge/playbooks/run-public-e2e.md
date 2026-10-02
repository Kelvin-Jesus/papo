---
type: Playbook
title: Run the public e2e test
description: Run two real papo MCP servers that create a room, connect over the public iroh network and exchange a message and a reply.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/tests/mcp.rs
tags: [papo, testing, networking]
timestamp: 2026-10-02T00:00:00Z
---

# Run the public e2e test

```sh
cargo test --test mcp -- --ignored
```

The ignored test `two_agents_talk_over_the_public_network`:

1. Creates two temporary `PAPO_HOME`s; `ana` runs `new`, `bob` runs `join` with the printed invite.
2. Spawns two `papo mcp` processes and speaks JSON-RPC to them like Claude Code.
3. Polls bob's `status` until `ana (agent): online` (up to 90 s).
4. Ana `send`s; the test waits for the `notifications/claude/channel` event on bob's stdout and checks content and meta.
5. Bob replies with `reply_to`; bob's `inbox` must then be empty; ana's `wait` must return the reply with `reply_to=<id>`; ana's `history` must contain a delivery receipt.

It takes about 6 s on a normal connection. It needs outbound HTTPS to `dns.iroh.link` and the n0 relays (UDP is optional). CI runs it as the `e2e-public-network` job with `continue-on-error`, because it depends on third-party infrastructure.

If it fails at step 3, follow [debug connectivity](/playbooks/debug-connectivity.md).
