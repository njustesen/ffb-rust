#!/usr/bin/env python3
"""
parity_run.py — orchestrate parity runs across Java and Rust engines.

Usage:
    python scripts/parity_run.py [options]

Options:
    --tier T1a|T1b|T2|T3|T3i     Which tier to run (default: T1a)
    --seeds N                     Run seeds 1..N (default: 100)
    --seeds START-END             Run seeds START..END
    --home RACE                   Home team race (default: lineman)
    --away RACE                   Away team race (default: lineman)
    --edition EDITION             bb2016|bb2020|bb2025|random (default: bb2025)
    --parallel N                  Number of seeds to run concurrently (default: 1)
    --no-abort                    Continue after first failure
    --rust-bin PATH               Path to the compiled ffb-parity binary
                                  (default: auto-detect from cargo build)
    --results-dir DIR             Where to write per-seed results (default: parity/results)

Examples:
    python scripts/parity_run.py --tier T1a --seeds 100
    python scripts/parity_run.py --home human --away orc --seeds 10
    python scripts/parity_run.py --tier T2 --seeds 10 --parallel 4
"""

import argparse
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
PARITY_DIR = REPO_ROOT / "parity"
DATA_DIR = REPO_ROOT / "data" / "rosters"

RACE_NAMES = [
    "amazon", "chaos", "chaos_dwarf", "chaos_pact", "dark_elf", "dwarf",
    "elf", "goblin", "halfling", "high_elf", "human", "khemri", "lizardman",
    "necromantic", "norse", "nurgle", "ogre", "orc", "renegades", "skaven",
    "slann", "undead", "underworld", "vampire", "wood_elf",
]

JAVA_TEAM_IDS = {
    "lineman":    "teamLinemanParityHome",  # special: away uses teamLinemanParityAway
    "human":      "teamHumanKalimar",
    "orc":        "teamOrcBattleLore",
    "dark_elf":   "teamDarkElfKalimar",
    "dwarf":      "teamDwarfKalimar",
    "skaven":     "teamSkavenKalimar",
    "chaos":      "teamChaosKalimar",
    "wood_elf":   "teamWoodElfKalimar",
    "high_elf":   "teamHighElfKalimar",
    "lizardman":  "teamLizardmanKalimar",
    "nurgle":     "teamNurgleKalimar",
    "undead":     "teamUndeadKalimar",
    "necromantic": "teamNecromanticKalimar",
    "norse":      "teamNorseKalimar",
    "halfling":   "teamHalflingKalimar",
    "ogre":       "teamOgreKalimar",
    "goblin":     "teamGoblinKalimar",
}

EDITIONS = ["bb2016", "bb2020", "bb2025"]


def java_team_id(race: str, side: str = "home") -> str:
    """Return the Java server team ID for a race name."""
    race_lower = race.lower()
    if race_lower == "lineman":
        return "teamLinemanParityAway" if side == "away" else "teamLinemanParityHome"
    return JAVA_TEAM_IDS.get(race_lower, race)


def find_rust_binary() -> Path:
    """Find the compiled ffb-parity binary."""
    candidates = [
        REPO_ROOT / "target" / "release" / "ffb-parity.exe",
        REPO_ROOT / "target" / "release" / "ffb-parity",
        REPO_ROOT / "target" / "debug" / "ffb-parity.exe",
        REPO_ROOT / "target" / "debug" / "ffb-parity",
    ]
    for c in candidates:
        if c.exists():
            return c
    return None


def find_java_cp() -> str | None:
    """Find the Java fat JAR classpath."""
    candidates = [
        r"C:\Users\Admin\niels\ffb\ffb\ffb-ai\target\ffb-ai-jar-with-dependencies.jar",
        str(REPO_ROOT.parent.parent / "ffb-java" / "ffb" / "ffb-ai" / "target" / "ffb-ai-jar-with-dependencies.jar"),
    ]
    env_cp = os.environ.get("PARITY_CP")
    if env_cp:
        return env_cp
    for c in candidates:
        if Path(c).exists():
            return c
    return None


def find_ffb_server_dir() -> str | None:
    """Find the ffb-server directory."""
    candidates = [
        r"C:\Users\Admin\niels\ffb\ffb\ffb-server",
        str(REPO_ROOT.parent.parent / "ffb-java" / "ffb" / "ffb-server"),
    ]
    env_dir = os.environ.get("FFB_SERVER_DIR")
    if env_dir:
        return env_dir
    for c in candidates:
        if Path(c).exists():
            return c
    return None


def run_java_seed(seed: int, home_java: str, away_java: str, output_path: Path,
                  java_cp: str, server_dir: str) -> bool:
    """Run Java parity for one seed. Returns True on success."""
    cmd = [
        "java", "-cp", java_cp,
        "com.fumbbl.ffb.ai.parity.ParityRunner",
        server_dir, home_java, away_java,
        str(seed), str(output_path),
    ]
    try:
        r = subprocess.run(cmd, capture_output=True, timeout=120)
        if r.returncode != 0:
            print(f"  Java seed={seed} exited {r.returncode}", file=sys.stderr)
        return output_path.exists()
    except subprocess.TimeoutExpired:
        print(f"  Java seed={seed} timed out", file=sys.stderr)
        return False
    except FileNotFoundError:
        print("  Java not found — skipping Java run", file=sys.stderr)
        return False


def run_rust_seed(seed: int, home: str, away: str, edition: str,
                  rust_bin: Path | None, no_abort: bool) -> bool:
    """Run Rust parity for one seed. Returns True on success."""
    if rust_bin and rust_bin.exists():
        cmd = [str(rust_bin), "--home", home, "--away", away,
               "--edition", edition, "--seeds", str(seed), "--no-abort"]
    else:
        cmd = [
            "cargo", "run", "--release", "-p", "ffb-parity", "--",
            "--home", home, "--away", away,
            "--edition", edition, "--seeds", str(seed), "--no-abort",
        ]
    try:
        r = subprocess.run(cmd, capture_output=True, timeout=180, cwd=str(REPO_ROOT))
        return r.returncode == 0
    except subprocess.TimeoutExpired:
        print(f"  Rust seed={seed} timed out", file=sys.stderr)
        return False


def run_seed(seed: int, home: str, away: str, edition: str,
             java_cp: str | None, server_dir: str | None, rust_bin: Path | None,
             results_dir: Path) -> dict:
    """Run one seed through both engines and compare. Returns a result dict."""
    import importlib.util, sys as _sys
    spec = importlib.util.spec_from_file_location("parity_compare", SCRIPT_DIR / "parity_compare.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)

    java_out = PARITY_DIR / f"seed_{seed}_java.jsonl"
    rust_out = PARITY_DIR / f"seed_{seed}_rust.jsonl"

    java_ok = False
    rust_ok = False

    if java_cp and server_dir:
        home_java = java_team_id(home, "home")
        away_java = java_team_id(away, "away")
        java_ok = run_java_seed(seed, home_java, away_java, java_out, java_cp, server_dir)
    else:
        java_ok = java_out.exists()

    run_rust_seed(seed, home, away, edition, rust_bin, no_abort=True)
    rust_ok = rust_out.exists()

    matched = False
    divergence_step = None
    if java_ok and rust_ok:
        matched = mod.compare(java_out, rust_out, verbose=False)
        if not matched:
            # re-run with verbose to get details
            mod.compare(java_out, rust_out, verbose=True)

    result = {
        "seed": seed,
        "home": home,
        "away": away,
        "edition": edition,
        "matched": matched,
        "java_ok": java_ok,
        "rust_ok": rust_ok,
    }

    results_dir.mkdir(parents=True, exist_ok=True)
    with open(results_dir / f"seed_{seed}.json", "w") as f:
        json.dump(result, f)

    status = "✓" if matched else "✗"
    print(f"  {status} seed={seed}  {home} vs {away} ({edition})")
    return result


def parse_args():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--tier", default="T1a",
                    choices=["T1a", "T1b", "T2", "T3", "T3i"],
                    help="Preset tier (overrides --home/--away if set)")
    ap.add_argument("--home", default=None)
    ap.add_argument("--away", default=None)
    ap.add_argument("--edition", default=None)
    ap.add_argument("--seeds", default="100")
    ap.add_argument("--parallel", type=int, default=1)
    ap.add_argument("--no-abort", action="store_true")
    ap.add_argument("--rust-bin", default=None)
    ap.add_argument("--results-dir", default=None)
    return ap.parse_args()


TIER_DEFAULTS = {
    "T1a": ("lineman", "lineman", "bb2025"),
    "T1b": ("human",   "orc",     "bb2020"),
    "T2":  (None,      None,      "random"),
    "T3":  (None,      None,      "random"),
    "T3i": (None,      None,      "random"),
}


def main():
    args = parse_args()

    # Seed range
    seeds_str = args.seeds
    if "-" in seeds_str:
        start, end = seeds_str.split("-", 1)
        seed_start, seed_end = int(start), int(end)
    else:
        seed_start, seed_end = 1, int(seeds_str)

    # Team defaults from tier
    tier_home, tier_away, tier_edition = TIER_DEFAULTS.get(args.tier, ("lineman", "lineman", "bb2025"))
    home = args.home or tier_home or "lineman"
    away = args.away or tier_away or "lineman"
    edition = args.edition or tier_edition or "bb2025"

    results_dir = Path(args.results_dir) if args.results_dir else REPO_ROOT / "parity" / "results" / args.tier

    rust_bin = Path(args.rust_bin) if args.rust_bin else find_rust_binary()
    java_cp = find_java_cp()
    server_dir = find_ffb_server_dir()

    PARITY_DIR.mkdir(exist_ok=True)

    if not java_cp:
        print("WARNING: Java classpath not found. Java logs will use pre-existing files if present.", file=sys.stderr)
    if not server_dir:
        print("WARNING: ffb-server directory not found.", file=sys.stderr)

    seeds = list(range(seed_start, seed_end + 1))
    total = len(seeds)
    passed = 0
    failed = 0

    print(f"Running {total} seeds: {home} vs {away} ({edition}) | tier={args.tier}")

    import random as _rnd
    _rnd.seed(42)  # deterministic random race selection for T2/T3

    if args.parallel > 1:
        with ThreadPoolExecutor(max_workers=args.parallel) as ex:
            futures = {}
            for seed in seeds:
                run_home = home
                run_away = away
                run_edition = edition
                if edition == "random":
                    run_edition = _rnd.choice(EDITIONS)
                if run_home is None:
                    run_home = _rnd.choice(RACE_NAMES)
                if run_away is None:
                    run_away = _rnd.choice(RACE_NAMES)
                f = ex.submit(run_seed, seed, run_home, run_away, run_edition,
                              java_cp, server_dir, rust_bin, results_dir)
                futures[f] = seed
            for f in as_completed(futures):
                result = f.result()
                if result["matched"]:
                    passed += 1
                else:
                    failed += 1
                    if not args.no_abort:
                        print(f"ABORT: seed={futures[f]} failed. Use --no-abort to continue.")
                        sys.exit(1)
    else:
        for seed in seeds:
            run_home = home
            run_away = away
            run_edition = edition
            if edition == "random":
                run_edition = _rnd.choice(EDITIONS)
            if run_home is None:
                run_home = _rnd.choice(RACE_NAMES)
            if run_away is None:
                run_away = _rnd.choice(RACE_NAMES)
            result = run_seed(seed, run_home, run_away, run_edition,
                              java_cp, server_dir, rust_bin, results_dir)
            if result["matched"]:
                passed += 1
            else:
                failed += 1
                if not args.no_abort:
                    print(f"ABORT at seed={seed}. Use --no-abort to continue.")
                    sys.exit(1)

    print(f"\nSummary: {passed}/{total} passed, {failed} failed")

    # Write summary
    summary = {
        "tier": args.tier, "home": home, "away": away, "edition": edition,
        "seed_start": seed_start, "seed_end": seed_end,
        "total": total, "passed": passed, "failed": failed,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
    }
    results_dir.mkdir(parents=True, exist_ok=True)
    with open(results_dir / "summary.json", "w") as f:
        json.dump(summary, f, indent=2)
    print(f"Results written to {results_dir}/")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
