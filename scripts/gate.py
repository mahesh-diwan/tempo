#!/usr/bin/env python3
import json, sys

path = sys.argv[1] if len(sys.argv) > 1 else "spike-report.json"
report = json.load(open(path))
tracks = report["tracks"]
queries = report["queries"]
played = sum(1 for t in tracks if t["status"] == "played")
with_results = len({t["query"] for t in tracks if t["status"] not in ("search_empty", "search_failed")})
ok = with_results == len(queries) and len(tracks) == 50 and played * 100 >= 90 * len(tracks)
print(f"pass={played} fail={len(tracks) - played} queries={with_results}/{len(queries)} gate={'pass' if ok else 'fail'}")
sys.exit(0 if ok else 2)
