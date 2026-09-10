"""Diff the Java and Rust state strings at the first diverging step of a parity trace.

Both engines print their per-step state into an FFB_TRACE log: Java as `JSTEP i=<n> ... state=<s>`
and Rust as `RUST_STEP i=<n> ... state=<s>`. Java's block is emitted up front (batched JVM) and
Rust's during the comparison loop, so the two are far apart in the file and the Java block holds
one game per seed in order.

This pulls out both sides' state at a given step of a given game and reports which FIELD differs
-- the header (half/turn/active/score/ball/weather/re-rolls) or an individual player slot. Player
slots are POSITIONAL indices into the nr-sorted list, not ids: `h05` is the sixth home player.

Usage:
  python scripts/statediff_step.py <trace.log> --step 59 [--game -1]

`--game` selects which game in the Java block (default -1 = the last, which is the one the run
aborted on when the runner stops at the first failing seed).
"""

import argparse
import re
import sys


def java_states(lines, game_index):
    """The `i -> state` map for one game in the Java trace block."""
    games, cur = [], {}
    for l in lines:
        if not l.startswith("JSTEP i="):
            continue
        m = re.match(r"JSTEP i=(\d+) .*? state=(.*)$", l)
        if not m:
            continue
        i, state = int(m.group(1)), m.group(2)
        if i == 1 and cur:
            games.append(cur)
            cur = {}
        cur[i] = state
    if cur:
        games.append(cur)
    if not games:
        return {}, 0
    return games[game_index], len(games)


def rust_states(lines):
    out = {}
    for l in lines:
        if not l.startswith("RUST_STEP i="):
            continue
        m = re.match(r"RUST_STEP i=(\d+) .*? state=(.*)$", l)
        if m:
            out[int(m.group(1))] = m.group(2)
    return out


def split(state):
    """(header, {slot: value}) for one state string."""
    if " pa" not in state:
        return state, {}
    head, rest = state.split(" pa", 1)
    slots = {}
    for part in ("pa" + rest).split("|"):
        if ":" in part:
            k, v = part.split(":", 1)
            slots[k] = v
    return head, slots


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log")
    ap.add_argument("--step", type=int, required=True)
    ap.add_argument("--game", type=int, default=-1)
    args = ap.parse_args()

    lines = open(args.log, encoding="utf-8", errors="replace").read().split("\n")
    j, n_games = java_states(lines, args.game)
    r = rust_states(lines)
    print(f"java games in trace: {n_games}; java steps {min(j, default=0)}..{max(j, default=0)}; "
          f"rust steps {min(r, default=0)}..{max(r, default=0)}")

    js, rs = j.get(args.step), r.get(args.step)
    if js is None or rs is None:
        print(f"MISSING: java={js is not None} rust={rs is not None} at step {args.step}")
        return 1

    jh, jslots = split(js)
    rh, rslots = split(rs)
    if jh != rh:
        print(f"HEADER differs:\n  java: {jh}\n  rust: {rh}")
    else:
        print(f"header identical: {jh}")
    diffs = 0
    for k in sorted(set(jslots) | set(rslots)):
        a, b = jslots.get(k), rslots.get(k)
        if a != b:
            diffs += 1
            print(f"  {k}: java={a}   rust={b}")
    if not diffs and jh == rh:
        print("states identical at this step")
    else:
        print(f"{diffs} player slot(s) differ")
    return 0


if __name__ == "__main__":
    sys.exit(main())
