#!/bin/sh
# Labalaba deb pre-remove maintainer script.
# Runs as root before package files are removed.
# Best-effort: never fail the package removal.
#
# LIMITATION: this script runs as root, so labalaba-daemon resolves the root
# user's data dir (e.g. /root/.local/share/labalaba or CWD), not the
# installing user's dir. For per-user installs the daemon cleanup will find
# no running service to stop and exit cleanly — it is effectively a no-op.
# The primary uninstall path on Linux is the in-app Settings "Remove
# Background Service" button or running `labalaba-daemon cleanup` as the
# correct user before removing the package.

if command -v labalaba-daemon >/dev/null 2>&1; then
  labalaba-daemon cleanup || true
elif [ -x /usr/bin/labalaba-daemon ]; then
  /usr/bin/labalaba-daemon cleanup || true
fi

exit 0
