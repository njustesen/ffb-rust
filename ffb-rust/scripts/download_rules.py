"""Download BB2025 ruleset from bloodbowlbase.ru to local markdown files."""

import re
import time
from pathlib import Path

import html2text
import requests
from bs4 import BeautifulSoup

BASE_URL = "https://bloodbowlbase.ru/bb2025/"
OUT_DIR = Path(__file__).parent.parent / "rules"

CORE_SECTIONS = [
    ("01", "game_essentials"),
    ("02", "rules_and_regulations"),
    ("03", "the_game_of_blood_bowl"),
    ("04", "drafting_a_blood_bowl_team"),
    ("05", "league_play"),
    ("06", "matched_play"),
    ("07", "exhibition_play"),
    ("08", "skills_and_traits"),
    ("09", "inducements"),
    ("10", "the_teams"),
    ("11", "latest_faq"),
]

HEADERS = {
    "User-Agent": "Mozilla/5.0 (compatible; ffb-rust-rules-downloader/1.0)"
}


def fetch(url: str) -> BeautifulSoup:
    resp = requests.get(url, headers=HEADERS, timeout=30)
    resp.raise_for_status()
    return BeautifulSoup(resp.text, "html.parser")


def extract_main(soup: BeautifulSoup) -> str:
    # Try common content containers in order of preference
    for selector in ["main", "article", ".content", ".page-content", "#content"]:
        el = soup.select_one(selector)
        if el:
            # Remove nav/sidebar elements inside main if any
            for tag in el.select("nav, .sidebar, .menu, aside"):
                tag.decompose()
            return str(el)
    # Fallback: body without header/footer/nav
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
    # Clean up excessive blank lines
    md = re.sub(r"\n{3,}", "\n\n", md)
    return f"<!-- source: {source_url} -->\n\n" + md.strip() + "\n"


def download_core_rules():
    dest = OUT_DIR / "core_rules"
    dest.mkdir(parents=True, exist_ok=True)
    for num, slug in CORE_SECTIONS:
        url = f"{BASE_URL}core_rules/{slug}/"
        print(f"  Fetching {url}")
        soup = fetch(url)
        html = extract_main(soup)
        md = to_markdown(html, url)
        outfile = dest / f"{num}_{slug}.md"
        outfile.write_text(md, encoding="utf-8")
        print(f"    -> {outfile} ({len(md):,} chars)")
        time.sleep(0.5)


def get_star_player_slugs() -> list[str]:
    url = f"{BASE_URL}starplayers/"
    soup = fetch(url)
    slugs = []
    for a in soup.find_all("a", href=True):
        href = a["href"]
        # Match relative links like "Griff_Oberwald/" or absolute with starplayers
        if re.match(r"^[A-Z][A-Za-z0-9_%'.-]+/$", href):
            slug = href.rstrip("/")
            if slug not in slugs:
                slugs.append(slug)
        elif "starplayers/" in href:
            m = re.search(r"starplayers/([^/]+)/?$", href)
            if m:
                slug = m.group(1)
                if slug not in slugs:
                    slugs.append(slug)
    return slugs


def download_star_players():
    dest = OUT_DIR / "star_players"
    dest.mkdir(parents=True, exist_ok=True)
    print("  Fetching star player index...")
    slugs = get_star_player_slugs()
    print(f"  Found {len(slugs)} star players")
    for slug in slugs:
        url = f"{BASE_URL}starplayers/{slug}/"
        print(f"  Fetching {url}")
        try:
            soup = fetch(url)
            html = extract_main(soup)
            md = to_markdown(html, url)
            outfile = dest / f"{slug}.md"
            outfile.write_text(md, encoding="utf-8")
            print(f"    -> {outfile} ({len(md):,} chars)")
        except Exception as e:
            print(f"    ERROR: {e}")
        time.sleep(0.5)


def get_team_slugs() -> list[str]:
    """Team page slugs, parsed from the already-downloaded the_teams chapter."""
    teams_md = OUT_DIR / "core_rules" / "10_the_teams.md"
    slugs: list[str] = []
    if teams_md.exists():
        text = teams_md.read_text(encoding="utf-8")
        for m in re.finditer(r"\.\./\.\./teams/([A-Za-z0-9_]+)/", text):
            slug = m.group(1)
            if slug not in slugs:
                slugs.append(slug)
    if not slugs:
        # Fallback: scrape the teams index page
        soup = fetch(f"{BASE_URL}teams/")
        for a in soup.find_all("a", href=True):
            m = re.search(r"teams/([A-Za-z0-9_]+)/?$", a["href"])
            if m and m.group(1) not in slugs:
                slugs.append(m.group(1))
    return sorted(slugs)


def download_teams():
    dest = OUT_DIR / "teams"
    dest.mkdir(parents=True, exist_ok=True)
    slugs = get_team_slugs()
    print(f"  Found {len(slugs)} team pages")
    for slug in slugs:
        url = f"{BASE_URL}teams/{slug}/"
        print(f"  Fetching {url}")
        try:
            soup = fetch(url)
            html = extract_main(soup)
            md = to_markdown(html, url)
            outfile = dest / f"{slug}.md"
            outfile.write_text(md, encoding="utf-8")
            print(f"    -> {outfile} ({len(md):,} chars)")
        except Exception as e:
            print(f"    ERROR: {e}")
        time.sleep(0.5)


def main():
    import sys

    if "--teams-only" in sys.argv:
        print("=== Downloading BB2025 Team Rosters ===")
        download_teams()
        print("\nDone.")
        return
    print("=== Downloading BB2025 Core Rules ===")
    download_core_rules()
    print("\n=== Downloading Star Players ===")
    download_star_players()
    print("\n=== Downloading BB2025 Team Rosters ===")
    download_teams()
    print("\nDone.")


if __name__ == "__main__":
    main()
