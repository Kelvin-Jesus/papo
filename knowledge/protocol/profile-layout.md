---
type: On-disk Data Layout
title: Profile layout
description: Files kept per profile under $PAPO_HOME/profiles/<profile>/ and the write discipline of each.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/store.rs
tags: [papo, storage]
timestamp: 2026-10-02T00:00:00Z
---

# Profile layout

Root: `$PAPO_HOME/profiles/<profile>/` (default `~/.papo/profiles/default/`). On Unix the directory is created 0700. See [profile](/concepts/profile.md).

# Schema

| File | Content | Write discipline |
|---|---|---|
| `profile.json` | `{name, about?, room (base32 secret), created_ms}` | Atomic (tmp + rename), mode 0600. |
| `secret.key` | 32-byte iroh endpoint secret key, lowercase hex | Generated on first use, atomic, mode 0600; kept by `--force`. |
| `peers.json` | Map endpoint id -> `{name, last_seen_ms}` | Atomic; unparseable ids are skipped on read. |
| `inbox.json` | Array of envelopes not yet consumed by the agent | Atomic, rewritten on every change. |
| `outbox.json` | Array of envelopes not yet acked | Atomic, rewritten on every change. |
| `log.jsonl` | One JSON entry per line, tagged by `ev` | Append-only; each line is one `write_all` with O_APPEND so `papo mcp` and `papo say` never interleave. A torn last line is ignored on read. |
| `closed` | Empty marker | Written when a member closes the room; the node starts in tombstone mode (no dialing, answers `close` to whoever connects), `send` fails. |
| `lock` | Empty | Exclusive `File::try_lock` held by the running MCP server. |

`log.jsonl` entries:

```json
{"ev":"in","msg":{...envelope...}}
{"ev":"out","msg":{...envelope...}}
{"ev":"delivered","id":"a51b766958","by":"colega","ts":1790000000000}
```

Envelope fields: [wire frames](/protocol/wire-frames.md).
