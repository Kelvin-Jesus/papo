#!/usr/bin/env bash
# Builds the book and assembles _site/ exactly like .github/workflows/pages.yml:
#   _site/            <- site/ (landing page)
#   _site/assets/logo <- assets/logo (without the source/ folder)
#   _site/docs/       <- mdbook build docs
# Fails if mdBook prints any warning, so a broken SUMMARY or include never ships.
# Uses `mdbook` from PATH, or MDBOOK=/path/to/mdbook.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
MDBOOK="${MDBOOK:-mdbook}"

rm -rf docs/book _site
log=$("$MDBOOK" build docs 2>&1)
echo "$log"
if grep -qE 'WARN|ERROR' <<<"$log"; then
  echo "build-site: o mdBook emitiu avisos; corrija antes de publicar" >&2
  exit 1
fi

mkdir -p _site/assets
cp -R site/. _site/
cp -R assets/logo _site/assets/logo
rm -rf _site/assets/logo/source
cp -R docs/book _site/docs
touch _site/.nojekyll
echo "build-site: _site pronto"
