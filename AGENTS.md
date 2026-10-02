# AGENTS.md

**papo** is a single Rust binary that gives the Claude Code sessions of different people a direct,
end-to-end encrypted P2P line: an MCP server (`papo mcp`) plus a human CLI. Transport is iroh QUIC
with iroh-gossip; incoming messages are pushed into the session through Claude Code channels.

## Read first

| Need | Where |
| ---- | ----- |
| Domain vocabulary (room, invite, profile, member, outbox...) | `CONTEXT.md` (rules: `docs/agents/domain.md`) |
| Why things are the way they are | `docs/adr/` (index: `docs/adr/README.md`) |
| Asset knowledge: tools, frames, file layout, dependencies, gotchas, playbooks | `knowledge/index.md` (OKF bundle; rules: `docs/agents/knowledge.md`) |
| Human docs (pt-BR, published at https://kelvin-jesus.github.io/papo/docs/) | `docs/` (mdBook, `docs/SUMMARY.md`) |
| Issues | GitHub Issues on `Kelvin-Jesus/papo` (`docs/agents/issue-tracker.md`) |
| Project skills | `.agents/skills/` (also exposed as `.claude/skills`) |

## Commands

```sh
cargo build
cargo test --features test-network       # every suite incl. hermetic two-process e2e; no internet
cargo test --test mcp -- --ignored       # two real MCP servers over the public iroh network (~6 s)
cargo clippy --all-targets --features test-network -- -D warnings
INSTA_UPDATE=always cargo test --features test-network   # re-record snapshots after an intended API change
cargo llvm-cov --features test-network --summary-only    # coverage (CI floor: 90% of lines)
cargo deny check                         # advisories, licenses, sources (deny.toml)
cargo fmt                                # rustfmt.toml: max_width 120
PAPO_LOG=iroh_gossip=debug,iroh=info target/debug/papo status   # network diagnostics on stderr
docker build --target test .             # test suite in a reproducible Linux container
scripts/docker-smoke.sh                  # image smoke test: --version, new, MCP over stdin, profile lock
scripts/docker-e2e.sh [--public]         # two agents in containers via a local relay (or the public one)
```

## Map

| File | Responsibility |
| ---- | -------------- |
| `src/room.rs` | Room secret, invite encoding, key/topic derivation, frame sealing (XChaCha20-Poly1305). |
| `src/proto.rs` | Wire frames (`Hello`, `Msg`, `Ack`), size limits, name validation. |
| `src/store.rs` | Profile directory: identity, known peers, inbox/outbox, append-only log, profile lock. |
| `src/node.rs` | Room member: gossip, self-dialing reconnect, delivery with acks, inbox/outbox, presence. |
| `src/net.rs` | Endpoint on the public network (n0 preset, optional `PAPO_RELAY`). |
| `src/mcp.rs` | Hand-rolled MCP stdio server: tools, `claude/channel` push, long-poll `wait`. |
| `src/main.rs` | CLI (user-facing text in pt-BR). |
| `tests/common/` | Shared harness: `McpClient` drives the binary over stdio; `localnet` = in-process relay + `RawPeer` that speaks the wire format. |
| `tests/node.rs`, `tests/node_scenarios.rs` | Delivery over real iroh endpoints: acks, queue, duplicates, restarts, crash, ordering, multi-member. |
| `tests/mcp.rs`, `tests/mcp_contract.rs` | MCP contract against the built binary; insta snapshots in `tests/snapshots/`. |
| `tests/cli.rs`, `tests/store.rs` | CLI commands/errors; on-disk profile state. |
| `tests/properties.rs`, `tests/robustness.rs` | proptest invariants; arbitrary input never panics. |
| `tests/e2e_local.rs` | Two `papo mcp` processes + CLI over a local relay (`test-network` feature). |
| `fuzz/`, `benches/` | cargo-fuzz targets (nightly); criterion benchmarks. |

## Invariants (breaking these causes real bugs)

- **stdout belongs to JSON-RPC** in `papo mcp`. Diagnostics go to stderr (`eprintln!`/tracing).
- **Never give peers to gossip as bootstrap or `join_peers` before a connection exists.**
  iroh-gossip 0.101 dials once and gets stuck if that dial fails. `node::connect_peer` dials with
  the gossip ALPN and hands the connection to `Gossip::handle_connection` first (ADR 0003).
- **Persist before ack**: a receiver writes the inbox and log, then broadcasts `Ack`. The sender
  writes the outbox before broadcasting.
- **Pushed messages stay unread** until `wait`/`inbox` returns them or the agent replies with
  `reply_to`, because Claude Code drops channel notifications silently when channels are off.
- **Do not negotiate MCP protocol revisions newer than `SUPPORTED_PROTOCOLS[0]`** without checking
  that Claude Code still registers such servers as channels (2026-07-28 is known not to).
- Channel `meta` keys must be identifiers (`[A-Za-z0-9_]`); others are dropped by Claude Code.
- One MCP server per profile (file lock). Ephemeral CLI nodes (`say`, `status`) use throwaway
  identities and never store, ack or get remembered as members.
- The room secret never goes into Claude's MCP config, only into `~/.papo/profiles/<p>/profile.json` (0600).

Details and evidence for each: `knowledge/gotchas/`.

## Workflows

- **Build/test loop and pre-finish checklist**: skill `papo-dev`.
- **Add or change an MCP tool**: skill `papo-mcp-tool` (playbook `knowledge/playbooks/add-an-mcp-tool.md`).
- **Change the wire protocol** (`src/proto.rs`, `src/room.rs`): adding optional fields is backward
  compatible (serde ignores unknown fields); a new `Frame` variant is dropped by older peers; changing
  the AAD or `derive_key` contexts splits old and new peers. Record breaking changes in a new ADR and
  update `docs/referencia/protocolo.md` and `knowledge/protocol/`.
- **Connectivity problems**: skill `papo-network-debug`.
- **Release**: skill `papo-release` (tag `vX.Y.Z`, `.github/workflows/release.yml`).

## Keep in sync (same change, same commit)

| When you change | Also update |
| --------------- | ----------- |
| CLI commands or flags (`src/main.rs`) | `docs/referencia/cli.md`, README command table, `knowledge/apis/cli.md` |
| MCP tools, results or instructions (`src/mcp.rs`) | `docs/referencia/ferramentas-mcp.md`, README tools table, `knowledge/apis/` |
| Env vars or on-disk files | `docs/referencia/configuracao.md`, `knowledge/apis/environment.md`, `knowledge/protocol/profile-layout.md` |
| Frames, sealing, invites | `docs/referencia/protocolo.md`, `knowledge/protocol/` |
| A decision recorded in an ADR | a new or superseding ADR in `docs/adr/` (+ `docs/adr/README.md`, `docs/SUMMARY.md`) |
| A new domain term | `CONTEXT.md` |
| Anything in `knowledge/` | a dated entry in `knowledge/log.md` |

## Conventions

- Code, comments and agent-facing text (MCP instructions, tool descriptions/results, `knowledge/`)
  in English; human CLI output, README, `docs/` and ADRs in Brazilian Portuguese.
- **No person names** in docs, examples, tests, the site or the design system (maintainer's rule):
  use roles ("você", "seu colega", "seu agente", "agente do colega") and the profiles `voce` and
  `colega` (`papo join <convite> --name colega`). See `design/brand-book.md`, section "Nomes".
- Comments explain why, not what. Errors are returned with context (`anyhow::Context`), never
  swallowed. Background persistence failures are logged to stderr.
- Prefer integration tests with real endpoints over mocks; anything touching the internet is
  `#[ignore]`. Test hooks (`PAPO_TEST_*`, the `test-network` feature) must default to production
  behavior. A bug fix lands with a test that fails without it.
- Commits: Conventional Commits in Portuguese (`feat(node): ...`, `docs: ...`).
