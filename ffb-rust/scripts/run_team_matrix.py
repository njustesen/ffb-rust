"""Run the full mirror-parity matrix with the hand-drafted teams and write
green/red report docs (reds are recorded, not fixed).

For each matrix CLI key x edition: ffb-parity --home K --away K --edition E
--tier 3 --seeds A-B --no-abort, parsed for pass count + first divergence.

Usage:
  python scripts/run_team_matrix.py --edition bb2025 [--seeds 1-100] [--parallel 6] [--only human,orc]
  python scripts/run_team_matrix.py --edition all --reuse-java   # the full three-edition gate
Writes docs/TEAM_MATRIX_<EDITION>.md

`--edition all` gates bb2016+bb2020+bb2025 in ONE thread pool rather than three sequential
runs; the parity logs are edition-scoped (`parity/<edition>/<matchup>/`), so the editions no
longer overwrite each other's Java logs mid-comparison.

`--reuse-java` skips the JVM for any matchup whose cached Java logs provably came from the
same jar and the same Java server data — safe for a Rust-only iteration, and the bulk of the
gate's wall-clock. Each run prints REUSE / REUSE declined so the choice is never silent.

NEVER run `cargo build` while a gate is in flight: it rewrites `target/release/ffb-parity.exe`
underneath the running matrix and produces phantom reds that do not reproduce in isolation.
"""

import argparse
import concurrent.futures
import datetime
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).parent.parent
BIN = ROOT / "target" / "release" / "ffb-parity.exe"

KEYS = [
    "lineman", "amazon", "chaos", "chaos_dwarf", "chaos_pact", "dark_elf",
    "dark_elf_league_fumbbl", "dwarf", "elf", "goblin", "halfling", "high_elf",
    "human", "khemri", "khemri_fumbbl", "lizardman", "necromantic", "nippon",
    "norse", "nurgle", "ogre", "orc", "renegades", "skaven", "slann",
    "slann_fumbbl", "undead", "underworld", "vampire", "wood_elf",
]
LEGACY = {
    "bb2025": {"nippon", "slann", "chaos_pact", "dark_elf_league_fumbbl",
               "khemri_fumbbl", "slann_fumbbl", "lineman"},
    "bb2016": {"nippon", "renegades", "dark_elf_league_fumbbl", "khemri_fumbbl",
               "slann_fumbbl", "lineman"},
    # bb2020 draws the same non-official rosters as bb2025 (the FUMBBL imports and the
    # synthetic lineman team have no official BB2020 team page to audit against).
    "bb2020": {"nippon", "slann", "chaos_pact", "dark_elf_league_fumbbl",
               "khemri_fumbbl", "slann_fumbbl", "lineman"},
}


def run_one(key: str, edition: str, seeds: str, reuse_java: bool = False) -> dict:
    env = dict(os.environ)
    env.setdefault("PARITY_JVM_CORES", "2")
    cmd = [str(BIN), "--home", key, "--away", key, "--edition", edition,
           "--tier", "3", "--seeds", seeds, "--no-abort"]
    if reuse_java:
        cmd.append("--reuse-java")
    proc = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True,
                          text=True, encoding="utf-8", errors="replace",
                          timeout=7200)
    out = proc.stdout + "\n" + proc.stderr
    passed = len(re.findall(r"^✓ seed ", out, re.M))
    fails = re.findall(r"PARITY FAIL seed=(\d+)[^\n]*, step (\d+)[^\n]*", out)
    first_fail = None
    m = re.search(r"PARITY FAIL seed=(\d+)[^,]*, step (\d+): java=(.{0,120}).*?\n\s*java_hash=(\S+)\n\s*rust_hash=(\S+)", out, re.S)
    if m:
        first_fail = {"seed": int(m.group(1)), "step": int(m.group(2)),
                      "java_hash": m.group(4), "rust_hash": m.group(5)}
    return {"key": key, "passed": passed, "failed": len(fails),
            "first_fail": first_fail, "rc": proc.returncode}


def write_report(edition: str, keys, results, seeds: str, total_seeds: int) -> tuple:
    legacy = LEGACY[edition]
    lines = [
        f"# Team-Parity Matrix — {edition.upper()} (hand-drafted teams)",
        "",
        f"Run {datetime.date.today().isoformat()} — mirror matchups, tier 3, seeds {seeds},",
        "teams from `data/teams/" + edition + "/` (see docs/TEAM_DRAFTS_"
        + edition.upper() + ".md), Java XMLs from scripts/gen_java_parity_data.py.",
        "Reds are RECORDED, not fixed (scope of the 2026-08-08 team-creation task).",
        "",
        "| Roster | Result | First divergence | Notes |",
        "|---|---|---|---|",
    ]
    green = red = 0
    for k in keys:
        r = results[k]
        ok = r["passed"] == total_seeds
        green += ok
        red += not ok
        ff = r.get("first_fail")
        div = f"seed {ff['seed']}, step {ff['step']}, java {ff['java_hash']} vs rust {ff['rust_hash']}" if ff else ""
        notes = "FUMBBL-legacy roster" if k in legacy else ""
        badge = f"🟢 {r['passed']}/{total_seeds}" if ok else f"🔴 {r['passed']}/{total_seeds}"
        lines.append(f"| `{k}` | {badge} | {div} | {notes} |")
    lines += ["", f"**{green} green / {red} red of {len(keys)}.**", ""]
    doc = ROOT / "docs" / f"TEAM_MATRIX_{edition.upper()}.md"
    doc.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {doc}: {green} green / {red} red")
    return green, red


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--edition", required=True,
                    choices=["bb2016", "bb2020", "bb2025", "all"])
    ap.add_argument("--seeds", default="1-100")
    ap.add_argument("--parallel", type=int, default=6)
    ap.add_argument("--only", default=None)
    ap.add_argument("--reuse-java", action="store_true",
                    help="reuse cached Java logs when the jar and Java data are unchanged")
    args = ap.parse_args()

    editions = ["bb2016", "bb2020", "bb2025"] if args.edition == "all" else [args.edition]
    keys = args.only.split(",") if args.only else KEYS
    total_seeds = 1 + int(args.seeds.split("-")[1]) - int(args.seeds.split("-")[0])

    results = {e: {} for e in editions}
    tasks = [(k, e) for e in editions for k in keys]
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as ex:
        futs = {ex.submit(run_one, k, e, args.seeds, args.reuse_java): (k, e)
                for k, e in tasks}
        for fut in concurrent.futures.as_completed(futs):
            k, e = futs[fut]
            try:
                r = fut.result()
            except Exception as exc:
                r = {"key": k, "passed": 0, "failed": -1, "first_fail": None,
                     "rc": -1, "error": str(exc)}
            results[e][k] = r
            status = "GREEN" if r["passed"] == total_seeds else "RED"
            ff = r.get("first_fail")
            extra = f" first-div seed {ff['seed']} step {ff['step']}" if ff else ""
            print(f"[{status}] {e} {k}: {r['passed']}/{total_seeds}{extra}", flush=True)

    print()
    totals = [(e,) + write_report(e, keys, results[e], args.seeds, total_seeds)
              for e in editions]
    if len(totals) > 1:
        # Plain ASCII: this line is routinely redirected to a file, and Windows' default
        # cp1252 stdout encoding raises UnicodeEncodeError on the badge emoji — crashing the
        # script with exit 1 AFTER a fully green gate, which reads like a failed gate.
        print("\nGATE:", "  ".join(f"{e} {g} green/{r} red" for e, g, r in totals))


if __name__ == "__main__":
    main()
