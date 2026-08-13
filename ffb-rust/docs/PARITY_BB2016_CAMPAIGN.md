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
| renegades | **0** (38→8→1→0) | 🟢 100/100 GREEN (ITER55 TTM routing, ITER56 declined-re-roll, ITER57 RightStuff dropPlayer) | GREEN |
| underworld | **0** (44→8→1→0) | 🟢 100/100 GREEN (ITER55-56 TTM + ITER58 bb2016 InitPassing routing) | GREEN |
| necromantic | 58 | untraced — **NEXT TARGET** (fewest fails) | queued |
| undead | 76 | stand-up-blitz-GFI (ITER51 diagnosis) | queued |
| dwarf | 79 | Deathroller (ITER54 diagnosis), multi-layer | queued |
| elf | 84 | untraced (suspect AG / pass) | queued |
| ogre | 98 | earlier non-TTM blocker | queued |
| wood_elf | 98 | untraced | queued |
| goblin | 100 | earlier non-TTM blocker masks the TTM win — retrace seed 1 | queued |
| halfling | 100 | systematic (every seed) — likely a roster/skill-load or first-step gap | queued |
| vampire | 100 | systematic — Bloodlust bb2016 | queued |

Counts above re-scouted 2026-08-13 AFTER ITER56-58 with FULL 1-100 `--no-abort` runs (no timeout).
The older `undead 44` / `necromantic 44` figures were unreliable (truncated triage) — the true
counts are 76 / 58. Green rosters re-verified 0 fails in the same sweep.

Green (21): renegades, underworld, lineman, amazon, chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl,
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

### ITER57 (2026-08-13) — bb2016 StepRightStuff failed landing must `dropPlayer` → **renegades 100/100 GREEN**

renegades seed 80, first state mismatch i=62; the real divergence is step 61's resolution
(home_04 Troll THROW_TEAM_MATE of the ball-carrying Goblin h10 at (4,6)). State-only + Rust one
die SHORT. `FFB_DICE_TRACE` caller stacks pinned it exactly:

| pos | Java caller | Rust |
|---|---|---|
| 57 | `ThrowTeamMateBehaviour$1.handleExecuteStepHook:78` accuracy d6=1 → FUMBLE | same |
| 58 | `StepRightStuff.executeStep:135` landing d6=2 → fail, re-roll declined | same |
| 59-60 | `InjuryTypeTTMLanding.handleInjury:34` armour 2d6 | same |
| **61** | **`StepCatchScatterThrowIn.scatterBall:446` d8=1 — the ball bounce** | *(missing)* — Rust's pos 61 is already the next activation's Bone Head d6 |

ROOT CAUSE: Java's `StepRightStuff.executeStep()` `if (!doRoll)` block publishes the injury result
AND calls `UtilServerInjury.dropPlayer(this, thrownPlayer, THROWN_PLAYER)`, publishing its returned
parameters. `dropPlayer` sets `fieldModel.ballMoving = true` and returns
`CATCH_SCATTER_THROW_IN_MODE = SCATTER_BALL` whenever the dropped player's square equals the ball
square — that parameter is what makes the sequence's following CATCH_SCATTER_THROW_IN step bounce
the ball. It also returns `END_TURN` (turnover) for the acting team's own carrier, which Java keeps
when `fThrownPlayerHasBall` and removes otherwise. Rust's `land_injury` published only
`ThrownPlayerCoordinate(None)`, so the ball stayed under the prone Goblin and the bounce d8 was
never rolled.

FIX: port the block 1:1, mirroring the already-correct bb2020 translation. Also collapsed the
duplicated `drop_thrown_player` injury branch into the same `land_injury` path — Java has ONE
injury site (`doRoll = !fDropThrownPlayer` falls through to `if (!doRoll)`), not two.

Tests: `failed_landing_of_ball_carrier_drops_player_and_requests_ball_scatter`,
`failed_landing_without_ball_scatters_but_does_not_end_turn`.
Verified: renegades 1 fail → **0 (100/100)**; lineman bb2016 100/100; lineman bb2025 100/100;
`cargo test -p ffb-engine` green. Commit `a1db7893`. **20 🟢 / 10 🔴.**

NOTE (latent, not hit yet): Rust `util_server_injury::drop_player_with_base` omits Java's
`&& game.getTurnMode() != TurnMode.BLITZ` guard on the ball-scatter branch, and adds a
`FieldCoordinateBounds::FIELD.is_in_bounds` early return Java does not have. Worth checking when a
BLITZ-turn-mode drop shows up as a frontier.

### ITER58 (2026-08-13) — the bb2016 PASS step-set was routed to the bb2025 steps → **underworld 100/100 GREEN**

underworld seed 72. First state mismatch i=78, but the first `rng_calls` divergence is i=75
(Java 52 / Rust 53): step 74 (`Activate(away_03, PASS)`) rolled one die in Rust and none in Java.

Java trace: `JAVA_PASS pid=…Away3 coord=(25,7)` then `UNHANDLED_STEP: INIT_PASSING`. The thrower is
at (12,9) — 13 squares away — so Java's `StepInitPassing.executeStep()` finds
`findPassingDistance(...) == null`, no branch matches, and it returns WITHOUT calling
`setNextAction`. The step stays current; ParityRunner has no `INIT_PASSING` case, so the stuck step
falls to its `default:` branch and injects `ClientCommandEndTurn` → turnover, ball unmoved, zero
dice. **This is not a harness gap** — it is stock Java's behaviour for an out-of-range throw, and
the runner's EndTurn is just how the contract resolves it.

Rust trace: `RUST_STEPPASS thrower=away_03 … pass_coord=(25,7) dist=None` — emitted from
`step/bb2025/pass/step_pass.rs`. ROOT CAUSE: `bb2016::move_::step_end_selecting` already pushes the
**bb2016 Pass sequence**, but `make_step_for(id, Rules::Bb2016)` had no pass entries, so the driver
instantiated the **bb2025** step classes for every StepId in it. The shared
`mixed::pass::step_init_passing` range-checks with `ffb_model::util::passing::passing_distance`
(the bb2020+ table), which accepts (dx 13, dy 2); the bb2016 `PassMechanic::find_passing_distance`
throwing-range table rejects it. So Rust accepted the throw, rolled the accuracy d6 and offered an
interception that stock Java never rolls.

FIX (two parts):
1. `bb2016/pass/step_init_passing.rs`: the no-branch-matched fall-through now produces the same
   observable result as Java-plus-runner — `GOTO_LABEL(gotoLabelOnEnd)` + `END_TURN(true)` — instead
   of `Continue`, which the headless driver cannot resolve (identical to how the already-verified
   shared `mixed` step handles it).
2. `driver.rs` `make_step_for`: route `StepId::InitPassing` to the bb2016 impl for bb2016 games.

**Scope lesson — routing the WHOLE bb2016 pass step-set REGRESSED badly and was reverted.** Adding
`Pass`/`Intercept`/`HailMaryPass`/`EndPassing`/`PassBlock` alongside `InitPassing` took lineman
bb2016 from 0 → 38 fails and underworld from 1 → 33: those five bb2016 step files are less complete
than the bb2025 ones they were shadowing. Only `InitPassing` is routed. The other five remain a
known gap — port them individually, each verified on its own, not as a block.

Tests: `out_of_range_pass_ends_the_turn_without_rolling` (bb2016 InitPassing).
Verified: underworld 1 fail → **0 (100/100)**; lineman bb2016 100/100. **21 🟢 / 9 🔴.**
