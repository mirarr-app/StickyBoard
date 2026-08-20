#!/usr/bin/env bash
set -euo pipefail

text=${1-}
if [[ -z ${text} ]]; then
  echo "No note text" >&2
  exit 1
fi

if command -v stickyboard >/dev/null 2>&1; then
  exec stickyboard new --text "${text}"
fi

for candidate in "${HOME}/.local/bin/stickyboard" /usr/bin/stickyboard /usr/local/bin/stickyboard; do
  if [[ -x ${candidate} ]]; then
    exec "${candidate}" new --text "${text}"
  fi
done

echo "stickyboard not found. Is StickyBoard installed?" >&2
exit 127
