#!/usr/bin/env bash
# Post-sweep pipeline: turn a finished 330-gate sweep into the committed artifacts.
#
#   bash scripts/post_sweep.sh <sweep-root> <date-tag> [expected-gates]
#   e.g. bash scripts/post_sweep.sh D:/ffb_sweep3 2026-09-16 330
#
# Respects the six-processor cap: agg.py's pool is pinned to NPROC=6 (its default is 8).
set -euo pipefail

ROOT="${1:?usage: post_sweep.sh <sweep-root> <date-tag> [expected]}"
TAG="${2:?missing date tag, e.g. 2026-09-16}"
EXPECT="${3:-330}"

cd "$(dirname "$0")/.."
SCRATCH="${SCRATCH:-sweepjobs/scratch}"   # override for a sweep run with -scratch <dir>
OUTDIR=scripts/sweep_census/out_$(echo "$TAG" | tr -d '-')

echo "== 1/5 verdict table"
python scripts/sweep_verdicts.py "$SCRATCH/sweep.log" \
  --out "docs/SWEEP_${TAG}_REPORTS.txt" \
  --title "FFB-Rust full parity matrix - ${TAG}" \
  --expect "$EXPECT"

echo "== 2/5 per-gate event aggregation -> $OUTDIR"
CENSUS_ROOT="$ROOT" CENSUS_PREFIX="parity_" NPROC=6 \
  python scripts/sweep_census/agg.py "$OUTDIR"

# The census tail is the ORACLE CHECK (H.61): every reportId and every modifier/skill NAME
# the JAVA twin writes that Rust does not. Stock Java is never modified, so each line there is
# a Rust fidelity bug by definition -- no judgement call. Tee it to a file, because it is the
# actionable half of the whole census and scrollback loses it.
echo "== 3/5 report-stream census -> docs/report_census.json (+ oracle check)"
CENSUS_ROOT="$ROOT" \
  python scripts/sweep_census/report_census.py "parity_*" docs/report_census.json \
  | tee "docs/oracle_check_${TAG}.txt"
if grep -q "JAVA REPORTS IT, RUST DOES NOT" "docs/oracle_check_${TAG}.txt"; then
  echo
  echo "   ^^ oracle check found Rust naming gaps - see docs/oracle_check_${TAG}.txt"
fi

echo "== 4/5 coverage gaps -> docs/COVERAGE_GAPS_${TAG}.md inputs"
python scripts/sweep_census/gaps.py "$OUTDIR" docs/report_census.json \
  | tee "docs/coverage_gaps_${TAG}.txt"

# Instrument guard (BACKLOG §J): steps can stay 100% green while the REPORT stream regresses,
# because report_compare is advisory. Every sweep therefore records how many games have a fully
# identical report stream, per edition, plus the ranked divergence families - compare the
# IDENTICAL GAMES block against the previous sweep's docs/REPORT_PARITY_<tag>.txt.
echo "== 5/5 report-stream parity -> docs/REPORT_PARITY_${TAG}.txt"
python scripts/report_first_diff.py "$ROOT" --nproc 6   | tee "docs/REPORT_PARITY_${TAG}.txt"

echo
echo "done. Artifacts:"
echo "  docs/SWEEP_${TAG}_REPORTS.txt"
echo "  docs/report_census.json"
echo "  $OUTDIR/ (330 per-gate JSONs)"
echo "  scripts/sweep_census/gaps.json + docs/coverage_gaps_${TAG}.txt"
echo "  docs/REPORT_PARITY_${TAG}.txt (identical report streams per edition + families)"
echo
echo "Then: write docs/COVERAGE_GAPS_${TAG}.md from the gaps output and diff its"
echo "headline against docs/COVERAGE_GAPS_2026-09-15.md to show what the fixes moved."
