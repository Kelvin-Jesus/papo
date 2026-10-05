---
type: CLI
title: papo CLI
description: Human-facing commands (output in Brazilian Portuguese) to create and join rooms, register the MCP server, talk, watch the log and test connectivity.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/main.rs
tags: [papo, cli]
timestamp: 2026-10-02T00:00:00Z
---

# papo CLI

Global option: `--profile <name>` (env `PAPO_PROFILE`, default `default`). Errors print `erro: <message>` to stderr and exit 1.

# Schema

| Command | Options | Effect |
|---|---|---|
| `new` | `--name <n>` (required), `--about <text>`, `--force` | Creates a [room](/concepts/room.md) and [profile](/concepts/profile.md); prints the [invite](/concepts/invite.md) and next steps. |
| `join <invite>` | `--name <n>` (required), `--about <text>`, `--force` | Creates a profile from an invite; stores the invite's endpoint ids as known peers. |
| `invite` | | Prints an invite with the caller's id plus up to 3 most recently seen peers. |
| `install` | `--scope local\|user\|project` (default `local`), `--print` | Runs `claude mcp add --scope <s> papo -- <abs path to papo> mcp [--profile <p>]`. `--print` only prints the `.mcp.json` snippet. Falls back to `claude.cmd` on Windows; if `claude` is missing it prints the command to run. Requires a configured profile. |
| `mcp` | | Runs the [MCP server](/apis/mcp-server.md) on stdio. |
| `say <text...>` | `--to <name>` | Ephemeral human node: waits up to 30 s for a neighbor, sends, waits up to 15 s for an ack. Fails if nobody is online (nothing is queued). |
| `log` | `-n/--lines <N>` (default 30), `-f/--follow` | Prints the last N log entries; `--follow` polls the file every 500 ms. |
| `status` | `--timeout <secs>` (default 20) | Ephemeral node that looks for members, then lists peers with online state and about text, plus queued/unread counts from disk. |

| `leave` | `--yes` | Needs the profile lock. Ephemeral node broadcasts `Bye` (twice) as the real endpoint id, then deletes the profile dir. |
| `close` | `--yes` | Same, but broadcasts `Close`: receivers forget everyone and write the `closed` marker. The room owner (`Profile.owner`, set by `new`) leaving counts as `close`. If no neighbor answered, `into_tombstone` keeps only `profile.json`, `secret.key` and `closed`; a second `leave` deletes it. |

`say` and `status` refuse to run when the profile knows no peers yet. Launch command printed after setup: `claude --dangerously-load-development-channels server:papo`.

# Examples

```sh
papo new --name voce
papo join papo1... --name colega
papo install                 # inside the project directory
papo log -f
papo say --to colega "pergunta pro Claude do colega sobre o deploy"
papo status --timeout 15
papo --profile time-pagamentos new --name voce
```
