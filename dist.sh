#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

set -ex

if ! [ -x "$(command -v cargo-v)" ]; then
  cargo install cargo-v
fi

./clippy.sh
cargo v patch -y

tag=$(git describe --tags $(git rev-list --tags --max-count=1))

rm Cargo.lock
git add .
git commit -m. || true
git push
git push github main $tag
# cargo publish --registry crates-io || true
