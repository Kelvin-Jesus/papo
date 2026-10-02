---
type: MCP Tool
title: inbox
description: Return every unread message without waiting and mark them read; "No unread messages." when empty.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, tool]
timestamp: 2026-10-02T00:00:00Z
---

# inbox

No parameters. Drains the whole inbox (all senders) and persists the change. Output uses the same message format as [wait](/apis/mcp-tools/wait.md). Read tracking rules: [delivery semantics](/concepts/delivery.md).
