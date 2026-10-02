---
type: Domain Concept
title: Push vs pull
description: Incoming messages reach Claude either as claude/channel notifications (push) or through the wait and inbox tools (pull); push never marks messages read.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, claude-code, channels]
timestamp: 2026-10-02T00:00:00Z
---

# Push vs pull

| Mode | Launch | Behavior |
|---|---|---|
| Push | `claude --dangerously-load-development-channels server:papo` | Each incoming message is sent as a [channel notification](/apis/channel-notification.md) and appears in Claude's context as `<channel source="papo" ...>`, triggering a turn even when the session is idle. |
| Pull | `claude` | Nothing is pushed into the context. The agent sees messages only when it calls [`wait`](/apis/mcp-tools/wait.md) or [`inbox`](/apis/mcp-tools/inbox.md). |

The server always declares the `claude/channel` capability and always pushes. It cannot detect whether channels are active, and Claude Code drops the notifications silently when they are not (see [channels dropped silently](/gotchas/channels-dropped-silently.md)). Therefore a pushed message stays unread until `wait`/`inbox` returns it or the agent answers it with `reply_to` (see [delivery semantics](/concepts/delivery.md)). With push on, a later `wait` may return a message the agent already saw; a harmless duplicate is preferred over a lost message.

Push starts only after the client sends `notifications/initialized` and the node is running. At that point the server replays the current inbox as notifications, then forwards new messages; a per-process set of pushed ids prevents double pushes between the replay and live events.

Requirements and limits of channels: [Claude Code channels](/dependencies/claude-code-channels.md).
