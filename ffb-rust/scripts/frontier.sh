#!/bin/sh
# frontier.sh <edition> [seed ...]
#
# The MIDDLE loop's instrument: run first_state_divergence.sh over every current red of an edition
# and print one table. With no seeds given, the reds are taken from the most recent gate log
# ($FRONTIER_LOG, default: the scratchpad's latest iter*_<edition>_amz.log) by its PARITY FAIL lines.
#
# Read the table as a whole. Every row is "the activation whose resolution diverged"; the columns
# that matter are the declared action, whether the declaration itself differs, and whether it was
# the first activation after a side/turn flip. A family is a set of rows sharing those. Fix the
# family with the most rows, as a unit; never chase one row.
#
# Cost: one heuristic re-run per seed (~30 s), so 16 reds ~ 8 min. Pass --no-rerun as the FIRST seed
# argument to read the jsonl already on disk when you know it is fresh.
cd /c/Users/Admin/niels/ffb-rust/ffb-rust
E=$1; shift
NORERUN=""
if [ "$1" = "--no-rerun" ]; then NORERUN="--no-rerun"; shift; fi
if [ $# -eq 0 ]; then
  S=/c/Users/Admin/AppData/Local/Temp/claude/C--Users-Admin-niels-ffb-rust/09776488-5c36-4b65-a667-eeb487670a51/scratchpad
  LOG=${FRONTIER_LOG:-$(ls -t $S/iter*_${E}_amz.log 2>/dev/null | head -1)}
  [ -z "$LOG" ] && { echo "no gate log for $E; pass seeds explicitly"; exit 1; }
  echo "reds from $LOG"
  set -- $(grep -oE "^PARITY FAIL seed=[0-9]+" "$LOG" | grep -oE "[0-9]+$")
fi
echo "=== $E frontier: $# seeds ==="
for s in "$@"; do
  sh scripts/first_state_divergence.sh $E $s $NORERUN | head -1
done
