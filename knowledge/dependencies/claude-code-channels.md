---
type: External Platform Feature
title: Claude Code channels
description: Research-preview Claude Code feature that lets an MCP server push events into a running session; papo uses it for incoming messages.
resource: https://code.claude.com/docs/en/channels
tags: [papo, claude-code, channels, dependency]
timestamp: 2026-10-02T00:00:00Z
---

# Claude Code channels

## Contract papo relies on

* Server capability `experimental["claude/channel"] = {}`; event method `notifications/claude/channel` with `content` (string) and optional `meta` (string map). Meta keys must be identifiers; others are dropped silently. Events render as `<channel source="<server name>" key="value">content</channel>`; events that arrive while Claude is busy are delivered together on the next turn.
* Custom (non-plugin) servers need `claude --dangerously-load-development-channels server:<name>` plus a confirmation dialog. The server must already be configured (papo registers it with `claude mcp add`). `--channels` only accepts allowlisted plugins.
* Auth: claude.ai login or Console API key. Not available on Bedrock, Vertex or Foundry.
* Team and Enterprise organizations must enable Channels (admin setting or `channelsEnabled: true` in managed settings); the dev flag does not bypass this. When disabled, the server and its tools still work and only push is lost.
* When channels are not active, notifications are dropped silently and the server cannot detect it (see [channels dropped silently](/gotchas/channels-dropped-silently.md)).
* With `MCP_PROTOCOL_NEGOTIATION=auto`, a server negotiating protocol revision 2026-07-28 is not registered as a channel (see [MCP protocol version cap](/gotchas/mcp-protocol-version-cap.md)).
* Flags and protocol may change while in research preview. The minimum Claude Code version for channels was not stated in the docs consulted.

## Tool timeouts (relevant to `wait`)

* `MCP_TOOL_TIMEOUT` defaults to a very long wall-clock limit (about 28 hours when unset); a per-server `timeout` in `.mcp.json` is a hard limit that progress does not extend.
* Idle timeout `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`: 30 minutes for stdio servers; progress notifications reset it. papo caps `wait` at 20 minutes and sends progress every 15 s.

papo does not declare the permission relay capability (`claude/channel/permission`).

# Citations

[1] [Channels](https://code.claude.com/docs/en/channels)
[2] [Channels reference](https://code.claude.com/docs/en/channels-reference)
[3] [MCP in Claude Code (timeouts, push messages with channels)](https://code.claude.com/docs/en/mcp)
