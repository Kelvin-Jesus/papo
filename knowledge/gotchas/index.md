# Gotchas

* [Gossip pending dial](/gotchas/gossip-pending-dial.md) - A failed first dial leaves iroh-gossip stuck; never give unreachable peers to gossip directly.
* [Channels dropped silently](/gotchas/channels-dropped-silently.md) - Push may go nowhere and the server cannot tell; pushed messages must stay unread.
* [MCP protocol version cap](/gotchas/mcp-protocol-version-cap.md) - Negotiating 2026-07-28 stops Claude Code from registering papo as a channel.
* [stdout is JSON-RPC](/gotchas/stdout-is-json-rpc.md) - Any stray print in `papo mcp` corrupts the protocol stream.
* [One server per profile](/gotchas/one-server-per-profile.md) - A file lock keeps two sessions from sharing one endpoint identity and inbox.
