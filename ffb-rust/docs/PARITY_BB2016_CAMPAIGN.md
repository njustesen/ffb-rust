# PARITY_BB2016_CAMPAIGN.md — drive all 30 bb2016 mirror matchups to 100/100

GOAL: `docs/TEAM_MATRIX_BB2016.md` all 30 rosters 🟢 100/100 (mirror, `--edition bb2016 --tier 3 --seeds 1-100`).

Ground rules (see `docs/PARITY_PROCESS.md`, non-negotiable):
- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, harness `ParityRunner.java` (needs a jar rebuild).
- Every Rust change is a **1:1 port** of the corresponding Java class/method. No hacks, no
  parity-only special-cases. Read the Java, port the Java.
- Every fix lands with a colocated `#[cfg(test)]` regression test.
- Verify advance **and** no regression (lineman bb2016+bb2025 100/100, `cargo test -p ffb-engine`)
  before committing. REVERT if regressed.

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
# full 1-100 for one roster, counting every failing seed (do NOT trust parallel/timeout triage)
./target/release/ffb-parity --home R --away R --edition bb2016 --tier 3 --seeds 1-100 --no-abort
# single seed, first divergence only
./target/release/ffb-parity --home R --away R --edition bb2016 --tier 3 --seeds N-N
```

**NEVER run two parity runs of the SAME matchup concurrently** (even different editions): both
write `parity/<home>_vs_<away>/seed_N_{java,rust}.jsonl`, so they clobber each other and produce
bogus mass failures (observed: lineman bb2016 reported 95/100 fails, actually 0). Different
matchups run in parallel safely — they use different directories.

Tracing: `FFB_TRACE=1` (RUST_STEP/JSTEP state strings), `FFB_DICE_TRACE=1` (per-die + Java
`caller=` stack), `FFB_DRIVE_TRACE=1` (Rust step order / stalls). Redirect to a file — the
volume is large. Reliable signal = per-step `rng_calls` deltas + state-string diffs, NOT
`DICE_TRACE pos=` (Java counts per call, Rust per die).

## Status (update every iteration)

Baseline entering this run (post-commit `5e86d749`): **19 🟢 / 11 🔴**.

| Roster | fails /100 | Diagnosis | State |
|---|---:|---|---|
| renegades | **1** (38→8→1) | ITER55 TTM routing + ITER56 declined-re-roll. Residual = seed 80 step 61 | ACTIVE |
| underworld | **1** (44→8→1) | same two fixes. Residual = seed 72 | ACTIVE |
| undead | 44 | stand-up-blitz-GFI (ITER51 diagnosis) | queued |
| dwarf | 80 | Deathroller (ITER54 diagnosis), multi-layer | queued |
| elf | 84 | untraced (suspect AG / pass) | queued |
| wood_elf | 99 | untraced | queued |
| goblin | 100 | earlier non-TTM blocker masks the TTM win — retrace seed 1 | queued |
| ogre | 98 | earlier non-TTM blocker | queued |
| halfling | 100 | systematic (every seed) — likely a roster/skill-load or first-step gap | queued |
| vampire | 100 | systematic — Bloodlust bb2016 | queued |
| necromantic | 44* | *stale count, re-verify | queued |

Green (19): lineman, amazon, chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl,
high_elf, human, khemri, khemri_fumbbl, lizardman, nippon, norse, nurgle, orc, skaven, slann,
slann_fumbbl.

## Iteration protocol

1. Pick the roster with the FEWEST fails (cheapest green), unless a shared root cause makes a
   higher-fail roster higher leverage (a fix hitting 4 rosters beats one hitting 1).
2. Run its lowest failing seed alone; find the first divergent step.
3. Root-cause to ONE Java-vs-Rust difference. Read the Java class named in the trace.
4. Port it 1:1 + regression test. `cargo test --release -p ffb-engine`.
5. Re-run the roster 1-100 `--no-abort`; confirm the fail count DROPS and lineman stays 100/100.
6. Commit with an explicit path list (never `git add -A` — it would sweep `parity/*.jsonl` and
   `.claude/worktrees`). Append the diagnosis here and update the table + matrix doc.
7. Chain to the next frontier.

## Iteration log

(append newest at the bottom)

### ITER56 (2026-08-13) — bb2016 TTM declined re-roll must keep `re_rolled_action`

Java `skillbehaviour/bb2016/ThrowTeamMateBehaviour.handleExecuteStepHook` gates on
`ReRolledActions.THROW_TEAM_MATE == step.getReRolledAction()` and, when the source is null or the
re-roll is unusable, `GOTO_LABEL(goToLabelOnFailure)` — it NEVER clears `reRolledAction`. Rust's
`bb2016/ttm/step_throw_team_mate.rs::handle_command` cleared BOTH `re_rolled_action` and
`re_roll_source` on `UseSkill{false}` / `UseReRoll{false}`, so `execute_step` re-entered as a FRESH
throw and rolled the accuracy d6 a second time — an extra die that desynced the shared stream (and
often flipped a fumble into a spurious success + 3× scatter). The parity harness always declines
team re-rolls (`ParityRunner.sendUseReRoll(action, null)`), so a fumbled bb2016 TTM must stay
fumbled. FIX: clear only `re_roll_source`. Test
`declined_reroll_keeps_action_clears_source_and_gotos_failure`.

IMPACT (measured, much larger than expected): **renegades 8 fails → 1** (only seed 80 left),
**underworld 8 → 1** (only seed 72). The spurious second accuracy die was the dominant residual
TTM desync for both rosters.
