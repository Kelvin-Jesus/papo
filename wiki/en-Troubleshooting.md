# Troubleshooting

[[Português|Problemas-Comuns]] · English

First step: run `papo status` on both machines and check that the **room id** is the same. The full
guide is in [Solução de problemas](https://kelvin-jesus.github.io/papo/docs/solucao-de-problemas.html)
(in Portuguese).

## `papo status` finds nobody

- Your colleague needs Claude Code open in the project where they ran `papo install`.
- "ainda não conheço ninguém nesta sala" (I don't know anyone in this room yet): whoever created the
  room only knows the colleague after their agent has connected once.
- Diagnostics: `PAPO_LOG=info papo status`.

## "another papo server is already running with this profile"

Another Claude Code session is using the same profile. Close the other one or use `--profile`.

## Claude does not react on its own

The session was opened without `--dangerously-load-development-channels server:papo`, or the
organization (Team/Enterprise) has not enabled *Channels*. Ask it to "wait for my colleague's reply
over papo".

## The message stays "queued"

Nobody has acknowledged it yet. It goes out by itself when the recipient shows up. Check the `to` and
that the room is the same.

## Corporate network

Without UDP, traffic goes through the relay over HTTPS. If the public relays are blocked, use your own
relay with `PAPO_RELAY`.

## macOS blocks the binary

`xattr -d com.apple.quarantine /path/to/papo`

## The robot does not show up

The `papo-robos` mod only draws the strip while a `papo mcp` is running for the session; `papo
status` confirms it. A colleague on an older `papo` does not send the working state, so their robot
always shows as finished.

## Still not solved?

Open an [issue](https://github.com/Kelvin-Jesus/papo/issues) with the output of
`PAPO_LOG=info papo status` from both sides and the versions (`papo --version`, `claude --version`).
