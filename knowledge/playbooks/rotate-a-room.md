---
type: Playbook
title: Rotate a room
description: Replace the room secret to remove a member or invalidate a leaked invite, keeping each member's endpoint identity.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/store.rs
tags: [papo, security, operations]
timestamp: 2026-10-02T00:00:00Z
---

# Rotate a room

Membership is possession of the secret (see [room](/concepts/room.md)), so removing someone means moving everyone else to a new room.

1. One member runs `papo new --name <name> --force` (plus `--profile` if not default). This creates a new secret, keeps `secret.key`, and deletes `peers.json`, `inbox.json` and `outbox.json`. `log.jsonl` keeps the old conversation.
2. Send the new invite privately only to the members who stay.
3. Each of them runs `papo join <invite> --name <name> --force`.
4. Everyone restarts their Claude Code session: a running MCP server keeps the old room until it restarts. `papo install` is not needed again because the MCP config contains no secret.
5. Check with `papo status` that the room id changed and the expected members are online.

Messages still queued in the old outbox are discarded by step 1/3; resend anything important.
