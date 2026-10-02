---
type: MCP Tool
title: send
description: Broadcast a message to the room (or one member via to), wait up to 8 s for an ack, and report delivered or queued.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool]
timestamp: 2026-10-02T00:00:00Z
---

# send

# Schema

| Param | Type | Required | Description |
|---|---|---|---|
| `message` | string | yes | Full, self-contained message. Trimmed; empty is an error. Max 48 KiB. |
| `to` | string | no | Member name (case-insensitive). Omit to address everyone. |
| `reply_to` | string | no | `msg_id` being answered; marks it and that sender's earlier inbox messages read. |

Annotations: `openWorldHint: true`. No additional properties.

## Results

* `Delivered to <name> (msg_id <id>). If you need their answer to continue, call \`wait\`.`
* `Not acknowledged yet (msg_id <id>). It is queued and will be delivered automatically when a peer is reachable. <Online: a, b. | No peer is online right now.>`
* If the gossip subscription died: `Queued (msg_id <id>), but WARNING: papo lost its network subscription ...`
* Errors (`isError: true`): missing `message`, body too large, invalid `to`, or the rate limit of 40 sends per 10 minutes (`PAPO_MAX_SENDS_PER_10MIN`).

Semantics: [delivery](/concepts/delivery.md).

# Examples

```json
{"name": "send", "arguments": {"message": "8443, com TLS.", "reply_to": "a51b766958"}}
```
