---
type: Gotcha
title: stdout is JSON-RPC in papo mcp
description: In MCP mode stdout carries only protocol messages; every diagnostic must go to stderr or the client breaks.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, gotcha]
timestamp: 2026-10-02T00:00:00Z
---

# stdout is JSON-RPC in papo mcp

All output of `papo mcp` goes through a single writer task that serializes JSON values, one per line. Any `println!` reachable from the MCP path, or a library printing to stdout, corrupts the stream for Claude Code.

## Rule

* Use `eprintln!` or `tracing` for diagnostics in `src/node.rs`, `src/mcp.rs`, `src/net.rs` and `src/store.rs`. `PAPO_LOG` sends tracing output to stderr.
* `println!` is fine only in human CLI commands in `src/main.rs` (`new`, `join`, `invite`, `install`, `say`, `log`, `status`).
* `tests/mcp.rs` parses every stdout line as JSON and panics on anything else, which catches regressions.
