"""Build the per-edition STAR PLAYER catalogue used by the star-parity tier (BACKLOG §L).

Sources (no drafting decisions are made here, only conversion):
  bb2025  rules/star_players/*.md          (bloodbowlbase.ru pages: profile + "Accept to play for")
  bb2020  rules/bb2020/star_players/*.md   (same format, BB2020 profiles)
  bb2016  ffb-java/ffb/ffb-server/rosters/roster_<squad>.xml (FFB's original LRB6 rosters, which
          list each race's hireable stars as <type>Star</type> positions)

Output: data/star_players/<edition>.json
  {"edition", "stars": [{"id", "name", "cost",
                         "players": [{"position": <roster-JSON-shaped position>,
                                      "position_xml": <the same position as Java roster XML>}],
                         "eligible": [<squad stems from data/teams/<edition>>],
                         "dropped_skills": [<names Java cannot resolve in this edition>]}]}

Every skill name is resolved against the JAVA engine's SkillFactory for the edition (exact
case-insensitive canonical names, scripts/check_skill_names.py). A name Java cannot resolve is
silently dropped by Java's roster loader, so it is dropped here for BOTH engines and listed in
`dropped_skills` -- an honest record of star rules that cannot be tested because the oracle does
not implement them for that edition.

A star is a one-game Inducement; the star tier adds exactly ONE star (a pair counts as one hire)
per team on top of the R6-legal squad. The squads themselves stay untouched (R6).

Usage: python scripts/build_star_catalog.py [--check]
"""

import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from urllib.parse import unquote
from xml.sax.saxutils import escape

sys.path.insert(0, str(Path(__file__).parent))
from check_skill_names import java_skill_names  # noqa: E402

ROOT = Path(__file__).parent.parent
JAVA_ROSTERS = ROOT.parent / "ffb-java" / "ffb" / "ffb-server" / "rosters"
RULES_SRC = {"bb2025": ROOT / "rules" / "star_players", "bb2020": ROOT / "rules" / "bb2020" / "star_players"}

OFFICIAL = {
    "Amazon": "amazon", "Black Orc": "black_orc", "Bretonnian": "bretonnian",
    "Chaos Chosen": "chaos", "Chaos Dwarf": "chaos_dwarf", "Chaos Renegades": "renegades",
    "Dark Elf": "dark_elf", "Dwarf": "dwarf", "Elven Union": "elf", "Gnome": "gnome",
    "Goblin": "goblin", "Halfling": "halfling", "High Elf": "high_elf", "Human": "human",
    "Imperial Nobility": "imperial_nobility", "Khorne": "khorne", "Lizardmen": "lizardman",
    "Necromantic Horror": "necromantic", "Norse": "norse", "Nurgle": "nurgle", "Ogre": "ogre",
    "Old World Alliance": "old_world_alliance", "Orc": "orc", "Skaven": "skaven",
    "Snotling": "snotling", "Tomb Kings": "khemri", "Shambling Undead": "undead",
    "Underworld Denizens": "underworld", "Vampire": "vampire", "Wood Elf": "wood_elf",
}
# Squads that are not an official team page: the official team whose star list they take.
# R3 variants share their base race; FUMBBL legacy imports take their nearest official team
# (Chaos Pact became Chaos Renegades, Slann play in the Lustrian Superleague like Lizardmen).
SQUAD_BASE = {
    "chaos_chaosogre": "chaos", "chaos_chaostroll": "chaos", "chaos_ogre": "chaos",
    "chaos_troll": "chaos", "old_world_alliance_ogre": "old_world_alliance",
    "old_world_alliance_treeman": "old_world_alliance", "renegades_37730": "renegades",
    "renegades_37733": "renegades", "underworld_37844": "underworld",
    "underworld_underworldtroll": "underworld", "chaos_pact": "renegades",
    "dark_elf_league_fumbbl": "dark_elf", "khemri_fumbbl": "khemri", "slann": "lizardman",
    "slann_fumbbl": "lizardman",
}
BB2016_BASE = {"renegades": "chaos_pact", "dark_elf_league_fumbbl": "dark_elf"}
# Nippon belongs to no official league: it takes the stars that play for (nearly) any team.
ANY_TEAM_SQUADS = {"nippon"}
ANY_TEAM_FRACTION = 0.9


# Rulebook spelling -> Java canonical spelling, where they differ beyond case/punctuation.
SKILL_ALIASES = {"claws": "claw", "claw": "claws", "dwarvenscourge": "dwarfenscourge",
                 "dwarfenscourge": "dwarvenscourge", "ballchain": "ballandchain"}


def norm(name: str) -> str:
    return re.sub(r"[^a-z0-9]", "", name.lower())


def java_canon(edition: str) -> dict[str, str]:
    """normalised name -> Java canonical name, for one edition."""
    names = java_skill_names()[edition.upper()]
    # check_skill_names lowercases; recover the canonical spelling from the source.
    skill_dir = ROOT.parent / "ffb-java" / "ffb" / "ffb-common" / "src" / "main" / "java" / "com" / "fumbbl" / "ffb" / "skill"
    canon = {}
    for f in skill_dir.rglob("*.java"):
        m = re.search(r'super\(\s*"((?:[^"\\]|\\.)*)"', f.read_text(encoding="utf-8", errors="replace"))
        if m:
            n = m.group(1).replace('\\"', '"')
            if n.lower() in names:
                canon.setdefault(norm(n), n)
    return canon


def squads(edition: str) -> list[str]:
    return sorted(p.stem[5:] for p in (ROOT / "data" / "teams" / edition).glob("team_*.json"))


def skill_xml(s) -> str:
    if isinstance(s, dict):
        return f'<skill value="{escape(str(s["value"]))}">{escape(s["name"])}</skill>'
    return f"<skill>{escape(s)}</skill>"


def position_xml(p: dict) -> str:
    L = [f'\t<position id="{escape(p["id"])}">',
         f"\t\t<quantity>1</quantity>",
         f"\t\t<name>{escape(p['name'])}</name>",
         f"\t\t<type>Star</type>",
         f"\t\t<cost>{p['cost']}</cost>",
         f"\t\t<movement>{p['ma']}</movement>",
         f"\t\t<strength>{p['st']}</strength>",
         f"\t\t<agility>{p['ag']}</agility>",
         f"\t\t<passing>{p['pa']}</passing>",
         f"\t\t<armour>{p['av']}</armour>"]
    if p["skills"]:
        L.append("\t\t<skillList>")
        L += [f"\t\t\t{skill_xml(s)}" for s in p["skills"]]
        L.append("\t\t</skillList>")
    else:
        L.append("\t\t<skillList/>")
    L.append("\t\t<skillCategoryList>")
    L.append("\t\t</skillCategoryList>")
    if p.get("keywords"):
        L.append("\t\t<keywords>")
        L += [f"\t\t\t<keyword>{escape(k)}</keyword>" for k in p["keywords"]]
        L.append("\t\t</keywords>")
    L.append("\t</position>")
    return "\n".join(L) + "\n"


# (position id, canonical skill name) -> skill value the rules give in prose.
# The Swift Twins (rules/star_players/The_Swift_Twins.md): Lucien re-rolls a block die (block
# variant), Valen passes to Lucien without range modifiers (pass variant).
SKILL_VARIANTS = {
    ("star.lucien_swift", "Working in Tandem"): "block",
    ("star.valen_swift", "Working in Tandem"): "pass",
}


def resolve_skill(raw: str, arg: str | None, canon: dict, edition: str, dropped: list):
    c = canon.get(norm(raw)) or canon.get(SKILL_ALIASES.get(norm(raw), ""))
    if c is None:
        dropped.append(raw)
        return None
    if arg is None:
        return c
    m = re.fullmatch(r"\+?(\d+)\+?", arg.strip())
    if m:
        v = int(m.group(1))
        # BB2025 Mighty Blow is a flat +1 with no value in the data (roster JSON convention).
        if edition == "bb2025" and norm(c) == "mightyblow":
            return c
        return {"name": c, "value": v}
    return {"name": c, "value": arg.strip().lower()}


def stat(v: str) -> int:
    v = v.replace("\\", "").strip()
    m = re.match(r"(\d+)", v)
    return int(m.group(1)) if m else 0


def slug(s: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", s.lower()).strip("_")


def parse_md(path: Path, edition: str, canon: dict):
    text = path.read_text(encoding="utf-8")
    name = re.search(r"^# (.+)$", text, re.M).group(1).strip()
    cost_m = re.search(r"(\d+)\s*K\b", text)
    cost = int(cost_m.group(1)) * 1000
    # Split into the header (single profile) or ### sub-profiles (pairs).
    tail_idx = re.search(r"^### (plays for|special rules|accept to play for)", text, re.M | re.I).start()
    body = text[:tail_idx]
    blocks = re.split(r"^#{3,4} (.+)$", body, flags=re.M)
    profiles = []
    if len(blocks) == 1:
        profiles.append((name, blocks[0]))
    else:
        for i in range(1, len(blocks), 2):
            profiles.append((blocks[i].strip(), blocks[i + 1]))
    dropped: list[str] = []
    players = []
    for pname, ptxt in profiles:
        row = None
        for line in ptxt.splitlines():
            cells = [c for c in line.split("|")]
            vals = [c.strip() for c in cells if c.strip()]
            if len(vals) >= 5 and re.match(r"^\d+$", vals[-5].replace("\\", "")) and re.search(r"\d\+$", vals[-1]):
                row = vals[-5:]
        if row is None:
            raise SystemExit(f"{path}: no stat row for {pname}")
        kw_m = re.search(r"_\(([^)]*)\)_", ptxt)
        keywords = [k.strip() for k in kw_m.group(1).split(",")] if kw_m else []
        skills = []
        for m in re.finditer(r"^\s*\*\s+(?:\[([^\]]+)\]\([^)]*\)|\*\*(.+?)\*\*)\s*(?:\(([^)]*)\))?\s*$", ptxt, re.M):
            raw = (m.group(1) or m.group(2)).strip()
            arg = m.group(3)
            # "Hatred (Undead)" may also be written inline in the link text.
            im = re.fullmatch(r"(.+?)\s*\(([^)]*)\)", raw)
            if im and arg is None:
                raw, arg = im.group(1), im.group(2)
            s = resolve_skill(raw, arg, canon, edition, dropped)
            # A variant the rules text states in prose, not as "(arg)": Java reads it as the
            # skill value (WorkingInTandem.VARIANT_BLOCK / VARIANT_PASS); without it neither
            # engine can ever trigger the skill.
            if isinstance(s, str) and arg is None:
                v = SKILL_VARIANTS.get((f"star.{slug(pname)}", s))
                if v is not None:
                    s = {"name": s, "value": v}
            if s is not None:
                skills.append(s)
        players.append({
            "id": f"star.{slug(pname)}", "name": pname, "type": "Star", "quantity": 1,
            "cost": cost if not players else 0,
            "ma": stat(row[0]), "st": stat(row[1]), "ag": stat(row[2]), "pa": stat(row[3]),
            "av": stat(row[4]), "skills": skills, "keywords": keywords,
        })
    acc_idx = text.find("### Accept to play for")
    accepts = re.findall(r"\[([^\]]+)\]\(\.\./\.\./teams/", text[acc_idx:]) if acc_idx >= 0 else []
    return {"id": f"star.{slug(name)}", "name": name, "cost": cost, "players": players,
            "accepts": accepts, "dropped_skills": sorted(set(dropped))}


def eligible_squads(accepts: list[str], edition: str, n_official: int) -> list[str]:
    bases = set()
    for a in accepts:
        if a not in OFFICIAL:
            raise SystemExit(f"unknown team page name {a!r}")
        bases.add(OFFICIAL[a])
    out = []
    for sq in squads(edition):
        if sq in ANY_TEAM_SQUADS:
            if len(bases) >= ANY_TEAM_FRACTION * n_official:
                out.append(sq)
            continue
        if SQUAD_BASE.get(sq, sq) in bases:
            out.append(sq)
    return out


def build_rules_edition(edition: str) -> list[dict]:
    canon = java_canon(edition)
    stars = [parse_md(p, edition, canon) for p in sorted(RULES_SRC[edition].glob("*.md"))]
    official_present = set()
    for s in stars:
        official_present |= set(s["accepts"])
    n_official = len(official_present)
    out = []
    for s in stars:
        s["eligible"] = eligible_squads(s.pop("accepts"), edition, n_official)
        s["players"] = [{"position": p, "position_xml": position_xml(p)} for p in s["players"]]
        out.append(s)
    return out


def build_bb2016() -> list[dict]:
    canon = java_canon("bb2016")
    by_id: dict[str, dict] = {}
    for sq in squads("bb2016"):
        xml = JAVA_ROSTERS / f"roster_{sq}.xml"
        if not xml.exists():
            raise SystemExit(f"bb2016 squad {sq}: no {xml}")
        root = ET.parse(xml).getroot()
        for pos in root.findall("position"):
            if (pos.findtext("type") or "").strip() != "Star":
                continue
            pid = pos.get("id")
            # Numeric ids >= 39000 are FUMBBL's BB2020 star profiles that the legacy dark-elf
            # roster also carries; they are not LRB6 stars.
            if pid.isdigit() and int(pid) >= 39000:
                continue
            if pid not in by_id:
                dropped: list[str] = []
                skills = []
                for sk in pos.findall("skillList/skill"):
                    s = resolve_skill(sk.text.strip(), sk.get("value"), canon, "bb2016", dropped)
                    if s is not None:
                        skills.append(s)
                name = pos.findtext("name").strip()
                p = {"id": f"star.{slug(pid)}", "name": name, "type": "Star", "quantity": 1,
                     "cost": int(pos.findtext("cost")), "ma": int(pos.findtext("movement")),
                     "st": int(pos.findtext("strength")), "ag": int(pos.findtext("agility")),
                     "pa": int(pos.findtext("passing") or 0), "av": int(pos.findtext("armour")),
                     "skills": skills, "keywords": []}
                by_id[pid] = {"id": f"star.{slug(pid)}", "name": name, "cost": p["cost"],
                              "players": [{"position": p, "position_xml": position_xml(p)}],
                              "eligible": [], "dropped_skills": sorted(set(dropped))}
            by_id[pid]["eligible"].append(sq)
    # Squads whose legacy roster carries no LRB6 star: renegades are bb2016's Chaos Pact, the
    # FUMBBL dark-elf league squad takes the Dark Elf list, and Nippon (no LRB6 race at all)
    # takes only the mercenaries every race can hire (a star name listed by >= half the races).
    have = {sq for s in by_id.values() for sq in s["eligible"]}
    races = len(have)
    by_name: dict[str, list[dict]] = {}
    for s in by_id.values():
        by_name.setdefault(s["name"], []).append(s)
    for sq in squads("bb2016"):
        if sq in have:
            continue
        base = BB2016_BASE.get(sq)
        if base is not None:
            for s in by_id.values():
                if base in s["eligible"]:
                    s["eligible"].append(sq)
        elif sq in ANY_TEAM_SQUADS:
            for name, group in sorted(by_name.items()):
                if len(group) * 2 >= races:
                    min(group, key=lambda s: s["id"])["eligible"].append(sq)
    return sorted(by_id.values(), key=lambda s: s["id"])


def main() -> int:
    check = "--check" in sys.argv
    bad = 0
    for edition in ("bb2016", "bb2020", "bb2025"):
        stars = build_bb2016() if edition == "bb2016" else build_rules_edition(edition)
        stars = [s for s in stars if s["eligible"]]
        doc = {"edition": edition, "stars": stars}
        text = json.dumps(doc, indent=1, ensure_ascii=False) + "\n"
        path = ROOT / "data" / "star_players" / f"{edition}.json"
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                print(f"MISMATCH {path}")
                bad += 1
        else:
            path.write_text(text, encoding="utf-8")
        pairs = sum(len(s["eligible"]) for s in stars)
        dropped = sorted({d for s in stars for d in s["dropped_skills"]})
        print(f"{edition}: {len(stars)} stars, {pairs} (squad, star) pairs; "
              f"dropped (Java cannot resolve): {dropped}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
