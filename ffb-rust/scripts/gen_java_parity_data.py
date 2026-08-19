"""Emit Java-side XML mirrors of the hand-drafted parity teams.

The source of truth is checked-in data the teams were drafted into ONCE:
  data/rosters/<edition>/roster_<race>.json   (audited roster data)
  data/teams/<edition>/team_<race>.json       (hand-drafted team specs)

This script only CONVERTS that data into the XML formats the Java engine
loads (it makes no drafting decisions):
  <server>/rosters/roster_<race>_<edition>.xml   roster id "<race>.<edition>"
  <server>/teams/team_<race>_parity{25|20|16}_{home|away}.xml
                                                 team id  team<Race>Parity{25|20|16}{Home|Away}

Team special rules: only names Java's SpecialRule enum resolves are emitted,
and the same filtered list is written back into the team JSON so the Rust
side uses the identical set ("Favoured of..." maps to "Favoured of Chaos
Undivided"; unknown rules like "Team Captain"/"Favoured of Hashut" are
dropped on both sides).

Usage: python scripts/gen_java_parity_data.py [--check]
"""

import json
import re
import sys
from pathlib import Path
from xml.sax.saxutils import escape

ROOT = Path(__file__).parent.parent
SERVER_DIRS = [
    Path(r"C:/Users/Admin/niels/ffb/ffb/ffb-server"),          # runtime (parity harness)
    ROOT.parent / "ffb-java" / "ffb" / "ffb-server",           # in-repo mirror
]
SPECIAL_RULE_JAVA = (ROOT.parent / "ffb-java" / "ffb" / "ffb-common" / "src" / "main"
                     / "java" / "com" / "fumbbl" / "ffb" / "model" / "SpecialRule.java")

EDITIONS = {"bb2016": "16", "bb2020": "20", "bb2025": "25"}
SPECIAL_RULE_ALIASES = {"Favoured of...": "Favoured of Chaos Undivided"}


def java_special_rules() -> set[str]:
    text = SPECIAL_RULE_JAVA.read_text(encoding="utf-8")
    return {m.lower() for m in re.findall(r'\("([^"]+)"\)', text)}


def pascal(race: str) -> str:
    return "".join(w.capitalize() for w in race.split("_"))


def skill_xml(s) -> str:
    if isinstance(s, dict):
        return f'<skill value="{escape(str(s["value"]))}">{escape(s["name"])}</skill>'
    return f"<skill>{escape(s)}</skill>"


def roster_xml(d: dict, roster_id: str, drafted_star_ids: set = frozenset()) -> str:
    L = ['<?xml version="1.0" encoding="UTF-8"?>', "", f'<roster id="{roster_id}">']
    L.append(f"\t<name>{escape(d['name'])}</name>")
    L.append(f"\t<reRollCost>{d['reroll_cost']}</reRollCost>")
    L.append(f"\t<maxReRolls>{d.get('max_rerolls', 8)}</maxReRolls>")
    L.append(f"\t<apothecary>{'true' if d.get('apothecary') else 'false'}</apothecary>")
    if d.get("undead"):
        L.append("\t<undead>true</undead>")
    if d.get("necromancer"):
        L.append("\t<necromancer>true</necromancer>")
    rp = d.get("raised_position_id")
    if rp and any(p["id"] == rp for p in d["positions"]):
        L.append(f"\t<raisedPositionId>{escape(rp)}</raisedPositionId>")
    for p in d["positions"]:
        if p.get("type") in ("Star", "Infamous Staff") and p["id"] not in drafted_star_ids:
            continue
        L.append(f'\t<position id="{escape(p["id"])}">')
        L.append(f"\t\t<quantity>{p['quantity']}</quantity>")
        L.append(f"\t\t<name>{escape(p['name'])}</name>")
        if p.get("display_name"):
            L.append(f"\t\t<displayName>{escape(p['display_name'])}</displayName>")
        L.append(f"\t\t<type>{escape(p.get('type', 'Regular'))}</type>")
        L.append(f"\t\t<cost>{p['cost']}</cost>")
        L.append(f"\t\t<movement>{p['ma']}</movement>")
        L.append(f"\t\t<strength>{p['st']}</strength>")
        L.append(f"\t\t<agility>{p['ag']}</agility>")
        L.append(f"\t\t<passing>{p.get('pa', 0)}</passing>")
        L.append(f"\t\t<armour>{p['av']}</armour>")
        skills = p.get("skills", [])
        if skills:
            L.append("\t\t<skillList>")
            for s in skills:
                L.append(f"\t\t\t{skill_xml(s)}")
            L.append("\t\t</skillList>")
        else:
            L.append("\t\t<skillList/>")
        cats = p.get("skill_categories") or {}
        L.append("\t\t<skillCategoryList>")
        for c in cats.get("normal", []):
            L.append(f"\t\t\t<normal>{escape(c)}</normal>")
        for c in cats.get("double", []):
            L.append(f"\t\t\t<double>{escape(c)}</double>")
        L.append("\t\t</skillCategoryList>")
        kws = p.get("keywords") or []
        if kws:
            L.append("\t\t<keywords>")
            for k in kws:
                L.append(f"\t\t\t<keyword>{escape(k)}</keyword>")
            L.append("\t\t</keywords>")
        L.append("\t</position>")
    L.append("</roster>")
    return "\n".join(L) + "\n"


def team_xml(team: dict, roster: dict, roster_id: str, side: str, suffix: str) -> str:
    race = team["race"]
    team_id = f"team{pascal(race)}Parity{suffix}{side.capitalize()}"
    pos_by_id = {p["id"]: p for p in roster["positions"]}
    L = ['<?xml version="1.0" encoding="UTF-8"?>', "", f'<team id="{team_id}">', ""]
    L.append(f"\t<coach>{side.capitalize()}</coach>")
    L.append(f"\t<name>{escape(roster['name'])} Parity{suffix} {side.capitalize()}</name>")
    L.append(f"\t<race>{escape(roster['name'])}</race>")
    L.append(f"\t<rosterId>{escape(roster_id)}</rosterId>")
    L.append(f"\t<reRolls>{team['rerolls']}</reRolls>")
    L.append(f"\t<fanFactor>{team['fan_factor']}</fanFactor>")
    L.append(f"\t<dedicatedFans>{team['dedicated_fans']}</dedicatedFans>")
    L.append(f"\t<apothecaries>{team['apothecaries']}</apothecaries>")
    L.append("\t<cheerleaders>0</cheerleaders>")
    L.append("\t<assistantCoaches>0</assistantCoaches>")
    L.append(f"\t<currentTeamValue>{team['team_value']}</currentTeamValue>")
    L.append("\t<division>[X]</division>")
    L.append(f"\t<treasury>{team['treasury']}</treasury>")
    rules = team.get("special_rules") or []
    if rules:
        L.append("\t<specialRules>")
        for r in rules:
            L.append(f"\t\t<rule>{escape(r)}</rule>")
        L.append("\t</specialRules>")
    else:
        L.append("\t<specialRules/>")
    L.append("")
    for pl in team["players"]:
        pos = pos_by_id[pl["position_id"]]
        pname = pos.get("display_name") or pos["name"]
        L.append(f'\t<player nr="{pl["nr"]}" id="{team_id}{pl["nr"]}">')
        L.append(f"\t\t<name>{escape(pname)} {pl['nr']}</name>")
        L.append("\t\t<gender>male</gender>")
        L.append(f"\t\t<positionId>{escape(pl['position_id'])}</positionId>")
        L.append("\t\t<skillList/>")
        L.append('\t\t<playerStatistics currentSpps="0"/>')
        L.append("\t</player>")
    L.append("</team>")
    return "\n".join(L) + "\n"


def main() -> int:
    check = "--check" in sys.argv
    known_rules = java_special_rules()
    wrote, mismatched = 0, 0

    def emit(path: Path, content: str):
        nonlocal wrote, mismatched
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                print(f"MISMATCH {path}")
                mismatched += 1
            return
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        wrote += 1

    for edition, suffix in EDITIONS.items():
        for tf in sorted((ROOT / "data" / "teams" / edition).glob("team_*.json")):
            race = tf.stem.replace("team_", "")
            team = json.loads(tf.read_text(encoding="utf-8"))
            roster = json.loads((ROOT / "data" / "rosters" / edition /
                                 f"roster_{race}.json").read_text(encoding="utf-8"))
            # filter special rules to the Java-resolvable set; persist into the
            # team JSON so Rust uses the identical list
            raw = roster.get("special_rules") or []
            filtered = []
            for r in raw:
                r = SPECIAL_RULE_ALIASES.get(r, r)
                if r.lower() in known_rules:
                    filtered.append(r)
            if team.get("special_rules") != filtered:
                team["special_rules"] = filtered
                if not check:
                    tf.write_text(json.dumps(team, indent=2) + "\n", encoding="utf-8")
            roster_id = f"{race}.{edition}"
            # Star players drafted by the spec's "stars" list are fielded as ordinary rostered
            # players: the star's stat block from data/star_players/all_editions.json is emitted
            # as an extra <position> (type Star) in the roster XML and an extra <player> in the
            # team XML. The Rust side (make_team_from_file) injects the identical player from the
            # SAME star data, so both engines stay in lockstep.
            drafted_star_ids = set()
            stars = team.get("stars") or []
            if stars:
                star_file = json.loads((ROOT / "data" / "star_players" /
                                        "all_editions.json").read_text(encoding="utf-8"))
                stars_by_id = {sp["id"]: sp for sp in star_file["star_players"]}
                roster = dict(roster)
                roster["positions"] = list(roster["positions"])
                team = dict(team)
                team["players"] = list(team["players"])
                for entry in stars:
                    sp = stars_by_id.get(entry["star_id"])
                    if sp is None:
                        raise SystemExit(f"{tf}: star id {entry['star_id']!r} not in all_editions.json")
                    if not any(p["id"] == sp["id"] for p in roster["positions"]):
                        roster["positions"].append(sp)
                    drafted_star_ids.add(sp["id"])
                    team["players"].append({"nr": entry["nr"], "position_id": sp["id"]})
                # nr-sorted, matching the Rust side's players.sort_by_key(nr): the harness
                # activation snapshots index by position, so both engines must list the
                # players in the identical order.
                team["players"] = sorted(team["players"], key=lambda p: p["nr"])
            for server in SERVER_DIRS:
                emit(server / "rosters" / f"roster_{race}_{edition}.xml",
                     roster_xml(roster, roster_id, drafted_star_ids))
                for side in ("home", "away"):
                    emit(server / "teams" / f"team_{race}_parity{suffix}_{side}.xml",
                         team_xml(team, roster, roster_id, side, suffix))
    if check:
        print(f"{mismatched} mismatched files")
        return 1 if mismatched else 0
    print(f"wrote {wrote} XML files across {len(SERVER_DIRS)} server dirs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
