"""BACKLOG §M phase 2: draft the SKILLED parity squads.

For every hand-drafted squad data/teams/<ed>/team_<squad>.json this writes a variant
data/teams_skills/<ed>/team_<squad>_sk.json -- the same players, plus ONE advancement skill on
some of them -- so that every advancement-category skill Java defines for the edition sits on at
least one player whose position may take it.

Legality: BB2025 Matched Play skill points (rules/core_rules/06_matched_play.md), applied to every
edition, at the STRICTEST tier so it holds whatever a squad's real tier is (no local tier list):
  * Tier 1: 6 Skill Points, a Primary skill costs 1, a Secondary skill costs 2, at most 1
    Secondary per squad;
  * one additional skill per player; never a skill the player already has; Elite skills (Block,
    Dodge, Guard, Mighty Blow) at most 4 per roster;
  * Team Value does NOT increase in Matched Play, so team_value is copied unchanged.
Skills whose Java class for the edition has `eligible() == false` are not skills of that edition
(bb2020 PilingOn) and are never drafted.
Players who cannot use ball/skill advancements are skipped outright: No Hands (bb2016/bb2020),
No Ball (bb2025). Star / Infamous Staff positions are never skilled (the squads field none).

Not drafted (each deferred with its reason in BACKLOG §M):
  * STAT_INCREASE (MA/ST/AG/PA/AV +1): a characteristic improvement, not a skill, in all three
    rulebooks; Java applies it through PlayerModifier on load, which the Rust loader would need to
    mirror first.

Coverage target: a skill that is not LIVE in the phase-0 ledger gets 3 placements (on 3 different
squads), a LIVE one 1. Rarest-eligibility first; deterministic (sorted iteration, no RNG).

usage: python scripts/draft_skill_squads.py [--ledger D:/ffb_ledger/base/ledger.json] [--check]
"""
import argparse
import glob
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from skill_ledger import java_catalogue  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EDITIONS = ["bb2016", "bb2020", "bb2025"]
CATEGORY = {  # Java SkillCategory -> roster skill_categories spelling
    "GENERAL": "General", "AGILITY": "Agility", "STRENGTH": "Strength",
    "PASSING": "Passing", "MUTATION": "Mutation", "DEVIOUS": "Devious",
}
ELITE = {"Block", "Dodge", "Guard", "MightyBlow"}
VALUED = {("bb2020", "MightyBlow"): 1, ("bb2025", "MightyBlow"): 1,
          ("bb2020", "DirtyPlayer"): 1, ("bb2025", "DirtyPlayer"): 1}
UNSKILLABLE = {"bb2016": {"no hands"}, "bb2020": {"no hands"}, "bb2025": {"no ball"}}
SKILL_POINTS, MAX_SECONDARY, MAX_ELITE = 6, 1, 4
NEED_RARE, NEED_LIVE = 3, 1


def non_eligible():
    """{(edition, class)} whose Java class overrides `eligible()` to return false -- a class kept
    for engine plumbing that is NOT a skill of that edition (bb2020 PilingOn: "PilingOnBehavior
    is used to handle block knockdowns"). A player can never legally take one."""
    import re
    from skill_ledger import JAVA_SKILLS
    out = set()
    for path in glob.glob(os.path.join(JAVA_SKILLS, "**", "*.java"), recursive=True):
        src = open(path, encoding="utf-8", errors="replace").read()
        if not re.search(r"boolean\s+eligible\(\)\s*\{\s*return\s+false", src):
            continue
        cls = os.path.splitext(os.path.basename(path))[0]
        pkg = os.path.basename(os.path.dirname(path))
        eds = EDITIONS if pkg == "common" else (
            [r.lower() for r in re.findall(r"Rules\.(BB20\d\d)", src)] if pkg == "mixed" else [pkg])
        out.update((ed, cls) for ed in eds)
    return out


def norm(s):
    return "".join(c for c in s.lower() if c.isalnum())


def skill_name(entry):
    return entry["name"] if isinstance(entry, dict) else entry


def load(path):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def rosters(ed):
    by_id = {}
    for f in glob.glob(os.path.join(ROOT, "data", "rosters", ed, "roster_*.json")):
        r = load(f)
        by_id[r["id"]] = r
    return by_id


CAPTAIN_SKILLS = ("Team Captain", "Pro")


def team_captain(roster, spec, taken):
    """§M 2b(l): the BB2025 TEAM CAPTAIN special rule (rules/core_rules/10_the_teams.md: "you may
    nominate any player on your starting roster (with the exception of a Big Guy) to be your Team
    Captain. A Team Captain immediately gains the Pro Skill without increasing their cost").
    Java models it as the player skills `Team Captain` (skill/bb2025/special/TeamCaptain:
    canSaveReRolls + needsToBeSetUp) and `Pro` on the team file -- StepPlayerLoss.countAdvancements
    subtracts exactly those two. Nominated: the lowest shirt number that is not a Star / Infamous
    Staff / Big Guy, holds no drafted advancement and no Pro. Returns that player dict or None."""
    if "Team Captain" not in (roster.get("special_rules") or []):
        return None
    pos_by_id = {p["id"]: p for p in roster["positions"]}
    for pl in sorted(spec["players"], key=lambda p: p["nr"]):
        pos = pos_by_id[pl["position_id"]]
        big = pos.get("type") == "Big Guy" or "Big Guy" in (pos.get("keywords") or [])
        if pos.get("type") in ("Star", "Infamous Staff") or big or pl["nr"] in taken:
            continue
        if "pro" in {norm(skill_name(s)) for s in pos.get("skills", [])}:
            continue
        return pl
    return None


def draft_edition(ed, catalogue, status):
    rost = rosters(ed)
    squads = {}
    for f in sorted(glob.glob(os.path.join(ROOT, "data", "teams", ed, "team_*.json"))):
        name = os.path.basename(f)[len("team_"):-len(".json")]
        squads[name] = load(f)

    # slots[squad] = [(nr, position, set of normalised skills held)]
    slots = {}
    for sq, spec in squads.items():
        roster = rost[spec["roster_id"]]
        pos_by_id = {p["id"]: p for p in roster["positions"]}
        rows = []
        for pl in sorted(spec["players"], key=lambda p: p["nr"]):
            pos = pos_by_id[pl["position_id"]]
            if pos.get("type") in ("Star", "Infamous Staff"):
                continue
            held = {norm(skill_name(s)) for s in pos.get("skills", [])}
            if held & {norm(x) for x in UNSKILLABLE[ed]}:
                continue
            rows.append((pl["nr"], pos, held))
        slots[sq] = rows

    banned = non_eligible()
    skills = sorted((cls, cat, disp) for cls, (cat, disp) in catalogue[ed].items()
                    if cat in CATEGORY and (ed, cls) not in banned)

    def eligible(sq, cls, cat, disp):
        """(primary, nr) candidates on squad sq that may take the skill."""
        out = []
        for nr, pos, held in slots[sq]:
            if norm(disp) in held or norm(cls) in held:
                continue
            cats = pos.get("skill_categories") or {}
            if CATEGORY[cat] in cats.get("normal", []):
                out.append((True, nr, pos))
            elif CATEGORY[cat] in cats.get("double", []):
                out.append((False, nr, pos))
        return out

    budget = {sq: SKILL_POINTS for sq in squads}
    secondary = {sq: 0 for sq in squads}
    elite = {sq: 0 for sq in squads}
    taken = {sq: {} for sq in squads}           # nr -> (cls, disp, primary)
    placed = {cls: [] for cls, _, _ in skills}  # cls -> [squad]
    need = {cls: (NEED_LIVE if status.get((ed, cls)) == "LIVE" else NEED_RARE) for cls, _, _ in skills}

    def n_eligible(item):
        cls, cat, disp = item
        return sum(1 for sq in squads if any(p for p, _, _ in eligible(sq, cls, cat, disp)))

    order = sorted(skills, key=lambda it: (n_eligible(it), it[0]))
    for rnd in range(max(NEED_RARE, NEED_LIVE)):
        for cls, cat, disp in order:
            if len(placed[cls]) >= need[cls] or len(placed[cls]) > rnd:
                continue
            best = None
            for sq in sorted(squads):
                if sq in placed[cls]:
                    continue
                if cls in ELITE and elite[sq] >= MAX_ELITE:
                    continue
                for primary, nr, pos in sorted(eligible(sq, cls, cat, disp),
                                               key=lambda t: (not t[0], t[1])):
                    if nr in taken[sq]:
                        continue
                    cost = 1 if primary else 2
                    if budget[sq] < cost or (not primary and secondary[sq] >= MAX_SECONDARY):
                        continue
                    big = pos.get("type") == "Big Guy" or "Big Guy" in (pos.get("keywords") or [])
                    # prefer primary, then the squad with most points left, then non-Big-Guys,
                    # then squad name and shirt number (deterministic)
                    key = (not primary, -budget[sq], big, sq, nr)
                    if best is None or key < best[0]:
                        best = (key, sq, nr, primary)
                    break  # first eligible player of this squad (nr order) is its candidate
            if best is None:
                continue
            _, sq, nr, primary = best
            budget[sq] -= 1 if primary else 2
            secondary[sq] += 0 if primary else 1
            elite[sq] += 1 if cls in ELITE else 0
            taken[sq][nr] = (cls, disp, primary)
            placed[cls].append(sq)

    out = {}
    for sq, spec in squads.items():
        if not taken[sq]:
            continue
        v = json.loads(json.dumps(spec))
        v["skilled_from"] = sq
        v["skill_points"] = {"tier_assumed": 1, "spent": SKILL_POINTS - budget[sq],
                             "secondary": secondary[sq]}
        for pl in v["players"]:
            if pl["nr"] in taken[sq]:
                cls, disp, primary = taken[sq][pl["nr"]]
                val = VALUED.get((ed, cls))
                pl["skills"] = [{"name": disp, "value": val} if val is not None else disp]
                pl["advancement"] = "primary" if primary else "secondary"
        captain = team_captain(rost[spec["roster_id"]], v, taken[sq])
        if captain is not None:
            captain["skills"] = list(CAPTAIN_SKILLS)
            captain["advancement"] = "team_captain"
        out[f"{sq}_sk"] = v
    missing = sorted(cls for cls, _, _ in skills if not placed[cls])
    short = sorted(cls for cls, _, _ in skills if 0 < len(placed[cls]) < need[cls])
    return out, missing, short, placed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ledger", default="D:/ffb_ledger/base/ledger.json")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    status = {}
    if os.path.exists(a.ledger):
        status = {(r["edition"], r["skill"]): r["status"] for r in load(a.ledger)}
    else:
        print(f"warning: no ledger at {a.ledger}; every skill gets {NEED_RARE} placements")
    cat = java_catalogue()
    bad = 0
    for ed in EDITIONS:
        out, missing, short, placed = draft_edition(ed, cat, status)
        n = sum(len(v) for v in placed.values())
        print(f"{ed}: {len(out)} skilled squads, {n} placements, {len(placed)} skills; "
              f"unplaceable {len(missing)} {missing}; short {len(short)} {short}")
        d = os.path.join(ROOT, "data", "teams_skills", ed)
        for name, spec in sorted(out.items()):
            path = os.path.join(d, f"team_{name}.json")
            text = json.dumps(spec, indent=2) + "\n"
            if a.check:
                if not os.path.exists(path) or open(path, encoding="utf-8").read() != text:
                    print(f"MISMATCH {path}")
                    bad += 1
                continue
            os.makedirs(d, exist_ok=True)
            with open(path, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(text)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
