#!/usr/bin/env bash
# Bump ghurni's version.
#
# VERSION at the repo root is the single source of truth; cyrius.cyml derives it
# via `version = "${file:VERSION}"`, so there is nothing else to edit. This
# script writes VERSION, restamps the distlib bundle (whose header carries the
# version string consumers see), and then verifies the whole tree agrees.
#
# It does NOT commit, tag or push — that is the maintainer's call.
#
# Usage: scripts/version-bump.sh 2.0.3
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>   (e.g. $0 2.0.3)" >&2
    exit 1
fi

NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Semver, optionally with a pre-release / build suffix. release.yml's tag filter
# is '[0-9]+.[0-9]+.[0-9]+*', so keep the numeric core strict.
if ! printf '%s' "$NEW_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$'; then
    echo "error: '$NEW_VERSION' is not a semver version" >&2
    exit 1
fi

OLD_VERSION="$(tr -d '[:space:]' < VERSION)"
if [ "$NEW_VERSION" = "$OLD_VERSION" ]; then
    echo "error: already at $NEW_VERSION" >&2
    exit 1
fi

# No trailing newline: release.yml strips whitespace, but keeping the file byte
# for byte as it has always been avoids a spurious diff.
printf '%s' "$NEW_VERSION" > VERSION

# dist/ghurni.cyr carries the version in its header, and it is what consumers
# actually compile — a stale bundle ships the old number.
if command -v cyrius >/dev/null 2>&1; then
    cyrius distlib >/dev/null
else
    echo "warning: cyrius not on PATH; run 'cyrius distlib' before tagging" >&2
fi

# Verify the tree agrees with itself, the same way release.yml will.
FILE_VERSION="$(tr -d '[:space:]' < VERSION)"
CYML_RAW="$(grep '^version = ' cyrius.cyml | head -1 | sed 's/version = "\(.*\)"/\1/')"
if [ "$CYML_RAW" != '${file:VERSION}' ] && [ "$CYML_RAW" != "$FILE_VERSION" ]; then
    echo "error: cyrius.cyml version ($CYML_RAW) disagrees with VERSION ($FILE_VERSION)" >&2
    exit 1
fi

echo "Bumped $OLD_VERSION -> $NEW_VERSION"
echo
echo "Before tagging:"
echo "  1. Add a '## [$NEW_VERSION]' section to CHANGELOG.md — release.yml pulls"
echo "     the release body from it, and a missing section ships an empty release."
echo "  2. Refresh the measured numbers in docs/development/state.md."
echo "  3. cyrius audit   (must exit 0)"
echo "  4. git add -A && git commit && git tag $NEW_VERSION"
