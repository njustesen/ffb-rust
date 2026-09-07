"""Audit & fix per-edition roster JSONs.

BB2025: reconciles data/rosters/bb2025/*.json against the official team pages
fetched into rules/teams/*.md (bloodbowlbase.ru/bb2025).
BB2020: reconciles data/rosters/bb2020/*.json against rules/bb2020/teams/*.md
(bloodbowlbase.ru/bb2020) the same way -- stats, cost, quantity, skills and
skill categories, per team per position.
BB2016: cleans BB2020-era contamination out of data/rosters/bb2016/*.json per
docs/BB2016_DRAFTING_AND_ROSTERS.md section 2.1, and can regenerate that doc's
reference tables.

Usage:
  python scripts/audit_rosters.py --edition bb2025 --report          # diff only
  python scripts/audit_rosters.py --edition bb2025 --apply           # rewrite JSONs
  python scripts/audit_rosters.py --edition bb2020 --report          # diff only
  python scripts/audit_rosters.py --edition bb2020 --apply           # rewrite JSONs
  python scripts/audit_rosters.py --edition bb2016 --report|--apply  # contamination cleanup
  python scripts/audit_rosters.py --edition bb2016 --tables          # regen doc tables
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
TEAMS_DIR = ROOT / "rules" / "teams"
TEAMS_DIR_BB2020 = ROOT / "rules" / "bb2020" / "teams"
ROSTERS = {ed: ROOT / "data" / "rosters" / ed for ed in ("bb2016", "bb2020", "bb2025")}
DOC_BB2016 = ROOT / "docs" / "BB2016_DRAFTING_AND_ROSTERS.md"

# repo race key (roster_<key>.json) -> official BB2025 team page slug.
# Rosters absent here are FUMBBL-legacy for bb2025 and are left untouched.
OFFICIAL_BB2025 = {
    "amazon": "Amazon",
    "chaos": "Chaos_Chosen",
    "chaos_dwarf": "Chaos_Dwarf",
    "dark_elf": "Dark_Elf",
    "dwarf": "Dwarf",
    "elf": "Elven_Union",
    "goblin": "Goblin",
    "halfling": "Halfling",
    "high_elf": "High_Elf",
    "human": "Human",
    "khemri": "Tomb_Kings",
    "lizardman": "Lizardmen",
    "necromantic": "Necromantic_Horror",
    "norse": "Norse",
    "nurgle": "Nurgle",
    "ogre": "Ogre",
    "orc": "Orc",
    "renegades": "Chaos_Renegades",
    "skaven": "Skaven",
    "undead": "Shambling_Undead",
    "underworld": "Underworld_Denizens",
    "vampire": "Vampire",
    "wood_elf": "Wood_Elf",
}
# repo race key (roster_<key>.json) -> official BB2020 team page slug
# (rules/bb2020/teams/). The mapping is NOT identity: the bb2020 pages use the
# marketing team names, the repo uses race keys.
# Old_World_Alliance.md and Snotling.md are official bb2020 teams for which the
# repo has NO roster at all -- reported as missing rather than silently skipped.
OFFICIAL_BB2020 = {
    "amazon": "Amazon",
    "black_orc": "Black_Orc",
    "chaos": "Chaos_Chosen",
    "chaos_dwarf": "Chaos_Dwarf",
    "dark_elf": "Dark_Elf",
    "dwarf": "Dwarf",
    "elf": "Elven_Union",
    "gnome": "Gnome",
    "goblin": "Goblin",
    "halfling": "Halfling",
    "high_elf": "High_Elf",
    "human": "Human",
    "imperial_nobility": "Imperial_Nobility",
    "khemri": "Tomb_Kings",
    "khorne": "Khorne",
    "lizardman": "Lizardmen",
    "necromantic": "Necromantic_Horror",
    "norse": "Norse",
    "nurgle": "Nurgle",
    "ogre": "Ogre",
    "old_world_alliance": "Old_World_Alliance",
    "orc": "Orc",
    "renegades": "Chaos_Renegades",
    "skaven": "Skaven",
    "snotling": "Snotling",
    "undead": "Shambling_Undead",
    "underworld": "Underworld_Denizens",
    "vampire": "Vampire",
    "wood_elf": "Wood_Elf",
}

FUMBBL_LEGACY = {
    "nippon", "slann", "chaos_pact",
    "dark_elf_league_fumbbl", "khemri_fumbbl", "slann_fumbbl",
}

# Official position name -> existing JSON position id, where lineage is clear
# but automatic name matching fails. New positions get generated ids.
POSITION_ID_ALIASES = {
    ("amazon", "Eagle Warrior"): "amazon.linewoman",
    ("amazon", "Python Warrior"): "amazon.thrower",
    ("amazon", "Piranha Warrior"): "amazon.blitzer",
    ("amazon", "Jaguar Warrior"): "amazon.catcher",
    ("chaos", "Beastman Lineman"): "chaos.beastman",
    ("chaos", "Chaos Chosen"): "chaos.warrior",
    ("chaos_dwarf", "Hobgoblin Lineman"): "chaosdwarf.hobgoblin",
    ("dwarf", "Dwarf Lineman"): "dwarf.blocker",
    ("goblin", "Goblin Lineman"): "goblin.goblin",
    ("goblin", "Bomma"): "goblin.bombardier",
    ("goblin", "Trained Troll"): "goblin.troll",
    ("skaven", "Skaven Clanrat"): "skaven.lineman",
    ("undead", "Zombie Lineman"): "undead.zombie",
    ("undead", "Skeleton Lineman"): "undead.skeleton",
    ("undead", "Ghoul Runner"): "undead.ghoul",
    ("undead", "Wight Blitzer"): "undead.wight",
    ("norse", "Valkyrie"): "norse.thrower",
    ("halfling", "Halfling Hopeful"): "halfling.halfling",
    ("khemri", "Tomb Kings Blitzer"): "khemri.blitzra",
    ("khemri", "Tomb Kings Thrower"): "khemri.throra",
    ("khemri", "Skeleton Lineman"): "khemri.skeleton",
    ("lizardman", "Saurus Blocker"): "lizardman.saurus",
    ("lizardman", "Skink Lineman"): "lizardman.skink",
    ("lizardman", "Chameleon Skink"): "lizardman.chameleon_skink",
    ("orc", "Goblin Lineman"): "orc.goblin",
    ("necromantic", "Zombie Lineman"): "necromantic.zombie",
    ("necromantic", "Ghoul Runner"): "necromantic.ghoul",
    ("norse", "Norse Raider"): "norse.lineman",
    ("norse", "Yhetee"): "norse.troll.snow",
    ("nurgle", "Bloater"): "nurgle.warrior",
    ("nurgle", "Rotspawn"): "nurgle.beast",
    ("ogre", "Ogre Blocker"): "ogre.ogre",
    ("ogre", "Gnoblar Lineman"): "ogre.snotling",
    ("renegades", "Renegade Human"): "37724",
    ("renegades", "Ogre"): "37731",
    ("vampire", "Thrall Lineman"): "vampire.thrall",
    ("underworld", "Goblin Lineman"): "underworld.goblin",
    ("underworld", "Skaven Clanrat"): "underworld.skaven.lineman",
    ("underworld", "Skaven Thrower"): "underworld.skaven.thrower",
    ("underworld", "Skaven Blitzer"): "underworld.skaven.blitzer",
    ("underworld", "Troll"): "underworld.troll.warpstone",
    ("underworld", "Rat Ogre"): "37844",
}

# Page skill spelling -> engine-canonical bb2025 spelling (Java SkillFactory /
# Rust SkillId::from_class_name). Only add entries verified against the Java
# skill classes; everything else passes through and is caught by the
# roster-skill-resolution tests.
SKILL_ALIASES_BB2025 = {
    "Ball & Chain": "Ball and Chain",
    "Side Step": "Sidestep",
}

# Same idea for BB2020. The bb2020 pages spell four skills differently from the
# BB2020-canonical name the engines resolve; everything else on those 29 pages
# already matches the canonical vocabulary used by data/rosters/bb2020/*.json
# (which scripts/check_skill_names.py verifies against Java SkillFactory).
# NOTE the direction is page -> canonical, and for bb2020 the canonical spelling
# is "Side Step" (BB2025 renamed it to "Sidestep", hence the opposite mapping
# above -- do not unify the two tables).
SKILL_ALIASES_BB2020 = {
    # semantically required: Java SkillFactory.forName special-cases this one,
    # the class's canonical name is "Ball and Chain".
    "Ball & Chain": "Ball and Chain",
    # cosmetic-but-applied: the bb2020 rosters spell it "Side Step" (BB2025
    # renamed the skill to "Sidestep", hence the opposite mapping above --
    # do not unify the two tables).
    "Sidestep": "Side Step",
}

CATEGORY_LETTERS = {
    "G": "General",
    "A": "Agility",
    "S": "Strength",
    "P": "Passing",
    "D": "Devious",
    "M": "Mutation",
    "T": "Trait",
}

# BB2016 contamination rules (doc section 2.1)
BB2016_REMOVE_SKILLS = {
    "plague ridden", "multiple block", "projectile vomit", "animal savagery",
    "safe pair of hands", "on the ball", "hit and run", "defensive",
    "unchannelled fury", "hatred",
}
# dual-spelling duplicates: bb2016 canonical -> spellings to drop
BB2016_CANONICAL = {
    "bone-head": ("bone head", "bone-head", "bonehead"),
    "claw": ("claw", "claws"),
    "blood lust": ("bloodlust", "blood lust"),
}
BB2016_CANONICAL_NAME = {
    "bone-head": "Bone-Head",
    "claw": "Claw",
    "blood lust": "Blood Lust",
}


def norm(s: str) -> str:
    return re.sub(r"[^a-z0-9]", "", s.lower())


def parse_cost(s: str) -> int:
    m = re.search(r"(\d+)\s*K", s)
    if not m:
        raise ValueError(f"bad cost: {s!r}")
    return int(m.group(1)) * 1000


def parse_stat(s: str):
    # strikethrough marks a superseded value (e.g. "~~3+~~ 4+"): drop it
    s = re.sub(r"~~[^~]*~~", "", s).strip()
    if s in ("-", "–", "—", ""):
        return 0
    return int(s.rstrip("+"))


def parse_skill_value(raw: str):
    raw = raw.strip()
    m = re.fullmatch(r"\+?(\d+)\+?", raw)
    if m:
        return int(m.group(1))
    return raw.lower()


def parse_team_page(path: Path, edition: str = "bb2025") -> dict:
    """Parse an official team page.

    bb2025 (rules/teams/) and bb2020 (rules/bb2020/teams/) share the column
    layout of the Positionals table but differ in three ways:
      * bb2025 renders the position cell as ``Name _(Keyword, Keyword)_``;
        bb2020 has no keyword parenthetical at all (so no Big Guy signal).
      * skill anchors are ``#dodge-active`` on bb2025, ``#dodge`` on bb2020, and
        bb2020 prefixes each entry with a bullet -- neither affects the regex.
      * the special-rules heading is ``### League`` on bb2025 and
        ``### Special Rules`` on bb2020.
    Skill spellings go through the edition's alias table.
    """
    aliases = SKILL_ALIASES_BB2020 if edition == "bb2020" else SKILL_ALIASES_BB2025
    text = path.read_text(encoding="utf-8")
    out = {"tier": None, "positions": [], "reroll_cost": None,
           "apothecary": False, "special_rules": []}
    m = re.search(r"\*\*TIER (\d+)\*\*", text)
    if m:
        out["tier"] = int(m.group(1))

    pos_sec = re.search(r"### Positionals\n(.*?)\n###", text, re.S)
    if not pos_sec:
        raise ValueError(f"{path.name}: no Positionals section")
    body = pos_sec.group(1)
    # Rows start with a qty range like "0‑16 |" (non-ASCII hyphen U+2011 or '-')
    row_starts = [m.start() for m in re.finditer(r"^\d+[‑-]\d+ \|", body, re.M)]
    for i, start in enumerate(row_starts):
        end = row_starts[i + 1] if i + 1 < len(row_starts) else len(body)
        row = body[start:end].replace("\n", " ")
        cells = [c.strip() for c in row.split("|")]
        if len(cells) < 11:
            raise ValueError(f"{path.name}: bad row ({len(cells)} cells): {row[:80]}")
        qty_m = re.fullmatch(r"(\d+)[‑-](\d+)", cells[0])
        if edition == "bb2020":
            # no keyword parenthetical on the bb2020 pages
            nm = re.match(r"^(.*?)\*?\s*$", cells[1])
            name_m, keywords = nm, []
        else:
            # trailing '*' marks mutually-limited big-guy choices; strip it
            name_m = re.match(r"^(.*?)\*?\s*_\(([^)]*)\)_\s*$", cells[1])
            keywords = [k.strip() for k in name_m.group(2).split(",")] if name_m else []
        if not qty_m or not name_m or not name_m.group(1).strip():
            raise ValueError(f"{path.name}: bad qty/name: {cells[0]!r} {cells[1]!r}")
        skills = []
        for sm in re.finditer(r"\[([^\]]+)\]\([^)]*\)(?:\s*\(([^)]+)\))?", cells[7]):
            name = sm.group(1).strip()
            name = aliases.get(name, name)
            if sm.group(2):
                skills.append({"name": name, "value": parse_skill_value(sm.group(2))})
            else:
                skills.append(name)
        out["positions"].append({
            "quantity": int(qty_m.group(2)),
            "display_name": name_m.group(1).strip(),
            "keywords": keywords,
            "ma": parse_stat(cells[2]),
            "st": parse_stat(cells[3]),
            "ag": parse_stat(cells[4]),
            "pa": parse_stat(cells[5]),
            "av": parse_stat(cells[6]),
            "skills": skills,
            "normal": [CATEGORY_LETTERS[c] for c in cells[8].split() if c in CATEGORY_LETTERS],
            "double": [CATEGORY_LETTERS[c] for c in cells[9].split() if c in CATEGORY_LETTERS],
            "cost": parse_cost(cells[10]),
        })

    staff = re.search(r"### Staff\n(.*?)(\n### |\Z)", text, re.S)
    if staff:
        st = staff.group(1)
        rr = re.search(r"\[Re-roll\]\([^)]*\)[^0-9]*(\d+)\s*K", st)
        if rr:
            out["reroll_cost"] = int(rr.group(1)) * 1000
        out["apothecary"] = "[Apothecary]" in st
    # Both editions use "### Special Rules"; some bb2025 pages retitled it
    # "### League" (unchanged behaviour: those parse as no special rules, and
    # the bb2020 rosters carry no special_rules field at all -- see audit_bb2020).
    sr = re.search(r"### Special Rules\n(.*?)(\n### |\Z)", text, re.S)
    if sr:
        out["special_rules"] = re.findall(r"\* \[([^\]]+)\]", sr.group(1))
    return out


def match_position_id(race: str, official_name: str, existing: list) -> str | None:
    alias = POSITION_ID_ALIASES.get((race, official_name))
    if alias:
        return alias if any(p["id"] == alias for p in existing) else alias
    target = norm(official_name)
    # exact display_name/name match, then match with race words stripped
    for p in existing:
        if norm(p.get("display_name") or "") == target or norm(p["name"]) == target:
            return p["id"]
    for p in existing:
        for cand in (p.get("display_name") or "", p["name"]):
            if cand and (norm(official_name).endswith(norm(cand)) or norm(cand).endswith(target)):
                if abs(len(norm(cand)) - len(target)) <= len(race) + 8:
                    return p["id"]
    return None


def gen_position_id(roster_prefix: str, official_name: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "_", official_name.lower()).strip("_")
    return f"{roster_prefix}.{slug}"


def official_to_json(race: str, page: dict, current: dict) -> dict:
    """Build the new bb2025 roster JSON from the official page, preserving
    identity fields from the current JSON."""
    # id prefix for NEW positions: most common non-numeric prefix among
    # existing ids, else the race key (FUMBBL imports have numeric ids)
    prefixes = [p["id"].split(".")[0] for p in current["positions"]
                if not p["id"].split(".")[0].isdigit()]
    prefix = max(set(prefixes), key=prefixes.count) if prefixes else re.sub(r"[^a-z0-9]", "", race)
    existing = current["positions"]
    stars = [p for p in existing if p.get("type") in ("Star", "Infamous Staff")]
    new_positions = []
    used_ids: set[str] = set()
    for op in page["positions"]:
        pid = match_position_id(race, op["display_name"], existing) or \
            gen_position_id(prefix, op["display_name"])
        if pid in used_ids:  # fuzzy match collided with an earlier row
            pid = gen_position_id(prefix, op["display_name"])
        assert pid not in used_ids, f"{race}: duplicate position id {pid}"
        used_ids.add(pid)
        old = next((p for p in existing if p["id"] == pid), None)
        is_big = "Big Guy" in op["keywords"]
        pos = {
            "id": pid,
            "name": (old or {}).get("name") or op["display_name"],
            "display_name": op["display_name"],
            "type": "Big Guy" if is_big else "Regular",
            "quantity": op["quantity"],
            "cost": op["cost"],
            "ma": op["ma"], "st": op["st"], "ag": op["ag"],
            "pa": op["pa"], "av": op["av"],
            "skills": op["skills"],
            "skill_categories": {"normal": op["normal"], "double": op["double"]},
            "keywords": op["keywords"],
        }
        new_positions.append(pos)
    out = dict(current)
    out["reroll_cost"] = page["reroll_cost"]
    out["max_rerolls"] = 8
    out["apothecary"] = page["apothecary"]
    out["special_rules"] = page["special_rules"]
    out["positions"] = new_positions + stars
    # keep raised_position_id only if it still exists
    rp = out.get("raised_position_id")
    if rp and not any(p["id"] == rp for p in new_positions):
        out["raised_position_id"] = None
    return out


def skill_name(entry) -> str:
    return entry["name"] if isinstance(entry, dict) else entry


def diff_roster(race: str, new: dict, cur: dict) -> list[str]:
    msgs = []
    for f in ("reroll_cost", "apothecary", "special_rules"):
        if new.get(f) != cur.get(f):
            msgs.append(f"  {f}: {cur.get(f)} -> {new.get(f)}")
    cur_by_id = {p["id"]: p for p in cur["positions"] if p.get("type") not in ("Star", "Infamous Staff")}
    new_by_id = {p["id"]: p for p in new["positions"] if p.get("type") not in ("Star", "Infamous Staff")}
    for pid in sorted(set(cur_by_id) - set(new_by_id)):
        msgs.append(f"  - position removed: {pid}")
    for pid in sorted(set(new_by_id) - set(cur_by_id)):
        msgs.append(f"  + position added:   {pid}")
    for pid in sorted(set(new_by_id) & set(cur_by_id)):
        n, c = new_by_id[pid], cur_by_id[pid]
        for f in ("quantity", "cost", "ma", "st", "ag", "pa", "av", "type"):
            if n.get(f) != c.get(f):
                msgs.append(f"  {pid}.{f}: {c.get(f)} -> {n.get(f)}")
        ns = sorted(json.dumps(s, sort_keys=True) for s in n["skills"])
        cs = sorted(json.dumps(s, sort_keys=True) for s in c.get("skills", []))
        if ns != cs:
            msgs.append(f"  {pid}.skills: {[skill_name(s) for s in c.get('skills', [])]} -> {[skill_name(s) for s in n['skills']]}")
        if n.get("skill_categories") != c.get("skill_categories"):
            msgs.append(f"  {pid}.categories: {c.get('skill_categories')} -> {n.get('skill_categories')}")
    return msgs


def audit_bb2025(apply: bool) -> int:
    changed = 0
    for race, slug in sorted(OFFICIAL_BB2025.items()):
        page_path = TEAMS_DIR / f"{slug}.md"
        json_path = ROSTERS["bb2025"] / f"roster_{race}.json"
        page = parse_team_page(page_path, "bb2025")
        cur = json.loads(json_path.read_text(encoding="utf-8"))
        new = official_to_json(race, page, cur)
        msgs = diff_roster(race, new, cur)
        if msgs:
            changed += 1
            print(f"== {race} ({slug}, tier {page['tier']})")
            print("\n".join(msgs))
            if apply:
                json_path.write_text(json.dumps(new, indent=2) + "\n", encoding="utf-8")
                print(f"  APPLIED -> {json_path.name}")
    skipped = sorted(FUMBBL_LEGACY)
    print(f"\n{changed} rosters differ; FUMBBL-legacy untouched: {', '.join(skipped)}")
    return changed


# ---------------------------------------------------------------------------
# BB2020 reconciliation
# ---------------------------------------------------------------------------

# Official BB2020 position name -> existing bb2020 JSON position id, where the
# lineage is clear but automatic name matching fails.
POSITION_ID_ALIASES_BB2020: dict[tuple[str, str], str] = {}


def skill_key(entry) -> tuple[str, object]:
    """Comparison key for a roster skill: normalized name + value.

    Both engines resolve skill names case-insensitively and ignore punctuation
    (Java SkillFactory.forName is case-insensitive; Rust from_class_name strips
    non-alphanumerics), so a case/punctuation-only difference is COSMETIC and
    must not be reported as a stat mismatch.

    Numeric values (Loner 4+, Mighty Blow +1, Dirty Player +1) are compared
    strictly. STRING values are NOT: the pages carry free prose for them
    ("Animosity (All)", "(Underworld Goblin Linemen)", "(dwarfs and halflings)")
    while the roster JSONs sometimes encode the same restriction as a
    semicolon-separated list of position ids. Those differences are reported as
    a labelled representation note instead, so a prose-vs-ids encoding does not
    masquerade as a data error.
    """
    if isinstance(entry, dict):
        v = entry.get("value")
        return (norm(entry["name"]), "<str>" if isinstance(v, str) else v)
    return (norm(entry), None)


def skill_value(entry):
    return entry.get("value") if isinstance(entry, dict) else None


def skill_label(entry) -> str:
    """Human-readable skill incl. its value -- never print skills without the
    value, or a value-only mismatch renders as ``['X'] -> ['X']``."""
    if isinstance(entry, dict):
        return f"{entry['name']}({entry.get('value')})"
    return entry


# The bb2020 pages spell the Mutation category plural for some teams and the
# roster JSONs are themselves inconsistent ("Mutation" vs "Mutations"); both
# resolve to the same category in both engines.
CATEGORY_SYNONYMS = {"mutations": "mutation"}


def category_set(names) -> frozenset:
    return frozenset(CATEGORY_SYNONYMS.get(n.lower(), n.lower()) for n in (names or []))


def match_position_id_bb2020(race: str, official_name: str, existing: list) -> str | None:
    """Position matcher for bb2020.

    Deliberately does NOT consult POSITION_ID_ALIASES: that table maps BB2025
    page names onto BB2025 position ids, and applying it here made the checker
    hallucinate 11 phantom "position missing / position not on page" pairs
    (e.g. it sent goblin's "Bomma" to `goblin.bombardier`, which does not exist
    in the bb2020 JSON, while the real `goblin.bomma` sat unmatched).
    """
    target = norm(official_name)
    for p in existing:
        if norm(p.get("display_name") or "") == target or norm(p["name"]) == target:
            return p["id"]
    for p in existing:
        for cand in (p.get("display_name") or "", p["name"]):
            if cand and (target.endswith(norm(cand)) or norm(cand).endswith(target)):
                if abs(len(norm(cand)) - len(target)) <= len(race) + 8:
                    return p["id"]
    return None


def pair_positions_bb2020(race: str, page: dict, cur: dict):
    """(page position -> JSON position id) pairing, plus the unmatched sets."""
    existing = [p for p in cur["positions"]
                if p.get("type") not in ("Star", "Infamous Staff")]
    pairs: list[tuple[dict, str | None]] = []
    used: set[str] = set()
    for op in page["positions"]:
        pid = POSITION_ID_ALIASES_BB2020.get((race, op["display_name"]))
        if pid is None:
            pid = match_position_id_bb2020(
                race, op["display_name"], [p for p in existing if p["id"] not in used])
        if pid is None or pid in used or not any(p["id"] == pid for p in existing):
            pairs.append((op, None))
            continue
        used.add(pid)
        pairs.append((op, pid))
    orphans = [p for p in existing if p["id"] not in used]
    return existing, pairs, orphans


def diff_roster_bb2020(race: str, page: dict, cur: dict) -> tuple[list[str], list[str]]:
    """Reconcile one bb2020 roster JSON against its official page.

    Returns (real mismatches, cosmetic notes). Deliberately narrower than
    diff_roster(): the bb2020 pages carry no position keywords, so `type`
    (Regular / Big Guy) has no page-side truth and is not diffed; and the
    bb2020 roster JSONs carry no `special_rules` field, so the page's special
    rules are reported as informational only.
    """
    real: list[str] = []
    cosmetic: list[str] = []

    if page["reroll_cost"] != cur.get("reroll_cost"):
        real.append(f"  reroll_cost: {cur.get('reroll_cost')} -> {page['reroll_cost']}")
    if page["apothecary"] != cur.get("apothecary"):
        real.append(f"  apothecary: {cur.get('apothecary')} -> {page['apothecary']}")

    existing, pairs, orphans = pair_positions_bb2020(race, page, cur)
    matched: dict[str, dict] = {}
    for op, pid in pairs:
        if pid is None:
            real.append(f"  + position MISSING from JSON: {op['display_name']!r} "
                        f"(0-{op['quantity']}, {op['cost'] // 1000}k, "
                        f"{op['ma']}/{op['st']}/{op['ag']}+/{op['pa']}+/{op['av']}+)")
        else:
            matched[pid] = op
    for p in orphans:
        real.append(f"  - position NOT ON PAGE: {p['id']} "
                    f"({p.get('display_name') or p['name']!r})")

    for pid, op in matched.items():
        c = next(p for p in existing if p["id"] == pid)
        for f in ("quantity", "cost", "ma", "st", "ag", "pa", "av"):
            if c.get(f) != op[f]:
                real.append(f"  {pid}.{f}: {c.get(f)} -> {op[f]}")
        cs = sorted(skill_key(s) for s in c.get("skills", []))
        ps = sorted(skill_key(s) for s in op["skills"])
        if cs != ps:
            real.append(f"  {pid}.skills: {[skill_label(s) for s in c.get('skills', [])]} "
                        f"-> {[skill_label(s) for s in op['skills']]}")
        else:
            for a, b in zip(sorted(c.get("skills", []), key=skill_key),
                            sorted(op["skills"], key=skill_key)):
                an, bn = skill_name(a), skill_name(b)
                if an != bn:
                    cosmetic.append(f"  {pid}.skills: {an!r} spelled {bn!r} on the page")
                av, bv = skill_value(a), skill_value(b)
                if isinstance(av, str) and isinstance(bv, str) and av.lower() != bv.lower():
                    cosmetic.append(
                        f"  [representation] {pid}.{an} value: JSON {av!r} vs page prose {bv!r}"
                        " -- the page states the restriction in prose, the JSON encodes it;"
                        " NOT compared")
        # categories are a SET in both engines: order is meaningless, and
        # "Mutation"/"Mutations" are the same category (see CATEGORY_SYNONYMS).
        cur_cats = c.get("skill_categories") or {}
        for key, page_val in (("normal", op["normal"]), ("double", op["double"])):
            if category_set(cur_cats.get(key)) != category_set(page_val):
                real.append(f"  {pid}.categories.{key}: {cur_cats.get(key)} -> {page_val}")

    if page["special_rules"] and "special_rules" not in cur:
        cosmetic.append(f"  special_rules (page only, no JSON field): {page['special_rules']}")
    return real, cosmetic


def apply_bb2020(race: str, page: dict, cur: dict) -> dict:
    """Build the corrected bb2020 roster JSON from the official page.

    Identity fields (id, name, display_name, type, shorthand) are preserved
    from the current JSON; stats, cost, quantity, skills and skill categories
    come from the page. Star / Infamous Staff entries pass through untouched.
    New positions get a generated id and type "Regular" -- the bb2020 pages
    carry no Big Guy keyword, so a new big guy must be typed by hand.
    """
    existing = [p for p in cur["positions"]
                if p.get("type") not in ("Star", "Infamous Staff")]
    stars = [p for p in cur["positions"]
             if p.get("type") in ("Star", "Infamous Staff")]
    prefixes = [p["id"].split(".")[0] for p in existing
                if not p["id"].split(".")[0].isdigit()]
    prefix = max(set(prefixes), key=prefixes.count) if prefixes else re.sub(r"[^a-z0-9]", "", race)

    new_positions = []
    used: set[str] = set()
    for op in page["positions"]:
        pid = POSITION_ID_ALIASES_BB2020.get((race, op["display_name"]))
        if pid is None:
            pid = match_position_id_bb2020(race, op["display_name"],
                                           [p for p in existing if p["id"] not in used])
        if pid is None or pid in used:
            pid = gen_position_id(prefix, op["display_name"])
        assert pid not in used, f"{race}: duplicate position id {pid}"
        used.add(pid)
        old = next((p for p in existing if p["id"] == pid), None) or {}
        pos = dict(old)
        pos.update({
            "id": pid,
            "name": old.get("name") or op["display_name"],
            "display_name": old.get("display_name") or op["display_name"],
            "type": old.get("type") or "Regular",
            "quantity": op["quantity"],
            "cost": op["cost"],
            "ma": op["ma"], "st": op["st"], "ag": op["ag"],
            "pa": op["pa"], "av": op["av"],
            "skills": op["skills"],
            "skill_categories": {"normal": op["normal"], "double": op["double"]},
        })
        new_positions.append(pos)
    out = dict(cur)
    out["reroll_cost"] = page["reroll_cost"]
    out["apothecary"] = page["apothecary"]
    out["positions"] = new_positions + stars
    rp = out.get("raised_position_id")
    if rp and not any(p["id"] == rp for p in new_positions):
        out["raised_position_id"] = None
    return out


def audit_bb2020(apply: bool, verbose: bool = False) -> int:
    changed = 0
    clean: list[str] = []
    missing_roster: list[str] = []
    for race, slug in sorted(OFFICIAL_BB2020.items()):
        page_path = TEAMS_DIR_BB2020 / f"{slug}.md"
        json_path = ROSTERS["bb2020"] / f"roster_{race}.json"
        page = parse_team_page(page_path, "bb2020")
        if not json_path.exists():
            missing_roster.append(f"{race} ({slug})")
            continue
        cur = json.loads(json_path.read_text(encoding="utf-8"))
        real, cosmetic = diff_roster_bb2020(race, page, cur)
        print(f"== {race} ({slug}, tier {page['tier']}) "
              f"{'MISMATCH' if real else 'MATCHES PAGE'}")
        if verbose:
            _, pairs, orphans = pair_positions_bb2020(race, page, cur)
            for op, pid in pairs:
                print(f"  [pair] {op['display_name']!r} <- {pid or 'NO MATCH'}")
            for p in orphans:
                print(f"  [pair] (no page row) <- {p['id']}")
        if real:
            changed += 1
            print("\n".join(real))
        else:
            clean.append(race)
        for c in cosmetic:
            print(f"  [cosmetic]{c}")
        if real and apply:
            new = apply_bb2020(race, page, cur)
            json_path.write_text(json.dumps(new, indent=2) + "\n", encoding="utf-8")
            print(f"  APPLIED -> {json_path.name}")

    # official bb2020 teams with no roster JSON at all
    pages = {p.stem for p in TEAMS_DIR_BB2020.glob("*.md")}
    unmapped = sorted(pages - set(OFFICIAL_BB2020.values()))
    print(f"\n{len(OFFICIAL_BB2020)} official bb2020 teams mapped; "
          f"{len(clean)} match their page, {changed} differ")
    if clean:
        print(f"clean: {', '.join(sorted(clean))}")
    if missing_roster:
        print(f"mapped but roster JSON absent: {', '.join(missing_roster)}")
    if unmapped:
        print(f"official pages with NO roster: {', '.join(unmapped)}")
    legacy = sorted(p.stem.replace("roster_", "") for p in ROSTERS["bb2020"].glob("roster_*.json")
                    if p.stem.replace("roster_", "") not in OFFICIAL_BB2020)
    print(f"not official bb2020 teams (untouched): {', '.join(legacy)}")
    return changed


def selftest_bb2020() -> int:
    """Validate the bb2020 checker on a known-good and known-bad input.

    Two earlier ad-hoc validators in this campaign produced false alarms (73
    "illegal" squads, 81 "parity reds") and both were TOOL bugs. This one is
    checked before its output is believed:

      known-good : `human` must produce zero real mismatches on the real files,
                   AND every page row must pair with a distinct JSON position
                   (a matcher that pairs nothing would also report "clean").
      known-bad  : `amazon`'s reroll_cost really is 50k vs the page's 60k.
      mutations  : each field the checker claims to cover is perturbed IN MEMORY
                   on the known-good pair and must be caught.
      cosmetics  : case/punctuation/order-only perturbations must NOT be
                   reported as mismatches.
    """
    fails = 0

    def check(label: str, ok: bool) -> None:
        nonlocal fails
        print(f"  {'PASS' if ok else 'FAIL'}  {label}")
        if not ok:
            fails += 1

    def load(race):
        page = parse_team_page(TEAMS_DIR_BB2020 / f"{OFFICIAL_BB2020[race]}.md", "bb2020")
        cur = json.loads((ROSTERS["bb2020"] / f"roster_{race}.json").read_text(encoding="utf-8"))
        return page, cur

    print("known-good (human):")
    page, cur = load("human")
    real, _ = diff_roster_bb2020("human", page, cur)
    check("no real mismatches", real == [])
    existing, pairs, orphans = pair_positions_bb2020("human", page, cur)
    check(f"all {len(pairs)} page rows paired", all(pid for _, pid in pairs))
    check("no unmatched JSON positions", orphans == [])
    check("pairing is 1:1", len({pid for _, pid in pairs}) == len(pairs) == len(existing))

    print("known-bad (amazon reroll_cost 50k vs page 60k):")
    apage, acur = load("amazon")
    areal, _ = diff_roster_bb2020("amazon", apage, acur)
    check("reroll_cost reported", any("reroll_cost" in m for m in areal))
    check("50000 -> 60000 stated", any("50000 -> 60000" in m for m in areal))

    print("known-bad (mutated copies of the known-good pair):")
    muts = [
        ("ma", lambda p: p.update(ma=p["ma"] + 1), ".ma"),
        ("st", lambda p: p.update(st=p["st"] + 1), ".st"),
        ("ag", lambda p: p.update(ag=p["ag"] + 1), ".ag"),
        ("pa", lambda p: p.update(pa=p["pa"] + 1), ".pa"),
        ("av", lambda p: p.update(av=p["av"] + 1), ".av"),
        ("cost", lambda p: p.update(cost=p["cost"] + 10000), ".cost"),
        ("quantity", lambda p: p.update(quantity=p["quantity"] - 1), ".quantity"),
        ("skill dropped", lambda p: p.update(skills=p["skills"][1:]), ".skills"),
        ("skill added", lambda p: p.update(skills=p["skills"] + ["Frenzy"]), ".skills"),
        ("skill value changed",
         lambda p: p.update(skills=[{"name": skill_name(s), "value": 9} for s in p["skills"]]),
         ".skills"),
        ("category added",
         lambda p: p["skill_categories"]["normal"].append("Strength"), ".categories"),
        ("category removed",
         lambda p: p["skill_categories"].__setitem__("double", []), ".categories"),
    ]
    for label, mutate, expect in muts:
        m = json.loads(json.dumps(cur))
        # the skill mutations are no-ops on a position with no starting skills
        # (human's Lineman is the first entry and has none), so pick a target
        # that actually has some -- otherwise the test measures nothing.
        target = next(p for p in m["positions"]
                      if p.get("type") not in ("Star", "Infamous Staff")
                      and p.get("skills"))
        mutate(target)
        real, _ = diff_roster_bb2020("human", page, m)
        check(f"{label} -> {expect}", any(expect in x for x in real))

    m = json.loads(json.dumps(cur))
    m["positions"] = [p for p in m["positions"] if p is not m["positions"][0]]
    dropped = cur["positions"][0]["id"]
    real, _ = diff_roster_bb2020("human", page, m)
    check("position deleted -> MISSING from JSON",
          any("MISSING from JSON" in x for x in real))
    m = json.loads(json.dumps(cur))
    m["positions"].append(dict(m["positions"][0], id="human.invented",
                               name="Invented Guy", display_name="Invented Guy"))
    real, _ = diff_roster_bb2020("human", page, m)
    check("position invented -> NOT ON PAGE", any("NOT ON PAGE" in x for x in real))
    m = json.loads(json.dumps(cur))
    m["reroll_cost"] = m["reroll_cost"] + 10000
    real, _ = diff_roster_bb2020("human", page, m)
    check("reroll_cost mutated -> reported", any("reroll_cost" in x for x in real))
    m = json.loads(json.dumps(cur))
    m["apothecary"] = not m["apothecary"]
    real, _ = diff_roster_bb2020("human", page, m)
    check("apothecary mutated -> reported", any("apothecary" in x for x in real))

    print("cosmetic-only perturbations must NOT be reported:")
    m = json.loads(json.dumps(cur))
    for p in m["positions"]:
        p["skills"] = [skill_name(s).lower() if isinstance(s, str) else s
                       for s in p.get("skills", [])]
    real, _ = diff_roster_bb2020("human", page, m)
    check("skill names lower-cased", real == [])
    m = json.loads(json.dumps(cur))
    for p in m["positions"]:
        sc = p.get("skill_categories") or {}
        for k in sc:
            sc[k] = ["Mutations" if c == "Mutation" else c for c in reversed(sc[k])]
    real, _ = diff_roster_bb2020("human", page, m)
    check("categories reordered + Mutation->Mutations", real == [])

    # a STRING skill value (Animosity's target list) is prose on the page and an
    # id list in some rosters: a representation note, never a mismatch. But a
    # NUMERIC value must still be strict, and prose-vs-missing must still fire.
    upage, ucur = load("underworld")
    m = json.loads(json.dumps(ucur))
    for p in m["positions"]:
        p["skills"] = [dict(s, value="dwarfs and halflings")
                       if isinstance(s, dict) and isinstance(s.get("value"), str) else s
                       for s in p.get("skills", [])]
    real, cos = diff_roster_bb2020("underworld", upage, m)
    check("string skill value rewritten -> no mismatch", real == [])
    check("string skill value rewritten -> representation note",
          any("[representation]" in x for x in cos))
    m = json.loads(json.dumps(ucur))
    for p in m["positions"]:
        p["skills"] = [skill_name(s) if isinstance(s, dict) and isinstance(s.get("value"), str)
                       else s for s in p.get("skills", [])]
    real, _ = diff_roster_bb2020("underworld", upage, m)
    check("string skill value DELETED -> still a mismatch",
          any(".skills" in x for x in real))

    print(f"\nselftest: {'ALL PASS' if not fails else str(fails) + ' FAILURES'}")
    return fails


def clean_bb2016_position(pos: dict) -> tuple[dict, list[str]]:
    msgs = []
    skills = pos.get("skills", [])
    out_skills = []
    seen_canon = set()
    for s in skills:
        name = skill_name(s)
        low = name.lower()
        if low in BB2016_REMOVE_SKILLS or low.startswith("hatred"):
            msgs.append(f"drop {name}")
            continue
        # collapse dual spellings to the bb2016 canonical
        canon = None
        for key, variants in BB2016_CANONICAL.items():
            if low in variants:
                canon = key
                break
        if canon:
            if canon in seen_canon:
                msgs.append(f"dedup {name}")
                continue
            seen_canon.add(canon)
            cname = BB2016_CANONICAL_NAME[canon]
            if name != cname:
                msgs.append(f"rename {name} -> {cname}")
            out_skills.append(cname)
            continue
        # strip BB2020 parameterized values
        if isinstance(s, dict):
            msgs.append(f"strip value {name}({s['value']})")
            out_skills.append(name)
        else:
            out_skills.append(s)
    new = dict(pos)
    new["skills"] = out_skills
    return new, msgs


def audit_bb2016(apply: bool) -> int:
    changed = 0
    for json_path in sorted(ROSTERS["bb2016"].glob("roster_*.json")):
        race = json_path.stem.replace("roster_", "")
        cur = json.loads(json_path.read_text(encoding="utf-8"))
        new = dict(cur)
        all_msgs = []
        positions = []
        for pos in cur["positions"]:
            if pos.get("type") in ("Star", "Infamous Staff"):
                positions.append(pos)
                continue
            if pos.get("quantity", 0) == 0:
                all_msgs.append(f"  {pos['id']}: drop dead entry (quantity 0)")
                continue
            cleaned, msgs = clean_bb2016_position(pos)
            all_msgs.extend(f"  {pos['id']}: {m}" for m in msgs)
            positions.append(cleaned)
        new["positions"] = positions
        if all_msgs:
            changed += 1
            print(f"== {race}")
            print("\n".join(all_msgs))
            if apply:
                json_path.write_text(json.dumps(new, indent=2) + "\n", encoding="utf-8")
                print(f"  APPLIED -> {json_path.name}")
    print(f"\n{changed} bb2016 rosters cleaned")
    return changed


def regen_bb2016_tables() -> None:
    lines = []
    for json_path in sorted(ROSTERS["bb2016"].glob("roster_*.json")):
        race = json_path.stem.replace("roster_", "")
        d = json.loads(json_path.read_text(encoding="utf-8"))
        legacy = " *(FUMBBL-legacy)*" if race in FUMBBL_LEGACY else ""
        lines.append(f"\n#### {d['name']} (`{race}`){legacy}")
        lines.append(f"Re-rolls {d['reroll_cost'] // 1000}k · Apothecary {'yes' if d['apothecary'] else 'no'}")
        lines.append("")
        lines.append("| Qty | Position | Cost | MA | ST | AG | AV | Skills |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for p in d["positions"]:
            if p.get("type") in ("Star", "Infamous Staff"):
                continue
            sk = ", ".join(
                f"{skill_name(s)} ({s['value']})" if isinstance(s, dict) else s
                for s in p.get("skills", []))
            lines.append(
                f"| 0-{p['quantity']} | {p.get('display_name') or p['name']} | "
                f"{p['cost'] // 1000}k | {p['ma']} | {p['st']} | {p['ag']} | {p['av']} | {sk} |")
    block = "\n".join(lines) + "\n"
    doc = DOC_BB2016.read_text(encoding="utf-8")
    start = "<!-- BB2016_TABLES_START -->"
    end = "<!-- BB2016_TABLES_END -->"
    pre, rest = doc.split(start, 1)
    _, post = rest.split(end, 1)
    marker = "<!-- Regenerated by scripts/audit_rosters.py --edition bb2016 --tables -->"
    DOC_BB2016.write_text(pre + start + "\n" + marker + "\n" + block + end + post, encoding="utf-8")
    print(f"Regenerated tables in {DOC_BB2016}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--edition", choices=["bb2016", "bb2020", "bb2025"], required=True)
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--tables", action="store_true")
    ap.add_argument("--verbose", action="store_true",
                    help="bb2020: also print the page-row -> JSON-position pairing")
    ap.add_argument("--selftest", action="store_true",
                    help="bb2020: validate the checker on known-good/known-bad inputs")
    args = ap.parse_args()
    if args.selftest:
        if args.edition != "bb2020":
            ap.error("--selftest only supports bb2020")
        return 1 if selftest_bb2020() else 0
    if args.tables:
        if args.edition != "bb2016":
            ap.error("--tables only supports bb2016")
        regen_bb2016_tables()
        return 0
    if args.edition == "bb2025":
        audit_bb2025(args.apply)
    elif args.edition == "bb2020":
        audit_bb2020(args.apply, args.verbose)
    else:
        audit_bb2016(args.apply)
    return 0


if __name__ == "__main__":
    sys.exit(main())
