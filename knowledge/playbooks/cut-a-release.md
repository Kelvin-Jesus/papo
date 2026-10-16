---
type: Playbook
title: Cut a release
description: Bump the version, push a vX.Y.Z tag, verify the archives, deb/rpm and checksums, and the package-manager publication.
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

Linux builds also ship `papo_<version>_<amd64|arm64>.deb` and `papo-<version>-1.<x86_64|aarch64>.rpm` (nfpm, `packaging/nfpm.yaml`), each with a `.sha256`.

## Package managers

After `publish`, the `pacotes` job runs `scripts/pacotes.py <tag> dist pacotes`, which turns the `.sha256` files into the Homebrew formula, the AUR `PKGBUILD`/`.SRCINFO` (`papo-bin`) and the Scoop manifest, then pushes each one. Every push step is skipped when its secret is missing, so a release never fails on a manager that is not set up yet.

| Destination | Install command | Secret | One-time setup |
|---|---|---|---|
| this repo, `Formula/papo.rb` | `brew tap kelvin-jesus/papo https://github.com/Kelvin-Jesus/papo` then `brew install papo` | `PACOTES_DEPLOY_KEY` | done: deploy key with write access, allowed to bypass the `main` ruleset |
| this repo, `bucket/papo.json` | `scoop bucket add papo https://github.com/Kelvin-Jesus/papo` then `scoop install papo` | `PACOTES_DEPLOY_KEY` | same key |
| AUR `papo-bin` | `yay -S papo-bin` | `AUR_SSH_KEY` | AUR account with that key's public half |
| crates.io `papo` | `cargo install papo` / `cargo binstall papo` | `CARGO_REGISTRY_TOKEN` | crates.io account, token scoped to `publish-new`/`publish-update` |

The formula and the manifest live in this repo because Homebrew taps and Scoop buckets can be any git repo; `main` requires the quality gate and GitHub Actions cannot bypass rulesets on a personal repo, so the commit uses the `PACOTES_DEPLOY_KEY` deploy key, which the ruleset lists as a bypass actor. To check a manifest before pushing: `python3 scripts/pacotes.py vX.Y.Z <dir with the .sha256 files> /tmp/pacotes`, then `makepkg --printsrcinfo | diff - .SRCINFO` and `makepkg -f` in `/tmp/pacotes/aur`.

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
gh release view vX.Y.Z --json assets -q '.assets[].name'   # expect 18 files (10 archives/checksums + 4 deb/rpm + 4 checksums)
```

Dry run without publishing: `gh workflow run release --ref main`, then `gh run download <run-id>`. Artifacts are named after the ref (`papo-main-<target>`).
