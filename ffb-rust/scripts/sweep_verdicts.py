"""Turn a sweep worker log into the committed verdict table (docs/SWEEP_<date>.txt).

The workers append one line per gate to a shared `sweep.log`:

    <matchup> <edition> @<scale>    <minutes>m  PARITY: ...

This reads that, checks every expected gate reported exactly once, and writes the same shape of
summary the earlier sweeps committed: the headline, then the gates that are short on the
mechanic-coverage checklist, then the gates that actually FAILED parity.

Usage:
  python scripts/sweep_verdicts.py <sweep.log> --out docs/SWEEP_2026-09-10_REDRAFT.txt \\
      [--title "..."] [--expect 330]
"""

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LINE = re.compile(r"^(\S+) (bb\d{4}) @(\S+)\s+([\d.]+)m\s+(.*)$")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log")
    ap.add_argument("--out", required=True)
    ap.add_argument("--title", default="FULL MATRIX SWEEP")
    ap.add_argument("--expect", type=int, default=330)
    ap.add_argument("--json", help="also write the verdicts as JSON (for the review page)")
    args = ap.parse_args()

    gates, dupes = {}, []
    for raw in open(args.log, encoding="utf-8", errors="replace"):
        m = LINE.match(raw.strip())
        if not m:
            continue
        cell, ed, scale, mins, verdict = m.groups()
        key = (cell, ed, scale)
        if key in gates:
            dupes.append(key)
        gates[key] = (float(mins), verdict)

    green = [k for k, v in gates.items() if "games match" in v[1]]
    short = [k for k, v in gates.items() if "coverage items are MISSING" in v[1]]
    failed = [k for k, v in gates.items() if "FAILED" in v[1]]
    noverdict = [k for k, v in gates.items() if "NO VERDICT" in v[1]]
    total_min = sum(v[0] for v in gates.values())

    def fmt(k):
        return f"{k[0]} {k[1]} @{k[2]}"

    L = [f"{args.title} - {len(gates)} gates, {len(gates) // 3} cells x 3 scales"]
    L.append(f"parity: {len(green)}/{len(gates)} games match, {len(failed)} FAILED, "
             f"{len(noverdict)} without a verdict")
    L.append("")
    L.append(f"summed gate time {total_min:.0f} min ({total_min / 60:.2f} h)")
    if len(gates) != args.expect:
        L.append("")
        L.append(f"!! {len(gates)} gates reported, {args.expect} expected -- "
                 f"the sweep is INCOMPLETE, do not read this as a matrix result")
    if dupes:
        L.append(f"!! {len(dupes)} gate(s) reported more than once: "
                 f"{', '.join(fmt(k) for k in sorted(set(dupes)))}")

    if failed or noverdict:
        L += ["", f"PARITY RED - {len(failed) + len(noverdict)} gate(s):"]
        for k in sorted(failed + noverdict):
            L.append(f"  {fmt(k):38} {gates[k][1]}")

    L += ["", f"Separately, {len(short)} gate(s) report the mechanic-coverage checklist unmet",
          "(parity still 100/100 on every one of them):", ""]
    for k in sorted(short):
        L.append(f"  {fmt(k)}")
    L.append("")

    out = ROOT / args.out
    out.write_text("\n".join(L) + "\n", encoding="utf-8")
    print("\n".join(L[:6]))
    print(f"\n-> {out}")

    if args.json:
        import json
        payload = {
            "gates": len(gates), "cells": len(gates) // 3,
            "green": len(green), "failed": len(failed), "noverdict": len(noverdict),
            "short": len(short), "summed_minutes": round(total_min, 1),
            "expected": args.expect, "complete": len(gates) == args.expect,
            "red": [{"cell": k[0], "edition": k[1], "scale": k[2], "verdict": gates[k][1]}
                    for k in sorted(failed + noverdict)],
            "short_gates": [{"cell": k[0], "edition": k[1], "scale": k[2]} for k in sorted(short)],
        }
        (ROOT / args.json).write_text(json.dumps(payload, indent=1), encoding="utf-8")
        print(f"-> {ROOT / args.json}")
    return 0 if len(gates) == args.expect else 1


if __name__ == "__main__":
    sys.exit(main())
