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

# ── money / legality constants, all derived from the data (see module docstring) ──
BUDGET = 1_100_000
APOTHECARY_COST = 50_000
FAN_FACTOR_COST_BB2016 = 10_000       # LRB6 Fan Factor, counts in TV
DEDICATED_FAN_COST = 5_000            # bb2020+, first fan free, NOT in TV
MIN_PLAYERS = 11
MAX_PLAYERS = 16
MAX_SQUAD = 16                        # rostered players + fielded stars
DEDICATED_FANS_RANGE = (1, 6)         # bb2020+
FAN_FACTOR_RANGE = (0, 9)             # bb2016
BIG_GUY_TYPES = {"big guy", "bigguy"}


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


def single_big_guy(edition: str, race: str) -> bool:
    """True if the official page for this (edition, race) caps Big Guys at one."""
    page_dir, mapping = PAGES[edition]
    slug = mapping.get(race)
    if not slug:
        return False
    page = page_dir / f"{slug}.md"
    if not page.exists():
        return False
    return "single Big Guy" in page.read_text(encoding="utf-8", errors="replace")


def money(edition: str, spec: dict, roster: dict, positions: dict):
    """Return (players, staff, expected_team_value, expected_spent)."""
    players = sum(positions[p["position_id"]]["cost"]
                  for p in spec["players"] if p["position_id"] in positions)
    staff = spec["rerolls"] * roster["reroll_cost"] \
        + spec["apothecaries"] * APOTHECARY_COST
    if edition == "bb2016":
        tv = players + staff + spec.get("fan_factor", 0) * FAN_FACTOR_COST_BB2016
        spent = tv
    else:
        tv = players + staff
        spent = tv + max(0, spec.get("dedicated_fans", 1) - 1) * DEDICATED_FAN_COST
    return players, staff, tv, spent


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

    # C3 -- squad size
    n_players = len(spec["players"])
    n_stars = len(spec.get("stars", []))
    if not MIN_PLAYERS <= n_players <= MAX_PLAYERS:
        v.append(f"C3 {name}: {n_players} rostered players, legal range is "
                 f"{MIN_PLAYERS}..{MAX_PLAYERS}")
    if n_players + n_stars > MAX_SQUAD:
        v.append(f"C3 {name}: {n_players} players + {n_stars} stars = "
                 f"{n_players + n_stars} exceeds the {MAX_SQUAD}-player squad")

    # C4 -- per-position quantity cap
    for pid, n in sorted(counts.items()):
        pos = positions.get(pid)
        if pos and n > pos["quantity"]:
            v.append(f"C4 {name}: {n}x '{pid}' exceeds its quantity cap "
                     f"of {pos['quantity']}")

    # BG -- single Big Guy restriction from the official page
    if single_big_guy(edition, spec.get("race", "")):
        bgs = [pid for pid, n in counts.items()
               if positions.get(pid, {}).get("type", "").lower() in BIG_GUY_TYPES
               for _ in range(n)]
        if len(bgs) > 1:
            v.append(f"BG {name}: {len(bgs)} Big Guys ({sorted(set(bgs))}) but the "
                     f"official page allows a single Big Guy")

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

    # C6 -- staff limits
    if spec["rerolls"] > roster["max_rerolls"]:
        v.append(f"C6 {name}: {spec['rerolls']} re-rolls exceeds the roster's "
                 f"max_rerolls {roster['max_rerolls']}")
    if spec["rerolls"] < 0 or spec["apothecaries"] < 0:
        v.append(f"C6 {name}: negative rerolls/apothecaries")
    if spec["apothecaries"] > 1:
        v.append(f"C6 {name}: {spec['apothecaries']} apothecaries, max 1")
    if spec["apothecaries"] > 0 and not roster.get("apothecary", False):
        v.append(f"C6 {name}: apothecary bought but roster '{roster['id']}' "
                 f"may not hire one")
    if edition == "bb2016":
        lo, hi = FAN_FACTOR_RANGE
        if not lo <= spec.get("fan_factor", 0) <= hi:
            v.append(f"C6 {name}: fan_factor {spec.get('fan_factor')} outside "
                     f"{lo}..{hi}")
        if spec.get("dedicated_fans", 0) != 0:
            v.append(f"C6 {name}: bb2016 has no Dedicated Fans, but "
                     f"dedicated_fans = {spec.get('dedicated_fans')}")
    else:
        lo, hi = DEDICATED_FANS_RANGE
        if not lo <= spec.get("dedicated_fans", 0) <= hi:
            v.append(f"C6 {name}: dedicated_fans {spec.get('dedicated_fans')} "
                     f"outside {lo}..{hi}")
        if spec.get("fan_factor", 0) != 0:
            v.append(f"C6 {name}: bb2020+ has no Fan Factor, but "
                     f"fan_factor = {spec.get('fan_factor')}")

    # C7 -- squad numbers
    nrs = [p["nr"] for p in spec["players"]] + [s["nr"] for s in spec.get("stars", [])]
    dupes = sorted({n for n in nrs if nrs.count(n) > 1})
    if dupes:
        v.append(f"C7 {name}: duplicate squad numbers {dupes}")
    bad = sorted(n for n in nrs if not 1 <= n <= MAX_SQUAD)
    if bad:
        v.append(f"C7 {name}: squad numbers outside 1..{MAX_SQUAD}: {bad}")

    # C8 -- star ids resolve
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
        ]
        for tag, break_it in cases:
            bad = tmp / f"bad_{tag}"
            bad.mkdir()
            spec = json.loads(json.dumps(src))
            break_it(spec)
            (bad / "team_amazon.json").write_text(json.dumps(spec), encoding="utf-8")
            v, _ = validate_edition("bb2025", bad, rosters["bb2025"], stars)
            hit = [x for x in v if x.startswith(tag)]
            if not hit:
                fails.append(f"{tag}: deliberately broken squad NOT reported "
                             f"(got {v})")
            else:
                print(f"  selftest: {tag} broken squad -> {hit[0]}")

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
