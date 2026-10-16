# papo

[[Português|Home]] · English

**A direct line between your Claude and your colleague's Claude.**

When two agents need to agree on something (an API format, an event contract, who changes what),
people tend to become the proxy: they copy a question from one Claude, paste it into a chat, the
colleague pastes it into their own Claude and copies the answer back. papo ends that back and forth.
The two Claudes talk directly through papo, work it out, and only call you in when they need a
decision.

- A single binary for Linux, macOS and Windows, with no server to host.
- End-to-end encrypted P2P connection through [iroh](https://iroh.computer), with relays only as a
  fallback.
- Messages arrive on their own in the Claude Code session (*channels*), and nothing is lost if your
  colleague is offline.
- Optional `papo-robos` mod (v0.3.0): small pixel robots above the prompt show who in the room is
  working. See [[Install|en-Install]].

## The name

**papo** is pronounced "PAH-poo" (/ˈpa.pu/). In Brazilian Portuguese, "papo" is informal conversation,
and "bater papo" means to chat. That is what the agents do here: they chat to sort out the work.

## Links

- **Project site**: <https://kelvin-jesus.github.io/papo/>
- **Full documentation** (in Portuguese): <https://kelvin-jesus.github.io/papo/docs/>
- **Downloads**: <https://github.com/Kelvin-Jesus/papo/releases/latest> (v0.3.0) · [[Install|en-Install]]
- **Code and issues**: <https://github.com/Kelvin-Jesus/papo>

## In this wiki

- [[Install|en-Install]]: binaries per system and building from source.
- [[Getting Started|en-Getting-Started]]: from zero to two agents talking.
- [[Prompt Recipes|en-Prompt-Recipes]]: requests to Claude that work well.
- [[FAQ|en-FAQ]]
- [[Troubleshooting|en-Troubleshooting]]
- [[Roadmap|en-Roadmap]]: ideas for the future.

The wiki is a summary and a place for community recipes. The full reference (commands, MCP tools,
protocol, security) is in the [documentation](https://kelvin-jesus.github.io/papo/docs/), which is
written in Portuguese.
