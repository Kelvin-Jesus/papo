#!/usr/bin/env bash
# The CI quality gate, locally. Runs every check that works offline-ish on a dev machine and
# prints a summary; exits non-zero if any check failed. Checks whose tool is missing are
# reported as "pulado" (skipped) with the reason, never silently.
#
#   scripts/quality-gate.sh           # everything except Lighthouse and the MSRV build
#   scripts/quality-gate.sh --full    # also Lighthouse CI (needs Node + Chrome/Chromium)
#
# Optional tools: cargo-deny, mdbook (or MDBOOK=/path), node/npx, actionlint, shellcheck.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

FULL=0
[[ ${1:-} == --full ]] && FULL=1

declare -a NAMES RESULTS
record() { NAMES+=("$1"); RESULTS+=("$2"); }

run() { # run <name> <command...>
  local name=$1
  shift
  printf '\n==> %s\n' "$name"
  if "$@"; then record "$name" ok; else record "$name" FALHOU; fi
}

skip() { # skip <name> <reason>
  printf '\n==> %s (pulado: %s)\n' "$1" "$2"
  record "$1" "pulado: $2"
}

have() { command -v "$1" >/dev/null 2>&1; }

# Rust
run "fmt" cargo fmt --check
run "clippy" cargo clippy --all-targets --locked -- -D warnings
run "clippy (test-network)" cargo clippy --all-targets --locked --features test-network -- -D warnings
run "testes" cargo test --locked --features test-network
run "rustdoc" env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked --features test-network -q
if have cargo-deny; then run "cargo-deny" cargo deny check all; else skip "cargo-deny" "instale com cargo install cargo-deny"; fi

# Repository rules
run "nomes de pessoas" scripts/check-names.sh
if git rev-parse --verify -q origin/main >/dev/null; then
  run "commits" python3 scripts/check-commits.py origin/main..HEAD
else
  skip "commits" "sem origin/main"
fi
if have actionlint; then run "workflows (actionlint)" actionlint -no-color; else skip "workflows (actionlint)" "actionlint não instalado"; fi
if have shellcheck; then run "shellcheck" shellcheck scripts/*.sh .githooks/*; else skip "shellcheck" "shellcheck não instalado"; fi

# Docs and site
if have "${MDBOOK:-mdbook}"; then
  run "livro e _site" scripts/build-site.sh
  run "links (site + livro)" python3 scripts/check-book-links.py _site
else
  skip "livro e _site" "mdbook não instalado (ou defina MDBOOK=)"
fi
run "orçamento do site" python3 scripts/check-site-budget.py site
if have npx; then
  run "HTML válido" npx --yes html-validate@11.16.1 site/index.html
  if ((FULL)) && [[ -d _site ]]; then
    run "Lighthouse" npx --yes @lhci/cli@0.15.1 autorun
  elif ((FULL)); then
    skip "Lighthouse" "_site não foi montado"
  fi
else
  skip "HTML válido" "Node/npx não instalado"
fi

printf '\n%-28s %s\n' "check" "resultado"
failed=0
for i in "${!NAMES[@]}"; do
  printf '%-28s %s\n' "${NAMES[$i]}" "${RESULTS[$i]}"
  [[ ${RESULTS[$i]} == FALHOU ]] && failed=1
done
if ((failed)); then
  echo $'\nquality gate: FALHOU'
  exit 1
fi
echo $'\nquality gate: ok'
