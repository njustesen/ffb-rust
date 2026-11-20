"""BACKLOG §M phase 2b(h): draft the STAT-INCREASE squads (data/teams_stats/<ed>/team_<sq>_st.json).

The characteristic improvements +MA/+ST/+AG/+PA/+AV are Java "skills" of category STAT_INCREASE
(skill/common/MovementIncrease, skill/mixed/AgilityIncrease, ...). They do nothing in play: the
server behaviour of each registers ONE PlayerModifier that `RosterPlayer.updatePosition(..,
updateStats=true, ..)` applies when the team loads (skillbehaviour/{bb2016,bb2020,bb2025,mixed}/
*IncreaseBehaviour). The Rust parity loader applies the same modifiers
(`registry::apply_player_modifiers_from_behaviours`).

Legality: League Play (a characteristic improvement is a League advancement; Matched Play --
the rule set of the data/teams_skills `_sk` squads -- has no characteristic improvements, which is
why these squads live in their own directory and are not checked by the Matched Play test). One
improvement per player, and every improvement stays inside the edition's maxima (BB2016 caps at
position+2 and 10; BB2020/BB2025 MA <= 9, ST <= 8, AG/PA >= 1+, AV <= 11+), so Java's clamp never
bites. A player with no PA ("-", stored as 0) taking +PA becomes PA 6+ -- the Java behaviour's
`getPassing() <= 0` branch, drafted on purpose (bb2020 Troll Slayer, bb2025 Deathroller).

Deterministic, no RNG. The squads are kept out of data/teams AND data/teams_skills so the certified
mirror/cross/star sweeps and the cert6 skill sweep never pick them up; scripts/sksweep.py adds the
directory when SKSWEEP_DIRS names it, scripts/skill_ledger.py schedules them under kind `skill`.

usage: python -I scripts/draft_stat_squads.py [--check]
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# (edition, base squad) -> {shirt nr: improvement}
PLAN = {
    ("bb2016", "human"): {1: "+MA", 2: "+ST", 3: "+AG", 4: "+AV"},
    ("bb2016", "dwarf"): {1: "+MA", 2: "+AG", 3: "+ST", 6: "+AV"},
    ("bb2020", "human"): {1: "+MA", 2: "+PA", 3: "+AG", 4: "+ST", 7: "+AV"},
    ("bb2020", "dwarf"): {1: "+MA", 2: "+AG", 3: "+ST", 4: "+PA", 6: "+AV"},
    ("bb2025", "human"): {1: "+MA", 3: "+AG", 4: "+PA", 5: "+ST", 7: "+AV"},
    ("bb2025", "dwarf"): {1: "+MA", 2: "+AG", 3: "+ST", 5: "+PA", 6: "+AV"},
}
STAT = {"+MA": "ma", "+ST": "st", "+AG": "ag", "+PA": "pa", "+AV": "av"}


def legal(ed, pos, imp):
    """The improved value stays inside the edition maxima (so the Java clamp is a no-op)."""
    k = STAT[imp]
    v = pos.get(k) or 0
    if ed == "bb2016":
        return k != "pa" and v + 1 <= 10
    if k == "ma":
        return v + 1 <= 9
    if k == "st":
        return v + 1 <= 8
    if k == "av":
        return v + 1 <= 11
    if k == "pa" and v <= 0:
        return True                     # "-" -> 6+
    return v - 1 >= 1                   # AG / PA are target numbers


def build(ed, base, plan):
    team = json.load(open(os.path.join(ROOT, "data", "teams", ed, "team_%s.json" % base), encoding="utf-8"))
    roster = json.load(open(os.path.join(ROOT, "data", "rosters", ed, "roster_%s.json" % base), encoding="utf-8"))
    pos = {p["id"]: p for p in roster["positions"]}
    seen = set()
    for pl in team["players"]:
        imp = plan.get(pl["nr"])
        if imp is None:
            continue
        if not legal(ed, pos[pl["position_id"]], imp):
            raise SystemExit("%s %s nr %d (%s): %s is not legal" % (ed, base, pl["nr"], pl["position_id"], imp))
        pl["skills"] = [imp]
        pl["advancement"] = "characteristic"
        seen.add(pl["nr"])
    missing = set(plan) - seen
    if missing:
        raise SystemExit("%s %s: no player nr %s" % (ed, base, sorted(missing)))
    team["stat_increased_from"] = base
    return team


def main():
    check = "--check" in sys.argv
    bad = 0
    for (ed, base), plan in sorted(PLAN.items()):
        out = os.path.join(ROOT, "data", "teams_stats", ed, "team_%s_st.json" % base)
        text = json.dumps(build(ed, base, plan), indent=2) + "\n"
        if check:
            if not os.path.exists(out) or open(out, encoding="utf-8").read() != text:
                print("MISMATCH", out)
                bad += 1
            continue
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, "w", encoding="utf-8", newline="\n").write(text)
        print("wrote", out)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
