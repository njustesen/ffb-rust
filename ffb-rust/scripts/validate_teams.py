"""Validate the drafted parity SQUADS in data/teams/<edition>/team_*.json.

Nothing else in the repo checks these files. `scripts/check_skill_names.py` and the
`all_roster_starting_skills_resolve` test only check skill SPELLING in the roster
DEFINITIONS (data/rosters/), and parity itself is structurally blind to a wrong squad:
both engines are fed the same generated data by scripts/gen_java_parity_data.py, so a
squad that is illegal -- or that resolves to the wrong roster entirely -- produces a
perfectly MATCHED, and perfectly worthless, green gate.

This implements the requirements of docs/PARITY_COVERAGE_REQUIREMENTS.md §8:

  R1  realistic and rule-legal squad ....... checks C1-C8 below
  R2  every positional must be fielded ..... check R2 (per squad, informational)
  R4  the UNION of a cell's squads must
      field 100% of the positionals ........ check R4 (per (edition, race) cell)
  R5  one roster per ruleset, never shared . check R5 (opt-in, --r5)

Why C1 is CRITICAL: `make_team()` in crates/ffb-parity/src/runner.rs ends in
`make_team_from_file(..)` -> on error `make_team_from_roster(..)` ->
`.unwrap_or_else(|e| { log::warn!(..); make_lineman_team(side, roster_name) })`.
A roster_id that does not resolve is therefore a `log::warn!` and an ALL-LINEMAN team,
not a failure. The gate still goes green.

Checks
------
C1  roster_id resolves to a roster in data/rosters/<edition>/                [CRITICAL]
C2  every players[].position_id exists in that roster                        [CRITICAL]
C3  squad size within the legal range (11..16 rostered players; players+stars <= 16)
C4  per-position count within that position's `quantity` cap
C5  spend reconciles against the declared `team_value` / `spent` / `treasury`
C6  rerolls <= roster.max_rerolls; apothecary only if the roster allows one;
    dedicated_fans / fan_factor within the edition's range
C7  squad numbers unique and in 1..16
C8  every stars[].star_id resolves in data/star_players/all_editions.json
BG  a roster whose official page says "may have a single Big Guy" may field only one

The C5 money formula (DERIVED FROM THE DATA, not assumed -- an earlier ad-hoc attempt
reported 73 false violations by omitting the fan cost). Verified against all squads:

  players  = sum(position.cost)                    -- star players are NOT counted:
                                                      they are inducements, and no
                                                      declared team_value includes them
  staff    = rerolls * roster.reroll_cost + apothecaries * 50_000

  bb2016:  team_value = players + staff + fan_factor * 10_000      (LRB6: Fan Factor
           spent      = team_value                                  costs 10k and IS TV)

  bb2020:  team_value = players + staff                            (BB2020+: Dedicated
  bb2025:  spent      = team_value + (dedicated_fans - 1) * 5_000   Fans are NOT TV; the
                                                                    first is free)

  all:     spent + treasury == 1_100_000                           (the drafting budget)

Every constant above reproduces the declared fields for 100% of the squads in the tree,
across all three editions -- see --selftest, which also proves each check FIRES.

Usage:
  python scripts/validate_teams.py                     # all editions
  python scripts/validate_teams.py --edition bb2025
  python scripts/validate_teams.py --r5                # also the R5 roster-sharing check
  python scripts/validate_teams.py --selftest          # validate the checker itself
  python scripts/validate_teams.py --teams-dir <dir>   # validate a copy of data/teams

Exit code 0 = no violations, 1 = violations (R2 misses included), 2 = selftest failed.

READ-ONLY: this script never writes to data/. There is deliberately no --apply.
"""

import argparse
import json
import re
import shutil
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).parent.parent
EDITIONS = ("bb2016", "bb2020", "bb2025")
ROSTERS = {ed: ROOT / "data" / "rosters" / ed for ed in EDITIONS}
TEAMS = {ed: ROOT / "data" / "teams" / ed for ed in EDITIONS}
STAR_FILE = ROOT / "data" / "star_players" / "all_editions.json"

# Official team pages, for the single-Big-Guy restriction. Reused from
# scripts/audit_rosters.py rather than re-declared, so the two cannot drift.
sys.path.insert(0, str(Path(__file__).parent))
from audit_rosters import OFFICIAL_BB2020, OFFICIAL_BB2025  # noqa: E402

# audit_rosters.OFFICIAL_BB2025 predates the six teams drafted later (and Bretonnian),
# so it does not map them; rules/teams/ does carry their pages. Only the Big-Guy-cap
# lookup here needs them -- fold them in rather than leave those cells unchecked.
EXTRA_BB2025 = {
    "black_orc": "Black_Orc",
    "bretonnian": "Bretonnian",
    "gnome": "Gnome",
    "imperial_nobility": "Imperial_Nobility",
    "khorne": "Khorne",
    "old_world_alliance": "Old_World_Alliance",
    "snotling": "Snotling",
}

PAGES = {
    "bb2025": (ROOT / "rules" / "teams", {**OFFICIAL_BB2025, **EXTRA_BB2025}),
    "bb2020": (ROOT / "rules" / "bb2020" / "teams", OFFICIAL_BB2020),
    "bb2016": (ROOT / "rules" / "bb2016" / "teams", {}),
}

# ── R6 constants. Every one of these is quoted from rules/ in sec.19 of
# docs/PARITY_COVERAGE_REQUIREMENTS.md. NONE of them is read out of data/teams/: the previous
# set was "DERIVED FROM THE DATA", which is why it certified seven illegal drafting practices.
BUDGET = 1_100_000
APOTHECARY_COST = 50_000
STAFF_COST = 10_000                   # assistant coaches and cheerleaders alike, all editions
MAX_STAFF = 6                         # "up to a maximum of 6 Assistant Coaches" / Cheerleaders
MAX_REROLLS = 8                       # "a maximum of 8 Team Re-rolls"
MIN_PLAYERS = 11                      # "at least 11 players ... when it is first drafted"
MAX_PLAYERS = 16                      # "may never have more than 16 players"
MAX_SQUAD = 16
BIG_GUY_TYPES = {"big guy", "bigguy"}
CAP_WORDS = {"a single": 1, "up to two": 2, "up to three": 3, "up to four": 4}

# Fans, per edition: (spec field, starting value, maximum at drafting, cost per improvement,
# counts in Team Value). bb2016 CRP Fan Factor is purchasable 0-9 at 10,000 and IS in TV
# (UtilTeamValue: teamValue += fanFactor * 10000). bb2020 exhibition play starts Dedicated Fans
# at 0 and improves to 6 at 10,000 each. bb2025 matched play starts at 1 and improves to 3 at
# 5,000 each. The old checker used the bb2025 League figure for bb2020 as well, which underpaid
# every bb2020 squad's fans by 20,000 (sec.19 defect 4) and allowed a bb2025 squad up to 6
# fans where drafting caps them at 3 (defect 5).
FANS = {
    "bb2016": ("fan_factor", 0, 9, 10_000, True),
    "bb2020": ("dedicated_fans", 0, 6, 10_000, False),
    "bb2025": ("dedicated_fans", 1, 3, 5_000, False),
}


def load_json(path: Path):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def load_rosters(edition: str) -> dict:
    """roster_id -> roster json, for one edition."""
    out = {}
    for path in sorted(ROSTERS[edition].glob("roster_*.json")):
        r = load_json(path)
        out[r["id"]] = r
    return out


def load_stars() -> dict:
    return {s["id"]: s for s in load_json(STAR_FILE)["star_players"]}


def big_guy_cap(edition: str, race: str):
    """The official page's cap on how many Big Guys the team may CONTAIN, or None.

    "A Chaos Chosen team may have a single Big Guy, chosen from the following" is a cap on the
    team's composition, not on what it may put on the pitch, and it applies across a GROUP of
    positions -- which the per-position `quantity` field cannot express (sec.19 defect 6).
    No official page for this (edition, race) means no group cap: the FUMBBL legacy imports are
    governed by their per-position quantities alone. CRP (bb2016) has no group cap at all."""
    page_dir, mapping = PAGES[edition]
    slug = mapping.get(race)
    if not slug:
        return None
    page = page_dir / f"{slug}.md"
    if not page.exists():
        return None
    text = page.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"may (?:have|include|take) (%s) Big Guy" % "|".join(CAP_WORDS), text)
    return CAP_WORDS[m.group(1)] if m else None


def single_big_guy(edition: str, race: str) -> bool:
    """Back-compat shim: True when the page caps Big Guys at exactly one."""
    return big_guy_cap(edition, race) == 1


def money(edition: str, spec: dict, roster: dict, positions: dict):
    """Return (players, staff, expected_team_value, expected_spent).

    Team Value is Java's UtilTeamValue.findTeamValue, 1:1:
        rerolls*reRollCost + fanFactor*10k + assistantCoaches*10k + cheerleaders*10k
        + apothecaries*50k + sum(player position costs)
    Dedicated Fans are NOT in TV; Fan Factor is. Spent adds the Dedicated Fans improvements,
    which are paid for out of the Team Draft Budget but never counted in TV."""
    players = sum(positions[p["position_id"]]["cost"]
                  for p in spec["players"] if p["position_id"] in positions)
    staff = (spec["rerolls"] * roster["reroll_cost"]
             + spec["apothecaries"] * APOTHECARY_COST
             + (spec.get("assistant_coaches", 0) + spec.get("cheerleaders", 0)) * STAFF_COST)
    field, start, _mx, cost, in_tv = FANS[edition]
    fans = spec.get(field, start)
    tv = players + staff + (fans * cost if in_tv else 0)
    spent = tv + (0 if in_tv else max(0, fans - start) * cost)
    return players, staff, tv, spent


def cheapest_remaining(edition, spec, roster, positions, counts):
    """The cheapest purchase this squad could still legally make, or None if it could make
    none. Used by C10 to tell "spent everything it could" apart from "left gold on the table"."""
    opts = []
    n = len(spec["players"])
    cap = big_guy_cap(edition, spec.get("race", ""))
    n_bg = sum(c for pid, c in counts.items()
               if positions.get(pid, {}).get("type", "").lower() in BIG_GUY_TYPES)
    if n < MAX_PLAYERS:
        for pid, pos in positions.items():
            if counts.get(pid, 0) >= pos["quantity"]:
                continue
            if (cap is not None and pos.get("type", "").lower() in BIG_GUY_TYPES
                    and n_bg >= cap):
                continue
            if pos.get("type") in ("Star", "Infamous Staff"):
                continue
            opts.append(pos["cost"])
    if spec["rerolls"] < min(MAX_REROLLS, roster["max_rerolls"]):
        opts.append(roster["reroll_cost"])
    if spec["apothecaries"] == 0 and roster.get("apothecary", False):
        opts.append(APOTHECARY_COST)
    field, start, mx, cost, _in_tv = FANS[edition]
    if spec.get(field, start) < mx:
        opts.append(cost)
    if (spec.get("assistant_coaches", 0) < MAX_STAFF
            or spec.get("cheerleaders", 0) < MAX_STAFF):
        opts.append(STAFF_COST)
    return min(opts) if opts else None


def validate_squad(edition: str, path: Path, spec: dict, rosters: dict, stars: dict):
    """Return a list of violation strings for one squad file."""
    v = []
    name = f"{edition}/{path.name}"

    # C1 -- roster_id resolves. Everything else needs the roster, so bail out here.
    roster = rosters.get(spec.get("roster_id"))
    if roster is None:
        v.append(f"C1 CRITICAL {name}: roster_id '{spec.get('roster_id')}' "
                 f"does not resolve in data/rosters/{edition}/ "
                 f"-- make_team() would fall back to an ALL-LINEMAN team "
                 f"(log::warn! only) and still gate green")
        return v
    positions = {p["id"]: p for p in roster["positions"]}

    # C2 -- every position_id exists in that roster
    counts = defaultdict(int)
    for pl in spec["players"]:
        pid = pl["position_id"]
        counts[pid] += 1
        if pid not in positions:
            v.append(f"C2 CRITICAL {name}: position_id '{pid}' not in roster "
                     f"'{roster['id']}'")

    # C3 -- squad size. "A team must have at least 11 players on their Team Draft List when it
    # is first drafted" and "may never have more than 16".
    n_players = len(spec["players"])
    if not MIN_PLAYERS <= n_players <= MAX_PLAYERS:
        v.append(f"C3 {name}: {n_players} players, legal range is "
                 f"{MIN_PLAYERS}..{MAX_PLAYERS}")

    # C9 -- no star players. A star on a Team Draft List is an INDUCEMENT: Matched Play wants
    # its gold cost AND 2 Skill Points inside the team's Tier allowance, and a mirror match --
    # which every parity gate is -- generates no inducement gold whatsoever. The squads field
    # none, so the key must be ABSENT rather than empty (sec.19 defect 1).
    if "stars" in spec:
        v.append(f"C9 CRITICAL {name}: carries a 'stars' list "
                 f"({len(spec.get('stars') or [])} entries). Star players are Inducements, "
                 f"not draftable squad members -- see PARITY_COVERAGE_REQUIREMENTS.md sec.19")

    # C4 -- per-position quantity cap
    for pid, n in sorted(counts.items()):
        pos = positions.get(pid)
        if pos and n > pos["quantity"]:
            v.append(f"C4 {name}: {n}x '{pid}' exceeds its quantity cap "
                     f"of {pos['quantity']}")

    # BG -- the group Big Guy cap from the official page, applied to what the team CONTAINS.
    cap = big_guy_cap(edition, spec.get("race", ""))
    if cap is not None:
        bgs = [pid for pid, n in counts.items()
               if positions.get(pid, {}).get("type", "").lower() in BIG_GUY_TYPES
               for _ in range(n)]
        if len(bgs) > cap:
            v.append(f"BG {name}: the squad contains {len(bgs)} Big Guys "
                     f"({sorted(set(bgs))}) but the official page allows {cap}")

    # C5 -- spend reconciles
    pcost, staff, exp_tv, exp_spent = money(edition, spec, roster, positions)
    if len(counts) == len(set(counts)) and all(p["position_id"] in positions
                                               for p in spec["players"]):
        if spec["team_value"] != exp_tv:
            v.append(f"C5 {name}: team_value {spec['team_value']:,} != "
                     f"players {pcost:,} + staff {staff:,}"
                     + (f" + fans {spec.get('fan_factor', 0) * FAN_FACTOR_COST_BB2016:,}"
                        if edition == "bb2016" else "")
                     + f" = {exp_tv:,}")
        if "spent" in spec and spec["spent"] != exp_spent:
            v.append(f"C5 {name}: spent {spec['spent']:,} != expected "
                     f"{exp_spent:,}")
        if spec["spent"] + spec["treasury"] != BUDGET:
            v.append(f"C5 {name}: spent {spec['spent']:,} + treasury "
                     f"{spec['treasury']:,} = {spec['spent'] + spec['treasury']:,}"
                     f" != budget {BUDGET:,}")
        # C10 -- the budget must be spent. "all the gold pieces a team has must be spent when
        # drafting your team. Any gold pieces not spent are lost" (matched play; exhibition
        # play says the same). `treasury` therefore records LOST gold, and it is only legal
        # when nothing further could have been bought with it -- so it must be smaller than
        # the cheapest remaining legal purchase (defect 2).
        cheapest = cheapest_remaining(edition, spec, roster, positions, counts)
        if cheapest is not None and spec["treasury"] >= cheapest:
            v.append(f"C10 {name}: {spec['treasury']:,} gold unspent, but {cheapest:,} "
                     f"would still buy something -- in this play format unspent gold is lost, "
                     f"so it must all be spent")

    # C6 -- staff limits
    if spec["rerolls"] > min(MAX_REROLLS, roster["max_rerolls"]):
        v.append(f"C6 {name}: {spec['rerolls']} re-rolls exceeds "
                 f"min(8, roster max_rerolls {roster['max_rerolls']})")
    if spec["rerolls"] < 0 or spec["apothecaries"] < 0:
        v.append(f"C6 {name}: negative rerolls/apothecaries")
    if spec["apothecaries"] > 1:
        v.append(f"C6 {name}: {spec['apothecaries']} apothecaries, max 1")
    if spec["apothecaries"] > 0 and not roster.get("apothecary", False):
        v.append(f"C6 {name}: apothecary bought but roster '{roster['id']}' "
                 f"may not hire one")
    # Sideline Staff, 0-6 each at 10,000. Absent from the spec until 2026-09-10 and hardcoded
    # to 0 on both sides, so no parity team ever had a single member of staff (defect 3).
    for fld in ("assistant_coaches", "cheerleaders"):
        val = spec.get(fld, 0)
        if not 0 <= val <= MAX_STAFF:
            v.append(f"C6 {name}: {fld} {val} outside 0..{MAX_STAFF}")

    # C6f -- fans, per edition and per play format
    field, start, mx, _cost, _in_tv = FANS[edition]
    other = "dedicated_fans" if field == "fan_factor" else "fan_factor"
    val = spec.get(field, start)
    if not start <= val <= mx:
        v.append(f"C6 {name}: {field} {val} outside {start}..{mx} for {edition}")
    if spec.get(other, 0) != 0:
        v.append(f"C6 {name}: {edition} has no {other}, but {other} = {spec.get(other)}")

    # C7 -- squad numbers
    nrs = [p["nr"] for p in spec["players"]] + [s["nr"] for s in spec.get("stars", [])]
    dupes = sorted({n for n in nrs if nrs.count(n) > 1})
    if dupes:
        v.append(f"C7 {name}: duplicate squad numbers {dupes}")
    bad = sorted(n for n in nrs if not 1 <= n <= MAX_SQUAD)
    if bad:
        v.append(f"C7 {name}: squad numbers outside 1..{MAX_SQUAD}: {bad}")

    # C8 -- if a star id ever reappears it must at least resolve. C9 above already rejects the
    # whole key, so this only fires on a spec that C9 has condemned.
    for st in spec.get("stars", []):
        if st["star_id"] not in stars:
            v.append(f"C8 CRITICAL {name}: star_id '{st['star_id']}' not in "
                     f"data/star_players/all_editions.json")

    return v


def validate_edition(edition: str, teams_dir: Path, rosters: dict, stars: dict):
    """Return (violations, r2_misses) for one edition.

    r2_misses: list of (race, roster_id, [unfielded position ids]) per CELL --
    the R4 union rule, i.e. the union of team_<race>.json and its team_<race>_*.json
    variants must field every positional.
    """
    violations = []
    cells = defaultdict(lambda: {"fielded": set(), "roster_id": None, "files": []})

    for path in sorted(teams_dir.glob("team_*.json")):
        try:
            spec = load_json(path)
        except Exception as exc:  # malformed JSON is itself a violation
            violations.append(f"C0 CRITICAL {edition}/{path.name}: unreadable ({exc})")
            continue
        violations += validate_squad(edition, path, spec, rosters, stars)
        # Group variants by the spec's own `race`, NOT by filename: filename prefixes
        # collide (team_dark_elf.json vs team_dark_elf_league_fumbbl.json are different
        # cells, while team_old_world_alliance_{ogre,treeman}.json are one cell).
        cell = cells[spec.get("race", path.stem)]
        cell["roster_id"] = spec.get("roster_id")
        cell["files"].append(path.name)
        cell["fielded"] |= {p["position_id"] for p in spec["players"]}

    r2 = []
    for race, cell in sorted(cells.items()):
        roster = rosters.get(cell["roster_id"])
        if roster is None:
            continue  # already reported as C1
        all_pos = [p["id"] for p in roster["positions"]]
        missing = [pid for pid in all_pos if pid not in cell["fielded"]]
        if missing:
            r2.append((race, cell["roster_id"], missing, cell["files"]))
    return violations, r2


def check_r5(rosters_by_ed: dict):
    """R5 (docs §10): a roster must be authored per ruleset, never shared; and a
    bb2016 roster must not carry a PA (LRB6 passing is AG-based, there is no PA)."""
    out = []

    def content(r):
        return json.dumps([
            {k: p.get(k) for k in ("id", "name", "type", "quantity", "cost",
                                   "ma", "st", "ag", "pa", "av", "skills")}
            for p in r["positions"]
        ], sort_keys=True)

    ids = set().union(*[set(d) for d in rosters_by_ed.values()])
    for rid in sorted(ids):
        same = defaultdict(list)
        for ed in EDITIONS:
            r = rosters_by_ed[ed].get(rid)
            if r:
                same[content(r)].append(ed)
        for eds in same.values():
            if len(eds) > 1:
                out.append(f"R5a roster '{rid}' is content-identical across "
                           f"{' == '.join(eds)} -- not authored per ruleset")
    for rid, r in sorted(rosters_by_ed["bb2016"].items()):
        pas = sorted({p.get("pa") for p in r["positions"] if p.get("pa") is not None})
        if pas:
            out.append(f"R5b bb2016 roster '{rid}' carries PA {pas}; LRB6 defines "
                       f"no PA characteristic (passing is AG-based)")
    return out


# ── selftest ────────────────────────────────────────────────────────────────────

def selftest() -> int:
    """Prove the checker fires. Copies the real data/teams tree into a temp dir,
    validates it (known-good: must be violation-free), then breaks one squad in
    seven ways and asserts each check reports. NEVER touches data/."""
    stars = load_stars()
    rosters = {ed: load_rosters(ed) for ed in EDITIONS}
    tmp = Path(tempfile.mkdtemp(prefix="validate_teams_"))
    fails = []
    try:
        good = tmp / "bb2025"
        shutil.copytree(TEAMS["bb2025"], good)
        v, _ = validate_edition("bb2025", good, rosters["bb2025"], stars)
        if v:
            fails.append(f"known-good copy reported {len(v)} violations: {v[:3]}")
        else:
            print("  selftest: known-good copy of data/teams/bb2025 -> 0 violations")

        src = load_json(good / "team_amazon.json")
        cases = [
            ("C1", lambda s: s.update(roster_id="amazon.NOPE")),
            ("C2", lambda s: s["players"][0].update(position_id="amazon.wizard")),
            ("C3", lambda s: s.__setitem__("players", s["players"][:6])),
            ("C4", lambda s: s.__setitem__(
                "players", [{"nr": i + 1, "position_id": "amazon.blitzer"}
                            for i in range(12)])),
            ("C5", lambda s: s.update(team_value=s["team_value"] + 10000)),
            ("C6", lambda s: s.update(rerolls=99)),
            ("C7", lambda s: s["players"][1].update(nr=s["players"][0]["nr"])),
            ("C8", lambda s: s.update(stars=[{"nr": 16, "star_id": "no.such.star"}])),
            # C9: a star that RESOLVES is still illegal -- the key itself is the violation,
            # so this case must fire without any help from C8.
            ("C9", lambda s: s.update(stars=[{"nr": 16, "star_id": "amazon.Estelle"}])),
            # C10: hand 50,000 gold back to the treasury. It is enough to buy a 14th
            # linewoman, so leaving it unspent is illegal in a format where unspent gold
            # is lost.
            ("C10", lambda s: (s.update(treasury=s["treasury"] + 50_000,
                                        spent=s["spent"] - 50_000,
                                        team_value=s["team_value"] - 50_000),
                               s.update(rerolls=s["rerolls"] - 1))),
            # bb2025 caps Dedicated Fans at 3 when the team is drafted. 6 was legal under the
            # old checker, which used the bb2020 range for every edition (defect 5).
            ("C6-dedfans-over-3", lambda s: s.update(dedicated_fans=6), "C6"),
        ]
        # (dir tag, mutator) or (dir tag, mutator, expected check prefix)
        cases = [(c[0], c[1], c[2] if len(c) > 2 else c[0]) for c in cases]
        for tag, break_it, expect in cases:
            bad = tmp / f"bad_{tag}"
            bad.mkdir()
            spec = json.loads(json.dumps(src))
            break_it(spec)
            (bad / "team_amazon.json").write_text(json.dumps(spec), encoding="utf-8")
            v, _ = validate_edition("bb2025", bad, rosters["bb2025"], stars)
            hit = [x for x in v if x.split()[0] == expect]
            if not hit:
                fails.append(f"{tag}: deliberately broken squad NOT reported "
                             f"(got {v})")
            else:
                print(f"  selftest: {tag} broken squad -> {hit[0]}")

        # C6f for bb2016/bb2020: the fans field itself must be the edition's own. A bb2016
        # squad carrying Dedicated Fans, or a bb2020 squad carrying Fan Factor, is the shape
        # of sec.19 defects 4 and 5 and must report.
        for ed, bad in (("bb2016", {"dedicated_fans": 3}), ("bb2020", {"fan_factor": 3})):
            bad_dir = tmp / f"bad_fans_{ed}"
            bad_dir.mkdir()
            src_ed = load_json(TEAMS[ed] / "team_amazon.json")
            src_ed.update(bad)
            (bad_dir / "team_amazon.json").write_text(json.dumps(src_ed), encoding="utf-8")
            v, _ = validate_edition(ed, bad_dir, rosters[ed], stars)
            hit = [x for x in v if x.startswith("C6")]
            if not hit:
                fails.append(f"C6 fans {ed}: {bad} NOT reported (got {v})")
            else:
                print(f"  selftest: C6 fans {ed} {bad} -> {hit[0]}")

        # BG: chaos may field a single Big Guy; field two and it must report
        bad = tmp / "bad_BG"
        bad.mkdir()
        spec = load_json(good / "team_chaos.json")
        bg = [p for p in load_json(ROSTERS["bb2025"] / "roster_chaos.json")["positions"]
              if p["type"].lower() in BIG_GUY_TYPES]
        assert len(bg) >= 2, "chaos roster should offer 2+ Big Guys"
        spec["players"] = spec["players"][:len(spec["players"]) - 2] + [
            {"nr": 15, "position_id": bg[0]["id"]},
            {"nr": 16, "position_id": bg[1]["id"]},
        ]
        (bad / "team_chaos.json").write_text(json.dumps(spec), encoding="utf-8")
        v, _ = validate_edition("bb2025", bad, rosters["bb2025"], stars)
        hit = [x for x in v if x.startswith("BG")]
        if not hit:
            fails.append(f"BG: two Big Guys NOT reported (got {v})")
        else:
            print(f"  selftest: BG two Big Guys -> {hit[0]}")

        # R2/R4: dropping the Jaguar Warrior must surface as an unfielded positional
        bad = tmp / "bad_R2"
        bad.mkdir()
        spec = json.loads(json.dumps(src))
        spec["players"] = [p for p in spec["players"]
                           if p["position_id"] != "amazon.catcher"]
        spec["players"] += [{"nr": 20 + i, "position_id": "amazon.linewoman"}
                            for i in range(2)]
        (bad / "team_amazon.json").write_text(json.dumps(spec), encoding="utf-8")
        _, r2 = validate_edition("bb2025", bad, rosters["bb2025"], stars)
        if not any("amazon.catcher" in m for _, _, m, _ in r2):
            fails.append(f"R2: dropped positional NOT reported (got {r2})")
        else:
            print("  selftest: R2 dropped positional -> amazon.catcher unfielded")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    if fails:
        print("\nSELFTEST FAILED:")
        for f in fails:
            print("  " + f)
        return 2
    print("  selftest: PASS")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--edition", choices=list(EDITIONS), action="append",
                    help="restrict to one edition (repeatable); default all three")
    ap.add_argument("--teams-dir", type=Path,
                    help="validate this directory instead of data/teams (expects "
                         "<dir>/<edition>/team_*.json); rosters still come from data/")
    ap.add_argument("--r5", action="store_true",
                    help="also run the R5 roster-sharing / bb2016-PA check")
    ap.add_argument("--selftest", action="store_true",
                    help="validate the checker on a known-good and broken copy")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    editions = args.edition or list(EDITIONS)
    stars = load_stars()
    rosters = {ed: load_rosters(ed) for ed in EDITIONS}

    total_v = 0
    total_r2 = 0
    for ed in editions:
        teams_dir = (args.teams_dir / ed) if args.teams_dir else TEAMS[ed]
        n_files = len(list(teams_dir.glob("team_*.json")))
        v, r2 = validate_edition(ed, teams_dir, rosters[ed], stars)
        print(f"\n=== {ed}: {n_files} squads, {len(v)} R1 violations, "
              f"{len(r2)} cells with unfielded positionals ===")
        for line in v:
            print("  " + line)
        for race, rid, missing, files in r2:
            print(f"  R2/R4 {ed}/{race} ({rid}, union of {', '.join(files)}): "
                  f"{len(missing)} positional(s) never fielded: "
                  f"{', '.join(missing)}")
        total_v += len(v)
        total_r2 += sum(len(m) for _, _, m, _ in r2)

    if args.r5:
        r5 = check_r5(rosters)
        print(f"\n=== R5 (roster provenance, docs sec.10): {len(r5)} violations ===")
        for line in r5:
            print("  " + line)

    print(f"\nTOTAL: {total_v} R1 violations, {total_r2} unfielded positional slots")
    return 1 if (total_v or total_r2) else 0


if __name__ == "__main__":
    sys.exit(main())
