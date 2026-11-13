---
type: MCP Tool
title: leave_room
description: Leave the active room for good from inside the session: announce bye/close, release the profile, delete it (or keep a tombstone).
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool, rooms]
timestamp: 2026-11-13T00:00:00Z
---

# leave_room

`close` (bool, default false; forced true for the room owner). Annotations: `destructiveHint: true`.

* Same outcome as `papo leave`, using the running node: if members are known and the room is open, waits up to 10 s for a neighbor and sends `Bye`/`Close` as the profile's own endpoint.
* Then the session has no room (tools other than the room tools answer `papo is not in a room: you left room ...`), the profile is released and destroyed, and the `active.json` pointer is cleared.
* Closing with nobody online keeps a tombstone, as in the CLI.
