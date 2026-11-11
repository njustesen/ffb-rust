"""Rebuild a cross sweep's verdict log from the per-gate logs, and shard the REMAINING jobs.

Why: a `tail -f` on the shared sweep.log (a monitor) locked it on Windows, and every worker's
`Add-Content` failed for 30 minutes -- 1,110 gates ran and their verdict lines were lost. The
per-gate `cw_<home>_vs_<away>_<edition>.log[.err]` files survived, and the job files are
deterministic, so both are recoverable.

Usage:
  python scripts/cross_recover.py --since "2026-11-11 18:29:40" [--scratch crossjobs/scratch]
      [--jobs crossjobs] [--workers 5] [--out crossjobs/resume]

Writes <scratch>/recovered.log (sweep.log shape) from gate logs modified at/after --since, and
<out>/w1..wN.txt holding every job of <jobs>/w*.txt whose gate log is NOT recovered.
"""
import argparse
import datetime as dt
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def verdict_of(log: Path):
    v, ff = None, None
    for g in (log, Path(str(log) + ".err")):
        if not g.exists():
            continue
        for line in g.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith("PARITY: ") and (v is None or "games match" in line):
                v = line
            if ff is None and line.startswith("PARITY FAIL seed="):
                m = re.match(r"PARITY FAIL seed=(\d+)[^,]*, step (\d+)", line)
                if m:
                    ff = f"  first-fail seed={m.group(1)} step={m.group(2)}"
    if v is None:
        return None
    return v + (ff or "")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--since", required=True, help="YYYY-MM-DD HH:MM:SS local")
    ap.add_argument("--scratch", default="crossjobs/scratch")
    ap.add_argument("--jobs", default="crossjobs")
    ap.add_argument("--workers", type=int, default=5)
    ap.add_argument("--out", default="crossjobs/resume")
    ap.add_argument("--scale", default="1.0")
    args = ap.parse_args()

    since = dt.datetime.strptime(args.since, "%Y-%m-%d %H:%M:%S").timestamp()
    scratch = ROOT / args.scratch
    jobs = []
    for jf in sorted((ROOT / args.jobs).glob("w*.txt")):
        for line in jf.read_text(encoding="utf-8").splitlines():
            t = line.split()
            if len(t) == 3:
                jobs.append(tuple(t))

    recovered, remaining = [], []
    for ed, home, away in jobs:
        log = scratch / f"cw_{home}_vs_{away}_{ed}.log"
        if log.exists() and log.stat().st_mtime >= since:
            v = verdict_of(log)
            if v is not None:
                recovered.append(f"{home}_vs_{away} {ed} @{args.scale}".ljust(50) + "   0.0m  " + v)
                continue
        remaining.append((ed, home, away))

    (scratch / "recovered.log").write_text("\n".join(recovered) + "\n", encoding="utf-8")
    out = ROOT / args.out
    out.mkdir(exist_ok=True)
    shards = [[] for _ in range(args.workers)]
    for i, j in enumerate(remaining):
        shards[i % args.workers].append(j)
    for w, shard in enumerate(shards, 1):
        (out / f"w{w}.txt").write_text("".join(f"{ed} {h} {a}\n" for ed, h, a in shard), encoding="utf-8")
    print(f"jobs {len(jobs)}  recovered {len(recovered)}  remaining {len(remaining)} -> {out}/w1..w{args.workers}.txt")
    reds = [r for r in recovered if "games match" not in r]
    print(f"recovered reds: {len(reds)}")
    for r in reds:
        print("  " + r)


if __name__ == "__main__":
    main()
