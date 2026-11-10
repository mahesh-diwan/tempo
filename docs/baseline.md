# Ryotunes baseline

Recorded on 2026-11-10. These numbers exist only for later comparison against
Tempo. Nothing here is a target or a requirement.

## Measurement method

`scripts/baseline.sh ryotunes` reads `Pss` from `/proc/<pid>/smaps_rollup` and
`Threads` from `/proc/<pid>/status`. It must run while Ryotunes is playing a
track, because a paused or freshly launched app understates memory.

## Result: not measured

**Reason:** the old Ryotunes app is not installed on this machine and is not
running, so there was nothing to measure. Verified on 2026-11-10:

- `ps -eo pid,comm | grep -i ryotunes` returned no processes.
- `command -v ryotunes` found no installed binary.
- `/usr/bin/ryotunes`, `/usr/local/bin/ryotunes`, and `~/.local/bin/ryotunes`
  do not exist.
- `/home/mahesh-diwan/SPECTRE/Projects/ryotunes/src-tauri/target/release/`
  does not contain a release binary (the directory is absent).

Per the plan, the old Tauri app will not be built just to take baseline
numbers. When a running Ryotunes becomes available, fill in the table below by
running `scripts/baseline.sh ryotunes` during playback. Until then, treat every
value as unknown rather than zero.

| Field | Value |
| --- | --- |
| Binary path | not measured (no binary installed or present in source tree) |
| Track playing | not measured (no running process) |
| PSS (MiB) | not measured (no running process) |
| Threads | not measured (no running process) |
| Process count | not measured (no running process) |
| WebKit helper processes | not measured for Ryotunes (two unrelated system-wide `WebKitNetworkProcess` / `WebKitWebProcess` instances were running on 2026-11-10, but Ryotunes itself was not, so they cannot be attributed to it) |

## Script sanity checks (2026-11-10)

- `scripts/baseline.sh ryotunes` with no matching process prints
  `no process named ryotunes` to stderr and exits 1.
- `scripts/baseline.sh sleep` against a running `sleep 3` process prints
  `pid`, `pss_kib`, `pss_mib`, `threads`, `processes`, and the `ps` listing;
  exits 0.
