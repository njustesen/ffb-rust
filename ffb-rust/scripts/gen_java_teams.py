#!/usr/bin/env python3
"""
gen_java_teams.py — generate Java parity team XML files for all races.

Reads each Rust bb2025 roster JSON, computes the canonical 11-player composition
(sorted by position quantity asc then cost desc), and writes matching home/away XML
files into the ffb-server/teams/ directory.

Usage:
    python scripts/gen_java_teams.py [--dry-run] [--out-dir DIR]

The canonical composition mirrors make_team_from_roster() in Rust runner.rs exactly.
"""

import argparse
import json
import os
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
ROSTERS_DIR = REPO_ROOT / "data" / "rosters" / "bb2025"

# Default Java server teams directory
DEFAULT_SERVER_TEAMS = Path(r"C:\Users\Admin\niels\ffb\ffb\ffb-server\teams")

# Skip legacy "replaced" positions (qty=0 or type that shouldn't be fielded)
SKIP_TYPES = {"Star", "Infamous Staff"}

# skill -> XML skill name mapping for skills with unusual Java names
SKILL_NAME_OVERRIDES = {
    "bone head": "bone head",
    "bone-head": "bone-head",
}

RACE_TO_LOGO = {
    "amazon": "amazon", "chaos": "chaos", "chaos_dwarf": "chaos_dwarf",
    "chaos_pact": "chaos_pact", "dark_elf": "dark_elf",
    "dark_elf_league_fumbbl": "dark_elf",
    "dwarf": "dwarf", "elf": "elf", "goblin": "goblin",
    "halfling": "halfling", "high_elf": "high_elf", "human": "human",
    "khemri": "khemri", "khemri_fumbbl": "khemri",
    "lizardman": "lizardman", "necromantic": "necromantic",
    "nippon": "nippon", "norse": "norse", "nurgle": "nurgle",
    "ogre": "ogre", "orc": "orc", "renegades": "renegades",
    "skaven": "skaven", "slann": "slann", "slann_fumbbl": "slann",
    "undead": "undead", "underworld": "underworld",
    "vampire": "vampire", "wood_elf": "wood_elf",
}


def load_roster(json_path: Path) -> dict:
    with open(json_path) as f:
        return json.load(f)


def canonical_composition(roster: dict, max_players: int = 11) -> list[dict]:
    """
    Compute 11-player composition matching Rust's make_team_from_roster():
    Sort non-star positions by (quantity ASC, -cost ASC) (stable), fill to max_players.
    """
    eligible = [
        p for p in roster["positions"]
        if p.get("type", "Regular") not in SKIP_TYPES and p["quantity"] > 0
    ]
    # Stable sort: primary key = quantity asc, secondary = cost desc (i.e. -cost asc)
    eligible_sorted = sorted(eligible, key=lambda p: (p["quantity"], -p["cost"]))

    players = []
    nr = 1
    for pos in eligible_sorted:
        take = min(pos["quantity"], max_players - len(players))
        for _ in range(take):
            players.append({"nr": nr, "position_id": pos["id"], "position_name": pos["name"]})
            nr += 1
        if len(players) >= max_players:
            break
    return players


def compute_tv(roster: dict, players: list[dict]) -> int:
    """Sum position costs for the given player list."""
    cost_map = {p["id"]: p["cost"] for p in roster["positions"]}
    return sum(cost_map.get(pl["position_id"], 50000) for pl in players)


def race_name_from_path(path: Path) -> str:
    """Extract race name from roster_*.json filename."""
    return path.stem.replace("roster_", "")


def team_id(race: str, side: str) -> str:
    """Convert snake_case race name to PascalCase Java team ID.
    Mirrors java_team_id() in runner.rs exactly.
    dark_elf_league_fumbbl → teamDarkElfLeagueFumbblParityHome
    """
    suffix = "Home" if side == "home" else "Away"
    pascal = "".join(w.capitalize() for w in race.split("_"))
    return f"team{pascal}Parity{suffix}"


def race_display(race: str) -> str:
    """Human-readable race name."""
    return race.replace("_fumbbl", " (FUMBBL)").replace("_", " ").title()


def logo(race: str) -> str:
    base = RACE_TO_LOGO.get(race, race.split("_")[0])
    return f"teamlogos/{base}.gif"


def generate_xml(roster: dict, race: str, side: str) -> str:
    players = canonical_composition(roster)
    tv = compute_tv(roster, players)
    tid = team_id(race, side)
    coach = side.capitalize()
    suffix = "Home" if side == "home" else "Away"

    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '',
        f'<team id="{tid}">',
        '',
        f'    <coach>{coach}</coach>',
        f'    <name>{race_display(race)} Parity {suffix}</name>',
        f'    <race>{race_display(race)}</race>',
        f'    <rosterId>{roster["id"]}</rosterId>',
        f'    <reRolls>3</reRolls>',
        f'    <fanFactor>5</fanFactor>',
        f'    <apothecaries>{"1" if roster.get("apothecary") else "0"}</apothecaries>',
        f'    <cheerleaders>0</cheerleaders>',
        f'    <assistantCoaches>0</assistantCoaches>',
        f'    <teamRating>100</teamRating>',
        f'    <currentTeamValue>{tv}</currentTeamValue>',
        f'    <teamStrength>{tv}</teamStrength>',
        f'    <division>[X]</division>',
        f'    <treasury>0</treasury>',
        f'    <baseIconPath>http://localhost:2224/icons/</baseIconPath>',
        f'    <logo>{logo(race)}</logo>',
        f'    <specialRules/>',
        '',
    ]

    for pl in players:
        pid = f"{tid}{pl['nr']}"
        pos_label = pl["position_name"]
        nr = pl["nr"]
        lines += [
            f'    <player nr="{nr}" id="{pid}">',
            f'        <name>{pos_label} {nr}</name>',
            f'        <gender>male</gender>',
            f'        <positionId>{pl["position_id"]}</positionId>',
            f'        <skillList/>',
            f'        <playerStatistics currentSpps="0"/>',
            f'    </player>',
            '',
        ]

    lines += ['</team>', '']
    return '\n'.join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dry-run", action="store_true", help="Print what would be generated without writing")
    ap.add_argument("--out-dir", default=str(DEFAULT_SERVER_TEAMS), help="Output directory")
    args = ap.parse_args()

    out_dir = Path(args.out_dir)
    if not args.dry_run:
        out_dir.mkdir(parents=True, exist_ok=True)

    if not ROSTERS_DIR.exists():
        print(f"ERROR: Rosters dir not found: {ROSTERS_DIR}", file=sys.stderr)
        return 1

    roster_files = sorted(ROSTERS_DIR.glob("roster_*.json"))
    print(f"Processing {len(roster_files)} rosters from {ROSTERS_DIR}")

    generated = 0
    skipped = 0
    for rpath in roster_files:
        race = race_name_from_path(rpath)
        roster = load_roster(rpath)
        players = canonical_composition(roster)

        if len(players) < 11:
            print(f"  SKIP {race}: only {len(players)} non-star positions (not enough for 11)", file=sys.stderr)
            skipped += 1
            continue

        tv = compute_tv(roster, players)
        positions_used = {}
        for pl in players:
            positions_used[pl["position_id"]] = positions_used.get(pl["position_id"], 0) + 1
        pos_summary = ", ".join(f"{cnt}×{pid.split('.')[-1]}" for pid, cnt in positions_used.items())

        print(f"  {race}: TV={tv:,} | {pos_summary}")

        for side in ("home", "away"):
            fname = f"team_{race}_parity_{side}.xml"
            fpath = out_dir / fname

            # Skip if it already exists (don't overwrite manually-crafted files)
            if fpath.exists() and not args.dry_run:
                print(f"    SKIP (exists): {fname}")
                continue

            xml = generate_xml(roster, race, side)
            if args.dry_run:
                print(f"    Would write: {fname}")
            else:
                fpath.write_text(xml, encoding="utf-8")
                print(f"    Wrote: {fname}")
                generated += 1

    print(f"\nDone: {generated} files written, {skipped} races skipped.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
