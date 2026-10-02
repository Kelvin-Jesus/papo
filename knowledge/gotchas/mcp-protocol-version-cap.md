---
type: Gotcha
title: Do not negotiate MCP 2026-07-28
description: Claude Code does not register servers that negotiate MCP revision 2026-07-28 as channels, so papo answers with at most 2025-11-25.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/mcp.rs
tags: [papo, mcp, channels, gotcha]
timestamp: 2026-10-02T00:00:00Z
---

# Do not negotiate MCP 2026-07-28

`SUPPORTED_PROTOCOLS` in `src/mcp.rs` is `2025-11-25, 2025-06-18, 2025-03-26, 2024-11-05`. A client asking for anything else (including `2026-07-28`) gets `2025-11-25`. The documentation states that, with `MCP_PROTOCOL_NEGOTIATION=auto`, a server negotiating 2026-07-28 is not registered as a channel.

## Rule

Before adding a newer revision, confirm in the Claude Code channels documentation that servers on it are still registered as channels. `tests/mcp.rs` asserts that a `2026-07-28` request is answered with `2025-11-25`; update it only together with that confirmation.

# Citations

[1] [Channels reference](https://code.claude.com/docs/en/channels-reference)
