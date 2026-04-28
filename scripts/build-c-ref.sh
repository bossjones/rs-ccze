#!/usr/bin/env bash
# Build the Docker image that hosts the C ccze reference binary.
#
# After this completes, `docker run --rm -i ccze:reference ...` invokes the
# canonical C ccze used to mint snapshot .ok files.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "${HERE}/../.." && pwd)"

cd "${ROOT}"
docker build -f rust/scripts/Dockerfile -t ccze:reference .

echo
echo "Built ccze:reference. Smoke check:"
docker run --rm ccze:reference -V
