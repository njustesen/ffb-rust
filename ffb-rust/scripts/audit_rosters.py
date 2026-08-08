"""Audit & fix per-edition roster JSONs.

BB2025: reconciles data/rosters/bb2025/*.json against the official team pages
fetched into rules/teams/*.md (bloodbowlbase.ru/bb2025).
BB2016: cleans BB2020-era contamination out of data/rosters/bb2016/*.json per
docs/BB2016_DRAFTING_AND_ROSTERS.md section 2.1, and can regenerate that doc's
reference tables.

Usage:
  python scripts/audit_rosters.py --edition bb2025 --report          # diff only
  python scripts/audit_rosters.py --edition bb2025 --apply           # rewrite JSONs
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


def parse_team_page(path: Path) -> dict:
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
        # trailing '*' marks mutually-limited big-guy choices; strip it
        name_m = re.match(r"^(.*?)\*?\s*_\(([^)]*)\)_\s*$", cells[1])
        if not qty_m or not name_m:
            raise ValueError(f"{path.name}: bad qty/name: {cells[0]!r} {cells[1]!r}")
        skills = []
        for sm in re.finditer(r"\[([^\]]+)\]\([^)]*\)(?:\s*\(([^)]+)\))?", cells[7]):
            name = sm.group(1).strip()
            name = SKILL_ALIASES_BB2025.get(name, name)
            if sm.group(2):
                skills.append({"name": name, "value": parse_skill_value(sm.group(2))})
            else:
                skills.append(name)
        out["positions"].append({
            "quantity": int(qty_m.group(2)),
            "display_name": name_m.group(1).strip(),
            "keywords": [k.strip() for k in name_m.group(2).split(",")],
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
        page = parse_team_page(page_path)
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
    ap.add_argument("--edition", choices=["bb2016", "bb2025"], required=True)
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--tables", action="store_true")
    args = ap.parse_args()
    if args.tables:
        if args.edition != "bb2016":
            ap.error("--tables only supports bb2016")
        regen_bb2016_tables()
        return 0
    if args.edition == "bb2025":
        audit_bb2025(args.apply)
    else:
        audit_bb2016(args.apply)
    return 0


if __name__ == "__main__":
    sys.exit(main())
