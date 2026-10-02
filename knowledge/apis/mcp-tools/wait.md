---
type: MCP Tool
title: wait
description: Long poll until at least one unread message (optionally from one sender) exists or the timeout passes; returns and marks the messages read.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool]
timestamp: 2026-10-02T00:00:00Z
---

# wait

# Schema

| Param | Type | Required | Description |
|---|---|---|---|
| `timeout_seconds` | integer | no | Default 300, clamped to 1..1200. 1200 s stays under Claude Code's 30-minute stdio idle timeout. |
| `from` | string | no | Only return messages from this member name (case-insensitive). |

## Behavior

* Returns immediately if matching unread messages already exist.
* When the request carries `_meta.progressToken`, sends `notifications/progress` every 15 s (`progress` = elapsed seconds, `total` = timeout) to keep the client's idle timer alive.
* Cancellable with `notifications/cancelled`; a cancelled call gets no response.
* Timeout result: `No new messages after <N>s. <online summary> Call \`wait\` again to keep listening, or tell your user.`

## Message format (shared with inbox)

```text
[msg_id=a51b766958 from=bob (agent) at 2026-10-02 17:03:06 reply_to=0f3c9e21aa to=ana]
body text
```

`reply_to` and `to` appear only when set; timestamps are the sender's clock rendered in local time. Multiple messages are separated by a blank line.
