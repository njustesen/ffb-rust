"""Final result of every replay -> data/outcomes.json, for the outcome-conditioned policy (z).

    python outcomes.py --data data/setups_h.jsonl --replays D:/ffb_replays

{replay_id: {"home": touchdowns, "away": touchdowns, "conceded": "home" | "away" | null}}
Read from the replay's model changes (the last teamResultSetScore per side).
"""
from __future__ import annotations

import argparse
import glob
import gzip
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
SCORE = re.compile(r'"modelChangeId":"teamResultSetScore","modelChangeKey":"(home|away)","modelChangeValue":(\d+)')
CONCEDED = re.compile(r'"modelChangeId":"teamResultSetConceded","modelChangeKey":"(home|away)","modelChangeValue":true')


def outcome(path: str) -> dict:
    with gzip.open(path, "rt", encoding="utf-8") as f:
        text = f.read()
    out = {"home": 0, "away": 0, "conceded": None}
    for side, value in SCORE.findall(text):
        out[side] = int(value)
    conceded = CONCEDED.findall(text)
    if conceded:
        out["conceded"] = conceded[-1]
    return out


def z_of(result: dict, side: str) -> float:
    """1 = that side won, 0.5 = draw, 0 = lost. Conceding is a loss."""
    other = "away" if side == "home" else "home"
    if result["conceded"]:
        return 0.0 if result["conceded"] == side else 1.0
    a, b = result[side], result[other]
    return 1.0 if a > b else (0.5 if a == b else 0.0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=os.path.join(HERE, "data", "setups_h.jsonl"))
    ap.add_argument("--replays", default="D:/ffb_replays")
    args = ap.parse_args()
    ids = set()
    with open(args.data, encoding="utf-8") as f:
        for line in f:
            ids.add(re.search(r'"replay_id":\s*"([^"]+)"', line).group(1))
    files = {os.path.basename(p)[:-5]: p for p in glob.glob(os.path.join(args.replays, "**", "*.ffbr"), recursive=True)}
    result = {rid: outcome(files[rid]) for rid in sorted(ids) if rid in files}
    out = os.path.join(os.path.dirname(os.path.abspath(args.data)), "outcomes.json")
    with open(out, "w") as f:
        json.dump(result, f)
    zs = [z_of(r, "home") for r in result.values()]
    print(f"{len(result)} of {len(ids)} replays; home wins {zs.count(1.0)}, draws {zs.count(0.5)}, "
          f"home losses {zs.count(0.0)}; conceded {sum(1 for r in result.values() if r['conceded'])}")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
