# Phase AK: Full Method-Body Audit of Skill Files (2026-07-19)

## Scope

Full method-body re-audit of all 305 files under `crates/ffb-model/src/skill/` against their Java
sources, following Phase AJ's closing note that this was the largest never-fully-swept pool (only
constructor arguments/property registrations had been spot-checked in an earlier phase; method
bodies never systematically compared). Split into 8 parallel isolated-worktree batches:

| Batch | Directory | Files | Bugs found |
|---|---|---|---|
| 1 | `skill/bb2016/` | 57 | 12 |
| 2 | `skill/bb2020/` (+ flattened `special/`) | 57 | 24 property fixes + **36-file dead-code discovery** + 1 mechanics gap |
| 3 | `skill/bb2025/` (excl. `special/`) | 56 | 8 |
| 4 | `skill/bb2025/special/` + `skill/common/` | 45 | ~13 |
| 5 | `skill/mixed/` (accurate–leader) | 22 | 5 |
| 6 | `skill/mixed/` (loner–unchannelled_fury) | 20 | 3 |
| 7 | `skill/mixed/special/` (all_you_can_eat–old_pro) | 22 | 14 |
| 8 | `skill/mixed/special/` (primal_savagery–yoink) | 22 | 6 |

All 305 files covered exactly once (verified by diffing each worktree's file list against its
assignment before merging; `mixed/mod.rs` — deliberately excluded from both mixed/ batches to avoid
a merge conflict — was independently checked post-merge and found already complete, all 42 Java
classes correctly declared).

Two batches (bb2016, bb2020) initially returned empty/stalled placeholder responses on first launch
(apparent context confusion from unrelated concurrent worktree agents in the same repo) and had to
be relaunched from scratch with explicit "ignore other batch numbers, execute yourself" framing —
both retries succeeded cleanly. One additional batch's report was a truncated status-monitoring
loop; the actual committed work was verified directly instead of waiting for a final summary.

Tests: 17,736 → **17,902** (+166). 0 failures across `cargo test --workspace`; `cargo build --workspace`
and `cargo clippy --workspace --all-targets` both clean.

## Headline finding: 36 of 56 bb2020 skill files were compiled dead code

`crates/ffb-model/src/skill/bb2020/mod.rs` declared only 20 `pub mod` entries — the ones
corresponding to Java's `bb2020/special/*.java` subpackage. The other 36 files sitting in the same
directory were **complete, correct 1:1 translations of real, distinct `@RulesCollection(Rules.BB2020)`
Java classes** (Animosity, BallAndChain, Bombardier, BoneHead, Brawler, BreakTackle, BreatheFire,
Chainsaw, CloudBurster, Defensive, DirtyPlayer, Fumblerooskie, HitAndRun, HypnoticGaze, Leap,
MightyBlow, MonstrousMouth, NoHands, PassingIncrease, PileDriver, PilingOn, PogoStick,
ProjectileVomit, ReallyStupid, Regeneration, RunningPass, Shadowing, SideStep, SneakyGit, Stab,
StrengthIncrease, SureFeet, Swarming, Swoop, VeryLongLegs) that were **never declared as `pub mod`,
meaning they were never compiled, never tested, never part of the crate** — bb2020 games were
silently running without any of these 36 skills' effects. Fixed by rewriting `bb2020/mod.rs` to
declare all 56 modules.

## The dominant bug shape: missing/wrong `SkillId::properties()` entries

This codebase does not use per-skill-struct `postConstruct()`-equivalent methods as its live
property-registration mechanism — `Skill::register_property`/`register_reroll_source`/
`set_enhancements` are confirmed dead code with zero callers anywhere in the workspace. The actual
live mechanism is the centralized static table `SkillId::properties()` in
`crates/ffb-model/src/enums/skill_id.rs`, consumed via `player.has_skill_property(...)` at engine
call sites. Every batch found the same shape of bug: a skill's Java `postConstruct()` registers a
property that has no corresponding entry (or an incomplete/wrong one) in this table, silently
breaking that skill's mechanic while the per-file skill struct itself looks perfectly correct.

Roughly 70 property-table entries were added or corrected across all 8 batches (see individual
batch commits for full per-skill detail). Two confirmed **live gameplay breaks** traced directly to
missing entries: `GoredByTheBull`'s `canAddBlockDie` (checked by `step_init_moving.rs`/
`step_end_moving.rs`) and `HalflingLuck`'s `canRerollSingleDieOncePerPeriod` (checked by
`step_block_roll.rs`) — both skills' entire mechanic silently never fired.

## Other bug shapes found (recurring across batches)

- Missing inherent method overrides: `getConfusionMessage()`, `getSkillUseDescription()`,
  `evaluator()`, `getCost()`, `canBeAssignedTo()` — Java overrides these on specific skill classes;
  the Rust struct had no override at all, silently falling back to a base-class default (or, for
  methods without a base default, simply never being called).
- Wrong `SkillCategory`/`SkillUsageType`/`DeclareCondition` constructor arguments (e.g. LethalFlight
  tagged `Trait` instead of `Devious`; Leap invented a `OncePerTurn` usage type Java never has).
- A real `ffb-mechanics` gap: `dodge_modifier_factory.rs` only had a BB2016 arm for BreakTackle's
  strength-tiered dodge bonus — BB2020 and BB2025 players got zero benefit from the skill. Fixed
  with both editions' arms + tests.
- A systemic, still-unresolved gap flagged by two independent batches: **no skill file anywhere in
  the codebase calls `register_reroll_source`**, despite the `ReRollSource`/`ReRolledAction`
  infrastructure being fully implemented and tested. Several skills (SavageBlow, ThinkingMansTroll,
  UnstoppableMomentum, Dodge, and others) register reroll sources in Java that have no Rust-side
  effect. This is a candidate for a future phase — fixing it properly requires wiring a
  `SkillId → ReRollSource` lookup into the reroll-resolution path, not a per-file change.

## Merge-time reconciliation (all conflicts confined to `skill_id.rs`)

Every batch touching the shared `SkillId::properties()` table produced merge conflicts against
sibling batches also editing it, resolved by taking the union of both sides' property lists,
verified against Java source where the correct union was ambiguous:

- **`BallAndChain`**: found and removed a pre-existing (pre-dating this phase), invented
  `"blocksLikeChainsaw"` property that does not exist in *any* of the bb2016/bb2020/bb2025 Java
  `postConstruct()` methods (confirmed by reading all three directly) — replaced with the verified
  full property union across all three editions.
- **`HailMaryPass`/`ShotToNothing`**: one batch corrected `HailMaryPass` from the wrong
  `canGainHailMary` to the Java-correct `canPassToAnySquare`, but initially left `ShotToNothing`
  (the skill that actually registers `canGainHailMary` in Java) without the property — added
  during merge, along with fixing a pre-existing test in `step_init_selecting.rs` that had been
  built against the wrong mapping.
- Several smaller overlapping additions (`Incorporeal`, `CloudBurster`, `FuriousOutburst`,
  `RightStuff`, `NoHands`, `MonstrousMouth`, `SecretWeapon`) — straightforward unions, all
  re-verified against Java during reconciliation.

## What's left, in priority order (carried forward from Phase AJ, updated)

1. The ~137 client/server handler files (sampled clean twice across two phases, never fully swept).
2. The systemic `register_reroll_source` gap identified this phase — no skill file's reroll
   registration has any live effect; needs an engine-level wiring decision, not per-file fixes.
3. A small set of documented-not-fixed gaps carried from Phase AJ: `sequences.rs`'s
   `Rules`-ignoring kickoff sequences; BB2025 `step_apothecary.rs`'s missing "Getting Even"/"Raise
   Dead" subsystems; `step_select_blitz_target_end.rs`'s self-documented stub; Tentacles' reroll
   resolving against the wrong team.
4. Parity/integration testing remains the only large, entirely out-of-scope workstream.

**Honest completion estimate: still not behaviorally done.** This is the sixth consecutive phase
(AF, AG, AH, AI, AJ, AK) where fresh re-verification found real bugs — including, this time, an
entire 36-file dead-code discovery in a previously "100%-tracked" pool. File/method coverage
remains ~99.8–99.9%+ by the tracker's own metric, but that metric measures file existence, not
correctness, and six straight phases of finding real bugs on already-"done" files is a strong,
repeated signal that more remain.
