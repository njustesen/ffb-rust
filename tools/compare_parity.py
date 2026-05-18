#!/usr/bin/env python3
"""
Compare two parity JSONL logs (one from Java, one from Rust).

Usage:
    python compare_parity.py rust.jsonl java.jsonl
    python compare_parity.py rust.jsonl java.jsonl --verbose

Each file is produced by running the respective parity_runner binary:
    # Rust
    cargo run --bin parity_runner -- parity human orc 42 rust.jsonl
    # Java
    mvn -pl ffb-ai exec:java -Dexec.mainClass=...ParityRunner -Dexec.args="teamHumanKalimar teamHumanBattleLore 42 java.jsonl"

Exit code 0 = full match, 1 = divergence or error.
"""

import json
import sys


def load_jsonl(path):
    lines = []
    with open(path, "r", encoding="utf-8") as f:
        for lineno, raw in enumerate(f, 1):
            raw = raw.strip()
            if not raw:
                continue
            try:
                lines.append(json.loads(raw))
            except json.JSONDecodeError as e:
                print(f"ERROR: {path}:{lineno}: JSON parse error: {e}", file=sys.stderr)
                sys.exit(1)
    return lines


def compare(a_path, b_path, verbose=False, check_actions=False):
    a_lines = load_jsonl(a_path)
    b_lines = load_jsonl(b_path)

    def lines_of_type(lines, t):
        return [l for l in lines if l.get("type") == t]

    # Summarize headers
    a_start = lines_of_type(a_lines, "game_start")
    b_start = lines_of_type(b_lines, "game_start")
    a_end   = lines_of_type(a_lines, "game_end")
    b_end   = lines_of_type(b_lines, "game_end")
    a_steps = lines_of_type(a_lines, "step")
    b_steps = lines_of_type(b_lines, "step")

    print(f"  {a_path}: {len(a_steps)} steps")
    print(f"  {b_path}: {len(b_steps)} steps")

    # Check game_start seed match
    if a_start and b_start:
        if a_start[0].get("seed") != b_start[0].get("seed"):
            print(f"WARNING: seeds differ: {a_start[0].get('seed')} vs {b_start[0].get('seed')}")

    # Check game_end scores
    if a_end and b_end:
        ah = a_end[0].get("home_score"); aa = a_end[0].get("away_score")
        bh = b_end[0].get("home_score"); ba = b_end[0].get("away_score")
        score_match = (ah == bh and aa == ba)
        print(f"  Score A: {ah}-{aa}  Score B: {bh}-{ba}  {'MATCH' if score_match else 'DIVERGE'}")

    if not a_steps and not b_steps:
        print("MATCH: 0 steps (empty games)")
        return True

    n = min(len(a_steps), len(b_steps))
    total_dice = 0
    first_divergence = None

    for idx in range(n):
        sa = a_steps[idx]
        sb = b_steps[idx]
        total_dice += len(sa.get("dice", []))

        mismatches = []

        # Compare state_hash (will diverge in V1 due to player ID differences)
        if sa.get("state_hash") != sb.get("state_hash"):
            mismatches.append(("state_hash", sa.get("state_hash"), sb.get("state_hash")))

        # Compare actions (sorted, order-independent) — opt-in only
        if check_actions:
            aa_acts = sorted(sa.get("actions", []))
            bb_acts = sorted(sb.get("actions", []))
            if aa_acts != bb_acts:
                mismatches.append(("actions", aa_acts, bb_acts))

        # Compare chosen action
        if sa.get("chosen") != sb.get("chosen"):
            mismatches.append(("chosen", sa.get("chosen"), sb.get("chosen")))

        # Compare dice rolls
        da = sa.get("dice") or []
        db = sb.get("dice") or []
        if da != db:
            mismatches.append(("dice", da, db))

        # Compare half / turn / active (informational only in verbose mode)
        for field in ("half", "turn", "active"):
            if sa.get(field) != sb.get(field) and verbose:
                mismatches.append((field, sa.get(field), sb.get(field)))

        if mismatches and first_divergence is None:
            first_divergence = (idx, mismatches, sa, sb)

        if verbose and mismatches:
            print(f"\n  Step {idx} (i={sa.get('i')}/{sb.get('i')}) DIVERGE:")
            for field, va, vb in mismatches:
                print(f"    {field}:")
                print(f"      A: {va}")
                print(f"      B: {vb}")

    if len(a_steps) != len(b_steps):
        print(f"\nSTEP COUNT MISMATCH: {len(a_steps)} vs {len(b_steps)}")
        if first_divergence is None:
            first_divergence = (n, [("step_count", len(a_steps), len(b_steps))], None, None)

    if first_divergence is not None:
        idx, mismatches, sa, sb = first_divergence
        print(f"\nFIRST DIVERGENCE at step {idx}:")
        for field, va, vb in mismatches:
            print(f"  {field}:")
            print(f"    A ({a_path}): {va}")
            print(f"    B ({b_path}): {vb}")
        if sa:
            print(f"  A context: half={sa.get('half')} turn={sa.get('turn')} active={sa.get('active')}")
        if sb:
            print(f"  B context: half={sb.get('half')} turn={sb.get('turn')} active={sb.get('active')}")
        return False

    matched = min(len(a_steps), len(b_steps))
    print(f"\nMATCH: {matched} steps, {total_dice} dice rolls compared")
    return True


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    check_actions = "--check-actions" in sys.argv
    pos_args = [a for a in sys.argv[1:] if not a.startswith("-")]

    if len(pos_args) < 2:
        print("Usage: compare_parity.py <file_a.jsonl> <file_b.jsonl> [--verbose] [--check-actions]")
        sys.exit(1)

    ok = compare(pos_args[0], pos_args[1], verbose=verbose, check_actions=check_actions)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
