"""Rebuild an edition's roster JSON from its scraped rulebook team pages.

    python scripts/roster_from_rulebook.py --edition bb2020            # dry run, prints a diff
    python scripts/roster_from_rulebook.py --edition bb2020 --write

Why: the bb2020 rosters are LRB6 data wearing BB2020 clothes — 119 of 120 positions disagree with
the rulebook, because BB2016 stores bare characteristics and BB2020 stores roll targets (see
docs/PARITY_BB2020_CAMPAIGN.md ITER127). Parity cannot catch that: both engines read the same JSON,
so a wrong stat is wrong identically on both sides and the hashes still match.

**Position ids are preserved wherever a name matches an existing position**, because every
`data/teams/<edition>/team_*.json` references positions by id. Positions the rulebook has and we do
not get a generated id; positions we have and the rulebook does not are reported and dropped, which
will invalidate any squad using them — rerun the team legality test afterwards.
"""

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).parent.parent
CATEGORY = {"G": "General", "A": "Agility", "S": "Strength", "P": "Passing", "M": "Mutation"}

# our roster file stem -> rulebook team page stem
PAGE = {
    "amazon": "Amazon", "chaos": "Chaos_Chosen", "chaos_dwarf": "Chaos_Dwarf",
    "chaos_pact": "Chaos_Renegades", "dark_elf": "Dark_Elf", "dwarf": "Dwarf",
    "elf": "Elven_Union", "goblin": "Goblin", "halfling": "Halfling", "high_elf": "High_Elf",
    "human": "Human", "khemri": "Tomb_Kings", "lizardman": "Lizardmen",
    "necromantic": "Necromantic_Horror", "norse": "Norse", "nurgle": "Nurgle", "ogre": "Ogre",
    "orc": "Orc", "skaven": "Skaven", "undead": "Shambling_Undead",
    "underworld": "Underworld_Denizens", "vampire": "Vampire", "wood_elf": "Wood_Elf",
}

ROW = re.compile(r"^\s*(\d+)[‐-―\-](\d+)\s*\|")

# The rulebook's spelling is not always Java's `super("...")` name, and Java silently drops what it
# cannot resolve. Verified with scripts/check_skill_names.py after each rebuild.
SKILL_SPELLING = {
    "Sidestep": "Side Step",
}


def logical_rows(text: str) -> list[str]:
    """Join wrapped table rows: a row runs from `0-N |` until the line carrying its cost."""
    rows, buf = [], None
    for line in text.splitlines():
        if ROW.match(line):
            if buf:
                rows.append(buf)
            buf = line
        elif buf is not None:
            buf += " " + line.strip()
        if buf and re.search(r"\|\s*\d+K\s*$", buf.rstrip()):
            rows.append(buf)
            buf = None
    if buf:
        rows.append(buf)
    return rows


def skill_value(raw: str):
    """Match the stored convention: numeric values are INTEGERS, not display strings.

    The rulebook writes `Bloodlust (3+)`, `Loner (4+)`, `Mighty Blow (+1)`, `Animosity (All)`.
    The roster JSON stores `Bloodlust: 3`, `Loner: 4`, `Animosity: "all"`. Storing the display
    form instead means `get_skill_value_int` cannot parse it and silently falls back to the
    skill's DEFAULT — a Vargheist with `"3+"` rolled Blood Lust on 2+ in Rust against Java's 3+.
    """
    v = raw.strip().strip("()").strip()
    m = re.fullmatch(r"[+\-]?(\d+)\+?", v)
    if m:
        return int(m.group(1))
    return v.lower()


def parse_skills(cell: str) -> list:
    out = []
    for m in re.finditer(r"\[([^\]]+)\]\([^)]*\)\s*(\(([^)]*)\))?", cell):
        name, _, value = m.groups()
        name = SKILL_SPELLING.get(name.strip(), name.strip())
        if value:
            out.append({"name": name, "value": skill_value(value)})
        else:
            out.append(name)
    return out


def parse_page(path: Path) -> list[dict]:
    text = path.read_text(encoding="utf-8")
    # only the Positionals table, never the Star Players list below it
    start = text.find("### Positionals")
    end = text.find("### Special Rules", start)
    body = text[start: end if end > 0 else len(text)]

    out = []
    for row in logical_rows(body):
        cells = [c.strip() for c in row.split("|")]
        if len(cells) < 11:
            continue
        qty, name, ma, st, ag, pa, av = cells[0], cells[1], cells[2], cells[3], cells[4], cells[5], cells[6]
        skills_cell, primary, secondary, cost = cells[7], cells[8], cells[9], cells[10]
        # BB2025 pages append keywords: "Wood Elf Lineman _(Lineman, Elf)_"
        name = re.sub(r"\s*_\(.*?\)_\s*$", "", name).strip()
        big_guy = name.endswith("*")
        name = name.rstrip("*").strip()
        m = re.match(r"^(\d+)[‐-―\-](\d+)$", qty)
        if not m:
            continue
        out.append({
            "name": name,
            "quantity": int(m.group(2)),
            "ma": int(ma), "st": int(st),
            "ag": int(ag[0]) if ag[0].isdigit() else 0,
            "pa": int(pa[0]) if pa[0].isdigit() else 0,
            "av": int(av[:-1]) if av.endswith("+") else int(av),
            "skills": parse_skills(skills_cell),
            "normal": [CATEGORY[c] for c in primary.split() if c in CATEGORY],
            "double": [CATEGORY[c] for c in secondary.split() if c in CATEGORY],
            "cost": int(re.sub(r"[^\d]", "", cost)) * 1000,
            "big_guy": big_guy,
        })
    return out


def norm(s: str) -> str:
    s = s.lower()
    for a, b in (("linemen", "lineman"), ("men", "man"), ("wolves", "wolf"), ("elves", "elf")):
        s = s.replace(a, b)
    words = [w[:-1] if len(w) > 3 and w.endswith("s") and not w.endswith("ss") else w
             for w in re.sub(r"[^a-z ]", "", s).split()]
    return "".join(words)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--edition", default="bb2020")
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--race")
    args = ap.parse_args()

    book_dir = ROOT / "rules" / (args.edition if args.edition != "bb2025" else ".") / "teams"
    ro_dir = ROOT / "data" / "rosters" / args.edition

    total_new = total_gone = total_changed = 0
    for race, page in sorted(PAGE.items()):
        if args.race and race != args.race:
            continue
        page_path = book_dir / f"{page}.md"
        ro_path = ro_dir / f"roster_{race}.json"
        if not page_path.exists() or not ro_path.exists():
            continue
        book = parse_page(page_path)
        if not book:
            print(f"{race}: PARSED NOTHING from {page_path.name}")
            continue
        roster = json.loads(ro_path.read_text(encoding="utf-8"))
        old = {norm(p.get("display_name") or p["name"]): p for p in roster["positions"]}

        prefix = roster["positions"][0]["id"].split(".")[0] if roster["positions"] else race
        new_positions, seen = [], set()
        for b in book:
            key = norm(b["name"])
            match = old.get(key) if key not in seen else None
            if match is None:
                # BB2020 renamed several positions by appending a role: Skeleton -> Skeleton
                # Lineman, Wight -> Wight Blitzer. Match on either side being a prefix/suffix of
                # the other, but claim each old position ONCE — BB2020 also SPLITS positions
                # (Vampire -> Runner/Blitzer/Thrower), and reusing an id would emit duplicates.
                best = None
                for k, v in old.items():
                    if k in seen:
                        continue
                    if k.startswith(key) or key.startswith(k) or k.endswith(key) or key.endswith(k):
                        if best is None or len(k) > len(best[0]):
                            best = (k, v)
                if best:
                    key, match = best
            if match:
                seen.add(key)
                pid, ptype, short = match["id"], match["type"], match.get("shorthand", "")
            else:
                pid = f"{prefix}.{re.sub(r'[^a-z0-9]', '', b['name'].lower())}"
                ptype = "BigGuy" if b["big_guy"] else "Regular"
                short = b["name"][:2]
                total_new += 1
                print(f"  {race}: NEW position {b['name']} -> {pid}")
            new_positions.append({
                "id": pid, "name": b["name"], "display_name": b["name"], "type": ptype,
                "quantity": b["quantity"], "cost": b["cost"],
                "ma": b["ma"], "st": b["st"], "ag": b["ag"], "pa": b["pa"], "av": b["av"],
                "shorthand": short,
                "skill_categories": {"normal": b["normal"], "double": b["double"]},
                "skills": b["skills"],
            })
            if match:
                diffs = [f"{k} {match[k]}->{b[k]}" for k in ("ma", "st", "ag", "pa", "av", "cost", "quantity")
                         if match.get(k) != b[k]]
                if diffs:
                    total_changed += 1
        for k, v in old.items():
            if k not in seen:
                total_gone += 1
                print(f"  {race}: DROPPED {v.get('display_name') or v['name']} ({v['id']}) "
                      f"— not in the rulebook")
        roster["positions"] = new_positions
        if args.write:
            ro_path.write_text(json.dumps(roster, indent=2) + "\n", encoding="utf-8", newline="\n")

    print(f"\n{total_changed} positions changed, {total_new} added, {total_gone} dropped"
          f"{'' if args.write else '  (dry run — pass --write)'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
