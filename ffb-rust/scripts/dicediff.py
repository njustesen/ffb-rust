"""Diff the Java and Rust dice streams in an FFB_DICE_TRACE=1 capture.

Both engines print `DICE_TRACE pos=N sides=S result=R`; only the Java lines carry a `caller=`
field, which is what separates the two streams in a combined stderr capture. Compares by
(sides, result) in order and reports the first index where they disagree — positions are NOT
comparable directly (Java logs the count before the roll, Rust after).

Usage: python scripts/dicediff.py <capture-file> [context]
"""
import re
import sys

PAT = re.compile(r"^DICE_TRACE pos=(\d+) sides=(\d+) result=(\d+)")


def main() -> int:
    path = sys.argv[1]
    context = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    java, rust = [], []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = PAT.match(line)
            if not m:
                continue
            entry = (int(m.group(1)), int(m.group(2)), int(m.group(3)))
            (java if "caller=" in line else rust).append(entry)

    print(f"java dice: {len(java)}   rust dice: {len(rust)}")
    for i, (j, r) in enumerate(zip(java, rust)):
        if j[1:] != r[1:]:
            print(f"FIRST DIFF at index {i}: java d{j[1]}={j[2]} (pos {j[0]})  "
                  f"rust d{r[1]}={r[2]} (pos {r[0]})")
            lo, hi = max(0, i - context), i + context
            for k in range(lo, min(hi, len(java), len(rust))):
                mark = "  <--" if k == i else ""
                print(f"  [{k:3}] java d{java[k][1]}={java[k][2]}   "
                      f"rust d{rust[k][1]}={rust[k][2]}{mark}")
            return 0
    print("no value/sides difference in the common prefix")
    return 0


if __name__ == "__main__":
    sys.exit(main())
