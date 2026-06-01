#!/usr/bin/env python3
"""
extract_data.py — Extracts all FFB game data from Java source and roster XMLs
to JSON files in ffb-rust/data/.

Usage: python extract_data.py
Run from C:/Users/Admin/niels/ffb/ or any directory.
"""

import json
import os
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

# Paths
SCRIPT_DIR = Path(__file__).parent
ROSTER_DIR = SCRIPT_DIR / "ffb-server" / "rosters"
OUTPUT_DIR = SCRIPT_DIR / ".." / "ffb-rust" / "data"
OUTPUT_DIR = OUTPUT_DIR.resolve()


def mkdirs(*paths):
    for p in paths:
        Path(p).mkdir(parents=True, exist_ok=True)


def write_json(path, data):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Roster Extraction
# ---------------------------------------------------------------------------

def extract_rosters():
    mkdirs(
        OUTPUT_DIR / "rosters/bb2020",
        OUTPUT_DIR / "rosters/bb2016",
        OUTPUT_DIR / "rosters/bb2025",
        OUTPUT_DIR / "star_players",
    )

    all_star_players = {}  # id -> entry (deduplicated)

    for xml_file in sorted(ROSTER_DIR.glob("*.xml")):
        tree = ET.parse(xml_file)
        root = tree.getroot()

        roster_id = root.get("id") or root.get("team") or xml_file.stem
        roster_name = get_text(root, "name") or xml_file.stem
        reroll_cost = int(get_text(root, "reRollCost") or 50000)
        max_rerolls = int(get_text(root, "maxReRolls") or 8)
        apothecary = get_text(root, "apothecary")
        has_apothecary = apothecary != "false"
        is_undead = get_text(root, "undead") == "true"
        has_necromancer = get_text(root, "necromancer") == "true"
        name_generator = get_text(root, "nameGenerator")
        raised_position_id = get_text(root, "raisedPositionId")
        riotous_position_id = get_text(root, "riotousPositionId")
        max_big_guys = get_int(root, "maxBigGuys", 0)

        # Keywords at roster level
        roster_keywords = []
        kw_container = root.find("keywords")
        if kw_container is not None:
            for kw in kw_container.findall("keyword"):
                if kw.text:
                    roster_keywords.append(kw.text.strip())

        positions = []
        infamous_staff = []

        for pos in root.findall("position"):
            pos_id = pos.get("id", "")
            pos_name = get_child(pos, "name")
            display_name = get_child(pos, "displayName") or pos_name
            pos_type = get_child(pos, "type") or "Regular"
            quantity = int(get_child(pos, "quantity") or 16)
            cost = int(get_child(pos, "cost") or 0)
            ma = int(get_child(pos, "movement") or 6)
            st = int(get_child(pos, "strength") or 3)
            ag = int(get_child(pos, "agility") or 3)
            pa_str = get_child(pos, "passing")
            pa = int(pa_str) if pa_str and pa_str not in ("-", "") else -1
            av = int(get_child(pos, "armour") or 8)
            shorthand = get_child(pos, "shorthand")
            is_undead_pos = get_child(pos, "undead") == "true"
            is_thrall = get_child(pos, "thrall") == "true"

            # Skills
            skills = []
            skill_list = pos.find("skillList")
            if skill_list is not None:
                for sk in skill_list.findall("skill"):
                    name = sk.text.strip() if sk.text else ""
                    value = sk.get("value")
                    if value:
                        try:
                            skills.append({"name": name, "value": int(value)})
                        except ValueError:
                            skills.append({"name": name, "value": value})
                    else:
                        skills.append(name)

            # Skill categories
            skill_cats = {}
            cat_list = pos.find("skillCategoryList")
            if cat_list is not None:
                for cat_type in ["normal", "double"]:
                    cats = [c.text.strip() for c in cat_list.findall(cat_type) if c.text]
                    if cats:
                        skill_cats[cat_type] = cats

            entry = {
                "id": pos_id,
                "name": pos_name,
                "display_name": display_name,
                "type": pos_type,
                "quantity": quantity,
                "cost": cost,
                "ma": ma,
                "st": st,
                "ag": ag,
                "pa": pa,
                "av": av,
            }
            if shorthand:
                entry["shorthand"] = shorthand
            if is_undead_pos:
                entry["undead"] = True
            if is_thrall:
                entry["thrall"] = True
            if skills:
                entry["skills"] = skills
            if skill_cats:
                entry["skill_categories"] = skill_cats

            if pos_type in ("Star", "HiredStar"):
                # Deduplicate by id; track which races can hire
                if pos_id not in all_star_players:
                    sp = dict(entry)
                    sp["available_for"] = [roster_name]
                    all_star_players[pos_id] = sp
                else:
                    if roster_name not in all_star_players[pos_id]["available_for"]:
                        all_star_players[pos_id]["available_for"].append(roster_name)
            elif pos_type == "Infamous Staff":
                infamous_staff.append(entry)
            else:
                positions.append(entry)

        roster = {
            "id": roster_id,
            "name": roster_name,
            "reroll_cost": reroll_cost,
            "max_rerolls": max_rerolls,
            "apothecary": has_apothecary,
            "undead": is_undead,
            "necromancer": has_necromancer,
            "positions": positions,
        }
        if name_generator:
            roster["name_generator"] = name_generator
        if roster_keywords:
            roster["keywords"] = roster_keywords
        if raised_position_id:
            roster["raised_position_id"] = raised_position_id
        if riotous_position_id:
            roster["riotous_position_id"] = riotous_position_id
        if max_big_guys:
            roster["max_big_guys"] = max_big_guys
        if infamous_staff:
            roster["infamous_staff"] = infamous_staff

        base_name = xml_file.stem
        for edition in ("bb2020", "bb2016", "bb2025"):
            write_json(OUTPUT_DIR / f"rosters/{edition}/{base_name}.json", roster)

        print(f"  Extracted roster: {roster_name} ({base_name})")

    # Write all star players
    sp_list = list(all_star_players.values())
    write_json(OUTPUT_DIR / "star_players/all_editions.json", {"star_players": sp_list})
    print(f"  Extracted {len(sp_list)} unique star players")


# ---------------------------------------------------------------------------
# Inducements
# ---------------------------------------------------------------------------

def inducement(id_, name_plural, name_singular, cost, max_count, usage, availability=None):
    d = {
        "id": id_,
        "name": name_plural,
        "name_singular": name_singular,
        "cost": cost,
        "max_count": max_count,
        "usage": usage,
    }
    if availability:
        d["availability"] = availability
    return d


def write_inducements():
    bb2020 = [
        inducement("bloodweiserKegs", "Bloodweiser Kegs", "Bloodweiser Keg", 50000, 3, "KNOCKOUT_RECOVERY"),
        inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN"),
        inducement("briberyAndCorruption", "Bribery and Corruption ReRoll", "Bribery and Corruption ReRoll",
                   0, 0, "REROLL_ARGUE", "special_rule:BRIBERY_AND_CORRUPTION"),
        inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef",
                   300000, 1, "STEAL_REROLL"),
        inducement("mortuaryAssistant", "Mortuary Assistant", "Mortuary Assistant",
                   100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"),
        inducement("plagueDoctor", "Plague Doctor", "Plague Doctor",
                   200000, 2, "REGENERATION,APOTHECARY_JOURNEYMEN", "special_rule:FAVOURED_OF_NURGLE"),
        inducement("riotousRookies", "Riotous Rookies", "Riotous Rookies",
                   100000, 3, "ADD_LINEMEN", "special_rule:LOW_COST_LINEMEN"),
        inducement("tempCheerleader", "Temp Agency Cheerleaders", "Temp Agency Cheerleader", 20000, 5, "ADD_CHEERLEADER"),
        inducement("partTimeCoach", "Part-time Assistant Coaches", "Part-time Assistant Coach", 10000, 6, "ADD_COACH"),
        inducement("weatherMage", "Weather Mage", "Weather Mage", 30000, 1, "GAME_MODIFICATION"),
        inducement("biasedReferee", "Biased Referee", "Biased Referee", 130000, 1, "GAME_MODIFICATION"),
        inducement("bugmansXXXXXX", "Bugman's XXXXXX", "Bugman's XXXXXX", 100000, 1, "ADD_COACH"),
        inducement("throwARock", "Throw a Rock", "Throw a Rock", 0, 0, "GAME_MODIFICATION", "special_rule:SPIRITED_CROWD"),
        inducement("prayers", "Prayers of Nuffle", "Prayer of Nuffle", 100000, 3, "GAME_MODIFICATION"),
        inducement("infamousStaff", "Infamous Coaching Staff", "Infamous Coaching Staff", 0, 0, "ADD_INFAMOUS_STAFF", "roster_staff"),
        inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"),
    ]

    bb2016 = [
        inducement("bloodweiserBabes", "Bloodweiser Babes", "Bloodweiser Babe", 50000, 2, "KNOCKOUT_RECOVERY"),
        inducement("assistantCoaches", "Assistant Coaches", "Assistant Coach", 10000, 6, "ADD_COACH"),
        inducement("cheerleaders", "Cheerleaders", "Cheerleader", 10000, 12, "ADD_CHEERLEADER"),
        inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN"),
        inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef", 300000, 1, "STEAL_REROLL"),
        inducement("wizard", "Wizard", "Wizard", 150000, 1, "SPELL"),
        inducement("igor", "Igor", "Igor", 100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"),
        inducement("cards", "Cards", "Card", 15000, 3, "CARD"),
        inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"),
    ]

    bb2025 = [
        inducement("teamMascot", "Team Mascot", "Team Mascot", 100000, 1, "REROLL"),
        inducement("bloodweiserKegs", "Blitzer's Best Kegs", "Blitzer's Best Keg", 50000, 3, "KNOCKOUT_RECOVERY"),
        inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN"),
        inducement("briberyAndCorruption", "Bribery and Corruption ReRoll", "Bribery and Corruption ReRoll",
                   0, 0, "REROLL_ARGUE", "special_rule:BRIBERY_AND_CORRUPTION"),
        inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef", 300000, 1, "STEAL_REROLL"),
        inducement("mortuaryAssistant", "Mortuary Assistant", "Mortuary Assistant",
                   100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"),
        inducement("plagueDoctor", "Plague Doctor", "Plague Doctor",
                   200000, 2, "REGENERATION,APOTHECARY_JOURNEYMEN", "special_rule:FAVOURED_OF_NURGLE"),
        inducement("riotousRookies", "Riotous Rookies", "Riotous Rookies",
                   100000, 3, "ADD_LINEMEN", "special_rule:LOW_COST_LINEMEN"),
        inducement("tempCheerleader", "Temp Agency Cheerleaders", "Temp Agency Cheerleader", 20000, 5, "ADD_CHEERLEADER"),
        inducement("partTimeCoach", "Part-time Assistant Coaches", "Part-time Assistant Coach", 10000, 6, "ADD_COACH"),
        inducement("weatherMage", "Weather Mage", "Weather Mage", 30000, 1, "GAME_MODIFICATION"),
        inducement("biasedReferee", "Biased Referee", "Biased Referee", 130000, 1, "GAME_MODIFICATION"),
        inducement("bugmansXXXXXX", "Bugman's XXXXXX", "Bugman's XXXXXX", 100000, 1, "ADD_COACH"),
        inducement("throwARock", "Throw a Rock", "Throw a Rock", 0, 0, "GAME_MODIFICATION", "special_rule:SPIRITED_CROWD"),
        inducement("prayers", "Prayers of Nuffle", "Prayer of Nuffle", 100000, 3, "GAME_MODIFICATION"),
        inducement("infamousStaff", "Infamous Coaching Staff", "Infamous Coaching Staff", 0, 0, "ADD_INFAMOUS_STAFF", "roster_staff"),
        inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"),
    ]

    for edition, data in [("bb2020", bb2020), ("bb2016", bb2016), ("bb2025", bb2025)]:
        write_json(OUTPUT_DIR / f"inducements/{edition}_inducements.json",
                   {"edition": edition, "inducements": data})
        print(f"  Wrote {edition} inducements: {len(data)}")


# ---------------------------------------------------------------------------
# Prayers
# ---------------------------------------------------------------------------

def prayer(roll, id_, name, description, duration, exhibition_legal, league_only,
           grants_skill=None, effect=None):
    d = {
        "roll": roll,
        "id": id_,
        "name": name,
        "description": description,
        "duration": duration,
        "exhibition_legal": exhibition_legal,
        "league_only": league_only,
    }
    if grants_skill:
        d["grants_skill"] = grants_skill
    if effect:
        d["effect"] = effect
    return d


def write_prayers():
    bb2020 = [
        prayer(1, "TREACHEROUS_TRAPDOOR", "Treacherous Trapdoor",
               "Trapdoors appear. On a roll of 1 a player stepping on them falls through them",
               "UNTIL_END_OF_HALF", True, False),
        prayer(2, "FRIENDS_WITH_THE_REF", "Friends with the Ref",
               "Argue the call succeeds on 5+", "UNTIL_END_OF_GAME", True, False),
        prayer(3, "STILETTO", "Stiletto",
               "One random player without Loner gains Stab",
               "UNTIL_END_OF_GAME", False, True, "Stab"),
        prayer(4, "IRON_MAN", "Iron Man",
               "One chosen player without Loner improves AV by 1 (Max 11+)",
               "UNTIL_END_OF_GAME", False, True, effect={"stat": "AV", "delta": 1}),
        prayer(5, "KNUCKLE_DUSTERS", "Knuckle Dusters",
               "One chosen player without Loner gains Mighty Blow (+1)",
               "UNTIL_END_OF_GAME", False, True, "Mighty Blow"),
        prayer(6, "BAD_HABITS", "Bad Habits",
               "D3 random opponent players without Loner gain Loner (2+)",
               "UNTIL_END_OF_GAME", False, True, "Loner"),
        prayer(7, "GREASY_CLEATS", "Greasy Cleats",
               "One random opponent without Loner has MA reduced by 1",
               "UNTIL_END_OF_GAME", False, True, effect={"stat": "MA", "delta": -1}),
        prayer(8, "BLESSED_STATUE_OF_NUFFLE", "Blessed Statue of Nuffle",
               "One random player without Loner gains Pro",
               "UNTIL_END_OF_GAME", True, False, "Pro"),
        prayer(9, "MOLES_UNDER_THE_PITCH", "Moles under the Pitch",
               "GFI and rush rolls suffer a -1 modifier",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "GFI_MODIFIER", "value": -1}),
        prayer(10, "PERFECT_PASSING", "Perfect Passing",
               "Successful completions gain 2 SPP instead of 1",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "COMPLETION_SPP_BONUS", "value": 1}),
        prayer(11, "FAN_INTERACTION", "Fan Interaction",
               "Crowd push casualties gain 2 SPP instead of 1",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "CROWD_PUSH_CAS_SPP_BONUS", "value": 1}),
        prayer(12, "NECESSARY_VIOLENCE", "Necessary Violence",
               "Casualties caused against your players give 2 SPP to the opponent",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "OPP_CAS_SPP_BONUS", "value": 1}),
        prayer(13, "FOULING_FRENZY", "Fouling Frenzy",
               "Foul casualties gain 2 SPP instead of 1",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "FOUL_CAS_SPP_BONUS", "value": 1}),
        prayer(14, "THROW_A_ROCK", "Throw a Rock",
               "Once per game, knock down an opponent on a 4+",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "THROW_A_ROCK"}),
        prayer(15, "UNDER_SCRUTINY", "Under Scrutiny",
               "Fouls are always spotted if the target's armour is broken",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "FOULS_ALWAYS_SPOTTED"}),
        prayer(16, "INTENSIVE_TRAINING", "Intensive Training",
               "A random player gains a primary skill",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "GAIN_PRIMARY_SKILL"}),
    ]

    bb2025 = [
        prayer(1, "TREACHEROUS_TRAPDOOR", "Treacherous Trapdoor",
               "Trapdoors appear. On a roll of 1 a player stepping on them falls through them",
               "UNTIL_END_OF_HALF", True, False),
        prayer(2, "FRIENDS_WITH_THE_REF", "Friends with the Ref",
               "Argue the call succeeds on 5+", "UNTIL_END_OF_GAME", True, False),
        prayer(3, "STILETTO", "Stiletto",
               "One random player gains Stab", "UNTIL_END_OF_GAME", False, True, "Stab"),
        prayer(4, "IRON_MAN", "Iron Man",
               "One chosen player improves AV by 1 (Max 11+)",
               "UNTIL_END_OF_GAME", False, True, effect={"stat": "AV", "delta": 1}),
        prayer(5, "KNUCKLE_DUSTERS", "Knuckle Dusters",
               "One chosen player gains Mighty Blow (+1)",
               "UNTIL_END_OF_GAME", False, True, "Mighty Blow"),
        prayer(6, "BAD_HABITS", "Bad Habits",
               "D3 random opponent players gain Loner (2+)",
               "UNTIL_END_OF_GAME", False, True, "Loner"),
        prayer(7, "GREASY_CLEATS", "Greasy Cleats",
               "One random opponent has MA reduced by 1",
               "UNTIL_END_OF_GAME", False, True, effect={"stat": "MA", "delta": -1}),
        prayer(8, "BLESSING_OF_NUFFLE", "Blessing of Nuffle",
               "One random player gains Pro", "UNTIL_END_OF_GAME", True, False, "Pro"),
        prayer(9, "MOLES_UNDER_THE_PITCH", "Moles under the Pitch",
               "GFI rolls suffer a -1 modifier",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "GFI_MODIFIER", "value": -1}),
        prayer(10, "PERFECT_PASSING", "Perfect Passing",
               "Successful completions gain 2 SPP",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "COMPLETION_SPP_BONUS", "value": 1}),
        prayer(11, "DAZZLING_CATCHING", "Dazzling Catching",
               "Catches from passes gain 1 SPP",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "CATCH_SPP_BONUS", "value": 1}),
        prayer(12, "FAN_INTERACTION", "Fan Interaction",
               "Crowd push casualties gain 2 SPP",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "CROWD_PUSH_CAS_SPP_BONUS", "value": 1}),
        prayer(13, "FOULING_FRENZY", "Fouling Frenzy",
               "Foul casualties gain 2 SPP",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "FOUL_CAS_SPP_BONUS", "value": 1}),
        prayer(14, "THROW_A_ROCK", "Throw a Rock",
               "Once per game, knock down an opponent on a 4+",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "THROW_A_ROCK"}),
        prayer(15, "UNDER_SCRUTINY", "Under Scrutiny",
               "Fouls are always spotted if the target's armour is broken",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "FOULS_ALWAYS_SPOTTED"}),
        prayer(16, "INTENSIVE_TRAINING", "Intensive Training",
               "A random player gains a primary skill",
               "UNTIL_END_OF_GAME", False, True, effect={"type": "GAIN_PRIMARY_SKILL"}),
    ]

    for edition, data in [("bb2020", bb2020), ("bb2025", bb2025)]:
        write_json(OUTPUT_DIR / f"prayers/{edition}_prayers.json",
                   {"edition": edition, "prayers": data})
        print(f"  Wrote {edition} prayers: {len(data)}")


# ---------------------------------------------------------------------------
# Serious Injuries
# ---------------------------------------------------------------------------

def write_serious_injuries():
    # 2D6 roll injury table (used for armor + injury roll)
    # Roll on injury table after armor is broken
    injury_table = [
        {"roll_min": 2, "roll_max": 7, "type": "BADLY_HURT", "name": "Badly Hurt", "is_serious": False},
        {"roll_min": 8, "roll_max": 9, "type": "BADLY_HURT", "name": "Badly Hurt (KO)", "is_serious": False},
        {"roll_min": 10, "roll_max": 10, "type": "NIGGLE", "name": "Niggling Injury", "is_serious": True},
        {"roll_min": 11, "roll_max": 11, "type": "STAT_DECREASE", "name": "Serious Injury (stat decrease)", "is_serious": True},
        {"roll_min": 12, "roll_max": 12, "type": "DEAD", "name": "DEAD!", "is_serious": True},
    ]

    # Stat decrease sub-table (D6)
    stat_decrease_table = [
        {"roll_min": 1, "roll_max": 2, "type": "MOVEMENT_DECREASE", "stat": "MA", "delta": -1},
        {"roll_min": 3, "roll_max": 4, "type": "AGILITY_DECREASE", "stat": "AG", "delta": -1},
        {"roll_min": 5, "roll_max": 5, "type": "PASSING_DECREASE", "stat": "PA", "delta": -1},
        {"roll_min": 6, "roll_max": 6, "type": "STRENGTH_DECREASE", "stat": "ST", "delta": -1},
    ]

    data = {
        "bb2016": injury_table,
        "bb2020": injury_table,
        "bb2025": injury_table,
        "stat_decrease_table": stat_decrease_table,
    }

    write_json(OUTPUT_DIR / "injuries/serious_injuries.json", data)
    print("  Wrote serious injury tables")


# ---------------------------------------------------------------------------
# Skill list extraction from Java source
# ---------------------------------------------------------------------------

def extract_skill_names():
    """Walk the skill source directories and extract skill class names."""
    skill_dir = SCRIPT_DIR / "ffb-common/src/main/java/com/fumbbl/ffb/skill"
    if not skill_dir.exists():
        print(f"  Skill dir not found: {skill_dir}")
        return

    skills_by_edition = {}
    for edition in ("common", "mixed", "bb2016", "bb2020", "bb2025"):
        ed_dir = skill_dir / edition
        if not ed_dir.exists():
            continue
        names = []
        for f in sorted(ed_dir.rglob("*.java")):
            name = f.stem
            if not name.startswith("Abstract") and not name.endswith("Test"):
                names.append(name)
        skills_by_edition[edition] = names
        print(f"  Found {len(names)} {edition} skill classes")

    for edition, names in skills_by_edition.items():
        data = {"edition": edition, "skills": [{"class_name": n} for n in names]}
        write_json(OUTPUT_DIR / f"skills/{edition}_skills.json", data)


# ---------------------------------------------------------------------------
# XML helpers
# ---------------------------------------------------------------------------

def get_text(root, tag):
    el = root.find(tag)
    return el.text.strip() if el is not None and el.text else None


def get_int(root, tag, default=0):
    s = get_text(root, tag)
    try:
        return int(s) if s else default
    except ValueError:
        return default


def get_child(el, tag):
    """Get direct child text, but only from direct children (not nested)."""
    for child in el:
        if child.tag == tag:
            return child.text.strip() if child.text else None
    return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print(f"Output directory: {OUTPUT_DIR}")
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    print("\n--- Extracting Rosters ---")
    extract_rosters()

    print("\n--- Extracting Inducements ---")
    write_inducements()

    print("\n--- Extracting Prayers ---")
    write_prayers()

    print("\n--- Extracting Serious Injuries ---")
    write_serious_injuries()

    print("\n--- Extracting Skill Names ---")
    extract_skill_names()

    print("\nData extraction complete!")
    print(f"  Output: {OUTPUT_DIR}")
