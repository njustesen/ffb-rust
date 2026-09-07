"""Draft the parity squads for the 7 teams added for coverage (PARITY_COVERAGE_REQUIREMENTS §8).

The PURCHASES table below is the hand-draft: an explicit, reviewable purchase list per
(edition, squad). This script only does the arithmetic (spend / treasury / team value /
jersey numbers) and enforces the invariants, so a typo in a position id or a broken budget
fails here instead of silently producing a wrong squad.

Requirements satisfied (docs/PARITY_COVERAGE_REQUIREMENTS.md §8):
  R1 quantity caps + 1.1M budget + >=2 re-rolls + apothecary + Dedicated Fans 3
  R2 every positional of the roster appears in the squad ...
  R3 ... except where the official page caps the team at a single Big Guy
     (Old World Alliance, both editions) -- then one variant per Big Guy option,
     written as team_<race>_<bigguy>.json
  R4 the union of a cell's squads fields 100% of the positionals (asserted below)

Usage: python scripts/draft_coverage_teams.py [--check]
"""

import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUDGET = 1_100_000
APOTHECARY_COST = 50_000
DEDICATED_FANS_COST = 5_000

# (edition, squad_name) -> race (roster file), rerolls, apothecaries, dedicated_fans,
#                          buy = [(position_id, count), ...] premium-first so jerseys 1-11 start
PURCHASES = {
    ("bb2025", "black_orc"): dict(race="black_orc", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("blackorc.trained_troll", 1), ("blackorc.black_orc", 6), ("blackorc.goblin_bruiser", 5)]),
    ("bb2025", "gnome"): dict(race="gnome", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
        ("gnome.altern_forest_treeman", 2), ("gnome.gnome_beastmaster", 2),
        ("gnome.gnome_illusionist", 2), ("gnome.woodland_fox", 2), ("gnome.gnome_lineman", 7)]),
    ("bb2025", "imperial_nobility"): dict(race="imperial_nobility", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("imperialnobility.ogre", 1), ("imperialnobility.noble_blitzer", 2),
        ("imperialnobility.bodyguard", 3), ("imperialnobility.imperial_thrower", 2),
        ("imperialnobility.imperial_retainer", 4)]),
    ("bb2025", "khorne"): dict(race="khorne", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("khorne.bloodspawn", 1), ("khorne.bloodseeker", 3), ("khorne.khorngor", 2),
        ("khorne.bloodborn_marauder", 6)]),
    ("bb2025", "old_world_alliance_ogre"): dict(race="old_world_alliance", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("oldworldalliance.ogre", 1), ("oldworldalliance.dwarf_blitzer", 1),
        ("oldworldalliance.trollslayer", 1), ("oldworldalliance.human_blitzer", 1),
        ("oldworldalliance.dwarf_runner", 1), ("oldworldalliance.human_catcher", 1),
        ("oldworldalliance.human_thrower", 1), ("oldworldalliance.dwarf_lineman", 1),
        ("oldworldalliance.halfling_hopeful", 1), ("oldworldalliance.human_lineman", 3)]),
    ("bb2025", "old_world_alliance_treeman"): dict(race="old_world_alliance", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("oldworldalliance.altern_forest_treeman", 1), ("oldworldalliance.dwarf_blitzer", 1),
        ("oldworldalliance.trollslayer", 1), ("oldworldalliance.human_blitzer", 1),
        ("oldworldalliance.dwarf_runner", 1), ("oldworldalliance.human_catcher", 1),
        ("oldworldalliance.human_thrower", 1), ("oldworldalliance.dwarf_lineman", 1),
        ("oldworldalliance.halfling_hopeful", 1), ("oldworldalliance.human_lineman", 3)]),
    ("bb2025", "snotling"): dict(race="snotling", rerolls=5, apothecaries=1, dedicated_fans=3, buy=[
        ("snotling.trained_troll", 2), ("snotling.pump_wagon", 2), ("snotling.fungus_flinga", 2),
        ("snotling.stilty_runna", 2), ("snotling.fun_hoppa", 2), ("snotling.snotling_lineman", 6)]),
    ("bb2025", "bretonnian"): dict(race="bretonnian", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
        ("bretonnian.grail_knight", 2), ("bretonnian.knight_catcher", 2),
        ("bretonnian.knight_thrower", 2), ("bretonnian.squire", 6)]),

    ("bb2020", "black_orc"): dict(race="black_orc", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("blackorc.trained_troll", 1), ("blackorc.black_orc", 6),
        ("blackorc.goblin_bruiser_lineman", 5)]),
    ("bb2020", "gnome"): dict(race="gnome", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
        ("gnome.altern_forest_treeman", 2), ("gnome.gnome_beastmaster", 2),
        ("gnome.gnome_illusionist", 2), ("gnome.woodland_fox", 2), ("gnome.gnome_lineman", 7)]),
    ("bb2020", "imperial_nobility"): dict(race="imperial_nobility", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("imperialnobility.ogre", 1), ("imperialnobility.noble_blitzer", 2),
        ("imperialnobility.bodyguard", 2), ("imperialnobility.imperial_thrower", 1),
        ("imperialnobility.imperial_retainer_lineman", 6)]),
    ("bb2020", "khorne"): dict(race="khorne", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("khorne.bloodspawn", 1), ("khorne.bloodseeker", 2), ("khorne.khorngor", 2),
        ("khorne.bloodborn_marauder_lineman", 7)]),
    ("bb2020", "old_world_alliance_ogre"): dict(race="old_world_alliance", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("oldworldalliance.ogre", 1), ("oldworldalliance.trollslayer", 1),
        ("oldworldalliance.human_blitzer", 1), ("oldworldalliance.dwarf_runner", 1),
        ("oldworldalliance.human_thrower", 1), ("oldworldalliance.dwarf_blitzer", 1),
        ("oldworldalliance.dwarf_blocker", 1), ("oldworldalliance.human_catcher", 1),
        ("oldworldalliance.halfling_hopeful", 1), ("oldworldalliance.human_lineman", 3)]),
    ("bb2020", "old_world_alliance_treeman"): dict(race="old_world_alliance", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
        ("oldworldalliance.altern_forest_treeman", 1), ("oldworldalliance.trollslayer", 1),
        ("oldworldalliance.human_blitzer", 1), ("oldworldalliance.dwarf_runner", 1),
        ("oldworldalliance.human_thrower", 1), ("oldworldalliance.dwarf_blitzer", 1),
        ("oldworldalliance.dwarf_blocker", 1), ("oldworldalliance.human_catcher", 1),
        ("oldworldalliance.halfling_hopeful", 1), ("oldworldalliance.human_lineman", 3)]),
    ("bb2020", "snotling"): dict(race="snotling", rerolls=5, apothecaries=1, dedicated_fans=3, buy=[
        ("snotling.trained_troll", 2), ("snotling.pump_wagon", 2), ("snotling.fungus_flinga", 2),
        ("snotling.stilty_runna", 2), ("snotling.fun_hoppa", 2), ("snotling.snotling_lineman", 6)]),
}


def build(edition, squad, spec, roster):
    by_id = {p["id"]: p for p in roster["positions"]}
    players, nr = [], 1
    players_cost = 0
    for pid, cnt in spec["buy"]:
        pos = by_id[pid]                       # KeyError = typo'd position id, fails loudly
        assert cnt <= pos["quantity"], "%s/%s: %dx %s over cap %d" % (
            edition, squad, cnt, pid, pos["quantity"])
        for _ in range(cnt):
            players.append({"nr": nr, "position_id": pid})
            nr += 1
            players_cost += pos["cost"]
    rr_cost = spec["rerolls"] * roster["reroll_cost"]
    apo_cost = spec["apothecaries"] * APOTHECARY_COST
    fans_cost = (spec["dedicated_fans"] - 1) * DEDICATED_FANS_COST
    spent = players_cost + rr_cost + apo_cost + fans_cost
    return {
        "edition": edition,
        "race": spec["race"],
        "roster_id": roster["id"],
        "rerolls": spec["rerolls"],
        "reroll_cost": roster["reroll_cost"],
        "apothecaries": spec["apothecaries"],
        "dedicated_fans": spec["dedicated_fans"],
        "fan_factor": 0,
        "treasury": BUDGET - spent,
        "spent": spent,
        "team_value": players_cost + rr_cost + apo_cost,
        "players": players,
        "special_rules": [],
    }


def main():
    check = "--check" in sys.argv
    problems, written = [], 0
    fielded = {}
    for (edition, squad), spec in sorted(PURCHASES.items()):
        rpath = ROOT / "data" / "rosters" / edition / ("roster_%s.json" % spec["race"])
        roster = json.loads(rpath.read_text(encoding="utf-8"))
        team = build(edition, squad, spec, roster)

        n = len(team["players"])
        if not 11 <= n <= 16:
            problems.append("%s/%s: %d players (need 11-16)" % (edition, squad, n))
        if team["treasury"] < 0:
            problems.append("%s/%s: overspent by %d" % (edition, squad, -team["treasury"]))
        if team["rerolls"] < 2 or team["rerolls"] > roster.get("max_rerolls", 8):
            problems.append("%s/%s: %d re-rolls out of range" % (edition, squad, team["rerolls"]))
        if team["apothecaries"] and not roster.get("apothecary"):
            problems.append("%s/%s: apothecary not allowed by roster" % (edition, squad))

        fielded.setdefault((edition, spec["race"]), set()).update(
            p["position_id"] for p in team["players"])

        out = ROOT / "data" / "teams" / edition / ("team_%s.json" % squad)
        text = json.dumps(team, indent=2) + "\n"
        if check:
            if not out.exists() or out.read_text(encoding="utf-8") != text:
                print("DIFF %s" % out.relative_to(ROOT))
        else:
            out.write_text(text, encoding="utf-8")
            written += 1
        cnt = Counter(p["position_id"] for p in team["players"])
        print("%s %-28s %2d players  spent %9d  TV %9d  rr %d  treasury %8d  %d positionals"
              % (edition, squad, n, team["spent"], team["team_value"], team["rerolls"],
                 team["treasury"], len(cnt)))

    # R4: the union of each cell's squads must field 100% of the roster's positionals.
    for (edition, race), got in sorted(fielded.items()):
        roster = json.loads((ROOT / "data" / "rosters" / edition /
                             ("roster_%s.json" % race)).read_text(encoding="utf-8"))
        want = set(p["id"] for p in roster["positions"] if p.get("quantity", 0) > 0)
        missing = want - got
        if missing:
            problems.append("%s/%s: R4 union misses %s" % (edition, race, sorted(missing)))
        else:
            print("R4 OK  %s/%s: union fields all %d positionals" % (edition, race, len(want)))

    if problems:
        print("\nPROBLEMS:")
        for p in problems:
            print("  " + p)
        return 1
    print("\ncheck complete" if check else "\n%d squads written, all invariants OK" % written)
    return 0


if __name__ == "__main__":
    sys.exit(main())
