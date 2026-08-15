"""Print the Java and Rust step logs for one parity seed side by side.

Usage: python scripts/stepdiff.py <matchup-dir> <seed> [from] [to]
  e.g. python scripts/stepdiff.py parity/nurgle_vs_nurgle 2 28 34

Prints one line per step for each engine over the requested index range, and marks the first
index where the pre-state hash diverges — which is the step whose RESOLUTION differs, i.e. the
one before it.
"""
import json
import sys


def load(path):
    steps = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            try:
                d = json.loads(line)
            except ValueError:
                continue
            if d.get("type") == "step":
                steps[d["i"]] = d
    return steps


def fmt(d):
    if d is None:
        return "(absent)"
    return (f"t{d['turn']} h{d['half']} {d['active']:5} pre={d['state_hash']} "
            f"post={d['post_hash']} {d['chosen']}")


def main() -> int:
    base, seed = sys.argv[1], sys.argv[2]
    lo = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    hi = int(sys.argv[4]) if len(sys.argv) > 4 else lo + 10
    java = load(f"{base}/seed_{seed}_java.jsonl")
    rust = load(f"{base}/seed_{seed}_rust.jsonl")

    first = None
    for i in sorted(set(java) | set(rust)):
        j, r = java.get(i), rust.get(i)
        if j and r and j["state_hash"] != r["state_hash"] and first is None:
            first = i
    if first is not None:
        print(f"first diverging PRE-state at i={first} "
              f"-> step {first - 1} resolved differently\n")

    for i in range(lo, hi + 1):
        j, r = java.get(i), rust.get(i)
        mark = "  <--" if i == first else ""
        print(f"[{i:4}] JAVA {fmt(j)}{mark}")
        print(f"       RUST {fmt(r)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
