#!/usr/bin/env python3
"""
parity_report.py — read parity/results/ and generate a Markdown results table.

Usage:
    python scripts/parity_report.py
    python scripts/parity_report.py --results-dir parity/results

Reads all summary.json files under the results directory and prints a Markdown
table suitable for pasting into PARITY_PLAN.md's Results Log section.
"""

import argparse
import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent


def load_summaries(results_dir: Path) -> list[dict]:
    summaries = []
    for summary_file in sorted(results_dir.rglob("summary.json")):
        try:
            with open(summary_file) as f:
                data = json.load(f)
                data["_path"] = str(summary_file.relative_to(results_dir))
                summaries.append(data)
        except (json.JSONDecodeError, OSError) as e:
            print(f"WARN: could not load {summary_file}: {e}", file=sys.stderr)
    return summaries


def format_result(passed: int, total: int) -> str:
    if passed == total:
        return f"✓ {passed}/{total}"
    elif passed == 0:
        return f"✗ 0/{total}"
    else:
        return f"~ {passed}/{total}"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--results-dir", default=str(REPO_ROOT / "parity" / "results"))
    ap.add_argument("--format", choices=["markdown", "json"], default="markdown")
    args = ap.parse_args()

    results_dir = Path(args.results_dir)
    if not results_dir.exists():
        print(f"Results directory not found: {results_dir}", file=sys.stderr)
        print("Run `python scripts/parity_run.py` first to generate results.")
        return 1

    summaries = load_summaries(results_dir)
    if not summaries:
        print("No summary files found.", file=sys.stderr)
        return 1

    if args.format == "json":
        json.dump(summaries, sys.stdout, indent=2)
        return 0

    # Markdown table
    print("## Parity Results\n")
    print(f"| Date | Tier | Seeds | Home | Away | Edition | Passed | Notes |")
    print(f"|------|------|-------|------|------|---------|--------|-------|")

    for s in summaries:
        tier = s.get("tier", "?")
        home = s.get("home", "?")
        away = s.get("away", "?")
        edition = s.get("edition", "?")
        start = s.get("seed_start", 1)
        end = s.get("seed_end", "?")
        total = s.get("total", 0)
        passed = s.get("passed", 0)
        failed = s.get("failed", 0)
        ts = s.get("timestamp", "—")[:10]

        seed_str = f"{start}–{end}"
        result_str = format_result(passed, total)
        notes = f"{failed} failed" if failed > 0 else ""

        print(f"| {ts} | {tier} | {seed_str} | {home} | {away} | {edition} | {result_str} | {notes} |")

    print()

    # Summary by tier
    by_tier: dict[str, list[dict]] = {}
    for s in summaries:
        t = s.get("tier", "?")
        by_tier.setdefault(t, []).append(s)

    print("## Matrix Update\n")
    print("Copy the cells below into the PARITY_PLAN.md Test Matrix:\n")

    scale_map = {1: "1 seed", 10: "10 seeds", 100: "100 seeds", 1000: "1000 seeds"}

    for tier, tier_runs in sorted(by_tier.items()):
        for s in tier_runs:
            total = s.get("total", 0)
            passed = s.get("passed", 0)
            scale_label = scale_map.get(total, f"{total} seeds")
            result = format_result(passed, total)
            home = s.get("home", "?")
            away = s.get("away", "?")
            edition = s.get("edition", "?")
            print(f"  {tier} | {scale_label} | {result} | {home} vs {away} ({edition})")

    return 0


if __name__ == "__main__":
    sys.exit(main())
