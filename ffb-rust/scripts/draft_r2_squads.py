"""Close the 13 unfielded positional slots on EXISTING races (docs/PARITY_COVERAGE_REQUIREMENTS.md
§8, "Measured state of R2/R4 today").

Ten drafted squads never fielded 13 positionals between them, so those stat lines and starting
skills had NO parity evidence. This script is the hand-draft that closes them. As with
scripts/draft_coverage_teams.py, the PURCHASES table below IS the draft (an explicit, reviewable
purchase list); the code only does the arithmetic and enforces the invariants, so a typo'd
position id fails HERE instead of silently producing a wrong squad -- or, worse, an ALL-LINEMAN
fallback team (make_team() in crates/ffb-parity/src/runner.rs falls back on error with a
log::warn! only, and a fallback team still gates green).

Two shapes of fix, decided per cell by the official page:

  AMEND  -- the positional simply fits in the existing squad within the rules. The squad file is
            rewritten. THESE RACES MUST BE RE-GATED (their earlier gates were on a different
            squad):
              bb2016 orc        -- 1 Black Orc Blocker traded for 1 Orc Lineman
              bb2020 dwarf      -- Troll Slayer added (re-rolls 3 -> 2 to pay for it)
              bb2020 undead     -- 1 Zombie traded for 1 Skeleton (same cost)
              bb2020 underworld -- 2 Underworld Snotlings added (treasury spent to 0)

  VARIANT -- the official page caps the team ("may have a single Big Guy" / "up to three Big
            Guys"), so the missing positional CANNOT join the existing squad. A new squad file
            team_<race>_<positional>.json reuses the SAME roster (matched by its own roster_id
            field, exactly as make_team_from_file does) and swaps in the missing Big Guy. The
            base squad is untouched, so its existing gates stand; R2 is satisfied by the UNION
            of the cell's squads (R4).
              bb2020 chaos      -- single Big Guy: chaosogre, chaostroll variants
              bb2020 chaos_pact -- 4 Big Guys, 3 allowed: renegadetroll variant
              bb2020 renegades  -- 4 Big Guys, 3 allowed: 37730 (Troll) variant
              bb2020 underworld -- single Big Guy: underworldtroll variant
              bb2025 chaos      -- single Big Guy: ogre, troll variants
              bb2025 renegades  -- 4 Big Guys, 3 allowed: 37733 (Rat Ogre) variant
              bb2025 underworld -- single Big Guy: 37844 (Rat Ogre) variant

Money formula is the one scripts/validate_teams.py derived from the whole tree:
  players = sum(position.cost);  staff = rerolls*reroll_cost + apothecaries*50_000
  bb2016 : team_value = players + staff + fan_factor*10_000 ; spent = team_value
  bb2020+: team_value = players + staff ; spent = team_value + (dedicated_fans-1)*5_000
  always : spent + treasury == 1_100_000

Usage: python scripts/draft_r2_squads.py [--check]
Then:  python scripts/validate_teams.py   (expect 0 violations, 0 unfielded slots)
"""

import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUDGET = 1_100_000
APOTHECARY_COST = 50_000
DEDICATED_FAN_COST = 5_000
FAN_FACTOR_COST_BB2016 = 10_000

# (edition, squad file name) -> the draft. `race` is the ROSTER race (NOT the file name):
# validate_teams.py groups a cell's variants by this field, and gen_java_parity_data.py
# resolves the roster through the squad's roster_id.
PURCHASES = {
    # ---- AMENDED existing squads (re-gate these races) --------------------------------
    ("bb2016", "orc"): dict(
        race="orc", rerolls=2, apothecaries=0, dedicated_fans=0, fan_factor=1, buy=[
            ("orc.troll", 1), ("orc.blitzer", 4), ("orc.blackorc", 3),
            ("orc.thrower", 2), ("orc.goblin", 2), ("orc.lineman", 1)]),
    ("bb2020", "dwarf"): dict(
        race="dwarf", rerolls=2, apothecaries=0, dedicated_fans=3, buy=[
            ("dwarf.deathroller", 1), ("dwarf.trollslayer", 1), ("dwarf.blitzer", 1),
            ("dwarf.runner", 1), ("dwarf.dwarfblockerlineman", 8)]),
    ("bb2020", "undead"): dict(
        race="undead", rerolls=3, apothecaries=0, dedicated_fans=3, buy=[
            ("undead.mummy", 1), ("undead.wight", 2), ("undead.ghoul", 4),
            ("undead.zombie", 5), ("undead.skeleton", 1)]),
    ("bb2020", "underworld"): dict(
        race="underworld", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
            ("37844", 1), ("underworld.skaven.blitzer", 1), ("underworld.skaven.thrower", 1),
            ("37844.gutterrunner", 1), ("37844.skavenclanrat", 3),
            ("underworld.goblin", 6), ("37844.underworldsnotling", 2)]),

    # ---- R3 VARIANTS (base squads untouched) -----------------------------------------
    ("bb2020", "chaos_chaosogre"): dict(
        race="chaos", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
            ("chaos.chaosogre", 1), ("chaos.chosenblocker", 1),
            ("chaos.beastmanrunnerlineman", 10)]),
    ("bb2020", "chaos_chaostroll"): dict(
        race="chaos", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
            ("chaos.chaostroll", 1), ("chaos.chosenblocker", 2),
            ("chaos.beastmanrunnerlineman", 9)]),
    ("bb2020", "chaos_pact_renegadetroll"): dict(
        race="chaos_pact", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
            ("chaospact.renegadetroll", 1), ("chaospact.renegaderatogre", 1),
            ("chaospact.renegadeogre", 1), ("chaospact.renegadehumanthrower", 1),
            ("chaospact.renegadedarkelf", 1), ("chaospact.renegadeorc", 1),
            ("chaospact.renegadeskaven", 1), ("chaospact.renegadegoblin", 1),
            ("chaospact.renegadehumanlineman", 4)]),
    ("bb2020", "renegades_37730"): dict(
        race="renegades", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
            ("37730", 1), ("37731", 1), ("37733", 1), ("37725", 1), ("37729", 1),
            ("37727", 1), ("37728", 1), ("37726", 1), ("37724", 4)]),
    ("bb2020", "underworld_underworldtroll"): dict(
        race="underworld", rerolls=3, apothecaries=1, dedicated_fans=3, buy=[
            ("37844.underworldtroll", 1), ("underworld.skaven.blitzer", 1),
            ("underworld.skaven.thrower", 1), ("37844.gutterrunner", 1),
            ("37844.skavenclanrat", 3), ("underworld.goblin", 6),
            ("37844.underworldsnotling", 3)]),
    ("bb2025", "chaos_ogre"): dict(
        race="chaos", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
            ("chaos.ogre", 1), ("chaos.warrior", 4), ("chaos.beastman", 7)]),
    ("bb2025", "chaos_troll"): dict(
        race="chaos", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
            ("chaos.troll", 1), ("chaos.warrior", 4), ("chaos.beastman", 7)]),
    ("bb2025", "renegades_37733"): dict(
        race="renegades", rerolls=2, apothecaries=1, dedicated_fans=3, buy=[
            ("37733", 1), ("37731", 1), ("37730", 1), ("37725", 1), ("37729", 1),
            ("37727", 1), ("37728", 1), ("37726", 1), ("37724", 4)]),
    ("bb2025", "underworld_37844"): dict(
        race="underworld", rerolls=4, apothecaries=1, dedicated_fans=3, buy=[
            ("37844", 1), ("underworld.skaven.blitzer", 1), ("underworld.gutter_runner", 1),
            ("underworld.skaven.thrower", 1), ("underworld.skaven.lineman", 3),
            ("underworld.goblin", 2), ("underworld.snotling_lineman", 6)]),
}

BIG_GUY_TYPES = {"big guy", "bigguy"}


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def roster_for(edition, race):
    return load(ROOT / "data" / "rosters" / edition / ("roster_%s.json" % race))


def build(edition, squad, spec, roster):
    by_id = {p["id"]: p for p in roster["positions"]}
    players, nr, players_cost = [], 1, 0
    for pid, cnt in spec["buy"]:
        pos = by_id[pid]                     # KeyError = typo'd position id, fails loudly
        assert cnt <= pos["quantity"], "%s/%s: %dx %s over cap %d" % (
            edition, squad, cnt, pid, pos["quantity"])
        for _ in range(cnt):
            players.append({"nr": nr, "position_id": pid})
            nr += 1
            players_cost += pos["cost"]
    staff = spec["rerolls"] * roster["reroll_cost"] + spec["apothecaries"] * APOTHECARY_COST
    fan_factor = spec.get("fan_factor", 0)
    if edition == "bb2016":
        tv = players_cost + staff + fan_factor * FAN_FACTOR_COST_BB2016
        spent = tv
    else:
        tv = players_cost + staff
        spent = tv + max(0, spec["dedicated_fans"] - 1) * DEDICATED_FAN_COST
    return {
        "edition": edition,
        "race": spec["race"],
        "roster_id": roster["id"],
        "rerolls": spec["rerolls"],
        "reroll_cost": roster["reroll_cost"],
        "apothecaries": spec["apothecaries"],
        "dedicated_fans": spec["dedicated_fans"],
        "fan_factor": fan_factor,
        "treasury": BUDGET - spent,
        "spent": spent,
        "team_value": tv,
        "players": players,
        "special_rules": [],
    }


def main():
    check = "--check" in sys.argv
    problems, written = [], 0
    for (edition, squad), spec in sorted(PURCHASES.items()):
        roster = roster_for(edition, spec["race"])
        team = build(edition, squad, spec, roster)
        by_id = {p["id"]: p for p in roster["positions"]}

        n = len(team["players"])
        if not 11 <= n <= 16:
            problems.append("%s/%s: %d players (need 11-16)" % (edition, squad, n))
        if team["treasury"] < 0:
            problems.append("%s/%s: overspent by %d" % (edition, squad, -team["treasury"]))
        if team["rerolls"] < 2 or team["rerolls"] > roster.get("max_rerolls", 8):
            problems.append("%s/%s: %d re-rolls out of range" % (edition, squad, team["rerolls"]))
        if team["apothecaries"] and not roster.get("apothecary"):
            problems.append("%s/%s: apothecary not allowed by roster" % (edition, squad))
        # Big Guy cap: max_big_guys on the roster (renegades 3, underworld 2, ...), and 1 for a
        # page that says "may have a single Big Guy" -- both editions' Chaos Chosen and
        # Underworld Denizens pages do, and their rosters carry max_big_guys accordingly.
        bg = sum(1 for p in team["players"]
                 if by_id[p["position_id"]].get("type", "").lower() in BIG_GUY_TYPES)
        cap = SINGLE_BIG_GUY.get((edition, spec["race"]), roster.get("max_big_guys", 99))
        if bg > cap:
            problems.append("%s/%s: %d Big Guys, page allows %d" % (edition, squad, bg, cap))

        out = ROOT / "data" / "teams" / edition / ("team_%s.json" % squad)
        text = json.dumps(team, indent=2) + "\n"
        if check:
            if not out.exists() or out.read_text(encoding="utf-8") != text:
                print("DIFF %s" % out.relative_to(ROOT))
        else:
            out.write_text(text, encoding="utf-8")
            written += 1
        cnt = Counter(p["position_id"] for p in team["players"])
        print("%s %-28s %2d players  spent %9d  TV %9d  rr %d  apo %d  treasury %8d  "
              "%d positionals  %d big guy(s)"
              % (edition, squad, n, team["spent"], team["team_value"], team["rerolls"],
                 team["apothecaries"], team["treasury"], len(cnt), bg))

    if problems:
        print("\nPROBLEMS:")
        for p in problems:
            print("  " + p)
        return 1
    print("\ncheck complete" if check else "\n%d squads written, all invariants OK" % written)
    return 0


# Cells whose official page reads "may have a single Big Guy" (verified in
# rules/{,bb2020/}teams/{Chaos_Chosen,Underworld_Denizens}.md).
SINGLE_BIG_GUY = {
    ("bb2020", "chaos"): 1,
    ("bb2025", "chaos"): 1,
    ("bb2020", "underworld"): 1,
    ("bb2025", "underworld"): 1,
}


if __name__ == "__main__":
    sys.exit(main())
