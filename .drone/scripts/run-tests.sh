#!/usr/bin/env bash
set -ex

.drone/scripts/setup-pbmpr.sh
sudo apt-get install cargo libssl-dev pkg-config -y

cargo fmt --check
cargo clippy -- -D warnings
