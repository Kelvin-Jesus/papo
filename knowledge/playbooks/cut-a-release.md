---
type: Playbook
title: Cut a release
description: Bump the version, push a vX.Y.Z tag and verify the five platform archives and checksums published by the release workflow.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/.github/workflows/release.yml
tags: [papo, release, ci]
timestamp: 2026-10-02T00:00:00Z
---

# Cut a release

The `release` workflow builds on tag push (`v*`) or manual dispatch and publishes a GitHub Release only for tags.

| Target | Runner | Archive |
|---|---|---|
| `x86_64-unknown-linux-musl` | ubuntu-latest, cargo-zigbuild | `.tar.gz` |
| `aarch64-unknown-linux-musl` | ubuntu-latest, cargo-zigbuild | `.tar.gz` |
| `x86_64-apple-darwin` | macos-latest | `.tar.gz` |
| `aarch64-apple-darwin` | macos-latest | `.tar.gz` |
| `x86_64-pc-windows-msvc` | windows-latest | `.zip` |

Each archive is `papo-<tag>-<target>` containing the binary, `README.md` and `LICENSE`, plus a `.sha256` file.

`scripts/install.sh` (the `curl | sh` installer in the README) builds these names itself and resolves the latest tag by following the `/releases/latest` redirect. Changing the archive naming, the `<dir>/papo` layout inside the tarball or the `.sha256` format breaks it; after a release, smoke it with `sh scripts/install.sh --dir "$(mktemp -d)"`.

# Examples

```sh
# 1. Bump version = "X.Y.Z" in Cargo.toml, then refresh Cargo.lock
cargo build
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked

# 2. Commit and tag
git commit -am "chore: release vX.Y.Z"
git tag vX.Y.Z
git push origin main vX.Y.Z

# 3. Watch and verify
gh run watch "$(gh run list --workflow release --limit 1 --json databaseId -q '.[0].databaseId')"
gh release view vX.Y.Z --json assets -q '.assets[].name'   # expect 10 files
```

Dry run without publishing: `gh workflow run release --ref main`, then `gh run download <run-id>`. Artifacts are named after the ref (`papo-main-<target>`).
