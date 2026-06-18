#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
"""
Comprehensive T3 parity report: all events, actions, dice rolls and outcomes.

Usage:
    python scripts/t3_report.py [--seeds 1-100] [--passing-only]

Reads:
  parity/lineman_vs_lineman/seed_N_rust.jsonl       (activations + scores)
  parity/lineman_vs_lineman/seed_N_rust_events.jsonl (game events)
"""

import json, os
from collections import Counter, defaultdict

PARITY_DIR = os.path.join(os.path.dirname(__file__), "..", "parity", "lineman_vs_lineman")

FAILING_SEEDS: set = set()  # T3 100/100 as of 2026-06-18


def parse_args():
    seeds, passing_only = range(1, 101), False
    i = 1
    while i < len(sys.argv):
        arg = sys.argv[i]
        if arg == "--seeds" and i + 1 < len(sys.argv):
            i += 1
            lo, hi = (int(x) for x in sys.argv[i].split("-"))
            seeds = range(lo, hi + 1)
        elif arg == "--passing-only":
            passing_only = True
        i += 1
    return seeds, passing_only


def load_jsonl(path):
    if not os.path.exists(path):
        return []
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


# ---- Block die outcome from raw roll value -----------------------------------
def block_outcome(roll):
    if roll == 1:   return "Skull"
    if roll == 2:   return "BothDown"
    if roll in (3, 4): return "Pushback"
    if roll == 5:   return "PowPushback"
    if roll == 6:   return "Pow"
    return "?"


def header(title, width=62):
    print()
    print("=" * width)
    print(f"  {title}")
    print("=" * width)


def subheader(title):
    print(f"\n-- {title} " + "-" * max(0, 55 - len(title)))


def bar(label, n, total, width=20):
    pct = n / total if total else 0
    filled = round(pct * width)
    b = "#" * filled + "." * (width - filled)
    return f"  {label:<18} [{b}] {n:>5}  {pct:5.1%}"


def dice_hist(rolls, die_size=6):
    c = Counter(rolls)
    total = sum(c.values())
    faces = range(1, die_size + 1)
    lines = []
    for face in faces:
        n = c.get(face, 0)
        pct = n / total if total else 0
        bar_w = round(pct * 25)
        b = "#" * bar_w + "." * (25 - bar_w)
        lines.append(f"    {face}: [{b}] {n:>5}  {pct:5.1%}")
    expected = 1 / die_size
    lines.append(f"    (expected each: {expected:.1%})")
    return "\n".join(lines)


def two_d6_hist(pairs):
    """pairs is a list of [d1, d2] or (d1, d2) rolls."""
    sums = Counter(a + b for a, b in pairs)
    total = sum(sums.values())
    lines = []
    for s in range(2, 13):
        n = sums.get(s, 0)
        pct = n / total if total else 0
        bar_w = round(pct * 25)
        b = "#" * bar_w + "." * (25 - bar_w)
        lines.append(f"    {s:>2}: [{b}] {n:>5}  {pct:5.1%}")
    return "\n".join(lines)


# ---- Analyse one seed --------------------------------------------------------

def analyse(seed):
    steps = load_jsonl(os.path.join(PARITY_DIR, f"seed_{seed}_rust.jsonl"))
    events = load_jsonl(os.path.join(PARITY_DIR, f"seed_{seed}_rust_events.jsonl"))

    g = {
        "seed": seed,
        "home_score": 0, "away_score": 0,
        "total_steps": 0,
        "seen_turns_home": set(), "seen_turns_away": set(),
        "actions": Counter(), "actions_home": Counter(), "actions_away": Counter(),
        # Kickoff events
        "kickoff_results": Counter(),
        "weather_changes": [],
        # Block
        "block_dice": [],        # (nr_of_dice, own_choice) tuples
        "block_die_rolls": [],   # raw d6 values of selected die
        "block_outcomes": Counter(),
        "block_outcomes_by_nd": defaultdict(Counter),  # nd -> outcome -> count
        # Dodge
        "dodge_rolls": [],       # (roll, target, success)
        # Pickup
        "pickup_rolls": [],      # (roll, target, success)
        # Catch
        "catch_rolls": [],       # (roll, target, success)
        # Pass
        "pass_rolls": [],        # (roll, target, distance, result)
        # Injury (all)
        "injuries": [],          # (armor_roll, injury_roll, was_ko, was_cas, from_foul)
        # Foul
        "foul_count": 0,
        "next_is_foul": False,
        # Turns
        "turnovers": 0,
    }

    prev_active = None
    for line in steps:
        t = line.get("type")
        if t == "game_start":
            g["seed"] = line.get("seed")
        elif t == "step":
            g["total_steps"] += 1
            active = line.get("active", "")
            chosen = line.get("chosen", "")
            half = line.get("half", 1)
            turn = line.get("turn", 1)
            if chosen.startswith("Activate(") and chosen.endswith(")"):
                inner = chosen[len("Activate("):-1]
                parts = inner.split(",", 1)
                action = parts[1] if len(parts) == 2 else "Unknown"
            else:
                action = chosen
            g["actions"][action] += 1
            if active == "home":
                g["actions_home"][action] += 1
                g["seen_turns_home"].add((half, turn))
            else:
                g["actions_away"][action] += 1
                g["seen_turns_away"].add((half, turn))
            # Detect turnovers: active team switches mid-turn (turn/half same, side flips)
            if prev_active and prev_active["active"] == active:
                pass  # same team still acting
            prev_active = {"active": active, "half": half, "turn": turn}
        elif t == "game_end":
            g["home_score"] = line.get("home_score", 0)
            g["away_score"] = line.get("away_score", 0)

    g["turns_home"] = len(g["seen_turns_home"])
    g["turns_away"] = len(g["seen_turns_away"])

    # Process events
    expect_foul_injury = False
    for ev in events:
        etype = ev.get("type", "")

        if etype == "kickoffResultEvent":
            g["kickoff_results"][ev.get("result", "?")] += 1
        elif etype == "weatherChange":
            g["weather_changes"].append(ev.get("weather", "?"))

        elif etype == "foul":
            g["foul_count"] += 1
            expect_foul_injury = True

        elif etype == "blockRoll":
            nd = abs(ev.get("nr_of_dice", 1))
            own = ev.get("own_choice", True)
            dice = ev.get("dice", [])
            sel = ev.get("selected_index", 0)
            g["block_dice"].append((nd, own))
            if dice and sel < len(dice):
                roll = dice[sel]
                g["block_die_rolls"].append(roll)
                outcome = block_outcome(roll)
                g["block_outcomes"][outcome] += 1
                g["block_outcomes_by_nd"][nd][outcome] += 1

        elif etype == "dodgeRoll":
            g["dodge_rolls"].append((ev.get("roll", 0), ev.get("target", 0), ev.get("success", False)))

        elif etype == "pickupRoll":
            g["pickup_rolls"].append((ev.get("roll", 0), ev.get("target", 0), ev.get("success", False)))

        elif etype == "catchRoll":
            g["catch_rolls"].append((ev.get("roll", 0), ev.get("target", 0), ev.get("success", False)))

        elif etype == "passRoll":
            g["pass_rolls"].append((
                ev.get("roll", 0), ev.get("target", 0),
                ev.get("distance", "?"), ev.get("result", "?")
            ))

        elif etype == "injury":
            ar = ev.get("armor_roll")
            ir = ev.get("injury_roll")
            was_ko = ev.get("was_ko", False)
            was_cas = ev.get("was_cas", False)
            si = ev.get("serious_injury")  # None | str e.g. "SmashedKneeMa" | "Dead"
            g["injuries"].append((ar, ir, was_ko, was_cas, expect_foul_injury, si))
            expect_foul_injury = False

    return g


# ---- Aggregate ---------------------------------------------------------------

def aggregate(games):
    a = {
        "n": len(games), "n_pass": sum(1 for g in games if g.get("passing")),
        "scores": Counter(),
        "actions": Counter(), "actions_home": Counter(), "actions_away": Counter(),
        "total_steps": 0,
        "turns_home": [], "turns_away": [],
        "kickoff_results": Counter(),
        "weather": Counter(),
        # Block
        "block_dice": [],
        "block_die_rolls": [],
        "block_outcomes": Counter(),
        "block_outcomes_by_nd": defaultdict(Counter),
        # Rolls
        "dodge_rolls": [], "pickup_rolls": [], "catch_rolls": [], "pass_rolls": [],
        # Injury
        "injuries": [], "foul_count": 0,
    }
    for g in games:
        a["scores"][(g["home_score"], g["away_score"])] += 1
        a["actions"].update(g["actions"])
        a["actions_home"].update(g["actions_home"])
        a["actions_away"].update(g["actions_away"])
        a["total_steps"] += g["total_steps"]
        a["turns_home"].append(g["turns_home"])
        a["turns_away"].append(g["turns_away"])
        a["kickoff_results"].update(g["kickoff_results"])
        a["weather"].update(g["weather_changes"])
        a["block_dice"].extend(g["block_dice"])
        a["block_die_rolls"].extend(g["block_die_rolls"])
        a["block_outcomes"].update(g["block_outcomes"])
        for nd, oc in g["block_outcomes_by_nd"].items():
            a["block_outcomes_by_nd"][nd].update(oc)
        a["dodge_rolls"].extend(g["dodge_rolls"])
        a["pickup_rolls"].extend(g["pickup_rolls"])
        a["catch_rolls"].extend(g["catch_rolls"])
        a["pass_rolls"].extend(g["pass_rolls"])
        a["injuries"].extend(g["injuries"])
        a["foul_count"] += g["foul_count"]
    return a


# ---- Render ------------------------------------------------------------------

def render(a, ref):
    n = len(ref)
    if n == 0:
        print("No games."); return

    def avg(x): return x / n

    header(f"T3 lineman_vs_lineman  --  {a['n']} seeds  ({a['n_pass']} passing)")
    print(f"  Reference set: {n} passing games")

    # ---- Scores
    subheader("SCORES")
    for (h, v), c in sorted(a["scores"].items(), key=lambda x: -x[1]):
        print(f"  {h}-{v}  x{c}")

    # ---- Actions
    subheader("ACTIONS")
    total_acts = sum(a["actions"].values())
    print(f"  {'Action':<18} {'Total':>7} {'Avg/g':>7} {'Home':>7} {'Away':>7} {'%':>6}")
    print(f"  {'-'*55}")
    for act, c in sorted(a["actions"].items(), key=lambda x: -x[1]):
        ch = a["actions_home"].get(act, 0)
        ca = a["actions_away"].get(act, 0)
        print(f"  {act:<18} {c:>7} {c/n:>7.1f} {ch:>7} {ca:>7} {100*c/total_acts:>5.1f}%")
    print(f"  {'-'*55}")
    print(f"  {'TOTAL':<18} {total_acts:>7} {total_acts/n:>7.1f}")

    # ---- Events summary
    subheader("EVENTS SUMMARY")
    ev_counts = Counter()
    for g in ref:
        ev_counts["blockRoll"]  += len(g["block_dice"])
        ev_counts["dodgeRoll"]  += len(g["dodge_rolls"])
        ev_counts["pickupRoll"] += len(g["pickup_rolls"])
        ev_counts["catchRoll"]  += len(g["catch_rolls"])
        ev_counts["passRoll"]   += len(g["pass_rolls"])
        ev_counts["injury"]     += len(g["injuries"])
        ev_counts["foul"]       += g["foul_count"]
    # Add kickoff/weather from aggregate
    ev_counts["kickoffResult"] = sum(a["kickoff_results"].values())
    ev_counts["weatherChange"] = sum(a["weather"].values())
    ev_counts["coinThrow"] = n
    ev_counts["receiveChoice"] = n
    ev_counts["startHalf"] = n * 2
    ev_counts["kickoffScatter"] = n * 2
    total_evs = sum(ev_counts.values())
    print(f"  {'Event':<22} {'Total':>7} {'Avg/g':>7}")
    print(f"  {'-'*38}")
    for etype, c in sorted(ev_counts.items(), key=lambda x: -x[1]):
        print(f"  {etype:<22} {c:>7} {c/n:>7.1f}")
    print(f"  {'-'*38}")
    print(f"  {'TOTAL':<22} {total_evs:>7} {total_evs/n:>7.1f}")

    # ---- Block rolls
    subheader("BLOCK ROLLS")
    blk = a["block_dice"]
    total_blk = len(blk)
    n1  = sum(1 for (nd,_) in blk if nd == 1)
    n2o = sum(1 for (nd, own) in blk if nd == 2 and own)
    n2d = sum(1 for (nd, own) in blk if nd == 2 and not own)
    n3  = sum(1 for (nd,_) in blk if nd >= 3)
    print(f"  Total blocks: {total_blk}  ({avg(total_blk):.1f}/game)")
    print()
    print(f"  Dice count breakdown:")
    for label, cnt in [("1 die", n1), ("2 dice (attacker picks)", n2o),
                       ("2 dice (defender picks)", n2d), ("3+ dice", n3)]:
        pct = cnt/total_blk if total_blk else 0
        print(f"    {label:<28} {cnt:>5}  {pct:5.1%}")
    print()
    print(f"  Block die outcomes (all blocks):")
    outcome_order = ["Skull", "BothDown", "Pushback", "PowPushback", "Pow"]
    for o in outcome_order:
        c = a["block_outcomes"].get(o, 0)
        pct = c/total_blk if total_blk else 0
        print(bar(o, c, total_blk))
    print()
    print(f"  Block die outcomes by dice count:")
    for nd in sorted(a["block_outcomes_by_nd"].keys()):
        nd_total = sum(a["block_outcomes_by_nd"][nd].values())
        print(f"    {nd}-die blocks (n={nd_total}):")
        for o in outcome_order:
            c = a["block_outcomes_by_nd"][nd].get(o, 0)
            pct = c/nd_total if nd_total else 0
            print(f"      {o:<16} {c:>5}  {pct:5.1%}")
    print()
    print(f"  Block die value distribution (d6, selected die):")
    print(dice_hist(a["block_die_rolls"]))

    # ---- Dodge rolls
    subheader("DODGE ROLLS")
    dg = a["dodge_rolls"]
    ds = sum(1 for r,t,s in dg if s)
    df = sum(1 for r,t,s in dg if not s)
    dt = len(dg)
    print(f"  Total: {dt}  ({avg(dt):.1f}/game)   Success: {ds} ({100*ds/dt:.0f}%)   Fail: {df} ({100*df/dt:.0f}%)" if dt else "  Total: 0")
    if dt:
        print()
        print(f"  Target number distribution:")
        tgt_c = Counter(t for r,t,s in dg)
        for tgt in sorted(tgt_c):
            rolls_at_tgt = [(r,s) for r,t,s in dg if t == tgt]
            succ = sum(1 for r,s in rolls_at_tgt if s)
            total_at = len(rolls_at_tgt)
            print(f"    {tgt}+  {total_at:>5} rolls   {succ} success ({100*succ/total_at:.0f}%)")
        print()
        print(f"  Die value distribution (d6):")
        print(dice_hist([r for r,t,s in dg]))
        print()
        print(f"  Success by roll value:")
        by_roll = defaultdict(lambda: [0,0])  # roll -> [success, total]
        for r,t,s in dg:
            by_roll[r][1] += 1
            if s: by_roll[r][0] += 1
        for roll in sorted(by_roll):
            succ, tot = by_roll[roll]
            print(f"    rolled {roll}:  {succ}/{tot} success  ({100*succ/tot:.0f}%)")

    # ---- Pickup rolls
    subheader("PICKUP ROLLS")
    pu = a["pickup_rolls"]
    ps = sum(1 for r,t,s in pu if s)
    pf = sum(1 for r,t,s in pu if not s)
    pt = len(pu)
    print(f"  Total: {pt}  ({avg(pt):.1f}/game)   Success: {ps} ({100*ps/pt:.0f}%)   Fail: {pf} ({100*pf/pt:.0f}%)" if pt else "  Total: 0")
    if pt:
        print()
        print(f"  Target number distribution:")
        tgt_c = Counter(t for r,t,s in pu)
        for tgt in sorted(tgt_c):
            rolls_at_tgt = [(r,s) for r,t,s in pu if t == tgt]
            succ = sum(1 for r,s in rolls_at_tgt if s)
            total_at = len(rolls_at_tgt)
            print(f"    {tgt}+  {total_at:>5} rolls   {succ} success ({100*succ/total_at:.0f}%)")
        print()
        print(f"  Die value distribution (d6):")
        print(dice_hist([r for r,t,s in pu]))

    # ---- Catch rolls
    subheader("CATCH ROLLS")
    cr = a["catch_rolls"]
    cs_ = sum(1 for r,t,s in cr if s)
    cf_ = sum(1 for r,t,s in cr if not s)
    ct = len(cr)
    print(f"  Total: {ct}  ({avg(ct):.1f}/game)   Success: {cs_} ({100*cs_/ct:.0f}%)   Fail: {cf_} ({100*cf_/ct:.0f}%)" if ct else "  Total: 0")
    if ct:
        print()
        print(f"  Target number distribution:")
        tgt_c = Counter(t for r,t,s in cr)
        for tgt in sorted(tgt_c):
            rolls_at_tgt = [(r,s) for r,t,s in cr if t == tgt]
            succ = sum(1 for r,s in rolls_at_tgt if s)
            total_at = len(rolls_at_tgt)
            print(f"    {tgt}+  {total_at:>5} rolls   {succ} success ({100*succ/total_at:.0f}%)")
        print()
        print(f"  Die value distribution (d6):")
        print(dice_hist([r for r,t,s in cr]))

    # ---- Pass rolls
    subheader("PASS ROLLS")
    pr = a["pass_rolls"]
    pt2 = len(pr)
    if pt2 == 0:
        print("  Total: 0")
    else:
        p_acc = sum(1 for r,t,d,res in pr if res == "Complete")
        p_inac = sum(1 for r,t,d,res in pr if res == "Inaccurate")
        p_fum = sum(1 for r,t,d,res in pr if res == "Fumble")
        print(f"  Total: {pt2}  ({avg(pt2):.1f}/game)")
        print(f"  Accurate:    {p_acc:>4}  ({100*p_acc/pt2:.0f}%)")
        print(f"  Inaccurate:  {p_inac:>4}  ({100*p_inac/pt2:.0f}%)")
        print(f"  Fumble:      {p_fum:>4}  ({100*p_fum/pt2:.0f}%)")
        print()
        print(f"  Distance breakdown:")
        dist_c = Counter(d for r,t,d,res in pr)
        for dist, c in sorted(dist_c.items(), key=lambda x: -x[1]):
            dist_rolls = [(r,t,res) for r,t2,d,res in pr if d == dist]
            acc = sum(1 for r,t,res in dist_rolls if res == "Complete")
            inac = sum(1 for r,t,res in dist_rolls if res == "Inaccurate")
            fum = sum(1 for r,t,res in dist_rolls if res == "Fumble")
            print(f"    {dist:<20} n={c:>3}  acc={acc} inac={inac} fum={fum}")
        print()
        print(f"  Die value distribution (d6):")
        print(dice_hist([r for r,t,d,res in pr if r > 0]))

    # ---- Injury chain
    subheader("INJURY CHAIN")
    inj = a["injuries"]
    it = len(inj)
    if it == 0:
        print("  Total: 0")
    else:
        armor_held  = sum(1 for ar,ir,ko,cas,foul,si in inj if ir is None)
        stunned_    = sum(1 for ar,ir,ko,cas,foul,si in inj if ir and not ko and not cas)
        ko_         = sum(1 for ar,ir,ko,cas,foul,si in inj if ko)
        casualty_   = sum(1 for ar,ir,ko,cas,foul,si in inj if cas)
        from_foul   = sum(1 for ar,ir,ko,cas,foul,si in inj if foul)
        from_block  = it - from_foul
        print(f"  Total knockdowns: {it}  ({avg(it):.1f}/game)")
        print(f"  Cause:  from block/dodge: {from_block}   from foul: {from_foul}")
        print()
        print(f"  Armor break result:")
        print(bar("Armor held",  armor_held,  it))
        print(bar("Stunned",     stunned_,    it))
        print(bar("KO",          ko_,         it))
        print(bar("Casualty",    casualty_,   it))
        print()
        # Casualty tier breakdown (d16 result)
        if casualty_ > 0:
            badly_hurt = sum(1 for ar,ir,ko,cas,foul,si in inj if cas and si is None)
            dead_      = sum(1 for ar,ir,ko,cas,foul,si in inj if cas and si == "Dead")
            serious    = casualty_ - badly_hurt - dead_
            si_counts  = Counter(si for ar,ir,ko,cas,foul,si in inj if cas and si and si != "Dead")
            print(f"  Casualty breakdown (d16):")
            print(f"    Badly Hurt (1-8):  {badly_hurt:>4}  ({100*badly_hurt/casualty_:.0f}%)  — miss next game")
            print(f"    Serious Injury (9-14): {serious:>4}  ({100*serious/casualty_:.0f}%)")
            for kind, cnt in sorted(si_counts.items(), key=lambda x: -x[1]):
                stat_loss = {
                    "SmashedKneeMa": "-MA", "HeadInjuryAv": "-AV", "BrokenArmPa": "-PA",
                    "NeckInjuryAg": "-AG", "DislocatedHipAg": "-AG", "DislocatedShoulderSt": "-ST",
                }.get(kind, "")
                print(f"      {kind:<26} {cnt:>3}  {stat_loss}")
            print(f"    Dead / RIP (15-16):    {dead_:>4}  ({100*dead_/casualty_:.0f}%)")
            print()
        armor_pairs = [ar for ar,ir,ko,cas,foul,si in inj if ar is not None]
        if armor_pairs:
            print(f"  Armor roll distribution (2d6 sum, n={len(armor_pairs)}):")
            print(two_d6_hist(armor_pairs))
        injury_pairs = [ir for ar,ir,ko,cas,foul,si in inj if ir is not None]
        if injury_pairs:
            print()
            print(f"  Injury roll distribution (2d6 sum, when armor broken, n={len(injury_pairs)}):")
            print(two_d6_hist(injury_pairs))

    # ---- Fouls
    subheader("FOULS")
    fc = a["foul_count"]
    print(f"  Total fouls: {fc}  ({avg(fc):.1f}/game)")
    if fc:
        foul_inj = sum(1 for ar,ir,ko,cas,foul,si in a["injuries"] if foul)
        foul_broke = sum(1 for ar,ir,ko,cas,foul,si in a["injuries"] if foul and ir is not None)
        foul_ko = sum(1 for ar,ir,ko,cas,foul,si in a["injuries"] if foul and ko)
        foul_cas = sum(1 for ar,ir,ko,cas,foul,si in a["injuries"] if foul and cas)
        foul_eject = fc - foul_inj  # fouls with no injury event (shouldn't happen) -- approx
        print(f"  Foul outcomes (armor):")
        print(f"    Armor held: {fc - foul_broke:>5}  ({100*(fc-foul_broke)/fc:.0f}%)")
        print(f"    Armor broken: {foul_broke:>5}  ({100*foul_broke/fc:.0f}%)")
        print(f"      -> Stunned:   {foul_broke - foul_ko - foul_cas:>5}")
        print(f"      -> KO:        {foul_ko:>5}")
        print(f"      -> Casualty:  {foul_cas:>5}")

    # ---- Kickoff events
    subheader("KICKOFF EVENTS")
    print(f"  Total kickoffs: {sum(a['kickoff_results'].values())}  ({avg(sum(a['kickoff_results'].values())):.1f}/game)")
    for res, c in sorted(a["kickoff_results"].items(), key=lambda x: -x[1]):
        print(f"    {res:<28} {c:>5}  ({100*c/sum(a['kickoff_results'].values()):.1f}%)")

    # ---- Weather
    subheader("WEATHER")
    all_weather = a["weather"]
    total_w = sum(all_weather.values())
    for w, c in sorted(all_weather.items(), key=lambda x: -x[1]):
        print(f"  {w:<20} {c:>5} occurrences")

    # ---- Game length
    subheader("GAME LENGTH")
    steps_list = [g["total_steps"] for g in ref]
    th = a["turns_home"]
    ta = a["turns_away"]
    print(f"  Activations: min={min(steps_list)}  max={max(steps_list)}  avg={sum(steps_list)/n:.1f}")
    print(f"  Home turns:  min={min(th)}  max={max(th)}  avg={sum(th)/n:.1f}")
    print(f"  Away turns:  min={min(ta)}  max={max(ta)}  avg={sum(ta)/n:.1f}")

    # ---- Not tracked
    subheader("NOT TRACKED / NOT IMPLEMENTED")
    missing = [
        "GoForIt (GFI) rolls     -- Step::GoForIt declared but has no handler in engine",
        "StandUp rolls           -- none needed (BB2025: stand up costs movement, no d6)",
        "Touchdown event         -- never happens in these 0-0 games",
        "TurnEnd event           -- not emitted",
        "Pushback destinations   -- auto_push() not instrumented",
        "KickoffPitchInvasion    -- handler exists, event not emitted",
        "KickoffRiot             -- handler exists, event not emitted",
        "KickoffThrowARock       -- handler exists, event not emitted",
        "Ball scatter directions -- d8 values not recorded",
        "Argue-the-call roll     -- d6 roll in apply_foul_injury not emitted",
    ]
    for m in missing:
        print(f"  - {m}")
    print()


# ---- Main --------------------------------------------------------------------

def main():
    seeds, passing_only = parse_args()
    all_games = []
    for seed in seeds:
        if passing_only and seed in FAILING_SEEDS:
            continue
        steps = load_jsonl(os.path.join(PARITY_DIR, f"seed_{seed}_rust.jsonl"))
        if not steps:
            continue
        events = load_jsonl(os.path.join(PARITY_DIR, f"seed_{seed}_rust_events.jsonl"))
        g = analyse(seed)
        g["passing"] = seed not in FAILING_SEEDS
        all_games.append(g)

    if not all_games:
        print("No games found."); return

    passing = [g for g in all_games if g["passing"]]
    ref = passing if passing else all_games
    agg = aggregate(ref)
    agg["n"] = len(all_games)
    agg["n_pass"] = len(passing)
    render(agg, ref)


if __name__ == "__main__":
    main()
