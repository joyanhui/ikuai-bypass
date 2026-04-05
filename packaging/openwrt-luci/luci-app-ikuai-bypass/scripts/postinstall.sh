#!/bin/sh

set -e

if [ -n "${IPKG_INSTROOT}" ]; then
	exit 0
fi

rm -rf /tmp/luci-indexcache /tmp/luci-modulecache/* 2>/dev/null || true
/etc/init.d/rpcd reload >/dev/null 2>&1 || true
/etc/init.d/uhttpd reload >/dev/null 2>&1 || true

exit 0
