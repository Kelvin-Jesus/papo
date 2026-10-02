//! papo: a direct line between AI coding agents of different people.
//!
//! Each person runs `papo mcp` inside their Claude Code session. The MCP server joins
//! a private, end-to-end encrypted room over iroh (QUIC with hole punching and relay
//! fallback) and lets the agents talk to each other without the humans relaying.

pub mod room;
