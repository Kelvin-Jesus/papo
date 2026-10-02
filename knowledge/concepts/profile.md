---
type: Domain Concept
title: Profile
description: Local identity in one room, stored under $PAPO_HOME/profiles/<name>; several profiles allow being in several rooms.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/store.rs
tags: [papo, storage, identity]
timestamp: 2026-10-02T00:00:00Z
---

# Profile

A profile binds one person to one [room](/concepts/room.md) on one machine. It holds the member name, the optional `about` text, the room secret, a stable iroh endpoint identity, the known peers, the inbox, the outbox and the conversation log. File layout: [profile layout](/protocol/profile-layout.md).

* Selected with `--profile <name>` on any command or `PAPO_PROFILE`; default `default`. Names are 1-32 chars of `[a-zA-Z0-9_-]`.
* Created by `papo new` or `papo join`. Both refuse to overwrite an existing profile unless `--force` is given. `--force` keeps `secret.key` (so peers still recognize the endpoint) and deletes `peers.json`, `inbox.json` and `outbox.json`; `log.jsonl` is kept.
* The endpoint identity (`secret.key`) is generated on first use and must stay stable: peers remember endpoint ids in their `peers.json` and redial them after restarts.
* Only one MCP server may run per profile at a time; see [one server per profile](/gotchas/one-server-per-profile.md). One-shot CLI commands (`say`, `status`) do not take the lock because they use throwaway identities (see [members](/concepts/members.md)).

The MCP server registered by `papo install` passes only `mcp [--profile <p>]` to the binary; the room secret stays in the profile directory.
