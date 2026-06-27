#!/bin/sh

set -e

if [ -n "${IPKG_INSTROOT}" ]; then
	exit 0
fi

rm -rf /tmp/luci-indexcache /tmp/luci-modulecache/* 2>/dev/null || true

exit 0
