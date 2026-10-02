---
name: papo-dev
description: Build, test and lint loop for the papo Rust codebase, plus the invariant checklist to run before declaring any change done. Use whenever editing src/ or tests/ in this repo, fixing a failing test, or before committing or opening a PR.
---

# papo dev loop

## Loop

```sh
cargo build
cargo test                                  # unit + integration (local relay, no internet), ~5 s
cargo clippy --all-targets -- -D warnings   # CI uses -D warnings
cargo fmt                                   # max_width 120 (rustfmt.toml)
```

Targeted runs:

```sh
cargo test --lib                            # unit tests in src/
cargo test --test node                      # delivery semantics over real iroh endpoints
cargo test --test mcp                       # JSON-RPC against the built binary
cargo test --test mcp -- --ignored          # public network e2e (needs internet, ~6 s)
```

Manual run against throwaway state (never touch `~/.papo` in experiments):

```sh
export PAPO_HOME="$(mktemp -d)"
target/debug/papo new --name ana
```

## Before declaring done

- [ ] `cargo fmt --check`, clippy with `-D warnings` and `cargo test` pass.
- [ ] Nothing in the `papo mcp` path prints to stdout (`knowledge/gotchas/stdout-is-json-rpc.md`).
- [ ] No peer handed to gossip (`subscribe` bootstrap / `join_peers`) without a live connection (`knowledge/gotchas/gossip-pending-dial.md`).
- [ ] Delivery order kept: outbox before broadcast; inbox + log before ack (`knowledge/concepts/delivery.md`).
- [ ] Push still never marks messages read.
- [ ] `SUPPORTED_PROTOCOLS` not raised past `2025-11-25` without checking channels docs.
- [ ] Channel `meta` keys still identifiers.
- [ ] Room secret still only in `profile.json`, never in MCP config or logs.
- [ ] Docs synced per the "Keep in sync" table in `AGENTS.md` (docs/referencia, README, knowledge/, ADR, CONTEXT.md).
- [ ] Tests that need the internet are `#[ignore]`.

Language: code, comments and agent-facing strings in English; CLI output, README, docs/ and ADRs in pt-BR.
