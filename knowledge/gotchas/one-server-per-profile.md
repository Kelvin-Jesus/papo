---
type: Gotcha
title: One MCP server per profile
description: A file lock stops a second papo mcp from using the same profile, because two processes with one endpoint id fight on the network and corrupt the inbox.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/store.rs
tags: [papo, storage, gotcha]
timestamp: 2026-10-02T00:00:00Z
---

# One MCP server per profile

`Store::lock` takes an exclusive `File::try_lock` on `<profile>/lock` for the lifetime of the MCP server. A second server on the same profile (for example a second Claude Code session in another project with papo installed) fails to start; its tools return `another papo server is already running with this profile (another Claude Code session?). Close it or use a different --profile.`

## Why

Two processes with the same endpoint identity would compete for the same id on relays and in the gossip mesh, and both would rewrite `inbox.json`/`outbox.json`.

## Consequences

* `papo install` defaults to `--scope local` (one project) so not every session grabs the lock.
* `papo say` and `papo status` use throwaway identities and never take the lock, so they run next to the MCP server.
* For parallel sessions in different rooms, use different profiles (`--profile`).
