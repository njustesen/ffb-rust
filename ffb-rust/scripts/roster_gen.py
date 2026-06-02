#!/usr/bin/env python3
"""
roster_gen.py — generate deterministic random team_spec.json files.

Usage:
    python scripts/roster_gen.py [options]

Options:
    --seed N              Deterministic seed for roster generation (required)
    --edition EDITION     bb2016|bb2020|bb2025|random (default: random)
    --home-race RACE      Race for home team, or "random" (default: random)
    --away-race RACE      Race for away team, or "random" (default: random)
    --tv N                Target team value for each side (default: 1000000)
    --imbalance N         Extra TV for away team (default: 0; use for T3i)
    --out-dir DIR         Output directory (default: parity/specs)
    --list-races          Print available races for the given edition and exit

The generated files follow the team_spec.json format documented in PARITY_PLAN.md.
Both engines (Rust + Java) can load these files to build identical teams.

Examples:
    python scripts/roster_gen.py --seed 42 --edition bb2020 --home-race human --away-race orc
    python scripts/roster_gen.py --seed 42 --edition random --home-race random --away-race random
    python scripts/roster_gen.py --seed 42 --tier T3i --imbalance 200000
"""

import argparse
import json
import random
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
DATA_DIR = REPO_ROOT / "data" / "rosters"

EDITIONS = ["bb2016", "bb2020", "bb2025"]

RACE_NAMES = [
    "amazon", "chaos", "chaos_dwarf", "chaos_pact", "dark_elf", "dwarf",
    "elf", "goblin", "halfling", "high_elf", "human", "khemri", "lizardman",
    "necromantic", "norse", "nurgle", "ogre", "orc", "renegades", "skaven",
    "slann", "undead", "underworld", "vampire", "wood_elf",
]

SKILL_CATEGORIES = ["General", "Agility", "Passing", "Strength", "Mutation", "Extraordinary", "Trait"]


def load_roster(race: str, edition: str) -> dict | None:
    """Load roster JSON for a race+edition. Returns None if not found."""
    path = DATA_DIR / edition / f"roster_{race}.json"
    if path.exists():
        with open(path) as f:
            return json.load(f)
    # try normalized name
    norm = race.lower().replace(" ", "_").replace("-", "_")
    path2 = DATA_DIR / edition / f"roster_{norm}.json"
    if path2.exists():
        with open(path2) as f:
            return json.load(f)
    return None


def list_available_races(edition: str) -> list[str]:
    """List all roster race names for an edition."""
    roster_dir = DATA_DIR / edition
    if not roster_dir.exists():
        return []
    return sorted(
        p.stem.replace("roster_", "")
        for p in roster_dir.glob("roster_*.json")
    )


def build_canonical_team(roster: dict, side: str, seed_rng: random.Random,
                          max_players: int = 11,
                          extra_skills_per_player: int = 0) -> dict:
    """
    Build a canonical team spec from a roster JSON.

    Fills premium (low-quantity / high-cost) positions first, then fills
    remaining slots with the most plentiful position (typically linemen).

    max_players: stop after this many players (default 11).
    extra_skills_per_player: how many random legal skills to add per player (for T3).
    """
    positions = [p for p in roster["positions"]
                 if p.get("type", "Regular") not in ("Star", "Infamous Staff")]

    # Sort: low-quantity (premium) positions first, high-quantity (linemen) last
    # Within same quantity, higher cost comes first
    positions = sorted(positions, key=lambda p: (p["quantity"], -p["cost"]))

    players = []
    nr = 1
    for pos in positions:
        qty = pos["quantity"]
        take = min(qty, max_players - len(players))
        for _ in range(take):
            if len(players) >= max_players:
                break
            entry = {
                "nr": nr,
                "name": f"{side.capitalize()} {pos['name']} {nr}",
                "position": pos["id"],
            }
            if extra_skills_per_player > 0:
                # Pick random skills from this position's legal categories
                normal_cats = pos.get("skill_categories", {}).get("normal", [])
                double_cats = pos.get("skill_categories", {}).get("double", [])
                all_cats = normal_cats + double_cats
                if all_cats:
                    num_skills = seed_rng.randint(0, extra_skills_per_player)
                    # Simple: pick skill names from the category lists
                    # (real implementation would use the skills data files)
                    extra = []
                    for _ in range(num_skills):
                        cat = seed_rng.choice(all_cats)
                        # Placeholder skill names by category
                        cat_skills = {
                            "General": ["Block", "Dodge", "Sure Hands", "Tackle", "Strip Ball"],
                            "Agility": ["Catch", "Leap", "Sprint", "Sure Feet"],
                            "Passing": ["Pass", "Safe Throw", "Nerves of Steel"],
                            "Strength": ["Guard", "Mighty Blow", "Stand Firm", "Pile On"],
                            "Mutation": ["Claws", "Two Heads", "Extra Arms"],
                            "Extraordinary": ["Bone Head", "Really Stupid"],
                            "Trait": ["Loner"],
                        }.get(cat, [])
                        if cat_skills:
                            extra.append(seed_rng.choice(cat_skills))
                    if extra:
                        entry["extra_skills"] = list(set(extra))
            players.append(entry)
            nr += 1
        if len(players) >= max_players:
            break

    return {"players": players}


def compute_tv(roster: dict, players: list[dict]) -> int:
    """Compute team value from the player list and roster position costs."""
    pos_costs = {p["id"]: p["cost"] for p in roster["positions"]}
    tv = 0
    for p in players:
        pos_id = p.get("position", "")
        tv += pos_costs.get(pos_id, 50000)
    return tv


def generate_team_spec(race: str, edition: str, side: str, seed_rng: random.Random,
                       target_tv: int = 1_000_000,
                       extra_skills: int = 0) -> dict:
    """Generate a complete team_spec.json dict."""
    roster = load_roster(race, edition)
    if roster is None:
        raise ValueError(f"Roster not found: {race} ({edition})")

    team_data = build_canonical_team(roster, side, seed_rng, max_players=11,
                                      extra_skills_per_player=extra_skills)
    tv = compute_tv(roster, team_data["players"])

    return {
        "id": f"{side}_{race}_{edition}_gen",
        "name": f"{side.capitalize()} {roster['name']}",
        "race": race,
        "edition": edition,
        "rerolls": 3,
        "apothecary": roster.get("apothecary", False),
        "team_value": tv,
        "players": team_data["players"],
    }


def parse_args():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--seed", type=int, required=False, default=42,
                    help="Deterministic seed for random choices")
    ap.add_argument("--edition", default="random",
                    help="bb2016|bb2020|bb2025|random (default: random)")
    ap.add_argument("--home-race", default="random",
                    help="Race for home team, or 'random'")
    ap.add_argument("--away-race", default="random",
                    help="Race for away team, or 'random'")
    ap.add_argument("--tv", type=int, default=1_000_000,
                    help="Target team value (default: 1000000)")
    ap.add_argument("--imbalance", type=int, default=0,
                    help="Extra TV for away team; >0 triggers inducements")
    ap.add_argument("--extra-skills", type=int, default=0,
                    help="Random extra skills per player (0=none, for T3)")
    ap.add_argument("--out-dir", default=None,
                    help="Output directory (default: parity/specs)")
    ap.add_argument("--list-races", action="store_true",
                    help="Print available races and exit")
    return ap.parse_args()


def main():
    args = parse_args()

    rng = random.Random(args.seed)

    edition = args.edition
    if edition == "random":
        edition = rng.choice(EDITIONS)

    if args.list_races:
        races = list_available_races(edition)
        print(f"Available races for {edition}:")
        for r in races:
            print(f"  {r}")
        return 0

    available = list_available_races(edition)
    if not available:
        print(f"ERROR: No rosters found for edition '{edition}' in {DATA_DIR}", file=sys.stderr)
        return 1

    home_race = args.home_race
    away_race = args.away_race
    if home_race == "random":
        home_race = rng.choice(available)
    if away_race == "random":
        away_race = rng.choice(available)

    print(f"Generating: {home_race} vs {away_race} ({edition}) seed={args.seed}")

    try:
        home_spec = generate_team_spec(home_race, edition, "home", rng,
                                        target_tv=args.tv,
                                        extra_skills=args.extra_skills)
        away_spec = generate_team_spec(away_race, edition, "away", rng,
                                        target_tv=args.tv + args.imbalance,
                                        extra_skills=args.extra_skills)
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1

    out_dir = Path(args.out_dir) if args.out_dir else REPO_ROOT / "parity" / "specs"
    out_dir.mkdir(parents=True, exist_ok=True)

    home_path = out_dir / f"home_seed{args.seed}.json"
    away_path = out_dir / f"away_seed{args.seed}.json"

    with open(home_path, "w") as f:
        json.dump(home_spec, f, indent=2)
    with open(away_path, "w") as f:
        json.dump(away_spec, f, indent=2)

    print(f"Written: {home_path}")
    print(f"Written: {away_path}")
    print(f"  Home TV: {home_spec['team_value']:,}  ({len(home_spec['players'])} players)")
    print(f"  Away TV: {away_spec['team_value']:,}  ({len(away_spec['players'])} players)")
    if args.imbalance:
        diff = away_spec["team_value"] - home_spec["team_value"]
        print(f"  TV imbalance: {diff:,} → home gets petty cash / inducements")
    return 0


if __name__ == "__main__":
    sys.exit(main())
