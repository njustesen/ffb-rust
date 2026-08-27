#!/usr/bin/env python3
"""Check that the two Java harness trees have not drifted apart.

There are two copies of the Java source:

  * ``C:/Users/Admin/niels/ffb/ffb``       -- the Maven build tree. Its jar is what
    ``ffb-parity`` actually loads, so this is the one that decides what a gate measures.
  * ``<repo>/ffb-java/ffb``                -- the git-TRACKED reference copy, which is what a
    reviewer reads and what survives a fresh clone.

Only the second is under version control, so an edit applied to one and not the other is
invisible to ``git status`` and silently makes the reviewed source differ from the measured
source. That is the worst shape of harness bug: the code you are reading is not the code that
ran.

Run before any gate that follows a ``ParityRunner``/agent edit::

    python scripts/check_java_trees.py

Exits non-zero and names every differing file. ``--fix`` copies build-tree -> tracked tree.
"""

from __future__ import annotations

import argparse
import filecmp
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
# ffb-java/ sits at the git-repo root, one level above the cargo workspace.
TRACKED = REPO.parent / "ffb-java" / "ffb"
BUILD_CANDIDATES = [
    Path("C:/Users/Admin/niels/ffb/ffb"),
    REPO.parent.parent / "ffb" / "ffb",
    REPO.parent.parent.parent / "ffb" / "ffb",
]

# Only the harness is co-editable; ffb-common / ffb-server are stock engine and are deliberately
# not compared (they are large, and we never touch them).
WATCHED = [
    Path("ffb-ai/src/main/java/com/fumbbl/ffb/ai/parity"),
    Path("ffb-ai/src/test/java/com/fumbbl/ffb/ai/parity"),
]


def find_build_tree() -> Path:
    for c in BUILD_CANDIDATES:
        if (c / "ffb-ai").is_dir():
            return c
    sys.exit(f"could not find the Maven build tree; looked in {BUILD_CANDIDATES}")


def java_files(root: Path, rel: Path) -> dict[Path, Path]:
    base = root / rel
    if not base.is_dir():
        return {}
    return {p.relative_to(root): p for p in base.rglob("*.java")}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--fix",
        action="store_true",
        help="copy the build tree over the tracked tree (never the other way: the build tree is "
        "what the jar was compiled from, so it is the one that describes the measurement)",
    )
    args = ap.parse_args()

    build = find_build_tree()
    problems: list[str] = []
    fixed = 0

    for rel in WATCHED:
        in_build = java_files(build, rel)
        in_tracked = java_files(TRACKED, rel)
        for name in sorted(set(in_build) | set(in_tracked)):
            b, t = in_build.get(name), in_tracked.get(name)
            if b is None:
                problems.append(f"  only in TRACKED tree: {name}")
                continue
            if t is None:
                if args.fix:
                    dest = TRACKED / name
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(b, dest)
                    fixed += 1
                else:
                    problems.append(f"  missing from tracked tree: {name}")
                continue
            if not filecmp.cmp(b, t, shallow=False):
                if args.fix:
                    shutil.copy2(b, t)
                    fixed += 1
                else:
                    problems.append(f"  DIFFERS: {name}")

    if args.fix:
        print(f"synced {fixed} file(s) from {build} -> {TRACKED}")
        return 0
    if problems:
        print("Java harness trees have drifted:")
        print("\n".join(problems))
        print("\nThe jar is built from:", build)
        print("The tracked copy is:   ", TRACKED)
        print("Re-run with --fix, then rebuild the jar before gating.")
        return 1
    print(f"Java harness trees agree ({build} == {TRACKED})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
