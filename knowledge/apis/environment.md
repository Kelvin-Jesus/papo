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
| `PAPO_RELAY` | n0 public relays | `src/net.rs` | URL of a self-hosted `iroh-relay`; sets `RelayMode::Custom`. Empty or blank means unset (container env files export empty variables); any other non-URL value fails startup. |
| `PAPO_INSTALL_COMMAND` | this executable | `src/main.rs` (`install`) | Replaces the command `papo install` registers, e.g. `docker exec -i papo-kj papo` when papo runs in a container. Split on whitespace; the `mcp` arguments are appended. |
| `PAPO_TEST_PROGRESS_MS` | 15000 | `src/mcp.rs` | Test hook: interval of `wait` progress notifications. |
| `PAPO_TEST_RELAY`, `PAPO_TEST_PEERS` | unset | `src/net.rs`, only with the `test-network` feature | Test hook: local relay URL and comma-separated endpoint ids reachable through it; address lookup becomes a static in-memory table. Absent from release builds. |
| `PAPO_LOG` | unset (silent) | `src/main.rs` | `tracing_subscriber` EnvFilter written to stderr, e.g. `info` or `iroh_gossip=debug,iroh=info`. See [debug connectivity](/playbooks/debug-connectivity.md). |
| `PAPO_MAX_SENDS_PER_10MIN` | `40` | `src/mcp.rs` | Anti-loop ceiling for the MCP `send` tool (see [delivery](/concepts/delivery.md)). |
