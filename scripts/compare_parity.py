#!/usr/bin/env python3
"""Compare parity output JSON files from Rust and Java engines."""
import json, sys, os

def compare(rust_file, java_file):
    rust_games = [json.loads(l) for l in open(rust_file)]
    java_games = [json.loads(l) for l in open(java_file)]

    if len(rust_games) != len(java_games):
        print(f"MISMATCH: rust={len(rust_games)} games, java={len(java_games)} games")
        sys.exit(1)

    mismatches = 0
    for r, j in zip(rust_games, java_games):
        if r['seed'] != j['seed']:
            print(f"Seed mismatch: {r['seed']} vs {j['seed']}")
            mismatches += 1
            continue
        if r['score_home'] != j['score_home'] or r['score_away'] != j['score_away']:
            print(f"Score mismatch seed={r['seed']}: rust={r['score_home']}-{r['score_away']} java={j['score_home']}-{j['score_away']}")
            mismatches += 1

    if mismatches == 0:
        print(f"✓ All {len(rust_games)} games match")
    else:
        print(f"✗ {mismatches}/{len(rust_games)} games differ")
        sys.exit(1)

if __name__ == '__main__':
    compare(sys.argv[1], sys.argv[2])
