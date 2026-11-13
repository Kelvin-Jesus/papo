---
type: MCP Tool
title: rooms
description: List the local profiles (one room each) with name, room id, owner/closed flags, unread count and which one the session serves.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool, rooms]
timestamp: 2026-11-13T00:00:00Z
---

# rooms

No parameters. Annotations: `readOnlyHint: true`. Works with no room running.

# Examples

```text
Rooms on this machine:
- default: you are "voce" in room 6635d855 (active in this session)
- sala-a1b2c3d4: you are "voce" in room a1b2c3d4, created by you, 2 unread
Use `switch_room` with a profile to move there.
```

* Unread is `inbox.json` of inactive profiles; the active one is shown by `status`.
* No profiles: `No rooms on this machine yet. Use \`join_room\` ...`.
