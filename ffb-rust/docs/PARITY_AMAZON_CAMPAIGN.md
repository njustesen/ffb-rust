# Parity campaign — HeuristicAgent on amazon vs amazon (all three rulesets)

Started 2026-08-29, immediately after the lineman heuristic campaign closed at ITER70
(`3b3746b12`, nine gates at 100/100).

**Command**: `/amz-iter` — `.claude/commands/amz-iter.md` holds the goal, the rules, the root-cause
procedure and the standing no-regression gate. Read it every iteration; read the TAIL of this file
for the frontier.

## Goal

Nine `PARITY: 100/100 games match` — amazon v amazon, tier 3, seeds 1-100, `--agent heuristic
--heur-classes all`, across bb2016 / bb2020 / bb2025 x `--heur-scale` 0 / 1.0 / 1e6 — **plus** two
things the lineman campaign did not have to deliver:

- the agent uses the amazon skills it now has (Dodge everywhere, Block, Catch, Pass, Safe Pass,
  On the Ball, Hit and Run, Jump Up, Defensive) in its scoring, kept fast and simple;
- the event coverage is analysed, and any skill present on the pitch but absent from the event
  stream is explained — agent gap, engine bug, or genuinely dead path.

## Why this roster

Lineman rosters carry no skills at all, so 70 iterations of parity work never executed a skill
re-roll, a block-die choice under Block, a catch under Catch, or a throw under Pass/Safe Pass.
Amazons carry one skill on every player in every edition, and the three editions carry *different*
ones — bb2016 has a Catcher with Catch and a Blitzer with Block; bb2020/bb2025 replace those with
On the Ball, Safe Pass, Hit and Run, Jump Up and Defensive. The editions therefore differ
structurally, not just by generator.

## Status

**ITER0 not yet run.** No baseline measured. The first iteration's job is the census: run the nine
gates (and `--agent random` amazon in all three editions as the control), record the numbers, and
name the frontier.

## Iterations

<!-- Append one `## ITER<n>` section per iteration: what diverged, the root cause, the fix, the
     gate numbers, `rust_total=` timing, and the next frontier. -->
