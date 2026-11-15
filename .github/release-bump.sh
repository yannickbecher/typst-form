#!/usr/bin/env bash
# The release step both jobs of .github/workflows/release.yml run: it works out
# the next version yy.nn.ff and writes it into every place that names it. With
# COMMIT=true it also commits. It runs no dependency code (only sed, awk, git
# and `cargo metadata`, which builds nothing).
#
# Environment:
#   KIND            feature | fix
#   COMMIT          true to commit the result (default: only change files)
#   EXPECT_VERSION  abort if the computed version differs (the second job uses
#                   this to be sure it releases what the first job tested)
set -euo pipefail

# Next version: from the latest tag and today's year. Without a tag yet, the
# first release is the version already in the files.
last=$(git tag --list 'v*' --sort=-v:refname | head -n1)
last=${last#v}
current=$(sed -n 's/^version = "\(.*\)"/\1/p' cli/Cargo.toml | head -n1)
if [ -z "$last" ]; then
  next=$current
else
  IFS=. read -r yy nn ff <<<"$last"
  year=$((10#$(date -u +%y)))
  if   [ "$year" -gt "$yy" ]; then next="$year.1.0"
  elif [ "${KIND:-feature}" = feature ]; then next="$yy.$((nn + 1)).0"
  else                                        next="$yy.$nn.$((ff + 1))"; fi
fi
if [ -n "${EXPECT_VERSION:-}" ] && [ "$EXPECT_VERSION" != "$next" ]; then
  echo "::error::version is $next now, the tests ran for $EXPECT_VERSION"
  exit 1
fi
echo "version=$next" >> "${GITHUB_OUTPUT:-/dev/null}"
echo "Release ${last:-none} -> $next"

# Write it everywhere.
sed -i "0,/^version = \".*\"/s//version = \"$next\"/" cli/Cargo.toml typst-package/typst.toml
{ grep -rlE '@local/typst-form:[0-9]+\.[0-9]+\.[0-9]+' \
    README.md typst-package book/src .github/ISSUE_TEMPLATE || true; } \
  | xargs -r sed -i -E "s#@local/typst-form:[0-9]+\.[0-9]+\.[0-9]+#@local/typst-form:$next#"

# Changelog (Keep a Changelog): [Unreleased] becomes the release and the compare
# links at the bottom move along. The very first release uses the entry that is
# already there. "Empty" means no entries, only headings and link lines.
repo="https://github.com/${GITHUB_REPOSITORY:-yannickbecher/typst-form}"
if [ -n "$last" ]; then
  body=$(awk '/^## /{if(f)exit} f{print} /^## \[Unreleased\]/{f=1}' CHANGELOG.md \
    | { grep -vE '^(###|\[)' || true; } | tr -d '[:space:]')
  [ -n "$body" ] || { echo "::error::write the changes under '## [Unreleased]' in CHANGELOG.md"; exit 1; }
  sed -i -e "s/^## \[Unreleased\].*/## [Unreleased]\n\n## [$next] - $(date -u +%F)/" \
         -e "s#^\[Unreleased\]: .*#[Unreleased]: $repo/compare/v$next...HEAD\n[$next]: $repo/compare/v$last...v$next#" \
    CHANGELOG.md
fi
grep -q "^## \[$next\] " CHANGELOG.md \
  || { echo "::error::CHANGELOG.md has no entry for $next"; exit 1; }

# The version bump changes the lock file; `cargo metadata` updates it without
# running any build script.
cargo metadata --manifest-path cli/Cargo.toml --format-version 1 > /dev/null

if [ "${COMMIT:-false}" = true ]; then
  git config user.name "github-actions[bot]"
  git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  git add -A
  git diff --cached --quiet || git commit -m "Release $next"
else
  git status --short
fi
