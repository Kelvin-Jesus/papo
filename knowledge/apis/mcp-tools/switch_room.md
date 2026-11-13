---
type: MCP Tool
title: switch_room
description: Move the running server to another profile's room without restarting Claude Code; remembered in active.json for the next session.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool, rooms]
timestamp: 2026-11-13T00:00:00Z
---

# switch_room

`profile` (required). Annotations: `openWorldHint: true`.

* Starts the new profile (`start_agent`: lock, endpoint, node) before releasing the old one, so a failure leaves the session in its room. Errors: unknown profile (lists the existing ones), lock held by another session, closed room.
* Releasing the old room: `Node::shutdown`, `activity.json` removed, profile lock dropped (`papo leave` works on it right away).
* Writes `active.json[launch] = profile` (see [profile layout](/protocol/profile-layout.md)).
* `Already in this room.` when `profile` is the active one.
* The channel pump and the activity bridge follow the new context through the `watch` channel; messages already in the new profile's inbox are pushed after the switch.
