#!/usr/bin/env bash
# Run generate-baseline.sh for every untested plugin. Assumes each plugin has
# a hand-crafted rust/testdata/snap-<plugin>.in already present.
#
# Partial-type plugins (apm, fetchmail, postfix, ulogd) only run after a Full
# plugin matches the line, so their .in fixtures use a syslog-wrapped form and
# the -p argument bundles syslog with the plugin.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

# Format: "<snap-name> <plugins-csv>"
ENTRIES=(
  "snap-apm syslog,apm"
  "snap-distcc distcc"
  "snap-exim exim"
  "snap-fetchmail syslog,fetchmail"
  "snap-ftpstats ftpstats"
  "snap-icecast icecast"
  "snap-oops oops"
  "snap-php php"
  "snap-postfix syslog,postfix"
  "snap-proftpd proftpd"
  "snap-squid squid"
  "snap-sulog sulog"
  "snap-super super"
  "snap-ulogd syslog,ulogd"
  "snap-vsftpd vsftpd"
  "snap-xferlog xferlog"
)

for entry in "${ENTRIES[@]}"; do
  read -r name plugins <<<"${entry}"
  echo "=== generating ${name} (-p ${plugins}) ==="
  "${HERE}/generate-baseline.sh" "${name}" "${plugins}"
done
