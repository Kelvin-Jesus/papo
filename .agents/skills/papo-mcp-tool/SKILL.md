---
name: papo-mcp-tool
description: Add or change a papo MCP tool (send, wait, inbox, history, status or a new one) - definition and dispatch in src/mcp.rs, node support, server instructions, tests/mcp.rs, docs reference and the knowledge bundle. Use when asked to add a tool, change a tool's parameters or result text, or change what Claude is told in the server instructions.
---

# Add or change a papo MCP tool

Checklist (details: `knowledge/playbooks/add-an-mcp-tool.md`; current surface: `knowledge/apis/mcp-tools/`):

1. `src/mcp.rs` `tool_definitions()`: name, agent-facing English description saying when to use it, `inputSchema` with `"additionalProperties": false`, `annotations` (`readOnlyHint` when read-only).
2. `src/mcp.rs` `call_tool`: add the `match` arm.
3. Implement `tool_<name>(ctx, args) -> Result<String>`; errors become `isError: true`. Use `str_arg`, clamp numbers.
4. Long-running? Copy the `tool_wait` pattern: `tokio::select!` + `notifications/progress` every 15 s when `_meta.progressToken` is present. Cancellation already aborts the task. Keep total duration under 20 minutes.
5. New node behavior goes in `src/node.rs`, preserving outbox-before-broadcast and persist-before-ack.
6. If agents should behave differently, update `instructions()` (keep the seven collaboration rules coherent).
7. Tests: update the exact tool list asserted in `tests/mcp.rs` (`speaks_mcp_and_advertises_the_channel_capability`) and add a JSON-RPC test for the tool; node logic gets a `tests/node.rs` test.
8. Docs: README tools table, `docs/referencia/ferramentas-mcp.md`, `knowledge/apis/mcp-tools/<name>.md` + `index.md`, `knowledge/log.md`.
9. Run the `papo-dev` checklist.

Never print to stdout from tool code; never mark messages read on push.
