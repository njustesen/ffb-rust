"""BACKLOG §M phase 3: regenerate the `coverage` column of data/agent/skill_policy.json.

Input: an offer census -- a directory of `<edition>_<race>.txt` files holding the RSKILL lines of
a Rust-only heuristic bench run with FFB_DRAWS=1 (one file per race mirror, all with the same seed
count). `coverage` is how strongly `--coverage-bias` pulls a skill toward "use"; a skill that is
already offered often needs no push, a rare one gets the full bias:

    offers per game (the LOWEST over the editions that offer it)
        >= 1.0   -> coverage    0
        >= 0.25  -> coverage  500
        <  0.25  -> coverage 1000      (also: never offered -> the default row, 1000)

`use` (the base policy) is hand-set and never touched by this script.

usage: python scripts/skill_policy_coverage.py <census-dir> <seeds-per-race> [--write]
"""
import collections
import glob
import json
import os
import re
import sys

POLICY = os.path.join(os.path.dirname(__file__), "..", "data", "agent", "skill_policy.json")
PAT = re.compile(r"^RSKILL skill=(\S+) ")


def coverage_for(rate):
    if rate >= 1.0:
        return 0
    if rate >= 0.25:
        return 500
    return 1000


def main():
    census, seeds = sys.argv[1], int(sys.argv[2])
    write = "--write" in sys.argv[3:]
    offers = collections.Counter()
    games = collections.Counter()
    for f in glob.glob(os.path.join(census, "*.txt")):
        ed = os.path.basename(f).split("_", 1)[0]
        games[ed] += seeds
        for line in open(f, encoding="utf-8", errors="replace"):
            m = PAT.match(line)
            if m:
                offers[(ed, m.group(1))] += 1
    rate = {}
    for (ed, skill), n in offers.items():
        r = n / games[ed]
        rate[skill] = min(rate.get(skill, r), r)

    with open(POLICY, encoding="utf-8") as fh:
        policy = json.load(fh)
    rows = policy.setdefault("skills", {})
    for skill in sorted(rate):
        cov = coverage_for(rate[skill])
        row = rows.setdefault(skill, {})
        old = row.get("coverage", policy.get("default", {}).get("coverage", 1000))
        row["coverage"] = cov
        print(f"{skill:<22} offers/game={rate[skill]:7.3f}  coverage {old:>4} -> {cov}")
    if write:
        with open(POLICY, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(dump(policy))
        print("wrote", os.path.normpath(POLICY))


def dump(policy):
    """One skill per line, aligned, so diffs of the table stay readable."""
    out = ["{"]
    for k, v in policy.items():
        if k == "skills":
            continue
        out.append(f"  {json.dumps(k)}: {json.dumps(v, ensure_ascii=False)},")
    out.append('  "skills": {')
    names = sorted(policy["skills"])
    width = max(len(n) for n in names) + 3
    for i, n in enumerate(names):
        row = ", ".join(f'"{f}": {policy["skills"][n][f]}' for f in ("use", "coverage") if f in policy["skills"][n])
        comma = "," if i < len(names) - 1 else ""
        out.append(f"    {(json.dumps(n) + ':').ljust(width)} {{ {row} }}{comma}")
    out.append("  }")
    out.append("}")
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    main()
