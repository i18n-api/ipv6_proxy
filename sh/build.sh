#!/usr/bin/env bash
set -ex
PWD=$(dirname $(realpath $BASH_SOURCE))

set -o allexport
. flag.sh
set +o allexport

rm -rf bin
mkdir -p bin

cargo build \
  --release \
  --out-dir bin \
  -Z unstable-options
