"""Extract the LRB6 (CRP) team rosters from the rulebook PDF into per-roster markdown.

    python scripts/extract_lrb6_rosters.py path/to/LRB6.pdf

Writes `rules/bb2016/teams/<Team>.md`, one file per race, plus `rules/bb2016/core_rules/` text
for the chapters. BB2016 in this project is the LRB6/CRP lineage, which is why the LRB6 rulebook is
the reference for it — see docs/BB2016_DRAFTING_AND_ROSTERS.md.

Note the notation: LRB6 prints bare characteristics (AG 4, AV 8) and derives the roll, where BB2020+
prints roll targets (AG 2+, AV 8+). Numbers here are NOT directly comparable to the bb2020/bb2025
pages — see `mechanics/roll.rs` and the two `agility_mechanic.rs` files for how each is resolved.

Requires PyMuPDF (imported as `fitz`).
"""

import re
import sys
from pathlib import Path

import fitz

OUT = Path(__file__).parent.parent / "rules" / "bb2016"
ROSTER_PAGES = range(55, 63)      # 0-indexed: "TEAM ROSTERS" through Chaos Pact
SOURCE = "LRB6 / Competition Rules Pack (CRP)"

# Heading forms on the roster pages. Chaos Pact heads without the "TEAMS" suffix.
HEAD = re.compile(r"^((?:[A-Z][A-Z'\-]*\s)*[A-Z][A-Z'\-]*)\s+TEAMS\s*$|^(CHAOS PACT)\s*$", re.M)


def page_text(page) -> str:
    """Rebuild lines from word coordinates.

    `page.get_text()` reads the roster tables in DOM order, which on several pages drops or merges
    the ST/AG/AV columns entirely — Wood Elf came out as `0-16 Linemen 70,000 7 None`, losing three
    characteristics. Grouping words by their y position and sorting each row by x recovers the
    columns: `0-16 Linemen 70,000 7 3 4 7 None GA SP`.
    """
    rows: dict[int, list[tuple[float, str]]] = {}
    for x0, y0, _x1, _y1, word, *_ in page.get_text("words"):
        rows.setdefault(round(y0 / 3), []).append((x0, word))
    return "\n".join(" ".join(w for _, w in sorted(rows[k])) for k in sorted(rows))


def clean(text: str) -> str:
    """Undo the PDF's mangled glyphs and page furniture."""
    text = text.replace("’", "'").replace("�", "'")
    out = []
    for line in text.splitlines():
        s = line.rstrip()
        if not s.strip():
            continue
        if re.match(r"^'?\s*BLOOD BOWL\s*'?$", s.strip()):
            continue
        if re.match(r"^BLOOD BOWL\s*'?$", s.strip()):
            continue
        if re.match(r"^\d{1,3}$", s.strip()):        # bare page number
            continue
        out.append(s.strip())
    return "\n".join(out)


def split_teams(pages_text: str) -> list[tuple[str, str]]:
    marks = [(m.start(), (m.group(1) or m.group(2)).strip()) for m in HEAD.finditer(pages_text)]
    teams = []
    for i, (pos, name) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(pages_text)
        teams.append((name, pages_text[pos:end].strip()))
    return teams


def main() -> int:
    pdf = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    if not pdf or not pdf.exists():
        print("usage: extract_lrb6_rosters.py <LRB6.pdf>")
        return 2
    doc = fitz.open(pdf)

    blob = clean("\n".join(page_text(doc[p]) for p in ROSTER_PAGES))
    teams = split_teams(blob)
    # drop the leading "TEAM ROSTERS" preamble if it got captured as a team
    teams = [(n, b) for n, b in teams if n not in ("TEAM ROSTERS",)]

    dest = OUT / "teams"
    dest.mkdir(parents=True, exist_ok=True)
    for name, body in teams:
        slug = name.title().replace(" ", "_").replace("'", "")
        md = (f"<!-- source: {SOURCE}, {pdf.name} pp.55-62 -->\n"
              f"<!-- LRB6 prints bare characteristics; BB2020+ prints roll targets. "
              f"AG 4 here is not AG 4+ there. -->\n\n"
              f"# {name.title()}\n\n```\n{body}\n```\n")
        (dest / f"{slug}.md").write_text(md, encoding="utf-8")
        print(f"  -> {slug}.md ({len(body):,} chars)")

    print(f"\n{len(teams)} rosters -> {dest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
