#!/usr/bin/env bash

set -ex
export NAME=ipv6_proxy
export SERVICE_SH=/usr/local/bin/$NAME.service.sh

URL=https://atomgit.com/i18n-in/ipv6_proxy/raw/dev

wget $URL/run.sh -O $SERVICE_SH

while true; do
  ipv6_proxy=$(which ipv6_proxy) && rm $ipv6_proxy || true
  if [[ -z "$ipv6_proxy" ]]; then
    break
  fi
done

cargo install --root /usr/local --force $NAME

cd /etc/systemd/system

wget $URL/service -O ipv6_proxy.service

curl -sSf $URL/_service.sh | bash

grep -q '^net.ipv6.ip_nonlocal_bind' /etc/sysctl.conf || echo 'net.ipv6.ip_nonlocal_bind=1' >>/etc/sysctl.conf

if [ ! -f "/usr/lib/networkd-dispatcher/routable.d/50-add-route" ]; then
  apt install -y networkd-dispatcher

  ipv6=$(ip -6 addr show dev eth0 | grep "inet6.*scope global" | awk '{print $2}' | sed 's/::1\//::\//')

  cat <<EOF >/usr/lib/networkd-dispatcher/routable.d/50-add-route
#!/bin/sh

if [ "\$IFACE" = "eth0" ]; then
    ip route add local $ipv6 dev eth0
fi
EOF

  chmod +x /usr/lib/networkd-dispatcher/routable.d/50-add-route
  systemctl enable --now networkd-dispatcher || true
  systemctl restart networkd-dispatcher
fi

sysctl -p
