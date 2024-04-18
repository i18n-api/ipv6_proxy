#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -ex

./build.sh

mkdir -p os/opt
rm -rf os/opt/bin
mv bin os/opt

NAME=$(cargo metadata --format-version=1 --no-deps | jq '.packages[] | .name' -r)

sh=os/opt/bin/$NAME.sh
cat <<EOF | tee $sh
#!/usr/bin/env bash
set -o allexport
$(cat env.sh)
set +o allexport
. /root/i18n/conf/env/$NAME.sh
$(tail -n +2 ./run.sh)
EOF

chmod +x $sh
case "$(uname -s)" in
"Darwin")
  OS="apple-darwin"
  ;;
"Linux")
  (ldd --version 2>&1 | grep -q musl) && clib=musl || clib=gun
  OS="unknown-linux-$clib"
  ;;
"MINGW*" | "CYGWIN*")
  OS="pc-windows-msvc"
  ;;
*)
  echo "Unsupported System"
  exit 1
  ;;
esac

ARCH=$(uname -m)

if [[ "$ARCH" == "arm64" || "$ARCH" == "arm" ]]; then
  ARCH="aarch64"
fi
TZT=$ARCH-$OS.tar.zst
cd os
ZSTD_CLEVEL=19 tar -I zstd -cvpf ../$TZT .
cd ..

# set +x
# . $ROOT/../../conf/env/GITHUB_TAR.sh
# $DIR/encrypt.sh $TZT $TZT_PASSWORD
# set -x

VER=$(cargo metadata --format-version=1 --no-deps | jq '.packages[] | .version' -r)
LOG=../log/$VER.md

if [ -f "$LOG" ]; then
  NOTE="-F $LOG"
else
  NOTE="-n $VER"
fi

gh release create -d $VER $NOTE
gh release upload $VER $TZT
gh release edit $VER --draft=false
rm -rf $TZT
