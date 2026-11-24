"""Who coached which side of every replay -> data/coaches.json, the key the data split is made on.

    python coaches.py --data data/setups_h.jsonl --replays D:/ffb_replays

A formation is a coach's habit, so the split has to keep a coach's games together (train.split_of).
Only a hash of the coach name is stored. The replay's first command carries the whole game; the
two team headers are read from the start of the stream without parsing the file.
"""
from __future__ import annotations

import argparse
import glob
import gzip
import hashlib
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
TEAM = re.compile(r'"(teamAway|teamHome)"\s*:\s*\{\s*"teamId"\s*:\s*"([^"]*)"\s*,\s*"teamName"\s*:\s*"(?:[^"\\]|\\.)*"\s*,'
                  r'\s*"coach"\s*:\s*"((?:[^"\\]|\\.)*)"')


def sides(path: str) -> dict | None:
    out, text = {}, ""
    with gzip.open(path, "rt", encoding="utf-8") as f:
        while len(out) < 2 and len(text) < 8_000_000:
            chunk = f.read(262144)
            if not chunk:
                break
            text += chunk
            for m in TEAM.finditer(text):
                side = "home" if m.group(1) == "teamHome" else "away"
                out.setdefault(side, {"team": m.group(2), "coach": hashlib.sha1(m.group(3).encode()).hexdigest()[:16]})
    return out if len(out) == 2 else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=os.path.join(HERE, "data", "setups_h.jsonl"))
    ap.add_argument("--replays", default="D:/ffb_replays")
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    ids = set()
    with open(args.data, encoding="utf-8") as f:
        for line in f:
            ids.add(re.search(r'"replay_id":\s*"([^"]+)"', line).group(1))
    files = {os.path.basename(p)[:-5]: p for p in glob.glob(os.path.join(args.replays, "**", "*.ffbr"), recursive=True)}
    result, missing = {}, []
    for rid in sorted(ids):
        got = sides(files[rid]) if rid in files else None
        if got:
            result[rid] = got
        else:
            missing.append(rid)
    out = args.out or os.path.join(os.path.dirname(os.path.abspath(args.data)), "coaches.json")
    with open(out, "w") as f:
        json.dump(result, f)
    coaches = {s["coach"] for r in result.values() for s in r.values()}
    teams = {s["team"] for r in result.values() for s in r.values()}
    print(f"{len(result)} replays, {len(coaches)} coaches, {len(teams)} teams; not resolved: {len(missing)} {missing[:5]}")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
