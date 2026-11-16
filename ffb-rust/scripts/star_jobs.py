"""Schedule the STAR-PLAYER parity tier (BACKLOG §L) into sweep job files.

Each game fields exactly ONE hired star per team (a pair such as Grak & Crumbleberry is one
hire) on top of the unchanged R6 squads; stars and their hiring lists come from
data/star_players/<edition>.json (scripts/build_star_catalog.py).

The schedule is a covering design over the cross matrix:
  * every squad T plays every other squad U at home (all ordered pairs, like cross_jobs.py);
  * T plays max(#stars(T), #squads-1) home games, cycling its home star through EVERY star it
    can hire, so every (squad, star) pair is played at home at least once;
  * the away squad cycles through ITS stars round-robin across the games it is the away side
    in, so every (squad, star) pair is also played away (when #stars <= #squads-1);
  * the same star never plays for both teams in one game (rules: neither could hire him).

Usage:
  python scripts/star_jobs.py [--workers 3] [--out starjobs] [--editions ...] [--limit N]

Writes <out>/w1.txt .. w<N>.txt, one gate per line:
  "<edition> <home> <home_star> <away> <away_star>"
and <out>/coverage.txt with the (squad, star) home/away counts. --limit N keeps the first N
gates per edition AFTER the deterministic shuffle (a pilot sample).
"""
import argparse
import json
import random
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def squads(edition: str):
    d = ROOT / "data" / "teams" / edition
    return sorted(p.stem[len("team_"):] for p in d.glob("team_*.json"))


def star_lists(edition: str) -> dict[str, list[str]]:
    cat = json.loads((ROOT / "data" / "star_players" / f"{edition}.json").read_text(encoding="utf-8"))
    out: dict[str, list[str]] = {sq: [] for sq in squads(edition)}
    for s in cat["stars"]:
        for sq in s["eligible"]:
            out.setdefault(sq, []).append(s["id"])
    for sq in out:
        out[sq].sort()
    return out


def schedule(edition: str):
    keys = squads(edition)
    stars = star_lists(edition)
    missing = [k for k in keys if not stars.get(k)]
    if missing:
        raise SystemExit(f"{edition}: squads with no hireable star: {missing}")
    away_turn = Counter()
    jobs = []
    for ti, home in enumerate(keys):
        others = [k for k in keys if k != home]
        hs = stars[home]
        n = max(len(hs), len(others))
        for i in range(n):
            home_star = hs[i % len(hs)]
            away = others[(i + ti) % len(others)]
            as_ = stars[away]
            for _ in range(len(as_)):
                away_star = as_[away_turn[away] % len(as_)]
                away_turn[away] += 1
                if away_star != home_star:
                    break
            else:
                raise SystemExit(f"{edition}: {away} can only hire {home_star}")
            jobs.append((edition, home, home_star, away, away_star))
    return jobs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--out", default="starjobs")
    ap.add_argument("--editions", default="bb2016,bb2020,bb2025")
    ap.add_argument("--seed", type=int, default=20261116, help="shuffle seed (deterministic)")
    ap.add_argument("--limit", type=int, default=0, help="pilot: first N gates per edition")
    args = ap.parse_args()

    jobs = []
    cov = []
    for ed in args.editions.split(","):
        ej = schedule(ed)
        home_c = Counter((h, hs) for _, h, hs, _, _ in ej)
        away_c = Counter((a, as_) for _, _, _, a, as_ in ej)
        pairs = {(sq, s) for sq, lst in star_lists(ed).items() for s in lst}
        pairs_cov = {p for p in pairs if home_c[p] and away_c[p]}
        print(f"{ed}: {len(ej)} gates; (squad, star) pairs {len(pairs)}, "
              f"home-covered {sum(1 for p in pairs if home_c[p])}, "
              f"home+away-covered {len(pairs_cov)}; ordered squad pairs "
              f"{len({(h, a) for _, h, _, a, _ in ej})}")
        for p in sorted(pairs):
            cov.append(f"{ed} {p[0]} {p[1]} home={home_c[p]} away={away_c[p]}")
        random.Random(args.seed).shuffle(ej)
        if args.limit:
            ej = ej[:args.limit]
        jobs += ej
    random.Random(args.seed + 1).shuffle(jobs)

    out = ROOT / args.out
    out.mkdir(exist_ok=True)
    (out / "scratch").mkdir(exist_ok=True)
    (out / "coverage.txt").write_text("\n".join(cov) + "\n", encoding="utf-8")
    shards = [[] for _ in range(args.workers)]
    for i, j in enumerate(jobs):
        shards[i % args.workers].append(j)
    for w, shard in enumerate(shards, 1):
        p = out / f"w{w}.txt"
        p.write_text("".join(" ".join(j) + "\n" for j in shard), encoding="utf-8")
        print(f"  {p}: {len(shard)} gates")
    print(f"total {len(jobs)} gates")


if __name__ == "__main__":
    main()
