#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: scripts/verify-url.sh <http(s) URL>" >&2
  exit 2
fi

node "$(dirname "$0")/verify-url.mjs" "$1"
