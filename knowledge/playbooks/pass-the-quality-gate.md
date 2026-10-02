---
type: Playbook
title: Pass the quality gate
description: What the required `quality gate` check aggregates, the thresholds and why, how to run it locally, and how to add or change a gate.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/.github/workflows/ci.yml
tags: [papo, ci, quality, testing]
timestamp: 2026-10-02T00:00:00Z
---

# Pass the quality gate

The `ci` workflow ends with an aggregate job named exactly **`quality gate`**. It `needs:` every
gating job and fails if any of them ended in `failure`, `cancelled` or `skipped`. That single check
is the one to require on `main`.

## Gating jobs

| Job | Fails when |
| --- | ---------- |
| `test (ubuntu/macos/windows)` | fmt diff, clippy warning (default and `test-network`), any test fails (`--no-fail-fast`), benches or fuzz targets stop compiling |
| `coverage` | line coverage under 90% (`cargo llvm-cov --fail-under-lines 90`) |
| `msrv` | the crate stops building with `rust-version` from Cargo.toml (1.91: iroh 1.3's floor; 1.90 fails) |
| `rustdoc` | any rustdoc warning (`RUSTDOCFLAGS=-D warnings`) |
| `supply-chain` | cargo-deny: advisory, license, ban or source violation (`deny.toml`) |
| `docs` | any mdBook warning, any broken local link/anchor in `_site` (site + book), any Mermaid diagram that fails to render |
| `site` | gzip budget exceeded (HTML 30 KB, CSS 15 KB, JS 20 KB; font 160 KB raw), invalid HTML (html-validate recommended), Lighthouse desktop median under perf 0.90 / a11y 0.95 / best practices 0.95 / SEO 0.90, or any color-contrast failure |
| `rules` | a person name from the denylist anywhere in tracked files; a commit subject not matching `tipo(escopo): descrição` |
| `secrets` | gitleaks finds a secret anywhere in history |
| `lint (workflows and scripts)` | actionlint (with shellcheck on `run:` blocks) or shellcheck on `scripts/*.sh` and `.githooks/*` |

Informative, outside the gate: `e2e-public-network (informative)`, the path-filtered `docker`
workflow, scheduled `fuzz` and `mutants`.

## Run it locally

```sh
git config core.hooksPath .githooks   # once: pre-commit (fmt + names), commit-msg, pre-push (clippy + tests)
scripts/quality-gate.sh               # everything except Lighthouse and MSRV; prints a summary
scripts/quality-gate.sh --full        # adds Lighthouse CI (Node + Chrome/Chromium)
```

Missing optional tools (cargo-deny, mdbook via `MDBOOK=`, npx, actionlint, shellcheck) are reported
as skipped, never ignored silently. MSRV locally: `docker run --rm -v "$PWD":/w -w /w rust:1.91-slim cargo check --locked --all-targets --features test-network`.

## Baseline (2026-10-02)

- Lighthouse desktop, median of 3: performance 100, accessibility 100, best practices 100, SEO 100;
  LCP ~0.4 s, CLS 0. Accessibility was 97 until the "telefone sem fio" labels moved from
  `--tinta-3` to `--tinta-2` (4.14:1 and 4.4:1 on the tinted panels, below 4.5:1).
- Site budget (gzip): HTML 7.9 KB, CSS 7.1 KB, JS 11.8 KB; font 139 KB.
- Links: 2,127 local links in 51 pages of `_site`, 0 problems.
- Commit subjects: all 84 commits in history pass.
- gitleaks: 86 commits scanned, no leaks.

## Change a gate

- New blocking check: add a job to `ci.yml` and to the `quality-gate` job's `needs:`. A
  path-filtered workflow cannot be a required check (it does not run on every PR).
- Thresholds live in `ci.yml` (coverage), `.lighthouserc.json` (Lighthouse),
  `scripts/check-site-budget.py` (budget) and `scripts/check-names.sh` (denylist). Raise them when
  the code gets better; never lower one to make a change pass.
- Update the gates table in `docs/contribuindo.md` and the reasons in
  `docs/engenharia/desenvolvimento.md` in the same commit.
