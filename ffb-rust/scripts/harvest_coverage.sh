#!/bin/sh
# harvest_coverage.sh <edition> [scale]
#
# MATCHUP=<race> (default amazon) selects the mirror matchup; the output file carries it.
#
# ONE clean heuristic amazon-vs-amazon run (seeds 1-100), then copy the tier-3 checklist the run
# wrote to docs/EVENT_COVERAGE_<edition>.md and append the full GameEvent catalog + per-skill
# tallies from the run's own event logs (docs/COVERAGE_REPORT.md procedure).
#
# Run it ALONE: T3_COVERAGE.md is rewritten by every ffb-parity invocation -- the random control
# and the lineman regression included -- so a checklist read while anything else is running is
# somebody else's. Run editions one after another, never together.
cd /c/Users/Admin/niels/ffb-rust/ffb-rust
E=$1; SC=${2:-1.0}
M=${MATCHUP:-amazon}
OUT=docs/EVENT_COVERAGE_$E.md
[ "$M" != "amazon" ] && OUT=docs/EVENT_COVERAGE_${M}_$E.md
./target/release/ffb-parity --home $M --away $M --edition $E --tier 3 \
    --seeds 1-100 --no-abort --agent heuristic --heur-scale $SC --heur-classes all > /tmp/cov_$E.log 2>&1
PAR=$(grep -E "^PARITY:" /tmp/cov_$E.log)
{
  echo "# Event coverage — HeuristicAgent, $M v $M, $E, --heur-scale $SC, seeds 1-100"
  echo
  echo "Harvested $(date +%Y-%m-%d) by \`MATCHUP=$M scripts/harvest_coverage.sh $E $SC\`. Parity for the run: \`$PAR\`."
  echo
  echo "## Tier-3 checklist (as written by the run)"
  echo
  sed -n '3,200p' T3_COVERAGE.md
  echo
  echo "## GameEvent catalog (from parity/$E/${M}_vs_${M}/seed_*_rust_events.jsonl)"
  echo
  cat parity/$E/${M}_vs_${M}/seed_*_rust_events.jsonl | tr -cd '[:print:]\n' > /tmp/allev_$E.txt
  echo "Total events: $(wc -l < /tmp/allev_$E.txt)"
  echo
  echo '```'
  grep -oE '"type":"[^"]+"' /tmp/allev_$E.txt | sed 's/"type":"//;s/"//' | sort | uniq -c | sort -rn
  echo '```'
  echo
  echo "## Player actions declared"
  echo
  echo '```'
  grep '"type":"playerAction"' /tmp/allev_$E.txt | grep -oE '"action":"[^"]+"' | sed 's/"action":"//;s/"//' | sort | uniq -c | sort -rn
  echo '```'
  echo
  echo "## Skill uses / re-rolls seen"
  echo
  echo '```'
  # skill_id serialises as a NUMBER ("skill_id":127); the old grep here required a
  # quoted value and so matched nothing for EVERY race, including ones with hundreds
  # of skillUse events. Resolve ids to names instead (found closing khemri 2026-09-05).
  python scripts/skill_use_report.py /tmp/allev_$E.txt
  echo '```'
} > $OUT
echo "wrote $OUT ($PAR)"
