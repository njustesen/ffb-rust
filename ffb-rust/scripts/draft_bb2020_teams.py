"""Draft the BB2020 parity teams ONCE, the same way bb2016/bb2025 were drafted by hand.

Source of truth: data/rosters/bb2020/roster_<race>.json (audited roster data).
Output:          data/teams/bb2020/team_<race>.json  (frozen team specs)

Heuristics, copied verbatim from docs/TEAM_DRAFTS_BB2025.md so the three editions
are drafted alike:
  * budget 1,100,000
  * buy every positional type at least once when affordable, INCLUDING a Big Guy,
    respecting both the per-position `quantity` cap and the per-team shared
    Big-Guy limit (BB2020: most teams 1; the exceptions are listed in BIG_GUY_LIMIT)
  * 12+ players when affordable
  * 2+ team re-rolls (3 when they still fit)
  * apothecary when the roster allows it and it fits after positionals
  * Dedicated Fans up to 3 (5k each; BB2020 teams start at 1)
  * remainder is treasury
  * jerseys run premium positions first, so the first 11 jerseys are the starters
    and linemen sit in reserve

Team value uses Java's `UtilTeamValue` shape for BB2020: players + re-rolls +
apothecary (Dedicated Fans excluded, as in BB2025).

Usage: python scripts/draft_bb2020_teams.py [--check]
  --check  recompute and diff against the checked-in specs without writing
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ROSTERS = ROOT / "data" / "rosters" / "bb2020"
TEAMS = ROOT / "data" / "teams" / "bb2020"

BUDGET = 1_100_000
DEDICATED_FANS_TARGET = 3
DEDICATED_FANS_COST = 5_000
APOTHECARY_COST = 50_000

# BB2020 shared Big-Guy allowance per team, where it is NOT the default of 1.
# Same numbers docs/TEAM_DRAFTS_BB2025.md records for the equivalent rosters.
BIG_GUY_LIMIT = {
    "renegades": 3,     # Troll / Ogre / Minotaur / Rat Ogre share a 3-slot pool
    "chaos_pact": 3,    # ditto (the FUMBBL name for the same roster)
    "underworld": 1,
    "chaos": 1,
    "ogre": 6,          # Ogres ARE the roster's positionals, not a Big-Guy slot
    "halfling": 2,      # two Treemen
    "lizardman": 6,     # Saurus are regular positionals
    "slann": 1,
    "nurgle": 1,
    "khemri": 4,        # Tomb Guardians are regular positionals
}

# Words that mark a position as consuming a Big-Guy slot. Matched against the position
# NAME, not the id: the FUMBBL rosters use numeric ids ("37733" = Renegade Rat Ogre), so an
# id-based check silently misses them and the shared Big-Guy pool goes unenforced.
BIG_GUY_WORDS = (
    "troll", "ogre", "minotaur", "treeman", "kroxigor", "yhetee",
    "beast of nurgle", "mummy", "tomb guardian", "deathroller", "rat ogre",
)


def is_big_guy(pos: dict) -> bool:
    name = (pos.get("name") or pos.get("display_name") or pos["id"]).lower()
    return any(w in name for w in BIG_GUY_WORDS)


def draft(race: str, roster: dict) -> dict:
    # Retired/unavailable entries carry quantity 0 (e.g. `human.catcher.old`) — never buyable.
    positions = [p for p in roster["positions"] if p.get("quantity", 0) > 0]
    reroll_cost = roster["reroll_cost"]
    max_rerolls = roster.get("max_rerolls", 8)
    allows_apo = bool(roster.get("apothecary"))
    bg_limit = BIG_GUY_LIMIT.get(race, 1)

    # The "lineman" is the slot we backfill with, so it must be BULK-buyable: cheapest
    # position with a big quantity cap. Picking the cheapest position outright breaks the
    # renegade rosters, whose cheapest entry is a quantity-1 Renegade Goblin — backfill then
    # capped at one player and the squad came out at 9.
    bulk = [p for p in positions if p["quantity"] >= 6]
    lineman = min(bulk or positions, key=lambda p: p["cost"])

    # Premium positions, most expensive first; linemen handled separately.
    premium = sorted(
        (p for p in positions if p["id"] != lineman["id"]),
        key=lambda p: -p["cost"],
    )

    counts: dict[str, int] = {}
    spent = 0
    bg_used = 0
    apo_reserve = APOTHECARY_COST if allows_apo else 0
    df_reserve = (DEDICATED_FANS_TARGET - 1) * DEDICATED_FANS_COST

    def afford(cost: int, reserve: int) -> bool:
        return spent + cost <= BUDGET - reserve

    # 1. One of each premium position (Big Guys within their shared pool). Each buy must
    #    still leave room for 2 re-rolls AND enough cheap players to reach a legal 11 —
    #    otherwise the Big-Guy-heavy rosters (renegades/chaos_pact draft 3) spend themselves
    #    down to a 9-man squad.
    for pos in premium:
        cost = pos["cost"]
        if is_big_guy(pos):
            if bg_used >= bg_limit:
                continue
        squad_after = sum(counts.values()) + 1
        # Reserve the apothecary here too: a 160k Deathroller bought first otherwise starves
        # it, and a dwarf team with no apothecary is legal but not a realistic draft.
        reserve = (2 * reroll_cost + apo_reserve + df_reserve
                   + max(0, 11 - squad_after) * lineman["cost"])
        if not afford(cost, reserve):
            continue
        counts[pos["id"]] = 1
        spent += cost
        if is_big_guy(pos):
            bg_used += 1

    # 2. Deepen premium positions up to their caps, still most-expensive-first,
    #    while keeping room for re-rolls, an apothecary and 12 players.
    for pos in premium:
        cap = pos["quantity"]
        while counts.get(pos["id"], 0) < cap:
            if is_big_guy(pos) and bg_used >= bg_limit:
                break
            need_players = max(0, 12 - sum(counts.values()) - 1)
            reserve = (
                3 * reroll_cost + apo_reserve + df_reserve
                + need_players * lineman["cost"]
            )
            if not afford(pos["cost"], reserve):
                break
            counts[pos["id"]] = counts.get(pos["id"], 0) + 1
            spent += pos["cost"]
            if is_big_guy(pos):
                bg_used += 1

    # 3. Re-rolls: 3 if they fit alongside 12 players + apo + fans, else 2.
    rerolls = 0
    for target in (3, 2):
        if target > max_rerolls:
            continue
        # 12 players is the goal, 11 the hard floor: try the richer reserve first, then the floor.
        for floor in (12, 11):
            need_players = max(0, floor - sum(counts.values()))
            reserve = apo_reserve + df_reserve + need_players * lineman["cost"]
            if afford(target * reroll_cost, reserve):
                rerolls = target
                spent += target * reroll_cost
                break
        if rerolls:
            break
    if rerolls == 0:  # last resort — a team must have at least one
        rerolls = 1
        spent += reroll_cost

    # 4. Apothecary.
    apothecaries = 0
    if allows_apo:
        need_players = max(0, 12 - sum(counts.values()))
        if afford(APOTHECARY_COST, df_reserve + need_players * lineman["cost"]):
            apothecaries = 1
            spent += APOTHECARY_COST

    # 5. Dedicated Fans 1 -> 3.
    dedicated_fans = 1
    while dedicated_fans < DEDICATED_FANS_TARGET:
        need_players = max(0, 12 - sum(counts.values()))
        if not afford(DEDICATED_FANS_COST, need_players * lineman["cost"]):
            break
        dedicated_fans += 1
        spent += DEDICATED_FANS_COST

    # 6. Backfill linemen to 12 (then to 16 while the money lasts).
    while sum(counts.values()) < 11 and counts.get(lineman["id"], 0) < lineman["quantity"]:
        if not afford(lineman["cost"], 0):
            break
        counts[lineman["id"]] = counts.get(lineman["id"], 0) + 1
        spent += lineman["cost"]
    while sum(counts.values()) < 13 and counts.get(lineman["id"], 0) < lineman["quantity"]:
        if not afford(lineman["cost"], 0):
            break
        counts[lineman["id"]] = counts.get(lineman["id"], 0) + 1
        spent += lineman["cost"]

    # Jerseys: premium first (so jerseys 1-11 are the starters), linemen last.
    order = [p["id"] for p in premium if counts.get(p["id"])]
    if counts.get(lineman["id"]):
        order.append(lineman["id"])
    players = []
    nr = 1
    for pid in order:
        for _ in range(counts[pid]):
            players.append({"nr": nr, "position_id": pid})
            nr += 1

    by_id = {p["id"]: p for p in positions}
    player_value = sum(by_id[pid]["cost"] * n for pid, n in counts.items())
    # Java UtilTeamValue (BB2020, as BB2025): players + re-rolls + apothecary.
    team_value = player_value + rerolls * reroll_cost + apothecaries * APOTHECARY_COST

    return {
        "edition": "bb2020",
        "race": race,
        "roster_id": roster["id"],
        "rerolls": rerolls,
        "reroll_cost": reroll_cost,
        "apothecaries": apothecaries,
        "dedicated_fans": dedicated_fans,
        "fan_factor": 0,
        "treasury": BUDGET - spent,
        "spent": spent,
        "team_value": team_value,
        "players": players,
        "special_rules": [],
    }


def main() -> int:
    check = "--check" in sys.argv
    TEAMS.mkdir(parents=True, exist_ok=True)
    problems = []
    for path in sorted(ROSTERS.glob("roster_*.json")):
        race = path.stem[len("roster_"):]
        roster = json.loads(path.read_text(encoding="utf-8"))
        spec = draft(race, roster)

        # Validate before writing — these are the invariants the campaign relies on.
        n = len(spec["players"])
        if not 11 <= n <= 16:
            problems.append(f"{race}: {n} players (need 11-16)")
        if spec["spent"] > BUDGET:
            problems.append(f"{race}: overspent {spec['spent']}")
        if spec["treasury"] < 0:
            problems.append(f"{race}: negative treasury")
        caps = {p["id"]: p["quantity"] for p in roster["positions"]}
        from collections import Counter
        for pid, cnt in Counter(p["position_id"] for p in spec["players"]).items():
            if cnt > caps[pid]:
                problems.append(f"{race}: {pid} x{cnt} exceeds cap {caps[pid]}")

        out = TEAMS / f"team_{race}.json"
        text = json.dumps(spec, indent=2) + "\n"
        if check:
            old = out.read_text(encoding="utf-8") if out.exists() else ""
            if old != text:
                print(f"DIFF {out.relative_to(ROOT)}")
        else:
            out.write_text(text, encoding="utf-8")
        print(f"{race:26s} {n:2d} players  spent {spec['spent']:>9,}  "
              f"TV {spec['team_value']:>9,}  rr {spec['rerolls']}  "
              f"apo {spec['apothecaries']}  df {spec['dedicated_fans']}")

    if problems:
        print("\nPROBLEMS:")
        for p in problems:
            print("  " + p)
        return 1
    print(f"\n{len(list(ROSTERS.glob('roster_*.json')))} bb2020 teams drafted, all invariants OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
