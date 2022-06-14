#!/usr/bin/env bash
set -ex

if echo "${DRONE_COMMIT_MESSAGE}" | grep -q 'GH SKIP'; then
    echo "Skipping GitHub release creation!"
    exit 0
fi

.drone/scripts/setup-pbmpr.sh
sudo apt-get install gh parse-changelog -y

source makedeb/PKGBUILD

release_notes="$(parse-changelog CHANGELOG.md "${pkgver}")"
echo "${github_api_key}" | gh auth login --with-token
gh release create "v${pkgver}" --title "v${pkgver}" --target "${DRONE_COMMIT_SHA}" -n "${release_notes}"

# vim: set sw=4 expandtab:
