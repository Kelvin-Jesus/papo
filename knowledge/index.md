---
okf_version: "0.1"
---

# papo Knowledge Bundle

[Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf) bundle for **papo**: a single Rust binary that gives the Claude Code sessions of different people a direct, end-to-end encrypted P2P line (MCP server + CLI). Asset-level knowledge for agents and maintainers; domain vocabulary lives in `CONTEXT.md` and decisions in `docs/adr/` at the repo root.

* [Concepts](/concepts/index.md) - Rooms, invites, profiles, members, delivery semantics, push vs pull.
* [APIs](/apis/index.md) - MCP server, MCP tools, the channel notification, CLI commands, environment variables.
* [Protocol and data](/protocol/index.md) - Wire frames, frame sealing, invite encoding, on-disk profile layout.
* [Dependencies](/dependencies/index.md) - iroh, iroh-gossip and Claude Code channels as papo uses them.
* [Gotchas](/gotchas/index.md) - Non-obvious failure modes and the rules that prevent them.
* [Playbooks](/playbooks/index.md) - Release, connectivity debugging, adding an MCP tool, rotating a room, public e2e test.
