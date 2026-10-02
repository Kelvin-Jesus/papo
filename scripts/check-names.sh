#!/usr/bin/env bash
# Quality gate: no person names anywhere in the repository (maintainer's rule; see the
# "Nomes" section of design/brand-book.md). Examples use roles ("você", "seu colega") and
# the profiles `voce` and `colega`. Exits non-zero and prints file:line for every hit.
#
# The denylist holds the names that were used as example characters before the rule and
# common placeholder names. "Kelvin" is not listed on purpose: it is the author and the
# GitHub owner (Kelvin Jesus, Kelvin-Jesus), which must stay.
set -euo pipefail

DENYLIST='ana|anas|bob|carol|dave|bruno|alice|maria|joao|joão'

cd "$(git rev-parse --show-toplevel)"

# -I skips binaries; -w matches whole words only, so "análise" or "banana" never match.
if hits=$(git grep -n -I -i -w -E "$DENYLIST" -- . ':!scripts/check-names.sh'); then
  echo "Nomes de pessoas encontrados (use papéis: você, seu colega; perfis voce e colega):"
  echo "$hits"
  exit 1
fi
echo "names: ok (nenhum nome de pessoa)"
