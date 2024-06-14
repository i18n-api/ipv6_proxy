#!/usr/bin/env bash

set -ex
export SERVICE_SH=/usr/local/bin/$NAME.service.sh

URL=https://raw.githubusercontent.com/i18n-api/ipv6_proxy/main/sh/os

curl -sSf $URL/init.sh | bash

cd /etc/systemd/system

get() {
  rm -rf $1
  mkdir -p $(dirname $1)
  wget $URL$1 -O $1
}

get /etc/systemd/system/ipv6_proxy.service
ipv6_proxy_sh=/opt/bin/ipv6_proxy.sh
get $ipv6_proxy_sh
chmod +x $ipv6_proxy_sh

systemctl daemon-reload
systemdctl enable --now ipv6_proxy
