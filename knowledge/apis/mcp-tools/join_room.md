---
type: MCP Tool
title: join_room
description: Create a profile from an invite (default name sala-<room id>, default member name from the current room) and switch to it.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool, rooms]
timestamp: 2026-11-13T00:00:00Z
---

# join_room

`invite` (required), `name`, `profile`. Annotations: `openWorldHint: true`.

* Same profile creation as `papo join` (`Store::join`: profile.json plus the invite's endpoints in `peers.json`).
* If some profile already holds the room secret (`Store::find_room`), it switches to that profile instead.
* `name` defaults to the member name in the active room; with no room, it is required.
* `profile` defaults to `sala-<room id>`; an existing profile is an error, never overwritten.
