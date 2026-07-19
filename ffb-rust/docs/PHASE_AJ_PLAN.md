# Phase AJ Plan — Delete Dead bb2020/special/ Duplicate, Then Full Sweep of step/ (464 files)

*Written 2026-07-19, immediately after Phase AI (17,423 tests).*

## Context

Phase AI fully swept the 3 largest previously-flagged pools (`ffb-mechanics/modifiers/`, dialog
parameters, `step/generator/`) and closed with two open items: a dead-code decision on
`skill/bb2020/special/`, and the need to scope a fresh area since no more partially-audited pools
remained. Investigation confirmed the dead directory was a true duplicate (git history showed the
20 Java `bb2020.special` classes were translated twice in one commit, then a same-day merge
orphaned the `special/` copies) — safe to delete outright. The largest never-audited pool was
`ffb-engine/src/step/` non-generator step files (464 files) — the actual step-execution logic,
arguably the most behaviorally important code in the engine. Two samples during scoping already
found a confirmed total stub (`step_wisdom_of_the_white_dwarf.rs`) and a likely second bug. User
confirmed: delete the directory, fully sweep all 464 files in this phase.

## What actually happened

**Stage 1**: deleted the confirmed-dead `crates/ffb-model/src/skill/bb2020/special/` (20 files +
`mod.rs`) after confirming the flat sibling files are the live, tracker-tracked translation. 17,423
tests unaffected (the directory carried no compiled tests).

**Stage 2**: fanned out 11 parallel isolated-worktree agents to fully sweep all 464 `step/` files,
split by subdirectory (bb2016 ×2, bb2020 ×2, bb2025 ×3, mixed ×2, core/framework+action/game/phase
×2). One worktree-creation attempt failed on the first try (lock contention, same issue Phase AI
saw) and was retried — the retry turned out to duplicate a batch that had actually succeeded under
a different agent ID, while a different batch (mixed pass/shared/special/start/ttm, including
`step_quick_bite.rs`) silently never ran. Both were caught and corrected before finalizing: the
duplicate was reconciled file-by-file rather than blindly merged (some of its findings were already
covered, several were new and genuinely additive, and two files needed a careful blend — see below),
and the missing batch was launched fresh and completed cleanly.

**Result: every one of the 12 agent runs (11 original + 1 gap-fill) found real, confirmed bugs.**
This is the fifth consecutive phase (AF, AG, AH, AI, AJ) where fresh re-verification found real
bugs in nearly everything it checked — this time in the largest pool yet, with no sign of drying up.

### Highlights (full per-batch detail in `SESSION.md`)

- **A live production-breaking hang**: `driver.rs`'s core dispatch loop busy-looped forever on any
  `Continue` outcome with no `AgentPrompt` attached — a real bug in the game loop itself, not a
  per-skill translation gap, hit by fixing `step_init_start_game.rs`'s missing both-coaches-ready
  gate. Fixed with a `waiting_for_command` flag on `DriverGameState`.
- **Total stubs found and fully ported**: `step_wisdom_of_the_white_dwarf.rs`, plus two more
  discovered during the sweep (`step_auto_gaze_zoat.rs`, `step_then_i_started_blastin.rs`,
  `step_look_into_my_eyes.rs`) and a fourth in the gap-fill batch (`ttm/step_swoop.rs`'s entire
  throw-scatter movement block was missing — Swoop's core mechanic was a no-op).
  `step_quick_bite.rs` — flagged during scoping — was confirmed on both counts: a silent no-op on
  its single-opponent path, and a nonsensical acting-team comparison.
- **Cross-cutting infrastructure fixes**: `StepParameter::KickedPlayerCoordinate` couldn't carry
  Java's `null` sentinel (changed to `Option<FieldCoordinate>`, rippling through 8 files);
  `CHAINSAW_TURNOVER`/`ALLOW_BALL_AND_CHAIN_RE_ROLL` game options were being ignored in favor of
  hardcoded defaults in several files; the `AgentPrompt::SwarmingPlayers` dialog was defined but
  never wired, silently stalling any autonomous agent at that decision point.
  Two `SkillId::properties()` table gaps (`WisdomOfTheWhiteDwarf`, `ExcuseMeAreYouAZoat`) were
  silently breaking their own skills' lookups regardless of the step logic.
- **Systemic per-edition bugs**: BB2016's `step_catch_scatter_throw_in.rs` and `step_pushback.rs`
  were bare re-exports of the BB2025 versions, leaking edition-specific rules into BB2016 play.
  `step_wrestle.rs` hardcoded BB2016 gating logic across all three rule editions despite BB2020/
  BB2025 having genuinely different conditions in Java.
- **Reconciling the duplicate batch-8 run**: diffed both independent runs against main file-by-file
  rather than merging either wholesale. 4 files were genuinely new in the duplicate (safe to bring
  in whole); 4 needed a careful blend where both runs found real, non-contradictory bugs in the
  same file (verified against Java source before combining, e.g. `step_foul_chainsaw.rs` kept the
  first run's `hasUnusedSkillWithProperty` fix *and* added the second run's `CHAINSAW_TURNOVER`
  option gating); the rest were already fully covered by the first run.

**Deferred, documented, not fixed** (flagged for a future phase, not force-fit into this one):
- `sequences.rs`'s `start_game_sequence()`/`h2_kickoff_sequence()` ignores `Rules` and is missing
  SWARMING/MASTER_CHEF/KICKOFF_RETURN steps for some editions — fixing it exposed a separate latent
  infinite-loop bug in `step_touchback.rs`, so the fix was reverted rather than ship a regression.
- BB2025's `step_apothecary.rs` is missing substantial "Getting Even"/"Raise Dead" subsystems.
- `step_select_blitz_target_end.rs` is a self-documented stub missing entire generator-sequence
  pushes; part of `step_play_card.rs`'s `CLIENT_SETUP_PLAYER` gap needs a new `Action` variant.
- Tentacles' re-roll resolves against the acting team's sources, but Java offers it to the
  *defender's* team — fixing this needs a broader signature change to shared reroll helpers.

## Estimate: how far are we, and how far after this phase?

- **File/method coverage:** unchanged, ~99.8–99.9%+ (0 `○`/`~` tracker rows).
- **Tests:** 17,423 → **17,736** (+313, after accounting for the duplicate-batch reconciliation).
- **Behavioral-correctness residual:** still present, still no sign of drying up after five
  consecutive phases. This phase swept the single largest remaining pool (464 files, the actual
  step-execution logic) and found real bugs in essentially all of it, including one live
  production-breaking hang in core engine infrastructure — not a narrow per-skill issue.
- **What's left:** (1) the ~305 skill files' full method-body logic (only constructor arguments
  were checked in a prior phase); (2) the ~137 client/server handler files (sampled clean twice,
  never fully swept); (3) the small set of documented-not-fixed gaps above; (4) parity/integration
  testing remains the only large, entirely out-of-scope workstream. The project is **not**
  behaviorally done, despite ~100%-looking file coverage — five straight phases of fresh
  re-verification finding real bugs is now a strong, repeated signal, not a one-off.
