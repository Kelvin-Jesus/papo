---
type: MCP Tool
title: history
description: Read-only view of the last N entries of the profile's conversation log (sent, received, delivery receipts).
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool]
timestamp: 2026-10-02T00:00:00Z
---

# history

# Schema

| Param | Type | Required | Description |
|---|---|---|---|
| `limit` | integer | no | Default 20, clamped to 1..200. |

Annotations: `readOnlyHint: true`. Reads `log.jsonl` (see [profile layout](/protocol/profile-layout.md)); does not change read state. Returns `No conversation yet.` for an empty log.

# Examples

```text
[2026-10-02 17:03:06] bob (human) -> ana (msg a51b766958): o endpoint /v2/users já aceita paginação?
[2026-10-02 17:03:40] ana -> bob (msg 0c1d2e3f4a, reply to a51b766958): Sim, cursor-based.
[2026-10-02 17:03:41] delivered 0c1d2e3f4a to bob
```

The same formatting is used by `papo log` (see [CLI](/apis/cli.md)).
