---
name: papo-release
description: Cut a papo release - bump the version in Cargo.toml, tag vX.Y.Z, watch the release workflow with gh and verify the five platform archives and checksums. Use when asked to release, publish binaries, tag a version, or check why a release build failed.
---

# Release papo

Full reference: `knowledge/playbooks/cut-a-release.md`. Workflow: `.github/workflows/release.yml` (tags `v*` publish; manual dispatch only builds).

1. **Preconditions**: on `main`, clean tree, CI green (`gh run list --workflow ci --limit 3`).
2. **Bump**: set `version = "X.Y.Z"` in `Cargo.toml`, run `cargo build` to refresh `Cargo.lock`.
3. **Check**: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`.
4. **Commit and tag** (ask the user before pushing a tag: it publishes a public release):
   ```sh
   git commit -am "chore: release vX.Y.Z"
   git tag vX.Y.Z
   git push origin main vX.Y.Z
   ```
5. **Watch**:
   ```sh
   gh run watch "$(gh run list --workflow release --limit 1 --json databaseId -q '.[0].databaseId')"
   ```
   On failure: `gh run view <id> --log-failed`. Linux targets build with `cargo zigbuild`; macOS and Windows natively.
6. **Verify** 10 assets (5 archives + 5 `.sha256`):
   ```sh
   gh release view vX.Y.Z -R Kelvin-Jesus/papo --json assets -q '.assets[].name'
   ```
   Expected archives: `papo-vX.Y.Z-{x86_64,aarch64}-unknown-linux-musl.tar.gz`, `papo-vX.Y.Z-{x86_64,aarch64}-apple-darwin.tar.gz`, `papo-vX.Y.Z-x86_64-pc-windows-msvc.zip`.
7. Smoke-test the installer against the new release: `sh scripts/install.sh --dir "$(mktemp -d)"` must
   print the new version (it resolves the latest tag and checks the `.sha256`).

Dry run without publishing: `gh workflow run release --ref main -R Kelvin-Jesus/papo`, then `gh run download <id>`.
