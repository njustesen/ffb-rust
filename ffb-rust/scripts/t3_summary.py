#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
"""
Summarise T3 parity JSONL log files.

Usage:
    python scripts/t3_summary.py [--seeds 1-100] [--passing-only]

Reads parity/lineman_vs_lineman/seed_N_rust.jsonl and
       parity/lineman_vs_lineman/seed_N_rust_events.jsonl for each seed.
Reports only what is actually present in the log files -- no estimates.
"""

import json
import os
from collections import Counter, defaultdict

# ---- Config ------------------------------------------------------------------

PARITY_DIR = os.path.join(os.path.dirname(__file__), "..", "parity", "lineman_vs_lineman")

# Seeds that currently fail parity (as of 2026-06-17)
FAILING_SEEDS: set = set()  # T3 100/100 as of 2026-06-19


def parse_args():
    seeds = range(1, 101)
    passing_only = False
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


def load_seed(seed):
    """Return list of parsed JSON lines, or None if file missing."""
    path = os.path.join(PARITY_DIR, f"seed_{seed}_rust.jsonl")
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def load_events(seed):
    """Return list of parsed event JSON lines, or [] if file missing."""
    path = os.path.join(PARITY_DIR, f"seed_{seed}_rust_events.jsonl")
    if not os.path.exists(path):
        return []
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


# ---- Per-game extraction -----------------------------------------------------

def analyse_game(lines, events):
    """Extract everything the log records for one game."""
    g = {
        "seed": None,
        "home_score": 0,
        "away_score": 0,
        "total_steps": 0,
        "turns_home": 0,
        "turns_away": 0,
        "actions": Counter(),
        "actions_home": Counter(),
        "actions_away": Counter(),
        # Events
        "event_types": Counter(),
        # Block breakdown
        "block_1d": 0, "block_2d_own": 0, "block_2d_opp": 0, "block_3d": 0,
        "block_skull": 0, "block_bothdown": 0, "block_pushback": 0,
        "block_powpushback": 0, "block_pow": 0,
        # Dodge
        "dodge_success": 0, "dodge_fail": 0,
        # Pickup
        "pickup_success": 0, "pickup_fail": 0,
        # Catch
        "catch_success": 0, "catch_fail": 0,
        # Pass
        "pass_accurate": 0, "pass_inaccurate": 0, "pass_fumble": 0,
        # Injury
        "armor_held": 0, "stunned": 0, "ko": 0, "casualty": 0,
    }

    seen_turns_home = set()
    seen_turns_away = set()

    for line in lines:
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
                seen_turns_home.add((half, turn))
            else:
                g["actions_away"][action] += 1
                seen_turns_away.add((half, turn))
        elif t == "game_end":
            g["home_score"] = line.get("home_score", 0)
            g["away_score"] = line.get("away_score", 0)

    g["turns_home"] = len(seen_turns_home)
    g["turns_away"] = len(seen_turns_away)
    all_halves = {line.get("half") for line in lines if line.get("type") == "step" and "half" in line}
    g["halves"] = max(all_halves) if all_halves else 0

    # Process events
    BLOCK_RESULTS = ["Skull", "BothDown", "Pushback", "PowPushback", "Pow"]
    for ev in events:
        etype = ev.get("type", "")
        g["event_types"][etype] += 1

        if etype == "blockRoll":
            nd = ev.get("nr_of_dice", 1)
            own = ev.get("own_choice", True)
            if nd == 1:
                g["block_1d"] += 1
            elif nd == 2 and own:
                g["block_2d_own"] += 1
            elif nd == 2 and not own:
                g["block_2d_opp"] += 1
            elif nd >= 3:
                g["block_3d"] += 1
            # Determine outcome from selected die
            dice = ev.get("dice", [])
            sel = ev.get("selected_index", 0)
            if dice and sel < len(dice):
                roll = dice[sel]
                # BB2025 block die: 1=Skull, 2=BothDown, 3-4=Pushback, 5=PowPushback, 6=Pow
                if roll == 1:
                    g["block_skull"] += 1
                elif roll == 2:
                    g["block_bothdown"] += 1
                elif roll in (3, 4):
                    g["block_pushback"] += 1
                elif roll == 5:
                    g["block_powpushback"] += 1
                elif roll == 6:
                    g["block_pow"] += 1

        elif etype == "dodgeRoll":
            if ev.get("success"):
                g["dodge_success"] += 1
            else:
                g["dodge_fail"] += 1

        elif etype == "pickupRoll":
            if ev.get("success"):
                g["pickup_success"] += 1
            else:
                g["pickup_fail"] += 1

        elif etype == "catchRoll":
            if ev.get("success"):
                g["catch_success"] += 1
            else:
                g["catch_fail"] += 1

        elif etype == "passRoll":
            result = ev.get("result", "")
            if result == "Complete":
                g["pass_accurate"] += 1
            elif result == "Inaccurate":
                g["pass_inaccurate"] += 1
            elif result == "Fumble":
                g["pass_fumble"] += 1

        elif etype == "injury":
            armor = ev.get("armor_roll")
            injury = ev.get("injury_roll")
            was_ko = ev.get("was_ko", False)
            was_cas = ev.get("was_cas", False)
            if injury is None:
                g["armor_held"] += 1
            elif was_ko:
                g["ko"] += 1
            elif was_cas:
                g["casualty"] += 1
            else:
                g["stunned"] += 1

    return g


# ---- Aggregation + display ---------------------------------------------------

def fmt(n, width=8):
    return str(n).rjust(width)


def fmtf(f, width=8, decimals=1):
    return f"{f:.{decimals}f}".rjust(width)


def pct(num, denom):
    return f"{100*num/denom:.0f}%" if denom else "n/a"


def main():
    seeds, passing_only = parse_args()

    games = []
    skipped = []
    for seed in seeds:
        if passing_only and seed in FAILING_SEEDS:
            continue
        lines = load_seed(seed)
        if lines is None:
            skipped.append(seed)
            continue
        events = load_events(seed)
        g = analyse_game(lines, events)
        g["passing"] = seed not in FAILING_SEEDS
        games.append(g)

    if not games:
        print("No games found.")
        return

    passing = [g for g in games if g["passing"]]
    n = len(games)
    n_pass = len(passing)

    print(f"\n{'='*60}")
    print(f"  T3 lineman_vs_lineman  --  {n} seeds loaded  ({n_pass} passing)")
    print(f"{'='*60}\n")

    ref = passing if passing else games
    n_ref = len(ref)

    # ---- Scores ---------------------------------------------------------------
    scores = Counter()
    for g in ref:
        scores[(g["home_score"], g["away_score"])] += 1
    print("Scores (passing seeds):")
    for (h, a), c in sorted(scores.items(), key=lambda x: -x[1]):
        print(f"  {h}-{a}  x{c}")
    print()

    # ---- Actions --------------------------------------------------------------
    totals = Counter()
    for g in ref:
        totals.update(g["actions"])

    all_actions = sorted(totals, key=lambda k: -totals[k])
    total_acts = sum(totals.values())

    print(f"{'Action':<18} {'Total':>8} {'Avg/game':>10} {'%':>7}")
    print("-" * 46)
    for act in all_actions:
        c = totals[act]
        print(f"{act:<18} {fmt(c)} {fmtf(c/n_ref):>10} {fmtf(100*c/total_acts, decimals=1):>6}%")
    print("-" * 46)
    print(f"{'TOTAL':<18} {fmt(total_acts)} {fmtf(total_acts/n_ref):>10}")
    print()

    # ---- Events ---------------------------------------------------------------
    ev_totals = Counter()
    for g in ref:
        ev_totals.update(g["event_types"])

    if ev_totals:
        total_evs = sum(ev_totals.values())
        print(f"{'Event':<22} {'Total':>8} {'Avg/game':>10}")
        print("-" * 42)
        for etype, c in sorted(ev_totals.items(), key=lambda x: -x[1]):
            print(f"  {etype:<20} {fmt(c)} {fmtf(c/n_ref):>10}")
        print(f"  {'TOTAL':<20} {fmt(total_evs)} {fmtf(total_evs/n_ref):>10}")
        print()

    # ---- Block detail ---------------------------------------------------------
    def sum_field(field):
        return sum(g[field] for g in ref)

    b1 = sum_field("block_1d")
    b2o = sum_field("block_2d_own")
    b2opp = sum_field("block_2d_opp")
    b3 = sum_field("block_3d")
    total_blocks = b1 + b2o + b2opp + b3
    sk = sum_field("block_skull")
    bd = sum_field("block_bothdown")
    pb = sum_field("block_pushback")
    pp = sum_field("block_powpushback")
    pw = sum_field("block_pow")

    print("Block rolls:")
    print(f"  1 die:              {b1:>5}  ({pct(b1, total_blocks)} of blocks)")
    print(f"  2 dice (own):       {b2o:>5}  ({pct(b2o, total_blocks)} of blocks)")
    print(f"  2 dice (opponent):  {b2opp:>5}  ({pct(b2opp, total_blocks)} of blocks)")
    print(f"  3 dice:             {b3:>5}  ({pct(b3, total_blocks)} of blocks)")
    print(f"  Total blocks:       {total_blocks:>5}  ({fmtf(total_blocks/n_ref).strip()} avg/game)")
    print()
    print("Block outcomes (selected die):")
    for label, val in [("Skull", sk), ("BothDown", bd), ("Pushback", pb), ("PowPushback", pp), ("Pow", pw)]:
        print(f"  {label:<16} {val:>5}  ({pct(val, total_blocks)} of blocks)")
    print()

    # ---- Dodge ----------------------------------------------------------------
    ds = sum_field("dodge_success")
    df = sum_field("dodge_fail")
    dtotal = ds + df
    print("Dodge rolls:")
    print(f"  Success: {ds:>5}  ({pct(ds, dtotal)})")
    print(f"  Fail:    {df:>5}  ({pct(df, dtotal)})")
    print(f"  Total:   {dtotal:>5}  ({fmtf(dtotal/n_ref).strip()} avg/game)")
    print()

    # ---- Pickup ---------------------------------------------------------------
    ps = sum_field("pickup_success")
    pf = sum_field("pickup_fail")
    ptotal = ps + pf
    print("Pickup rolls:")
    print(f"  Success: {ps:>5}  ({pct(ps, ptotal)})")
    print(f"  Fail:    {pf:>5}  ({pct(pf, ptotal)})")
    print(f"  Total:   {ptotal:>5}  ({fmtf(ptotal/n_ref).strip()} avg/game)")
    print()

    # ---- Catch ----------------------------------------------------------------
    cs = sum_field("catch_success")
    cf = sum_field("catch_fail")
    ctotal = cs + cf
    if ctotal > 0:
        print("Catch rolls:")
        print(f"  Success: {cs:>5}  ({pct(cs, ctotal)})")
        print(f"  Fail:    {cf:>5}  ({pct(cf, ctotal)})")
        print(f"  Total:   {ctotal:>5}  ({fmtf(ctotal/n_ref).strip()} avg/game)")
        print()

    # ---- Pass -----------------------------------------------------------------
    pa = sum_field("pass_accurate")
    pi = sum_field("pass_inaccurate")
    pfum = sum_field("pass_fumble")
    ptotal2 = pa + pi + pfum
    if ptotal2 > 0:
        print("Pass rolls:")
        print(f"  Accurate:   {pa:>5}  ({pct(pa, ptotal2)})")
        print(f"  Inaccurate: {pi:>5}  ({pct(pi, ptotal2)})")
        print(f"  Fumble:     {pfum:>5}  ({pct(pfum, ptotal2)})")
        print(f"  Total:      {ptotal2:>5}  ({fmtf(ptotal2/n_ref).strip()} avg/game)")
        print()

    # ---- Injury ---------------------------------------------------------------
    ah = sum_field("armor_held")
    st = sum_field("stunned")
    ko = sum_field("ko")
    cas = sum_field("casualty")
    itotal = ah + st + ko + cas
    print("Injury chain (per knockdown):")
    print(f"  Armor held:  {ah:>5}  ({pct(ah, itotal)})")
    print(f"  Stunned:     {st:>5}  ({pct(st, itotal)})")
    print(f"  KO:          {ko:>5}  ({pct(ko, itotal)})")
    print(f"  Casualty:    {cas:>5}  ({pct(cas, itotal)})")
    print(f"  Total:       {itotal:>5}  ({fmtf(itotal/n_ref).strip()} avg/game)")
    print()

    # ---- Game length ----------------------------------------------------------
    steps = [g["total_steps"] for g in ref]
    turns_h = [g["turns_home"] for g in ref]
    turns_a = [g["turns_away"] for g in ref]
    print(f"Game length (passing seeds, n={n_ref}):")
    print(f"  Activations/game:  min={min(steps)}  max={max(steps)}  avg={sum(steps)/n_ref:.1f}")
    print(f"  Home team-turns:   min={min(turns_h)}  max={max(turns_h)}  avg={sum(turns_h)/n_ref:.1f}")
    print(f"  Away team-turns:   min={min(turns_a)}  max={max(turns_a)}  avg={sum(turns_a)/n_ref:.1f}")
    print()

    if skipped:
        print(f"Skipped (no file): {skipped}")


if __name__ == "__main__":
    main()
