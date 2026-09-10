#!/usr/bin/env python
"""Aggregate every rust GameEvent log of the 2026-09-10 full-matrix sweep.

One JSON per gate (race, edition, scale) into out/, mirroring
crates/ffb-parity/src/coverage_report.rs `tally()` so the numbers mean the same
thing the harness's own checklist means.
"""
import json
import os
import re
import sys
from collections import Counter
from multiprocessing import Pool
from pathlib import Path

REPO = Path(r"C:\Users\Admin\niels\ffb-rust\ffb-rust")
OUT = Path(sys.argv[1] if len(sys.argv) > 1 else "out")
OUT.mkdir(parents=True, exist_ok=True)

RE_ACTION = re.compile(r'"action":"([^"]+)"')
RE_RESULT = re.compile(r'"result":"([^"]+)"')
RE_DICE = re.compile(r'"nr_of_dice":(-?\d+),"dice":\[([^\]]*)\],"selected_index":(\d+)')
RE_SI = re.compile(r'"serious_injury":(?:"([^"]+)"|null)')
RE_SKILL = re.compile(r'"skill_id":(\d+)')
RE_SRC = re.compile(r'"source":\{"name":"([^"]+)"')
RE_ACTED = re.compile(r'"rerolled_action":"([^"]*)"')
RE_PRAYER = re.compile(r'"prayer_id":"([^"]+)"')
RE_IND = re.compile(r'"inducement_id":"([^"]+)"')
RE_INDTYPE = re.compile(r'"inducement_type":"([^"]+)"')
RE_DIST = re.compile(r'"distance":"([^"]+)"')
RE_EFFECT = re.compile(r'"effect":"([^"]+)"')
RE_CARD = re.compile(r'"card_id":"([^"]+)"')
RE_SPELL = re.compile(r'"spell":"([^"]+)"')
RE_NOTE = re.compile(r'"note":"([^"]+)"')
RE_WEATHER = re.compile(r'"weather":"([^"]+)"')
RE_PID = re.compile(r'"player_id":"([^"]+)"')

# success/rerolled-bearing roll events: tally s/f + reroll
SUCCESS_KEY = {
    "dodgeRoll": "success", "goForItRoll": "success", "catchRoll": "success",
    "pickupRoll": "success", "interceptionRoll": "success", "jumpRoll": "success",
    "jumpUpRoll": "success", "dauntlessRoll": "success", "lonerRoll": "success",
    "proRoll": "success", "alwaysHungry": "success", "bloodLustRoll": "success",
    "animosityRoll": "success", "hypnoticGazeRoll": "success", "escapeRoll": "success",
    "rightStuffRoll": "success", "safeThrowRoll": "success", "standUpRoll": "success",
    "pickMeUpRoll": "success", "projectileVomitRoll": "success",
    "balefulHexRoll": "success", "lookIntoMyEyesRoll": "success",
    "argueTheCall": "success", "bribesRoll": "success", "animalSavageryRoll": "success",
    "animalSavagery": "success", "regenerationRoll": "success",
    "chainsawRoll": "success", "allYouCanEatRoll": "success",
    "thenIStartedBlastin": "success", "kegThrow": "success",
    "throwAtPlayer": "successful", "specialEffectRoll": "success",
    "throwAtStallingPlayer": "success", "confusionRoll": "confused",
    "foulAppearanceRoll": "failed", "breatheFireRoll": "knock_down",
    "trapDoor": "escaped", "biasedRefRoll": "referee_spots_foul",
    "refereeSpotsFoul": "referee_spots_foul", "spellEffectRoll": None,
}

BLOCK_RESULT = {1: "Skull", 2: "BothDown", 5: "PowPushback", 6: "Pow"}


# Which sweep's log roots to census. The 2026-09-10 sweep used parity_sw_*; the re-draft sweep
# of the same day used parity_rd_*. Set CENSUS_PREFIX to pick, so a census can be re-run against
# either without editing this file.
PREFIX = os.environ.get("CENSUS_PREFIX", "parity_sw_")


def gate_dirs():
    for root in sorted(REPO.glob(PREFIX + "*")):
        name = root.name[len(PREFIX):]
        # trailing _<edition>_<scale>
        parts = name.rsplit("_", 2)
        if len(parts) != 3:
            continue
        race, edition, scale = parts
        d = root / edition / ("%s_vs_%s" % (race, race))
        if d.is_dir():
            yield race, edition, scale, d
        else:
            cand = list(root.glob("*/*_vs_*"))
            if cand:
                yield race, edition, scale, cand[0]


def do_gate(job):
    race, edition, scale, d = job
    tag = "%s__%s__%s" % (race, edition, scale)
    dst = OUT / (tag + ".json")
    if dst.exists():
        return tag + " (cached)"

    types = Counter()
    actions = Counter()
    kickoff = Counter()
    block_dice = Counter()
    block_result = Counter()
    block_own_choice = Counter()
    block_rerolled = 0
    rolls = {}          # event -> [total, success, rerolled]
    inj = Counter()
    si = Counter()
    skill_used = Counter()
    skill_declined = Counter()
    reroll_src = Counter()
    reroll_action = Counter()
    prayers = Counter()
    inducements = Counter()
    ind_types = Counter()
    pass_dist = Counter()
    pass_res = Counter()
    effects = Counter()
    cards = Counter()
    spells = Counter()
    notes = Counter()
    push_len = Counter()
    apo = Counter()
    weather = Counter()
    by_player = Counter()

    games = 0
    for f in sorted(d.glob("seed_*_rust_events.jsonl")):
        games += 1
        with open(f, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if not line.startswith('{"type":"'):
                    continue
                e = line.index('"', 9)
                t = line[9:e]
                types[t] += 1
                rest = line[e:]
                if t == "playerAction":
                    m = RE_ACTION.search(rest)
                    if m:
                        actions[m.group(1)] += 1
                    m = RE_PID.search(rest)
                    if m:
                        by_player[m.group(1)] += 1
                elif t == "weatherChange":
                    m = RE_WEATHER.search(rest)
                    if m:
                        weather[m.group(1)] += 1
                elif t == "kickoffResultEvent":
                    m = RE_RESULT.search(rest)
                    if m:
                        kickoff[m.group(1)] += 1
                elif t == "blockRoll":
                    m = RE_DICE.search(rest)
                    if m:
                        nd = int(m.group(1))
                        dice = [int(x) for x in m.group(2).split(",") if x.strip()]
                        idx = int(m.group(3))
                        block_dice[nd] += 1
                        if idx < len(dice):
                            block_result[BLOCK_RESULT.get(dice[idx], "Pushback")] += 1
                    block_own_choice['"own_choice":true' in rest] += 1
                    if '"rerolled":true' in rest:
                        block_rerolled += 1
                elif t == "injury":
                    inj["total"] += 1
                    ko = '"was_ko":true' in rest
                    cas = '"was_cas":true' in rest
                    if ko:
                        inj["ko"] += 1
                    if cas:
                        inj["cas"] += 1
                    m = RE_SI.search(rest)
                    if m and m.group(1):
                        si[m.group(1)] += 1
                        if m.group(1) == "dead" or m.group(1) == "Dead":
                            inj["dead"] += 1
                    armor = '"armor_roll":[' in rest
                    injr = '"injury_roll":[' in rest
                    if armor and not injr and not ko and not cas:
                        inj["armor_only"] += 1
                    if armor:
                        inj["armor_rolled"] += 1
                    if injr:
                        inj["injury_rolled"] += 1
                elif t == "skillUse":
                    m = RE_SKILL.search(rest)
                    if m:
                        sid = int(m.group(1))
                        if '"used":true' in rest:
                            skill_used[sid] += 1
                        else:
                            skill_declined[sid] += 1
                elif t == "reRoll":
                    m = RE_SRC.search(rest)
                    if m:
                        reroll_src[m.group(1)] += 1
                    m = RE_ACTED.search(rest)
                    if m:
                        reroll_action[m.group(1)] += 1
                elif t == "prayerRoll":
                    m = RE_PRAYER.search(rest)
                    if m:
                        prayers[m.group(1)] += 1
                elif t == "buyInducement":
                    m = RE_IND.search(rest)
                    if m:
                        inducements[m.group(1)] += 1
                elif t == "inducement":
                    m = RE_INDTYPE.search(rest)
                    if m:
                        ind_types[m.group(1)] += 1
                elif t == "passRoll":
                    m = RE_DIST.search(rest)
                    if m:
                        pass_dist[m.group(1)] += 1
                    m = RE_RESULT.search(rest)
                    if m:
                        pass_res[m.group(1)] += 1
                    r = rolls.setdefault(t, [0, 0, 0])
                    r[0] += 1
                    if '"rerolled":true' in rest:
                        r[2] += 1
                elif t == "pushback":
                    n = rest.count('{"x":')
                    push_len[min(n, 3)] += 1
                elif t == "apothecaryChoice":
                    apo['"healed":true' in rest] += 1
                elif t in ("cardEffectRoll", "specialEffectRoll"):
                    m = RE_EFFECT.search(rest)
                    if m:
                        effects[m.group(1)] += 1
                elif t in ("playCard", "cardDeactivated"):
                    m = RE_CARD.search(rest)
                    if m:
                        cards[m.group(1)] += 1
                elif t == "wizardUse":
                    m = RE_SPELL.search(rest)
                    if m:
                        spells[m.group(1)] += 1
                elif t == "playerNote":
                    m = RE_NOTE.search(rest)
                    if m:
                        notes[m.group(1)] += 1

                if t in SUCCESS_KEY:
                    key = SUCCESS_KEY[t]
                    r = rolls.setdefault(t, [0, 0, 0])
                    r[0] += 1
                    if key and ('"%s":true' % key) in rest:
                        r[1] += 1
                    if '"rerolled":true' in rest:
                        r[2] += 1

    data = dict(
        race=race, edition=edition, scale=scale, games=games,
        types=dict(types), actions=dict(actions), kickoff=dict(kickoff),
        block_dice={str(k): v for k, v in block_dice.items()},
        block_result=dict(block_result),
        block_own_choice={str(k): v for k, v in block_own_choice.items()},
        block_rerolled=block_rerolled,
        rolls={k: {"total": v[0], "success": v[1], "rerolled": v[2]} for k, v in rolls.items()},
        injuries=dict(inj), serious_injury=dict(si),
        skill_used={str(k): v for k, v in skill_used.items()},
        skill_declined={str(k): v for k, v in skill_declined.items()},
        reroll_src=dict(reroll_src), reroll_action=dict(reroll_action),
        prayers=dict(prayers), inducements=dict(inducements), ind_types=dict(ind_types),
        pass_dist=dict(pass_dist), pass_res=dict(pass_res),
        effects=dict(effects), cards=dict(cards), spells=dict(spells),
        notes=dict(notes),
        push_len={str(k): v for k, v in push_len.items()},
        apothecary={str(k): v for k, v in apo.items()},
        weather=dict(weather), by_player=dict(by_player),
    )
    dst.write_text(json.dumps(data), encoding="utf-8")
    return "%s games=%d events=%d" % (tag, games, sum(types.values()))


if __name__ == "__main__":
    jobs = list(gate_dirs())
    print("gates:", len(jobs), flush=True)
    with Pool(int(os.environ.get("NPROC", "8"))) as p:
        for i, msg in enumerate(p.imap_unordered(do_gate, jobs), 1):
            print("[%3d/%d] %s" % (i, len(jobs), msg), flush=True)
