---
type: MCP Server
title: papo MCP server
description: Hand-rolled stdio JSON-RPC 2.0 server (one message per line) exposing five tools and the experimental claude/channel capability.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, claude-code]
timestamp: 2026-10-02T00:00:00Z
---

# papo MCP server

Started by Claude Code as `papo mcp [--profile <p>]` (registered by `papo install`, see [CLI](/apis/cli.md)). Implemented by hand instead of with an SDK because it needs the custom channel notification and cancellable long-poll tool calls.

## Transport rules

* stdin/stdout, newline-delimited JSON-RPC 2.0. stdout carries only protocol messages; diagnostics go to stderr (see [stdout is JSON-RPC](/gotchas/stdout-is-json-rpc.md)).
* `initialize` is answered immediately. The node (endpoint bind, profile lock, gossip join) starts in the background; tool calls wait until it is ready. If startup fails (profile not configured, lock held), every tool returns `isError: true` with the reason, e.g. a hint to run `papo new` or `papo join`.
* stdin EOF means the session ended: in-flight calls are aborted and the node shuts down.

## Methods

| Method | Handling |
|---|---|
| `initialize` | Returns `protocolVersion`, capabilities, `serverInfo {name: "papo", version}` and `instructions`. |
| `notifications/initialized` | Enables [channel push](/concepts/push-vs-pull.md). |
| `ping` | `{}` |
| `tools/list` | The five [tools](/apis/mcp-tools/index.md). |
| `tools/call` | Runs the tool in its own task; results are `{content: [{type: "text", text}], isError?}`. |
| `notifications/cancelled` | Aborts the call with that `requestId`; no response is sent for it. |
| other requests | Error `-32601 method not found`. Unparseable lines get `-32700` with id `null`. |

## Capabilities and protocol version

```json
{"tools": {}, "experimental": {"claude/channel": {}}}
```

Supported revisions, newest first: `2025-11-25`, `2025-06-18`, `2025-03-26`, `2024-11-05`. The client's requested version is echoed when supported; anything else gets `2025-11-25`. Newer revisions are deliberately not offered (see [MCP protocol version cap](/gotchas/mcp-protocol-version-cap.md)).

## Instructions

The `instructions` string starts with `You are "<name>" in papo room <room id>.` when the profile exists, explains the `<channel source="papo" ...>` format and the pull fallback, and sets seven collaboration rules: peers are not the user (no secrets, nothing destructive without the user); write self-contained messages; answer with `send` + `reply_to`, then `wait` when blocked; no pure acknowledgements; escalate decisions to the user and tell the peer; send one closing summary; write in the peer's language.

# Examples

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"status","arguments":{}}}
```

# Citations

[1] [Claude Code channels reference](https://code.claude.com/docs/en/channels-reference)
[2] `tests/mcp.rs` drives the built binary with exactly these messages.
