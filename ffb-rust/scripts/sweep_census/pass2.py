#!/usr/bin/env python
"""Pass 2: for every SPECIAL action declaration, what actually happened inside
that activation window (until the next playerAction / turnEnd)?

Answers "declared but never executed": an action whose window contains nothing
but playerMoved is a declaration that degenerated into a plain move.
"""
import json
import os
import re
import sys
from collections import Counter
from multiprocessing import Pool
from pathlib import Path

from agg import gate_dirs

OUT = Path(sys.argv[1] if len(sys.argv) > 1 else "out2")
OUT.mkdir(parents=True, exist_ok=True)

BASIC = {"Move", "Block", "Blitz", "BlitzMove", "BlitzSelect", "Foul", "FoulMove",
         "Pass", "PassMove", "HandOver", "HandOverMove", "StandUp"}
RE_ACTION = re.compile(r'"action":"([^"]+)"')


def do_gate(job):
    race, edition, scale, d = job
    tag = "%s__%s__%s" % (race, edition, scale)
    dst = OUT / (tag + ".json")
    if dst.exists():
        return tag + " (cached)"

    windows = {}     # action -> Counter of event types seen in its windows
    empty = Counter()   # action -> windows containing only playerMoved/nothing
    total = Counter()

    cur = None
    for f in sorted(d.glob("seed_*_rust_events.jsonl")):
        with open(f, "r", encoding="utf-8", errors="replace") as fh:
            seen = set()
            for line in fh:
                if not line.startswith('{"type":"'):
                    continue
                e = line.index('"', 9)
                t = line[9:e]
                if t in ("playerAction", "turnEnd"):
                    if cur:
                        if not (seen - {"playerMoved"}):
                            empty[cur] += 1
                        windows.setdefault(cur, Counter()).update(seen)
                    cur, seen = None, set()
                    if t == "playerAction":
                        m = RE_ACTION.search(line[e:])
                        a = m.group(1) if m else "?"
                        total[a] += 1
                        if a not in BASIC:
                            cur = a
                    continue
                if cur:
                    seen.add(t)
            if cur:
                if not (seen - {"playerMoved"}):
                    empty[cur] += 1
                windows.setdefault(cur, Counter()).update(seen)
                cur, seen = None, set()

    data = dict(race=race, edition=edition, scale=scale,
                total=dict(total), empty=dict(empty),
                windows={k: dict(v) for k, v in windows.items()})
    dst.write_text(json.dumps(data), encoding="utf-8")
    return "%s special=%d" % (tag, sum(empty.values()))


if __name__ == "__main__":
    jobs = list(gate_dirs())
    with Pool(int(os.environ.get("NPROC", "10"))) as p:
        for i, msg in enumerate(p.imap_unordered(do_gate, jobs), 1):
            print("[%3d/%d] %s" % (i, len(jobs), msg), flush=True)
