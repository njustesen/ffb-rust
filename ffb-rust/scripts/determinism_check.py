"""Rust-vs-Rust determinism gate (§Q1.6): the same seeds, two fresh processes, byte-identical logs.

    python -I scripts/determinism_check.py <exe> <work dir> <edition> <home> <away> <seeds> [extra args...]

Runs `<exe> --bench --bench-harness` (Rust only, no JVM) twice, each into its own
FFB_PARITY_ROOT under <work dir>, then compares every `seed_*_rust*.jsonl` file of the two runs.
std's HashMap/HashSet hash with a random per-process seed, so any engine path whose result depends
on map iteration order shows up here as a difference. Prints one line per differing file and ends
with `DETERMINISM: <files> files, <same> identical`. Exit 1 on any difference.
"""
import filecmp
import os
import shutil
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run(exe, root, edition, home, away, seeds, extra):
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(root, exist_ok=True)
    env = dict(os.environ, FFB_PARITY_ROOT=root)
    subprocess.run([exe, "--bench", "--bench-harness", "--home", home, "--away", away,
                    "--edition", edition, "--tier", "3", "--seeds", seeds, "--agent", "heuristic",
                    "--heur-classes", "all"] + extra,
                   cwd=REPO, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def logs(root):
    out = {}
    for d, _, files in os.walk(root):
        for f in files:
            if f.startswith("seed_") and "_rust" in f and f.endswith(".jsonl"):
                out[os.path.relpath(os.path.join(d, f), root)] = os.path.join(d, f)
    return out


def main():
    exe, work, edition, home, away, seeds = sys.argv[1:7]
    extra = sys.argv[7:]
    a, b = os.path.join(work, "run_a"), os.path.join(work, "run_b")
    run(exe, a, edition, home, away, seeds, extra)
    run(exe, b, edition, home, away, seeds, extra)
    la, lb = logs(a), logs(b)
    names = sorted(set(la) | set(lb))
    same = 0
    for n in names:
        if n in la and n in lb and filecmp.cmp(la[n], lb[n], shallow=False):
            same += 1
        else:
            print("DIFFERS", n)
    print(f"DETERMINISM: {len(names)} files, {same} identical ({edition} {home} vs {away} seeds {seeds})")
    sys.exit(0 if names and same == len(names) else 1)


if __name__ == "__main__":
    main()
