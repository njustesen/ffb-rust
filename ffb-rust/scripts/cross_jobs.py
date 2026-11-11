"""Enumerate every NON-mirror (home != away) ordered matchup per edition into sweep job files.

The mirror matrix runs each squad against ITSELF, so a rule whose two halves sit on OPPOSITE
teams (Tackle vs Dodge, Claws vs Thick Skull, Stab vs Regeneration, Diving Tackle vs a dodger,
...) is reachable only when one squad happens to carry both. This is the other half of the
matrix: every ordered pair of drafted squads, a few seeds each.

The squad list is the same source the 330-gate mirror sweep uses -- the hand-drafted specs in
`data/teams/<edition>/team_*.json` -- NOT the stale 30-key `KEYS` of `run_team_matrix.py`.

Ordered pairs (home/away is not symmetric: coin toss, kick-off, setup side):
  bb2016 29x28 = 812, bb2020 40x39 = 1560, bb2025 41x40 = 1640  ->  4012 pairs.

Usage:
  python scripts/cross_jobs.py [--workers 5] [--out crossjobs] [--editions bb2016,bb2020,bb2025]

Writes <out>/w1.txt .. w<N>.txt, one gate per line: "<edition> <home> <away>". Pairs are
dealt round-robin after a deterministic shuffle so every worker gets a mix of editions and
squads (a worker's wall time is then ~1/N of the total instead of one slow edition).
A given ordered pair appears exactly once, on exactly one worker, so the campaign rule "never
two runs of one edition+matchup at once" holds by construction.
"""
import argparse
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def squads(edition: str):
    d = ROOT / "data" / "teams" / edition
    return sorted(p.stem[len("team_"):] for p in d.glob("team_*.json"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--workers", type=int, default=5)
    ap.add_argument("--out", default="crossjobs")
    ap.add_argument("--editions", default="bb2016,bb2020,bb2025")
    ap.add_argument("--seed", type=int, default=20261111, help="shuffle seed (deterministic)")
    args = ap.parse_args()

    jobs = []
    for ed in args.editions.split(","):
        keys = squads(ed)
        n = 0
        for home in keys:
            for away in keys:
                if home != away:
                    jobs.append((ed, home, away))
                    n += 1
        print(f"{ed}: {len(keys)} squads -> {n} ordered pairs")
    random.Random(args.seed).shuffle(jobs)

    out = ROOT / args.out
    out.mkdir(exist_ok=True)
    (out / "scratch").mkdir(exist_ok=True)
    shards = [[] for _ in range(args.workers)]
    for i, j in enumerate(jobs):
        shards[i % args.workers].append(j)
    for w, shard in enumerate(shards, 1):
        p = out / f"w{w}.txt"
        p.write_text("".join(f"{ed} {h} {a}\n" for ed, h, a in shard), encoding="utf-8")
        print(f"  {p}: {len(shard)} gates")
    print(f"total {len(jobs)} gates")


if __name__ == "__main__":
    main()
