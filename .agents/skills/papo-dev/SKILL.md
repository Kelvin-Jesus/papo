---
name: papo-dev
description: Build, test and lint loop for the papo Rust codebase, plus the invariant checklist to run before declaring any change done. Use whenever editing src/ or tests/ in this repo, fixing a failing test, or before committing or opening a PR.
---

# papo dev loop

## Loop

```sh
cargo build
cargo test --features test-network          # all suites, no internet, ~40 s
cargo clippy --all-targets --features test-network -- -D warnings   # CI uses -D warnings
cargo fmt                                   # max_width 120 (rustfmt.toml)
```

Targeted runs (pick the suite that matches what you touched):

```sh
cargo test --lib                            # unit tests in src/
cargo test --doc                            # examples in the public API docs
cargo test --test properties                # proptest invariants (room, proto)
cargo test --test robustness                # arbitrary input never panics
cargo test --test store                     # on-disk profile state
cargo test --test node --test node_scenarios   # delivery over real iroh endpoints
cargo test --test mcp --test mcp_contract   # JSON-RPC against the built binary
cargo test --test cli                       # CLI commands and errors
cargo test --features test-network --test e2e_local   # two papo mcp processes, local relay
cargo test --test mcp -- --ignored          # public network e2e (needs internet, ~6 s)
```

Deeper checks before a release or after touching parsers/crypto:

```sh
cargo llvm-cov --features test-network --summary-only   # CI floor 90% lines
cargo deny check                            # advisories, licenses, sources
cargo +nightly fuzz run invite_decode -- -max_total_time=60   # also frame_decode, room_open
cargo mutants --features test-network --file src/room.rs      # surviving mutants = untested behavior
```

Snapshots (`tests/snapshots/`): after an intended change to `initialize`, `tools/list`, `--help` or
the `new` output, run `INSTA_UPDATE=always cargo test --features test-network` and review the diff.

Manual run against throwaway state (never touch `~/.papo` in experiments):

```sh
export PAPO_HOME="$(mktemp -d)"
target/debug/papo new --name voce
```

## Before declaring done

- [ ] `scripts/quality-gate.sh` ends with `quality gate: ok` (add `--full` when you touched `site/`).
      It covers fmt, clippy (both configs), tests, rustdoc, cargo-deny, names, commit subjects,
      workflows/scripts lint, the book, links and the site budget; CI adds MSRV, Lighthouse and gitleaks.
- [ ] `cargo fmt --check`, clippy with `-D warnings` and `cargo test --features test-network` pass.
- [ ] No person names anywhere (`scripts/check-names.sh`); commit subjects are `tipo(escopo): descrição`.
- [ ] A bug fix comes with a test that fails without it; new behavior has a test in the matching suite.
- [ ] Test hooks (`PAPO_TEST_*`, `test-network`) still default to production behavior.
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
