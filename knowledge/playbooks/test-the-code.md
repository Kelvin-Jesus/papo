---
type: Playbook
title: Test the code
description: Which test suite covers what, the commands to run them, and how to use snapshots, coverage, fuzzing, mutation testing and cargo-deny.
resource: https://github.com/Kelvin-Jesus/papo/tree/main/tests
tags: [papo, testing, ci]
timestamp: 2026-10-02T00:00:00Z
---

# Test the code

`cargo test --features test-network` runs every suite that needs no internet (about 40 s). The feature only adds the hermetic two-process e2e; it compiles a local-network hook into the binary that release builds never contain.

| Suite | File | Pins down |
|---|---|---|
| Unit | `src/*.rs` (`mod tests`) | Pure functions: sealing, invite, frames, formatting, rate limiter, CLI helpers, relay override |
| Doc tests | examples in `src/room.rs`, `src/proto.rs` | The documented public API works as shown |
| Properties | `tests/properties.rs` (proptest) | Invite roundtrip and truncation rejection, single-bit tamper detection, frame roundtrip, name and addressing rules |
| Robustness | `tests/robustness.rs` | Arbitrary input never panics; `papo mcp` survives random JSON-RPC barrages and only writes JSON to stdout |
| Store | `tests/store.rs` | Atomic writes, corrupt/torn files, profile lock, 0600 permissions |
| Node | `tests/node.rs`, `tests/node_scenarios.rs` | Delivery, acks, queue, duplicates, echoes, foreign frames, restart, crash, ordering, concurrency, 4-member chain |
| MCP contract | `tests/mcp.rs`, `tests/mcp_contract.rs` | JSON-RPC behavior, tool JSON Schemas, progress, push backlog, history/status text, snapshots |
| CLI | `tests/cli.rs` | Commands, error messages, files left on disk, `install` against a fake `claude` |
| E2E (local) | `tests/e2e_local.rs` | Two real `papo mcp` processes plus `say`/`status` through an in-process relay |
| E2E (internet) | `cargo test --test mcp -- --ignored` | The same over n0 DNS and relays |

Shared harness: `tests/common/mod.rs` (`McpClient` drives the binary; sessions end by EOF so the shutdown path and child-process coverage are exercised) and `tests/common/localnet.rs` (`LocalNet` with real endpoints on a private relay, `RawPeer` to inject frames a well-behaved node never sends).

# Examples

```sh
cargo test --features test-network                    # everything offline
cargo test --test node_scenarios                      # one suite
INSTA_UPDATE=always cargo test --features test-network   # re-record snapshots after an intended change, then review git diff
cargo llvm-cov --features test-network --summary-only # coverage; CI fails under 90% of lines (94% measured)
cargo deny check                                      # advisories, licenses, sources (deny.toml)
cargo +nightly fuzz run invite_decode -- -max_total_time=60   # also frame_decode, room_open
cargo mutants --features test-network --file src/room.rs      # surviving mutants = behavior no test pins down
cargo bench                                           # criterion: seal/open, frames, invites
```

Notes:

* A bug fix lands with a test that fails without it. Two were found this way: [truncated invites accepted](/protocol/invite-encoding.md) (property test) and a spurious "gossip subscription closed" on clean CLI exits (e2e).
* `cargo-mutants` without `--in-place` copies the tree and builds in `$TMPDIR`; on a small tmpfs it runs out of space. Use `--in-place` locally or point `TMPDIR` at a real disk.
* Test hooks must default to production behavior: `PAPO_TEST_PROGRESS_MS`, and `PAPO_TEST_RELAY`/`PAPO_TEST_PEERS` which exist only with the `test-network` feature ([environment](/apis/environment.md)).
* CI: `ci.yml` (tests on Linux/macOS/Windows, coverage, cargo-deny), `fuzz.yml` (60 s per target on parser changes, 30 min weekly), `mutants.yml` (weekly and on demand).
