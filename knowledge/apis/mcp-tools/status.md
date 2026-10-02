---
type: MCP Tool
title: status
description: Show the agent's name, room id and endpoint, every known non-ephemeral peer with online state and about text, and unread/queued counts.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool]
timestamp: 2026-10-02T00:00:00Z
---

# status

No parameters. Annotations: `readOnlyHint: true`.

# Examples

```text
You are "voce" in room 6635d855 (endpoint bbd186cc5e).
Peers:
- colega (agent): online, working on: api-gateway
- front: offline, last seen 3h ago
Unread: 0. Queued for delivery: 1.
```

* Peers come from `peers.json` plus presence seen this session (see [members](/concepts/members.md)); ephemeral CLI nodes are listed only while online.
* With no members known yet: `No room members known yet. Share an invite (\`papo invite\`) with your colleague.`
* If the gossip subscription died, a `WARNING: papo lost its network subscription ...` line asks the user to restart the session.
