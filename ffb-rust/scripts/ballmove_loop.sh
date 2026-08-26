#!/bin/sh
# One iteration of the ball-move improvement loop.
#
#   sh scripts/ballmove_loop.sh <tag> [seeds]
#
# Builds, sanity-checks that games still complete, then A/Bs the agent against itself with passing
# and hand-offs switched off (`--mode wide-noball`), both colours, and prints the result in standard
# errors on decisive games.
#
# Why head-to-head and not touchdowns per game: self-play scoring rate measures the PAIR, not the
# policy (docs §23.3). Why 1600 seeds: two readings at 800 games reversed sign at 3200 (§27.2), so
# anything smaller cannot see the effect being chased.
#
# The sanity check exists because a broken ball-move path parks the driver with no prompt and the
# game simply stops — 267 events instead of ~1500 — which an A/B would report as a huge loss without
# saying why.
set -e
TAG=${1:-loop}
SEEDS=${2:-1600}
ROOT=/c/Users/Admin/niels/ffb-rust/ffb-rust
# Two spellings of the SAME directory, deliberately. The Rust exe is a native Windows binary and
# MSYS rewrites an argument that looks like a POSIX path on its way in, so `$D` works there; the
# Python that reads the dumps back is also native and never sees that rewrite, so it needs the
# drive-letter form. Handing Python the POSIX spelling makes every glob come back empty, which the
# sanity check then reports as "0 events/game -- the driver is parking" when the games were fine.
D=/c/Users/Admin/AppData/Local/Temp/claude
DW=C:/Users/Admin/AppData/Local/Temp/claude

cd "$ROOT"
echo "== build"
cargo build --release -p ffb-parity --message-format short 2>&1 | grep -E '^error' && exit 1

echo "== sanity: do games still finish?"
cd "$ROOT/crates/ffb-parity"
./../../target/release/ffb-parity.exe --heuristic 0 --mode wide --seeds 1-20 \
  --home human --away human --edition bb2025 --out "$D/${TAG}_sane" >/dev/null 2>&1
EV=$(python -c "
import glob,os
fs=glob.glob(os.path.join(r'$DW/${TAG}_sane','seed_*_argmax_events.jsonl'))
print(int(sum(sum(1 for _ in open(f,errors='ignore')) for f in fs)/max(len(fs),1)))
")
echo "   mean events/game: $EV  (a healthy game is ~1400+; under ~600 means the driver is parking)"
if [ "$EV" -lt 600 ]; then
  echo "   ABORT: games are not completing. Fix that before reading any A/B number."
  exit 2
fi

echo "== A/B over $((SEEDS * 2)) games"
./../../target/release/ffb-parity.exe --heuristic 0 --mode wide --mode-away wide-noball \
  --seeds 1-$SEEDS --home human --away human --edition bb2025 --out "$D/${TAG}_bh" >/dev/null 2>&1
./../../target/release/ffb-parity.exe --heuristic 0 --mode wide-noball --mode-away wide \
  --seeds 1-$SEEDS --home human --away human --edition bb2025 --out "$D/${TAG}_nh" >/dev/null 2>&1
cd "$ROOT"
python scripts/ab_ballmoves.py "${TAG}_bh" "${TAG}_nh" "$TAG"
echo
echo "Keep the change only if the SE is positive and larger than about +2."
