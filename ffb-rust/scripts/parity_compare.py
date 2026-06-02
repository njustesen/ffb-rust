#!/usr/bin/env python3
"""
parity_compare.py — compare two JSONL parity log files line-by-line.

Usage:
    python scripts/parity_compare.py <java.jsonl> <rust.jsonl>
    python scripts/parity_compare.py parity/seed_1_java.jsonl parity/seed_1_rust.jsonl

Exit 0 on match, exit 1 on divergence.
"""

import json
import sys
from pathlib import Path


def load_steps(path: Path) -> tuple[dict | None, list[dict], dict | None]:
    """Load a JSONL log: returns (game_start, steps, game_end)."""
    game_start = None
    steps = []
    game_end = None
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            entry = json.loads(line)
            t = entry.get("type")
            if t == "game_start":
                game_start = entry
            elif t == "step":
                steps.append(entry)
            elif t == "game_end":
                game_end = entry
    return game_start, steps, game_end


def compare(java_path: Path, rust_path: Path, verbose: bool = True) -> bool:
    """Return True if logs match, False if they diverge."""
    if not java_path.exists():
        print(f"MISSING: {java_path}")
        return False
    if not rust_path.exists():
        print(f"MISSING: {rust_path}")
        return False

    j_start, j_steps, j_end = load_steps(java_path)
    r_start, r_steps, r_end = load_steps(rust_path)

    # Fast path: compare final game_end hashes
    j_final = j_end.get("state_hash") if j_end else None
    r_final = r_end.get("state_hash") if r_end else None

    if j_final and r_final and j_final == r_final:
        if verbose:
            print(f"MATCH  final_hash={j_final}  steps={len(j_steps)}")
        return True

    # Slow path: find first divergent step
    if verbose:
        print(f"DIVERGE  java_final={j_final}  rust_final={r_final}")
        print(f"  java_steps={len(j_steps)}  rust_steps={len(r_steps)}")

    compare_fields = ["state_hash", "chosen", "turn", "half", "active"]
    min_steps = min(len(j_steps), len(r_steps))

    first_divergence = None
    for i in range(min_steps):
        js = j_steps[i]
        rs = r_steps[i]
        for field in compare_fields:
            if js.get(field) != rs.get(field):
                first_divergence = i
                if verbose:
                    print(f"  First divergence at step index {i} (turn={js.get('turn')} half={js.get('half')} active={js.get('active')}):")
                    print(f"    Java: {field}={js.get(field)!r}")
                    print(f"    Rust: {field}={rs.get(field)!r}")
                    print(f"    Java step: {js}")
                    print(f"    Rust step: {rs}")
                break
        if first_divergence is not None:
            break

    if first_divergence is None and len(j_steps) != len(r_steps):
        if verbose:
            print(f"  Step count mismatch: java={len(j_steps)} rust={len(r_steps)}")

    return False


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2

    java_path = Path(sys.argv[1])
    rust_path = Path(sys.argv[2])
    matched = compare(java_path, rust_path, verbose=True)
    return 0 if matched else 1


if __name__ == "__main__":
    sys.exit(main())
