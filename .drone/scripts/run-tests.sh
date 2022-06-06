#!/usr/bin/env bash
set -ex

.drone/scripts/setup-pbmpr.sh
sudo apt-get install cargo -y

cargo fmt --check
cargo clippy -- -D warnings
