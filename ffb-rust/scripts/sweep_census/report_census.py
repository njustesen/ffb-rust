#!/usr/bin/env python
"""Census of the REPORT stream — the coverage record that is complete by construction.

usage: report_census.py <parity-root-glob> <out.json>       e.g. "parity_rc*" report_census.json

The Rust engine's `game.report_list` is the 1:1 port of Java's `getResult().addReport(...)`: 442
add-sites against Java's ~470, carrying the roll-modifier NAMES, the `SkillUse` reasons and the
re-roll sources that the `GameEvent` side channel throws away. `ffb-parity --reports` writes it per
game as `seed_N_rust_reports.jsonl`, with the Java twin captured from `sendModelSync`.

This tallies, per (race, edition, scale) gate and in total:
  * every `reportId` produced, and how many games produced it
  * every roll-modifier name (`rollModifiers`, `armorModifiers`, `injuryModifiers`,
    `casualtyModifiers`) — the ONLY place a passive skill (Guard, Mighty Blow, Stunty, Break
    Tackle, ...) is visible in either engine
  * every `skillUse` reason and the skills that raised them
  * every re-roll source, injury type, prayer and serious injury named in a report
  * whether the Java twin file exists and matches report-for-report
"""
import glob
import json
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
MODIFIER_KEYS = ("rollModifiers", "armorModifiers", "injuryModifiers", "casualtyModifiers")


# A full sweep's logs live on the data volume (`sweep_worker.ps1 -root D:/ffb_sweep`); point
# CENSUS_ROOT there to census it. Default: the repo, where a hand-run gate writes.
CENSUS_ROOT = Path(os.environ.get("CENSUS_ROOT", str(ROOT)))


def gate_dirs(pattern):
    """Yield (race, edition, scale, dir) for every matchup directory under the given roots."""
    for root in sorted(CENSUS_ROOT.glob(pattern)):
        if not root.is_dir():
            continue
        for d in sorted(root.glob("*/*_vs_*")):
            edition = d.parent.name
            race = d.name.split("_vs_")[0]
            # The scale is not in the path; the caller groups by (race, edition) only.
            yield race, edition, "", d


def census(pattern):
    total = Counter()
    games = 0
    ids = Counter()
    id_games = Counter()
    modifiers = defaultdict(Counter)
    skill_use_reasons = Counter()
    skill_use_skills = Counter()
    skill_declined = Counter()
    reroll_sources = Counter()
    injury_types = Counter()
    serious = Counter()
    prayers = Counter()
    player_events = Counter()
    per_gate = {}
    java_games = 0
    java_identical = 0
    java_ids = Counter()      # reportId -> count, from the JAVA twin
    java_names = Counter()    # modifier / skill NAMES Java wrote

    for race, edition, _scale, d in gate_dirs(pattern):
        g_ids = Counter()
        g_games = 0
        for f in sorted(d.glob("seed_*_rust_reports.jsonl")):
            g_games += 1
            games += 1
            seen_here = set()
            for line in open(f, encoding="utf-8", errors="replace"):
                line = line.strip()
                if not line:
                    continue
                try:
                    r = json.loads(line)
                except ValueError:
                    continue
                rid = r.get("reportId", "?")
                ids[rid] += 1
                g_ids[rid] += 1
                seen_here.add(rid)
                total["reports"] += 1
                for k in MODIFIER_KEYS:
                    v = r.get(k)
                    if isinstance(v, list):
                        for name in v:
                            if isinstance(name, str) and name:
                                modifiers[k][name] += 1
                if rid == "skillUse":
                    reason = r.get("skillUse")
                    skill = r.get("skill")
                    if reason:
                        skill_use_reasons[reason] += 1
                    if skill:
                        (skill_use_skills if r.get("used") else skill_declined)[skill] += 1
                elif rid == "reRoll":
                    src = r.get("reRollSource")
                    if src:
                        reroll_sources[src] += 1
                elif rid == "injury":
                    it = r.get("injuryType")
                    if it:
                        injury_types[it] += 1
                    for key in ("seriousInjury", "seriousInjuryDecay"):
                        si = r.get(key)
                        if si:
                            serious[si] += 1
                elif rid in ("prayerRoll", "prayerEnd", "prayerWasted"):
                    p = r.get("prayer") or r.get("prayerName")
                    if p:
                        prayers[p] += 1
                elif rid == "playerEvent":
                    msg = r.get("message")
                    if msg:
                        player_events[msg.strip()] += 1
            for rid in seen_here:
                id_games[rid] += 1
            jf = Path(str(f).replace("_rust_reports.jsonl", "_java_reports.jsonl"))
            if jf.exists():
                java_games += 1
                jl = [l for l in open(jf, encoding="utf-8", errors="replace") if l.strip()]
                rl = [l for l in open(f, encoding="utf-8", errors="replace") if l.strip()]
                if len(jl) == len(rl):
                    java_identical += 1
                # Tally the JAVA side's reportIds and modifier/skill NAMES too. Until 2026-09-16
                # this census read the Rust stream only, so "neither engine reports this skill"
                # and "OUR engine is missing it" were indistinguishable -- which is how Thick
                # Skull (Java 476, Rust 0), Iron Hard Skin (137 / 0) and Diving Tackle (72 / 0)
                # sat invisible behind 330 green gates and a coverage page that called them
                # expected-dark (H.59/H.61). Stock Java is the oracle and is never modified, so
                # anything Java names and Rust does not is a RUST BUG by definition.
                for line in jl:
                    try:
                        jr = json.loads(line)
                    except Exception:
                        continue
                    jrid = jr.get("reportId")
                    if jrid:
                        java_ids[jrid] += 1
                    for bucket in MODIFIER_KEYS:
                        for name in jr.get(bucket) or []:
                            if isinstance(name, str):
                                java_names[name] += 1
                    jskill = jr.get("skill")
                    if isinstance(jskill, str):
                        java_names[jskill] += 1
        if g_games:
            per_gate["%s__%s" % (race, edition)] = {
                "race": race, "edition": edition, "games": g_games,
                "reports": sum(g_ids.values()), "ids": len(g_ids),
            }

    return {
        "games": games, "reports": total["reports"],
        "ids": dict(ids), "id_games": dict(id_games),
        "modifiers": {k: dict(v) for k, v in modifiers.items()},
        "skill_use_reasons": dict(skill_use_reasons),
        "skill_use_skills": dict(skill_use_skills),
        "skill_declined": dict(skill_declined),
        "reroll_sources": dict(reroll_sources),
        "injury_types": dict(injury_types),
        "serious": dict(serious),
        "prayers": dict(prayers),
        "player_events": dict(player_events),
        "per_gate": per_gate,
        "java_games": java_games, "java_same_length": java_identical,
        "java_ids": dict(java_ids), "java_names": dict(java_names),
    }


def report_id_catalog():
    """Every `ReportId` the model defines, in enum order."""
    src = (ROOT / "crates/ffb-model/src/report/report_id.rs").read_text(encoding="utf-8")
    body = src.split("pub enum ReportId {", 1)[1].split("\n}", 1)[0]
    out = []
    for line in body.split("\n"):
        s = line.strip()
        if not s or s.startswith("//") or s.startswith("#"):
            continue
        m = re.match(r"([A-Z][A-Z0-9_]*)\s*(?:=\s*\d+)?\s*,", s)
        if m:
            out.append(m.group(1))
    return out


def report_id_names():
    """SCREAMING_CASE variant -> the wire name `get_name()` returns."""
    src = (ROOT / "crates/ffb-model/src/report/report_id.rs").read_text(encoding="utf-8")
    return dict(re.findall(r"ReportId::([A-Z][A-Z0-9_]*)\s*=>\s*\"([^\"]+)\"", src))


if __name__ == "__main__":
    pattern = sys.argv[1] if len(sys.argv) > 1 else "parity_rc*"
    out = Path(sys.argv[2] if len(sys.argv) > 2 else "report_census.json")
    data = census(pattern)
    names = report_id_names()
    catalog = [names.get(v, v) for v in report_id_catalog()]
    data["catalog"] = catalog
    data["never_produced"] = sorted(set(catalog) - set(data["ids"]) - {"none"})
    out.write_text(json.dumps(data), encoding="utf-8")
    print("games %d, reports %d, distinct reportIds %d of %d in the catalog"
          % (data["games"], data["reports"], len(data["ids"]), len(catalog)))
    print("modifier names seen:", {k: len(v) for k, v in data["modifiers"].items()})
    print("skillUse reasons:", len(data["skill_use_reasons"]),
          " re-roll sources:", len(data["reroll_sources"]))
    print("java twins:", data["java_games"], "same length:", data["java_same_length"])

    # The oracle check. Stock Java is never modified, so any reportId or modifier/skill NAME that
    # Java writes and Rust does not is a RUST fidelity bug -- by definition, with no judgement
    # call. Reading only the Rust stream hid three of these behind 330 green gates (H.59/H.61).
    rust_names = Counter()
    for bucket in MODIFIER_KEYS:
        rust_names.update(data["modifiers"].get(bucket, {}))
    rust_names.update(data["skill_use_skills"])
    id_gap = sorted(set(data["java_ids"]) - set(data["ids"]))
    name_gap = sorted(n for n in data["java_names"] if n not in rust_names)
    data["java_only_ids"] = id_gap
    data["java_only_names"] = name_gap
    out.write_text(json.dumps(data), encoding="utf-8")
    if id_gap or name_gap:
        print()
        print("!! JAVA REPORTS IT, RUST DOES NOT -- these are Rust bugs, not coverage gaps:")
        for i in id_gap:
            print("     reportId %-32s java=%d" % (i, data["java_ids"][i]))
        for n in name_gap:
            print("     name     %-32s java=%d" % (n, data["java_names"][n]))
    else:
        print("oracle check: Rust names everything Java names.")
