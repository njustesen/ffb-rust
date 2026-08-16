"""Run CROSS-team (non-mirror) mirror-parity matchups.

Why this exists: the 30-roster matrix only ever runs a roster against ITSELF, so a rule needing the
ATTACKER to have one skill and the DEFENDER another is only reachable when a SINGLE roster happens
to carry both. Where no roster does, the mirror matrix can never reach it however many seeds it
runs. `Tackle` vs `Dodge` is the clean case: dwarf has Tackle and no Dodge, wood_elf has Dodge and
no Tackle, so "Dodge cancelled by Tackle" cannot fire in either mirror -- only in dwarf-vs-wood_elf.

(Counter-example worth remembering before citing one of these: chaos_dwarf carries BOTH `Stab` and
`Iron Hard Skin`, so its own mirror *does* reach the stab `ignoresArmourModifiersFromSkills`
branch. Check both halves against one roster before claiming a mirror cannot reach a rule.)

Rather than brute-force 30x29 ordered pairs per edition, pairs are RANKED by how many known
cross-team skill interactions they would actually exercise, and the top N are run. Use --all to
brute-force anyway.

Usage:
  python scripts/run_cross_matrix.py --edition bb2020 [--seeds 1-25] [--top 12] [--parallel 4]
  python scripts/run_cross_matrix.py --edition bb2020 --pairs dark_elf:chaos_dwarf,skaven:dwarf
Writes docs/CROSS_MATRIX_<EDITION>.md
"""

import argparse
import concurrent.futures
import datetime
import json
from pathlib import Path

from run_team_matrix import KEYS, run_one

ROOT = Path(__file__).parent.parent

# (attacker-side skill, defender-side skill, what it gates).
# Each entry is a rule whose two halves must sit on OPPOSITE teams, so a mirror matchup can only
# reach it by coincidence and usually not at all.
INTERACTIONS = [
    ("Stab",          "Iron Hard Skin", "stab armour modifiers ignored (InjuryTypeStab:64-66)"),
    ("Chainsaw",      "Iron Hard Skin", "chainsaw +3 ignored"),
    ("Dirty Player",  "Iron Hard Skin", "foul armour modifiers ignored"),
    ("Claws",         "Thick Skull",    "AV reduction vs KO->Stunned"),
    ("Mighty Blow",   "Thick Skull",    "injury bonus vs KO->Stunned"),
    ("Mighty Blow",   "Stunty",         "injury bonus vs the Stunty injury table"),
    ("Tackle",        "Dodge",          "Dodge cancelled by Tackle"),
    ("Prehensile Tail", "Dodge",        "dodge modifier stack"),
    ("Horns",         "Foul Appearance", "blitz roll ordering"),
    ("Stab",          "Regeneration",   "stab casualty then regeneration"),
    ("Claws",         "Regeneration",   "claw casualty then regeneration"),
    ("Bombardier",    "Foul Appearance", "bomb vs foul-appearance roll"),
]


def roster_skills(edition: str, key: str) -> set:
    path = ROOT / "data" / "rosters" / edition / f"roster_{key}.json"
    if not path.exists():
        return set()
    data = json.loads(path.read_text(encoding="utf-8"))
    out = set()
    for pos in data.get("positions", []):
        for s in pos.get("skills", []):
            out.add(s if isinstance(s, str) else s.get("name", ""))
    return out


def rank_pairs(edition: str, keys):
    """Rank ordered pairs, weighting interactions NO single roster can reach on its own.

    An interaction whose two halves both sit on some one roster is already reachable by that
    roster's mirror, so pairing for it adds little. The valuable pairs are the ones carrying
    interactions that are mirror-unreachable across the whole roster set.
    """
    skills = {k: roster_skills(edition, k) for k in keys}
    unreachable = {
        (att, dfn) for att, dfn, _ in INTERACTIONS
        if not any(att in skills[k] and dfn in skills[k] for k in keys)
    }
    scored = []
    for home in keys:
        for away in keys:
            if home == away:
                continue
            hit, weight = [], 0
            for att, dfn, why in INTERACTIONS:
                if att in skills[home] and dfn in skills[away]:
                    only_cross = (att, dfn) in unreachable
                    hit.append(f"{why}{' **' if only_cross else ''}")
                    weight += 3 if only_cross else 1
            if hit:
                scored.append((weight, home, away, hit))
    # Highest weight first; name-sorted within a weight so the selection is deterministic.
    scored.sort(key=lambda t: (-t[0], t[1], t[2]))
    return scored


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--edition", required=True, choices=["bb2016", "bb2020", "bb2025"])
    ap.add_argument("--seeds", default="1-25")
    ap.add_argument("--parallel", type=int, default=4)
    ap.add_argument("--top", type=int, default=12)
    ap.add_argument("--all", action="store_true", help="every ordered pair, not just ranked ones")
    ap.add_argument("--pairs", default=None, help="explicit home:away,home:away list")
    args = ap.parse_args()

    total_seeds = 1 + int(args.seeds.split("-")[1]) - int(args.seeds.split("-")[0])

    if args.pairs:
        chosen = [(0, p.split(":")[0], p.split(":")[1], ["explicit"]) for p in args.pairs.split(",")]
    elif args.all:
        chosen = [(0, h, a, []) for h in KEYS for a in KEYS if h != a]
    else:
        chosen = rank_pairs(args.edition, KEYS)[:args.top]

    if not chosen:
        print("no interacting pairs found for this edition")
        return

    print(f"running {len(chosen)} cross matchups, seeds {args.seeds}", flush=True)
    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as ex:
        futs = {}
        for _, home, away, why in chosen:
            # run_one drives --home/--away; the harness keys its jsonl on the matchup directory, so
            # distinct pairs are safe to run concurrently (same-matchup runs are NOT).
            futs[ex.submit(run_one_pair, home, away, args.edition, args.seeds)] = (home, away, why)
        for fut in concurrent.futures.as_completed(futs):
            home, away, why = futs[fut]
            try:
                r = fut.result()
            except Exception as e:
                r = {"passed": 0, "first_fail": None, "error": str(e)}
            results[(home, away)] = (r, why)
            status = "GREEN" if r["passed"] == total_seeds else "RED"
            ff = r.get("first_fail")
            extra = f" first-div seed {ff['seed']} step {ff['step']}" if ff else ""
            print(f"[{status}] {args.edition} {home} vs {away}: "
                  f"{r['passed']}/{total_seeds}{extra}", flush=True)

    lines = [
        f"# Cross-Team Parity Matrix — {args.edition.upper()}",
        "",
        f"Run {datetime.date.today().isoformat()} — NON-mirror matchups, tier 3, seeds {args.seeds}.",
        "",
        "Mirror matchups cannot exercise a rule whose two halves sit on opposite teams.",
        "Pairs below are ranked by how many such interactions they reach.",
        "",
        "| Home | Away | Result | Interactions reached | First divergence |",
        "|---|---|---|---:|---|",
    ]
    green = red = 0
    for (home, away), (r, why) in sorted(results.items()):
        ok = r["passed"] == total_seeds
        green += ok
        red += not ok
        ff = r.get("first_fail")
        div = f"seed {ff['seed']}, step {ff['step']}" if ff else ""
        badge = f"🟢 {r['passed']}/{total_seeds}" if ok else f"🔴 {r['passed']}/{total_seeds}"
        lines.append(f"| `{home}` | `{away}` | {badge} | {'; '.join(why)} | {div} |")
    lines += ["", f"**{green} green / {red} red of {len(results)}.**", ""]
    doc = ROOT / "docs" / f"CROSS_MATRIX_{args.edition.upper()}.md"
    doc.write_text("\n".join(lines), encoding="utf-8")
    print(f"\nwrote {doc}: {green} green / {red} red")


def run_one_pair(home: str, away: str, edition: str, seeds: str) -> dict:
    """run_team_matrix.run_one hardcodes home==away; this is the two-key variant."""
    import os
    import re
    import subprocess
    from run_team_matrix import BIN
    env = dict(os.environ)
    env.setdefault("PARITY_JVM_CORES", "2")
    cmd = [str(BIN), "--home", home, "--away", away, "--edition", edition,
           "--tier", "3", "--seeds", seeds, "--no-abort"]
    proc = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True,
                          text=True, encoding="utf-8", errors="replace", timeout=7200)
    out = proc.stdout + "\n" + proc.stderr
    passed = len(re.findall(r"^✓ seed ", out, re.M))
    first_fail = None
    m = re.search(r"PARITY FAIL seed=(\d+)[^,]*, step (\d+)", out)
    if m:
        first_fail = {"seed": int(m.group(1)), "step": int(m.group(2))}
    return {"passed": passed, "first_fail": first_fail}


if __name__ == "__main__":
    main()
