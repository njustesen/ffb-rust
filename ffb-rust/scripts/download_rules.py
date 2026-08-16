"""Download a Blood Bowl ruleset from bloodbowlbase.ru to local markdown files.

    python scripts/download_rules.py                 # BB2025 (default)
    python scripts/download_rules.py --edition bb2020
    python scripts/download_rules.py --edition bb2020 --only teams

Output layout:
    BB2025  rules/core_rules/, rules/teams/, rules/star_players/      (historical, kept in place)
    others  rules/<edition>/core_rules/, .../teams/, .../star_players/

BB2025 keeps the flat layout because `scripts/audit_rosters.py`, CLAUDE.md and the drafting docs
all reference `rules/teams/*.md` directly. Newer editions are namespaced so the three can coexist.

Section and team slugs are discovered from the edition's index page rather than hardcoded — the
two editions do not share slugs (BB2025 has `game_essentials`, BB2020 has `the_rules_of_blood_bowl`).
"""

import argparse
import collections
import re
import time
from pathlib import Path

import html2text
import requests
from bs4 import BeautifulSoup

SITE = "https://bloodbowlbase.ru/"
OUT_ROOT = Path(__file__).parent.parent / "rules"

# BB2025 shipped with a curated order; keep it so filenames stay stable for existing references.
BB2025_CORE_ORDER = [
    "game_essentials", "rules_and_regulations", "the_game_of_blood_bowl",
    "drafting_a_blood_bowl_team", "league_play", "matched_play", "exhibition_play",
    "skills_and_traits", "inducements", "the_teams", "latest_faq",
]
# BB2020's chapters, in rulebook order (the index lists them alphabetically).
BB2020_CORE_ORDER = [
    "the_rules_of_blood_bowl", "rules_and_regulations", "skills_and_traits",
    "the_teams", "inducements_in_detail", "special_plays_card_pack",
    "league_and_exhibition_play", "post-game_sequence", "blood_bowl_stadia",
    "cheat_sheet", "faq_290525",
]
CORE_ORDER = {"bb2025": BB2025_CORE_ORDER, "bb2020": BB2020_CORE_ORDER}

HEADERS = {"User-Agent": "Mozilla/5.0 (compatible; ffb-rust-rules-downloader/1.0)"}


def fetch(url: str) -> BeautifulSoup:
    resp = requests.get(url, headers=HEADERS, timeout=30)
    resp.raise_for_status()
    return BeautifulSoup(resp.text, "html.parser")


def extract_main(soup: BeautifulSoup) -> str:
    for selector in ["main", "article", ".content", ".page-content", "#content"]:
        el = soup.select_one(selector)
        if el:
            for tag in el.select("nav, .sidebar, .menu, aside"):
                tag.decompose()
            return str(el)
    body = soup.find("body")
    if body:
        for tag in body.select("header, footer, nav, .sidebar, .menu, aside"):
            tag.decompose()
        return str(body)
    return soup.prettify()


def to_markdown(html: str, source_url: str) -> str:
    h = html2text.HTML2Text()
    h.ignore_links = False
    h.body_width = 0
    h.ignore_images = True
    h.ignore_tables = False
    md = h.handle(html)
    md = re.sub(r"\n{3,}", "\n\n", md)
    return f"<!-- source: {source_url} -->\n\n" + md.strip() + "\n"


def discover(base_url: str) -> dict[str, list[str]]:
    """section -> slugs, read off the edition's index page."""
    soup = fetch(base_url)
    found = collections.defaultdict(set)
    for a in soup.find_all("a", href=True):
        href = a["href"].replace("../", "").strip()
        m = re.match(r"^(?:/[a-z0-9]+/)?(core_rules|teams|starplayers)/([A-Za-z0-9_%'.\-]+)/?$", href)
        if m:
            found[m.group(1)].add(m.group(2))
    return {k: sorted(v) for k, v in found.items()}


def save(url: str, outfile: Path) -> bool:
    try:
        md = to_markdown(extract_main(fetch(url)), url)
    except Exception as exc:                                   # noqa: BLE001
        print(f"    ERROR {url}: {exc}")
        return False
    outfile.parent.mkdir(parents=True, exist_ok=True)
    outfile.write_text(md, encoding="utf-8")
    print(f"    -> {outfile.relative_to(OUT_ROOT.parent)} ({len(md):,} chars)")
    return True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--edition", default="bb2025", choices=["bb2025", "bb2020"])
    ap.add_argument("--only", default="all", choices=["all", "core", "teams", "stars"])
    ap.add_argument("--delay", type=float, default=0.5)
    args = ap.parse_args()

    base = f"{SITE}{args.edition}/"
    dest = OUT_ROOT if args.edition == "bb2025" else OUT_ROOT / args.edition
    print(f"Downloading {args.edition} from {base}\n  into {dest}")

    idx = discover(base)
    print(f"  index: " + ", ".join(f"{k} {len(v)}" for k, v in sorted(idx.items())))

    if args.only in ("all", "core"):
        slugs = idx.get("core_rules", [])
        order = CORE_ORDER.get(args.edition, [])
        # keep the rulebook order where known, append anything new the site has added
        ordered = [s for s in order if s in slugs] + [s for s in slugs if s not in order]
        print(f"\ncore_rules ({len(ordered)})")
        for i, slug in enumerate(ordered, 1):
            save(f"{base}core_rules/{slug}/", dest / "core_rules" / f"{i:02d}_{slug}.md")
            time.sleep(args.delay)

    if args.only in ("all", "teams"):
        slugs = idx.get("teams", [])
        print(f"\nteams ({len(slugs)}) — one file per roster")
        for slug in slugs:
            save(f"{base}teams/{slug}/", dest / "teams" / f"{slug}.md")
            time.sleep(args.delay)

    if args.only in ("all", "stars"):
        slugs = idx.get("starplayers", [])
        print(f"\nstar_players ({len(slugs)})")
        for slug in slugs:
            save(f"{base}starplayers/{slug}/", dest / "star_players" / f"{slug}.md")
            time.sleep(args.delay)

    print("\ndone")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
