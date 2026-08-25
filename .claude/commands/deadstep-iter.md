---
description: Run one iteration of the FFB never-dispatched-step (dead-step frontier) campaign
---

Run ONE iteration of the dead-step frontier campaign, then stop and report.

## Goal

Drive the count of never-dispatched `StepId`s toward zero. Baseline measurement:
`ffb-rust/docs/DEAD_STEP_INVENTORY.md` — **62 of 199 never reached** (2026-08-19 pm sweep).
Every step switched on so far has exposed real Rust engine bugs (TTM 10, interception 8,
All You Can Eat 6, Zzharg 2) — reaching the step IS the bug-finding method.

Ordered work queue (top to bottom, ONE item per iteration). Ticked state lives in
`ffb-rust/docs/BACKLOG.md` §11 and in the `## DEADSTEP-ITER<n>` log described below.

1. **§11 Batch B stars** (`docs/BACKLOG.md` §11) — Guffle Pusmaw ("Quick Bite") → bb2025
   nurgle; Swiftvine Glimmershard ("Furious Outburst", 4 dead ids) → bb2025 wood_elf;
   Thorsson Stoutmead ("Beer Barrel Bash!" → ThrowKeg/EndThrowKeg) → bb2025 dwarf;
   Grombrindal ("Wisdom of the White Dwarf") → bb2025 dwarf or halfling.
2. **§11 Batch C** — The Zoat ("Excuse Me, Are You a Zoat?" → `AutoGazeZoat`), already in
   `data/star_players/all_editions.json`, host on a bb2020 team.
3. **Multiple Block roster** — draft the Multiple Block skill into a parity roster to reach
   `MultipleBlockFork BlockRollMultiple FoulAppearanceMultiple ApothecaryMultiple
   DauntlessMultiple StateMultipleRolls ReportStabInjury DispatchDumpOff` (8 ids, one skill).
4. **BLITZ/GAZE select sub-chain** — `SelectBlitzTarget SelectBlitzTargetEnd
   SelectGazeTarget SelectGazeTargetEnd`. Both harnesses declare a folded-target BLITZ, so the
   BLITZ_MOVE→BLITZ_SELECT dialog chain has never run. This is recurring bug shape #2
   (lockstep decline) by construction — changing BOTH agents to use the split declaration is
   the work, and it needs a `ParityRunner.java` edit + jar rebuild.
5. **Long tail of skill/star specials** — `HailMaryPass Pro ThrowARock BlackInk Treacherous
   DoubleStrength EatTeamMate WeatherMage PileDriver` etc. One carrier per iteration.
6. **Inducements / cards / wizard / prayers** — `PlayCard Wizard MasterChef FanFactor
   PrayerRoll`. Needs the harness to actually BUY inducements on both sides.
7. **bb2016 Kick Team-Mate** — `InitKickTeamMate EndKickTeamMate KickTeamMate
   KickTeamMateDoubleRolled` are bb2016-generator-only ids and no bb2016 roster drafts the
   skill. A roster/draft change, not an engine fix.

`AssignTouchdowns InitPunt EndPunt PuntDirection PuntDistance` are OUT OF SCOPE for this loop —
they need the scoring-agent tier, which is a user decision. Do not start it; note it and skip.

## Non-negotiable rules

- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, harness `ParityRunner.java` (jar rebuild).
- **Every Rust change is a 1:1 port** of the corresponding Java class/method. Read the Java
  first, port the Java. No hacks, no parity-only special cases, no constants tuned to pass a seed.
- **Every fix lands with a colocated `#[cfg(test)]` regression test.**
- **Vacuous-green check is mandatory.** A 100/100 that never fired the mechanic proves nothing.
  After the run, grep the events/traces and record the FIRING RATE (declarations, and games out
  of 100). If it is zero, the item is NOT done — find why the offer never happened.
- **A sweep is valid ONLY if the process exits 0 AND prints `TIMING ... rust_total=`.** Absence
  of `PARITY FAIL` lines is not a measurement — a panic aborts before any comparison.
- **Never run two parity runs of the SAME matchup concurrently** — they clobber
  `parity/<edition>/<matchup>/seed_N_*.jsonl`. Different matchups in parallel are fine.
- **Any data/draft change invalidates the Java log cache** — regenerate Java logs, never
  `--reuse-java` after touching `data/`, a team spec, or `ParityRunner.java`.
- **Never `git checkout --` a probed file.** Remove probes with targeted edits, then `git diff`
  the file and read EVERY `-` line before gating.
- Parallel subagents sharing this working dir get **read-only git** (no stash/checkout/reset).
- Limited CPU: `--parallel 3`, `PARITY_JVM_CORES=1`, never 8. Never build while a gate is active.

## Iteration procedure

1. **Orient**: read the TAIL of `ffb-rust/docs/BACKLOG.md` (§11 and any `DEADSTEP-ITER` log)
   and `ffb-rust/docs/DEAD_STEP_INVENTORY.md` for what is already live and what the last
   iteration said to do next.
2. **Target** the topmost unticked queue item. Carry one item per iteration; if an item is
   larger than one iteration, carry it across iterations and say so.
3. **Wire it**: add/verify the data entry (skill names must resolve in BOTH engines — check the
   canonical Java name), draft the carrier at or near the LOS in the host team spec, rerun
   `scripts/gen_java_parity_data.py`, rebuild the jar if `ParityRunner.java` changed.
4. **Run the host matchup 1-100** on FRESH Java logs; root-cause every divergence Rust-side.
   Debugging tools that actually work:
   - `rng_calls` + `FFB_DRIVE_TRACE=1` + `JIDSTATE` — NOT `DICE_TRACE` global position
     (Java logs per-call, Rust per-die, so positions sit offset). Compare dice by SIDES sequence.
   - State-only divergence (dice match) → diff Rust vs Java state STRINGS at the post-step
     (`FFB_TRACE=1` → RUST_STEP / JSTEP). State-string player labels are POSITIONAL indices.
   - Find the LIVE code path with a gated `Backtrace::force_capture()`, never by reading —
     stale duplicate impls (per-edition twins are usually dead) have cost whole iterations.
   - A stall = a step returning `Continue` with `prompt.is_none()`.
   - Check the five recurring bug shapes at the tail of `docs/BACKLOG.md` FIRST.
5. **Full gate before committing** (all must pass, else revert):
   `python scripts/run_team_matrix.py --edition all --seeds 1-100 --parallel 3`
   → **bb2016 30/30, bb2020 30/30, bb2025 30/30**, plus `cargo test --workspace` clean.
   `--reuse-java` ONLY for a Rust-only change with no data/harness edit.
6. **Commit AND push** when the gate is green. If no progress, commit documented findings only.
7. **Log it**: tick the item in `docs/BACKLOG.md` §11 (or add the item if it is queue entry 3-7),
   append a `## DEADSTEP-ITER<n>` section recording: target, what was wired, the firing rate
   (vacuous-green evidence), every engine bug found and its Java source, gate numbers, commit
   hash, and the concrete next step. Re-run the dead-step sweep and update
   `docs/DEAD_STEP_INVENTORY.md`'s reached/never-reached counts when ids actually flip.
   State partial results as partial — never claim a green that was not measured.

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
./target/release/ffb-parity --home <roster> --away <roster> --edition bb2025 --tier 3 --seeds 1-100 --no-abort
./target/release/ffb-parity --home <roster> --away <roster> --edition bb2025 --tier 3 --seeds N-N
FFB_DRIVE_TRACE=1 ./target/release/ffb-parity --uniform --all-rosters --all-editions --seeds 1-3 --no-abort
```

Piping the parity binary's stdout through `grep` can lose the `PARITY: n/m` line — redirect to a
file first, then grep the file.

## Reporting

End the iteration with a short status line: target, what flipped from dead to live, firing rate,
gate results, commit hash (or "no commit — findings only"), and the next step.

**Do not stop the campaign on your own judgement.** The loop continues until the queue is
genuinely exhausted (every in-scope id live or classified with evidence) or the user says stop.
