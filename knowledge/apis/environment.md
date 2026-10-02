---
type: Configuration
title: Environment variables
description: Environment variables read by the papo binary.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/main.rs
tags: [papo, configuration]
timestamp: 2026-10-02T00:00:00Z
---

# Environment variables

# Schema

| Variable | Default | Read by | Effect |
|---|---|---|---|
| `PAPO_HOME` | `~/.papo` | `src/store.rs` | Root of all profiles (`<home>/profiles/<profile>/`). Tests point it at temp dirs. |
| `PAPO_PROFILE` | `default` | `src/main.rs` (clap env) | Profile used when `--profile` is absent. |
| `PAPO_RELAY` | n0 public relays | `src/net.rs` | URL of a self-hosted `iroh-relay`; sets `RelayMode::Custom`. Must be a valid URL or the endpoint fails to bind. |
| `PAPO_LOG` | unset (silent) | `src/main.rs` | `tracing_subscriber` EnvFilter written to stderr, e.g. `info` or `iroh_gossip=debug,iroh=info`. See [debug connectivity](/playbooks/debug-connectivity.md). |
| `PAPO_MAX_SENDS_PER_10MIN` | `40` | `src/mcp.rs` | Anti-loop ceiling for the MCP `send` tool (see [delivery](/concepts/delivery.md)). |
