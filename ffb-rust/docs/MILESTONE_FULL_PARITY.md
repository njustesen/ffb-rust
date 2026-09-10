# 🏁 Milestone — full-matrix heuristic parity, 333/333

**2026-09-10 · tag `parity-matrix-green-2026-09-10`**

The tag is the anchor: `git show parity-matrix-green-2026-09-10`. The sweep itself was run on
`4e999414b`, whose tree is identical to the tagged commit apart from these docs.

Every drafted team, in every ruleset it belongs to, at every sampling scale, now agrees
step-for-step between the Rust engine and the stock Java engine under the **heuristic** agent.

| | |
|---|---|
| Cells (roster × ruleset) | **111** |
| Gates (cells × scales 1.0 / 0 / 1e6) | **333** |
| `PARITY: 100/100 games match` | **333** |
| Failures | **0** |
| Rust panics | **0** |
| Missing verdicts | **0** |
| Games compared | **33,300** |

Per-gate verdicts: [`SWEEP_2026-09-10.txt`](SWEEP_2026-09-10.txt). Matrix and cell semantics:
[`PARITY_COVERAGE_REQUIREMENTS.md`](PARITY_COVERAGE_REQUIREMENTS.md). Work log:
[`BACKLOG.md`](BACKLOG.md) §§H.1–H.12.

## What this does and does not claim

**Does:** for these 333 gates the two engines produce identical per-step state hashes across 100
seeds each, under an agent that actually plays — moving, blocking, passing, fouling, spending
re-rolls — rather than the earlier random contract that declined nearly everything.

**Does not:** this is not "the engine is correct". It is "the two engines agree on what the drafted
teams do". Three standing limits, all of which have already hidden real bugs in this campaign:

1. **The state hash is narrow.** It covers half, turn counters, active team, scores, ball, and
   MA/ST/AG/AV for the first 11 players by number. It does **not** carry `passing`, `reroll_used`,
   a player's ACTIVE bit, prayer-granted skills, or anything about players numbered above 11.
   `elf` bb2020 sat green for weeks while the engines disagreed about a live Pro skill (§H.10).
2. **Green is scoped to the drafted teams**, not to the game. A mechanic no drafted roster carries
   is untested. Four mechanics in this campaign — Swarming, Grab, Trickster, Pogo — were **dead in
   both engines** until the team carrying them was drafted, and each hid engine bugs.
3. **Coverage is a separate check, and it is NOT green.** See below.

## The caveat: 26 gates fail the mechanic-coverage checklist

26 of the 333 print `100/100 games match, but required coverage items are MISSING` and **exit 1**.
Parity passes on all of them (`failed == 0`); the checklist asks a different question — whether the
100 games actually EXERCISED a required mechanic.

Sampled `lizardman bb2025 @1.0`: missing `action HandOver | 0 | needs carrier + adjacent teammate`
— the heuristic never hands off with a Saurus/Skink roster in 100 games. That is agent behaviour,
not engine disagreement.

**So the honest headline is: parity green everywhere, coverage green almost everywhere.** Earlier
matrix tables counted a cell green off the parity clause alone and hid this. The 26 are listed in
`SWEEP_2026-09-10.txt`; triaging them is the open follow-up in BACKLOG §H.12.

## How it was verified

- 4 sharded workers pinned to 8 of 16 cores; sharded by (edition, matchup) so all three scales of a
  matchup stay on one worker — two runs of the same edition+matchup must never overlap.
- **3.07 h wall** (23:11:36 → 02:15:36), against 11.99 h of summed gate time ÷ 4 workers = 3.00 h
  expected. Nothing was skipped; the speedup is sharding alone.
- **No `--reuse-java`.** A fresh JVM per gate, 333 distinct `FFB_PARITY_ROOT` values, and **0 of
  33,300** Java seed files predate the run. (A stale cache once turned a 100/100 gate into 30/100,
  so this is checked, not assumed.)

## The last two cells

| cell | was | fix |
|---|---|---|
| `imperial_nobility` bb2025 | 84/100/33 | Pro re-roll was never offered. Java lists `ReRollProperty.PRO` alongside TRR, so Pro raises the question alone once the team re-roll is spent. Gated to bb2025 — the gate is in the HARNESS (`reRollSourceFor` returns early when `!teamReRollOption`), not the rules. §H.10 |
| `necromantic` bb2016 | 98/99/99 | `can_raise_dead` read the edition-LESS property set, where BB2025's Regeneration deliberately drops `preventRaiseFromDead` — so every bb2016/bb2020 Regeneration player became raisable and Rust raised Zombies Java refused to raise. §H.11 |
