#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

set -ex

if ! [ -x "$(command -v cargo-v)" ]; then
  cargo install cargo-v
fi

cargo build
cargo v patch -y

git describe --tags $(git rev-list --tags --max-count=1) | xargs git tag -d

rm Cargo.lock
git add -u
git commit -m. || true
git push
cargo publish --registry crates-io || true
