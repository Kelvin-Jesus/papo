---
type: Gotcha
title: Channel notifications are dropped silently
description: Claude Code discards claude/channel notifications without error when channels are not active, so the server must never treat a push as delivery to the agent.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, claude-code, channels, gotcha]
timestamp: 2026-10-02T00:00:00Z
---

# Channel notifications are dropped silently

When Claude Code was started without `--dangerously-load-development-channels server:papo`, or the organization has not enabled Channels, notifications are discarded and the server gets no error. Nothing in `initialize` tells the server whether channels are active.

## Rule

* A pushed message stays in the inbox. It leaves only through `wait`, `inbox`, or a `send` with `reply_to` (see [push vs pull](/concepts/push-vs-pull.md)).
* Never add logic that marks messages read on push, and never drop the pull tools.
* Tool descriptions and server instructions must keep telling agents to call `wait` when they expect an answer.

# Citations

[1] [Channels reference](https://code.claude.com/docs/en/channels-reference)
