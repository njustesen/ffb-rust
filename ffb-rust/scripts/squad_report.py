"""Build a per-squad report for the 2026-09-10 re-draft: composition, spend, and the diff
against the squads it replaced.

Reads the NEW squads from data/teams/ and the OLD ones from a git revision (default HEAD, i.e.
before the re-draft is committed), and emits one JSON document with everything the review page
needs. Read-only.

Usage:
  python scripts/squad_report.py [--old-rev HEAD] [--out docs/squad_report.json] [--md]
"""

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(Path(__file__).resolve().parent))
from validate_teams import PAGES, BIG_GUY_TYPES, FANS, big_guy_cap  # noqa: E402

EDITIONS = ("bb2016", "bb2020", "bb2025")
BUDGET = 1_100_000
FORMAT = {"bb2016": "CRP tournament", "bb2020": "Exhibition Play", "bb2025": "Matched Play"}


def git_show(rev, path):
    try:
        out = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=ROOT.parent,
                             capture_output=True, check=True)
        return json.loads(out.stdout.decode("utf-8"))
    except subprocess.CalledProcessError:
        return None


def rosters(edition):
    out = {}
    for p in sorted((ROOT / "data" / "rosters" / edition).glob("roster_*.json")):
        r = json.loads(p.read_text(encoding="utf-8"))
        out[r["id"]] = r
    return out


def compose(spec, roster):
    by_id = {p["id"]: p for p in roster["positions"]}
    counts = Counter(p["position_id"] for p in spec["players"])
    rows = []
    for pid, n in counts.items():
        pos = by_id.get(pid, {})
        rows.append({
            "position_id": pid,
            "name": pos.get("display_name") or pos.get("name") or pid,
            "type": pos.get("type", "?"),
            "n": n,
            "quantity": pos.get("quantity"),
            "cost": pos.get("cost", 0),
            "total": pos.get("cost", 0) * n,
            "ma": pos.get("ma"), "st": pos.get("st"), "ag": pos.get("ag"),
            "pa": pos.get("pa"), "av": pos.get("av"),
            "skills": [s if isinstance(s, str) else s.get("name") for s in pos.get("skills", [])],
            "big_guy": pos.get("type", "").lower().replace(" ", "") in {"bigguy"},
        })
    rows.sort(key=lambda r: (-r["cost"], r["position_id"]))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--old-rev", default="HEAD")
    ap.add_argument("--out", default="docs/squad_report.json")
    ap.add_argument("--md", action="store_true",
                    help="also rewrite docs/TEAM_DRAFTS_<EDITION>.md from the report")
    args = ap.parse_args()

    doc = {"budget": BUDGET, "old_rev": args.old_rev, "editions": {}, "squads": []}
    for ed in EDITIONS:
        ros = rosters(ed)
        doc["editions"][ed] = {
            "format": FORMAT[ed],
            "fans_field": FANS[ed][0], "fans_start": FANS[ed][1], "fans_max": FANS[ed][2],
            "fans_cost": FANS[ed][3], "fans_in_tv": FANS[ed][4],
        }
        for path in sorted((ROOT / "data" / "teams" / ed).glob("team_*.json")):
            stem = path.stem[len("team_"):]
            spec = json.loads(path.read_text(encoding="utf-8"))
            roster = ros[spec["roster_id"]]
            rel = f"ffb-rust/data/teams/{ed}/{path.name}"
            old = git_show(args.old_rev, rel)

            rows = compose(spec, roster)
            n = len(spec["players"])
            unfielded = [p["id"] for p in roster["positions"]
                         if p.get("type") not in ("Star", "Infamous Staff")
                         and p["id"] not in {r["position_id"] for r in rows}]
            entry = {
                "edition": ed, "cell": stem, "race": spec["race"],
                "roster_id": spec["roster_id"], "roster_name": roster["name"],
                "players": n, "rerolls": spec["rerolls"], "reroll_cost": roster["reroll_cost"],
                "apothecaries": spec["apothecaries"],
                "apothecary_allowed": bool(roster.get("apothecary")),
                "assistant_coaches": spec.get("assistant_coaches", 0),
                "cheerleaders": spec.get("cheerleaders", 0),
                "dedicated_fans": spec.get("dedicated_fans", 0),
                "fan_factor": spec.get("fan_factor", 0),
                "team_value": spec["team_value"], "spent": spec["spent"],
                "treasury": spec["treasury"],
                "big_guy_cap": big_guy_cap(ed, spec["race"]),
                "big_guys": sum(r["n"] for r in rows if r["big_guy"]),
                "composition": rows,
                "unfielded_positionals": unfielded,
            }
            if old:
                old_counts = Counter(p["position_id"] for p in old["players"])
                new_counts = Counter(p["position_id"] for p in spec["players"])
                entry["old"] = {
                    "players": len(old["players"]),
                    "rerolls": old["rerolls"], "apothecaries": old["apothecaries"],
                    "assistant_coaches": old.get("assistant_coaches", 0),
                    "cheerleaders": old.get("cheerleaders", 0),
                    "dedicated_fans": old.get("dedicated_fans", 0),
                    "fan_factor": old.get("fan_factor", 0),
                    "team_value": old["team_value"], "spent": old["spent"],
                    "treasury": old["treasury"],
                    "stars": [s["star_id"] for s in (old.get("stars") or [])],
                }
                delta = {}
                for pid in set(old_counts) | set(new_counts):
                    d = new_counts[pid] - old_counts[pid]
                    if d:
                        delta[pid] = d
                entry["delta_positions"] = delta
            doc["squads"].append(entry)

    out = ROOT / args.out
    out.write_text(json.dumps(doc, indent=1), encoding="utf-8")
    stars = sum(len(s.get("old", {}).get("stars", [])) for s in doc["squads"])
    print(f"{len(doc['squads'])} squads -> {out}")
    print(f"stars removed: {stars}")
    print(f"squads with staff: {sum(1 for s in doc['squads'] if s['assistant_coaches'] + s['cheerleaders'])}")
    print(f"old treasury total: {sum(s.get('old',{}).get('treasury',0) for s in doc['squads']):,}")
    print(f"new treasury total: {sum(s['treasury'] for s in doc['squads']):,}")

    if args.md:
        for ed in EDITIONS:
            md = ROOT / "docs" / f"TEAM_DRAFTS_{ed.upper()}.md"
            md.write_text(markdown(doc, ed) + chr(10), encoding="utf-8")
            print(f"wrote {md.relative_to(ROOT)}")


# ── markdown spend tables (docs/TEAM_DRAFTS_<EDITION>.md) ───────────────────────────────

DOC_HEAD = {
    "bb2016": ("BB2016", "CRP tournament", "`docs/BB2016_DRAFTING_AND_ROSTERS.md`",
               "Fan Factor 0-9 at 10,000 each, which **counts in Team Value**"),
    "bb2020": ("BB2020", "Exhibition Play",
               "`rules/bb2020/core_rules/07_league_and_exhibition_play.md`",
               "Dedicated Fans from 0 up to 6, at 10,000 each, not in TV"),
    "bb2025": ("BB2025", "Matched Play",
               "`rules/core_rules/04_drafting_a_blood_bowl_team.md` + `06_matched_play.md`",
               "Dedicated Fans from 1 up to 3, at 5,000 each, not in TV"),
}


def k(n):
    return f"{n // 1000}k"


def markdown(doc, edition):
    name, fmt, src, fans = DOC_HEAD[edition]
    squads = [s for s in doc["squads"] if s["edition"] == edition]
    L = [f"# {name} Parity Team Drafts", ""]
    L += [
        f"Re-drafted **2026-09-10** by `scripts/draft_all_squads.py` under the {fmt} drafting",
        f"rules ({src}). Superseded the hand drafts of 2026-08-08, which were legal only against",
        "a checker calibrated on themselves -- see `docs/PARITY_COVERAGE_REQUIREMENTS.md` §19 (R6)",
        "for the seven defects that survived it.", "",
        "Budget 1,100,000 gold, **spent in full** (in this play format unspent gold is lost, and",
        f"the drafter proves nothing cheaper remained buyable). {fans}. Team re-rolls 0-8 at the",
        "roster's cost; assistant coaches and cheerleaders 0-6 each at 10,000; apothecary 50,000",
        "where the roster allows one. **No star players**: a star is an Inducement, needing gold",
        "and Skill Points, and a mirror match has no inducement gold.", "",
        "Purchase order: one of every positional the caps allow; then more players dearest-first",
        "while 11 bodies, 3 re-rolls and an apothecary stay reachable; then the 11th and 12th",
        "bodies; 2 re-rolls; apothecary; a 3rd re-roll; a 13th body; fans; assistant coaches and",
        "cheerleaders; then players to 16 and re-rolls to 8 with anything left.", "",
        "Jerseys are numbered round-robin over the positions, so one of every positional takes a",
        "shirt inside the first 11 and starts on the pitch -- both harnesses field the first 11 by",
        "number, and a positional on the bench has no parity evidence.", "",
        f"Specs: `data/teams/{edition}/team_<cell>.json` (home and away are identical builds).",
        "TV per the Java `UtilTeamValue.findTeamValue`.", "",
    ]
    for s in squads:
        L.append(f"## `{s['cell']}`")
        L.append("")
        if s["cell"] != s["race"]:
            L.append(f"R3 variant of the `{s['race']}` cell (roster `{s['roster_id']}`), fielding "
                     f"the Big Guy the base squad may not.")
            L.append("")
        L += ["| Purchase | Qty | Cost |", "|---|---|---|"]
        for r in s["composition"]:
            L.append(f"| {r['name']} | {r['n']} | {k(r['total'])} |")
        L.append(f"| Team re-rolls @{k(s['reroll_cost'])} | {s['rerolls']} "
                 f"| {k(s['rerolls'] * s['reroll_cost'])} |")
        if s["apothecaries"]:
            L.append("| Apothecary | 1 | 50k |")
        if s["assistant_coaches"]:
            L.append(f"| Assistant coaches | {s['assistant_coaches']} "
                     f"| {k(s['assistant_coaches'] * 10_000)} |")
        if s["cheerleaders"]:
            L.append(f"| Cheerleaders | {s['cheerleaders']} | {k(s['cheerleaders'] * 10_000)} |")
        if edition == "bb2016":
            if s["fan_factor"]:
                L.append(f"| Fan Factor 0→{s['fan_factor']} | +{s['fan_factor']} "
                         f"| {k(s['fan_factor'] * 10_000)} |")
        else:
            start = 0 if edition == "bb2020" else 1
            cost = 10_000 if edition == "bb2020" else 5_000
            step = max(0, s["dedicated_fans"] - start)
            if step:
                L.append(f"| Dedicated Fans {start}→{s['dedicated_fans']} | +{step} "
                         f"| {k(step * cost)} |")
        L.append(f"| **Total spent** | | **{k(s['spent'])}** |")
        L.append(f"| Gold lost (unspendable) | | {k(s['treasury'])} |")
        L.append("")
        note = (f"{s['players']} players, TV {k(s['team_value'])}.")
        if s["big_guy_cap"] is not None:
            note += (f" The official page allows {s['big_guy_cap']} Big Guy"
                     f"{'s' if s['big_guy_cap'] > 1 else ''}; this squad has {s['big_guys']}.")
        if s["unfielded_positionals"]:
            note += (" Held back by that cap: "
                     + ", ".join(f"`{x}`" for x in s["unfielded_positionals"])
                     + " -- fielded by this cell's other squad.")
        L.append(note)
        L.append("")
    return "\n".join(L)


if __name__ == "__main__":
    sys.exit(main())
