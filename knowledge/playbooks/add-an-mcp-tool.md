---
type: Playbook
title: Add or change an MCP tool
description: Every place a new or changed MCP tool must touch: definitions, dispatch, implementation, instructions, tests, docs and this bundle.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, development]
timestamp: 2026-10-02T00:00:00Z
---

# Add or change an MCP tool

1. **Definition**: add an entry to `tool_definitions()` in `src/mcp.rs`: `name`, an agent-facing English `description` that says when to use it, an `inputSchema` with `"additionalProperties": false`, and `annotations` (`readOnlyHint` for read-only tools).
2. **Dispatch**: add the name to the `match` in `call_tool`. Tools receive the ready `Ctx` (node, store, profile).
3. **Implementation**: write `tool_<name>(ctx, args) -> Result<String>`; returned errors become `isError: true` results. Read arguments with `str_arg` and clamp numeric ones. Long-running tools follow `tool_wait`: `tokio::select!` with periodic `notifications/progress` when a `progressToken` is present; cancellation is already handled by aborting the task.
4. **Node API**: if the tool needs new behavior, add it to `Node` in `src/node.rs` and keep the [delivery invariants](/concepts/delivery.md) (outbox before broadcast, persist before ack).
5. **Instructions**: if agents should change how they collaborate, update `instructions()` in `src/mcp.rs`.
6. **Tests**: `tests/mcp.rs` asserts the exact tool list in `speaks_mcp_and_advertises_the_channel_capability`; update it and add a test that calls the tool through JSON-RPC. Node behavior gets a test in `tests/node.rs`.
7. **Docs**: README tools table, `docs/referencia/ferramentas-mcp.md` (docs book), and `knowledge/apis/mcp-tools/<name>.md` plus its `index.md` and a `knowledge/log.md` entry.
8. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
