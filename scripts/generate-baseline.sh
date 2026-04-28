#!/usr/bin/env bash
# Generate a snapshot .ok by feeding rust/testdata/<name>.in to the C ccze
# reference (running inside the ccze:reference Docker image) in debug mode.
#
# Usage:
#   rust/scripts/generate-baseline.sh <test-name> <plugin>[,<plugin>...]
# Example:
#   rust/scripts/generate-baseline.sh snap-php php
#
# Run rust/scripts/build-c-ref.sh once to build the image before invoking this.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <test-name> <plugin>[,<plugin>...]" >&2
  exit 64
fi

NAME="$1"
PLUGINS="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"
TESTDATA="${HERE}/../testdata"

IN="${TESTDATA}/${NAME}.in"
OK="${TESTDATA}/${NAME}.ok"

if [[ ! -f "${IN}" ]]; then
  echo "missing input: ${IN}" >&2
  exit 1
fi

if ! docker image inspect ccze:reference >/dev/null 2>&1; then
  echo "ccze:reference image not found. Run rust/scripts/build-c-ref.sh first." >&2
  exit 1
fi

docker run --rm -i ccze:reference \
  -F /dev/null -d -o nolookups -p "${PLUGINS}" \
  < "${IN}" > "${OK}"

echo "wrote ${OK} ($(wc -c < "${OK}") bytes)"
