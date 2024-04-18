#!/usr/bin/env bash

set -ex

ipv6=$(ip -6 addr show dev eth0 | grep "inet6.*scope global" | awk '{print $2}' | sed 's/::1\//::\//')

exec /opt/bin/ipv6_proxy -b 0.0.0.0:$IPV6_PROXY_PORT -i $ipv6
