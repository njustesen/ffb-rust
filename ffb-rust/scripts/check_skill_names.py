"""Verify every roster-JSON skill name resolves in the JAVA engine for its edition.

Java's SkillFactory.forName is an exact case-insensitive match on each skill
class's canonical name (super("Name")), restricted to classes whose
@RulesCollection covers the ruleset. Unresolvable names are silently dropped -
the exact failure mode behind past parity bugs (Bone-head, No Hands).

Usage: python scripts/check_skill_names.py [path-to-java-root]
Exits non-zero if any audited roster skill cannot resolve.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
JAVA = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT.parent / "ffb-java" / "ffb"
SKILL_DIR = JAVA / "ffb-common" / "src" / "main" / "java" / "com" / "fumbbl" / "ffb" / "skill"

# bb2020 was excluded here as "an unaudited legacy clone (not used by the parity matrix)". That
# stopped being true: BB2020 is a full parity ruleset and its 30-roster matrix runs 100/100. The
# exclusion meant the BB2020 rosters were never checked at all. Included since 2026-08-16, see
# docs/PARITY_BB2020_CAMPAIGN.md ITER123.
EDITION_TO_RULES = {"bb2016": "BB2016", "bb2020": "BB2020", "bb2025": "BB2025"}


def java_skill_names() -> dict[str, set[str]]:
    """ruleset -> set of lowercase canonical skill names."""
    by_rules: dict[str, set[str]] = {"BB2016": set(), "BB2020": set(), "BB2025": set()}
    for f in SKILL_DIR.rglob("*.java"):
        text = f.read_text(encoding="utf-8", errors="replace")
        m = re.search(r'super\(\s*"((?:[^"\\]|\\.)*)"', text)
        if not m:
            continue
        name = m.group(1).replace('\\"', '"').lower()
        # @RulesCollection is @Repeatable - collect ALL annotations
        anns = re.findall(r"@RulesCollection\(([^)]*)\)", text)
        rules: set[str] = set()
        for a in anns:
            rules |= set(re.findall(r"Rules\.(BB\d{4})", a))
            if "Rules.COMMON" in a:
                rules |= {"BB2016", "BB2020", "BB2025"}
        if not rules:
            # no annotation: subdir convention decides
            rel = f.relative_to(SKILL_DIR).parts[0]
            if rel in ("bb2016", "bb2020", "bb2025"):
                rules = {rel.upper()}
            else:
                rules = {"BB2016", "BB2020", "BB2025"}
        for r in rules & by_rules.keys():
            by_rules[r].add(name)
    # Java SkillFactory special-cases "Ball & Chain" -> "Ball and Chain"
    for names in by_rules.values():
        if "ball and chain" in names:
            names.add("ball & chain")
    return by_rules


def main() -> int:
    names = java_skill_names()
    for r, s in names.items():
        print(f"{r}: {len(s)} Java skill names")
    failures = 0
    for edition, rules in EDITION_TO_RULES.items():
        valid = names[rules]
        for jf in sorted((ROOT / "data" / "rosters" / edition).glob("roster_*.json")):
            d = json.loads(jf.read_text(encoding="utf-8"))
            for pos in d["positions"]:
                if pos.get("type") in ("Star", "Infamous Staff"):
                    continue
                for s in pos.get("skills", []):
                    name = s["name"] if isinstance(s, dict) else s
                    if name.lower() not in valid:
                        print(f"FAIL {edition}/{jf.stem}/{pos['id']}: {name!r} "
                              f"not resolvable in Java {rules}")
                        failures += 1
    # -- Drafted STAR players ------------------------------------------------
    # The roster loop above deliberately skips `type in ("Star", "Infamous Staff")` positions,
    # which was fine while no star was fielded. Every star in a team spec's `stars` list IS
    # fielded, as an ordinary rostered player, by gen_java_parity_data.py - so its skill names go
    # through the very same exact-match SkillFactory.forName. Rust's from_class_name is LENIENT
    # (lowercases and strips non-alphanumerics), so a misspelled star skill resolves in Rust and
    # is SILENTLY DROPPED in Java: the exact shape of the slann_fumbbl "Bone-Head" divergence.
    # Verified against a real case rather than assumed: temporarily drafting star 39459 (Grak)
    # into a bb2016 team makes this check FAIL on "Bone Head" (bb2020+ spelling; bb2016 Java has
    # "Bone-Head") and "Two for One" (no bb2016 class at all). His "Kick Team-mate" is FINE -
    # forName is case-insensitive, so the lowercase m resolves. That is the whole point of
    # checking against the extracted Java names instead of eyeballing spellings.
    star_file = json.loads((ROOT / "data" / "star_players" / "all_editions.json")
                           .read_text(encoding="utf-8"))
    stars_by_id = {sp["id"]: sp for sp in star_file["star_players"]}
    star_checked = 0
    for edition, rules in EDITION_TO_RULES.items():
        valid = names[rules]
        for tf in sorted((ROOT / "data" / "teams" / edition).glob("team_*.json")):
            team = json.loads(tf.read_text(encoding="utf-8"))
            for entry in team.get("stars") or []:
                sp = stars_by_id.get(entry["star_id"])
                if sp is None:
                    print(f"FAIL {edition}/{tf.stem}: star id {entry['star_id']!r} "
                          f"not in all_editions.json")
                    failures += 1
                    continue
                star_checked += 1
                for s in sp.get("skills", []):
                    name = s["name"] if isinstance(s, dict) else s
                    if name.lower() not in valid:
                        print(f"FAIL {edition}/{tf.stem}/star {sp['id']} ({sp['name']}): "
                              f"{name!r} not resolvable in Java {rules}")
                        failures += 1
    print(f"{star_checked} drafted star players audited")

    print(f"\n{failures} unresolvable skill names")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
