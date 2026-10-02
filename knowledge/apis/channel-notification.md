---
type: MCP Notification
title: notifications/claude/channel
description: Notification the papo MCP server sends for every incoming message so Claude Code injects it into the session as a channel tag.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, claude-code, channels]
timestamp: 2026-10-02T00:00:00Z
---

# notifications/claude/channel

# Schema

| Field | Type | Description |
|---|---|---|
| `params.content` | string | Message body. Becomes the tag body. |
| `params.meta.from` | string | Sender name. |
| `params.meta.msg_id` | string | Message id (10 hex chars); pass it back as `reply_to`. |
| `params.meta.sender_kind` | `agent` or `human` | Whether an MCP agent or a person (`papo say`) wrote it. |
| `params.meta.reply_to` | string, optional | Id this message answers. |
| `params.meta.to` | string, optional | Addressee, when the message was addressed. |

Meta keys must be identifiers (letters, digits, underscore); Claude Code silently drops other keys. A unit test in `src/mcp.rs` enforces this.

# Examples

Wire:

```json
{"jsonrpc":"2.0","method":"notifications/claude/channel","params":{"content":"o endpoint /v2/users já aceita paginação?","meta":{"from":"voce","msg_id":"a51b766958","sender_kind":"human"}}}
```

What Claude sees:

```text
<channel source="papo" from="voce" msg_id="a51b766958" sender_kind="human">
o endpoint /v2/users já aceita paginação?
</channel>
```

Delivery and read rules: [push vs pull](/concepts/push-vs-pull.md). Platform requirements: [Claude Code channels](/dependencies/claude-code-channels.md).

# Citations

[1] [Claude Code channels reference](https://code.claude.com/docs/en/channels-reference)
