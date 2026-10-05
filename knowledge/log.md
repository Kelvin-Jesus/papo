# Log

## 2026-10-05
* **Update**: `playbooks/cut-a-release` - `scripts/install.sh` depends on the archive naming, layout and `.sha256` format; smoke it after each release.
* **Update**: `apis/cli`, `protocol/*` - owner leaving closes the room; with nobody online a tombstone profile is kept and answers `close` to returning members (`Profile.owner`, `Store::into_tombstone`).
* **Update**: `apis/cli`, `protocol/wire-frames`, `protocol/profile-layout` - `papo leave`/`papo close`, frames `Bye`/`Close` and the `closed` marker file.

## 2026-10-02
* **Creation**: `playbooks/pass-the-quality-gate` - the required `quality gate` check (aggregate of every gating job in `ci.yml`), thresholds with the 2026-10-02 baseline, `scripts/quality-gate.sh` and the `.githooks/`. MSRV corrected to 1.91 (iroh 1.3's floor, confirmed by building with rust:1.91 and failing with 1.90).
* **Update**: `apis/cli`, `apis/channel-notification`, `apis/mcp-tools/{history,status,wait}`, `protocol/{wire-frames,profile-layout}`, `playbooks/run-public-e2e` - examples use the role profiles `voce` and `colega` instead of person names, per the maintainer's no-names rule (`design/brand-book.md`).
* **Creation**: `playbooks/run-in-docker` - building, smoke-testing and running papo in Docker, the MCP server from a container and the two-agent e2e via a local iroh-relay. Sourced from `Dockerfile`, `docker/`, `scripts/docker-*.sh` and `.github/workflows/docker.yml`.
* **Creation**: `playbooks/test-the-code` - test suites, coverage (94% of lines), fuzzing, mutation testing and supply-chain checks. Sourced from `tests/`, `fuzz/`, `benches/`, `deny.toml` and `.github/workflows/{ci,fuzz,mutants}.yml`.
* **Update**: `protocol/invite-encoding` - invites now end with a 4-byte integrity check (truncation bug found by a property test); one-peer invite length is 114 characters.
* **Update**: `apis/environment` - empty `PAPO_RELAY` means unset; new `PAPO_INSTALL_COMMAND`; test hooks `PAPO_TEST_PROGRESS_MS`, `PAPO_TEST_RELAY`, `PAPO_TEST_PEERS`.
* **Update**: `playbooks/add-an-mcp-tool` - contract tests, schema samples and the `tools_list` snapshot.
* **Creation**: Bundle scaffolded with concepts, APIs, protocol/data, dependencies, gotchas and playbooks for papo 0.1.0. Sourced from `src/*.rs`, `tests/*.rs`, `.github/workflows/*.yml`, `README.md` and the Claude Code channels documentation.
