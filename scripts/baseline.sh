#!/usr/bin/env bash
# Measure a running Ryotunes process. Usage: scripts/baseline.sh [process-name]
set -u
name="${1:-ryotunes}"
pids=$(pgrep -x "$name" || true)
if [ -z "$pids" ]; then
  echo "no process named $name" >&2
  exit 1
fi
main=$(echo "$pids" | head -1)
pss=$(awk '/^Pss:/ {print $2}' "/proc/$main/smaps_rollup" 2>/dev/null)
pss=${pss:-0}
threads=$(awk '/^Threads:/ {print $2}' "/proc/$main/status")
threads=${threads:-0}
count=$(echo "$pids" | wc -l)
echo "pid=$main"
echo "pss_kib=$pss"
echo "pss_mib=$((pss / 1024))"
echo "threads=$threads"
echo "processes=$count"
ps -o pid=,comm= -p "$(echo "$pids" | paste -sd, -)"
