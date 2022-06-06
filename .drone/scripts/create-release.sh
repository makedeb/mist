#!/usr/bin/env bash
set -ex

.drone/scripts/setup-pbmpr.sh
sudo apt-get install gh parse-changelog -y

source makedeb/PKGBUILD

release_notes="$(parse-changelog CHANGELOG.md "${pkgver}")"
echo "${github_api_key}" | gh auth login --with-token
gh release create --target "${DRONE_COMMIT_SHA}" -n "${release_notes}" "v${pkgver}"
