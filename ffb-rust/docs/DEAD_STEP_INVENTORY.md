# Dead-step inventory — re-measured 2026-08-25

Supersedes the 2026-08-19 inventory (136/199), which predated the bomb chain, the star
batches (§§9-11) and the blitz-select chain (§12).

**167 of 200 `StepId` variants reached; 33 dead.**

## How this was measured — and why the obvious method is wrong

Two sweeps, unioned:

1. `FFB_DRIVE_TRACE=1 ffb-parity --uniform --all-rosters --all-editions --seeds 1-3 --no-abort`
   (Rust-only, 270 games) → 147 reached.
2. The **random** (parity-tier) agent over all 30 rosters x 3 editions x seeds 1-10, driven by a
   loop over `--home K --away K --edition E --tier 3` → 161 reached.

Union: 167. Three traps, all of which produced a wrong number first:

- **The uniform agent under-reports star mechanics.** It references the star specials 3 times
  against `random_agent`'s 22 - it does not *declare* most of them, so they cannot dispatch no
  matter what is on the pitch (the sweep had 261 star references). A uniform-only sweep is not an
  inventory.
- **`--all-rosters` / `--all-editions` are silently ignored without `--uniform`.** A run that looks
  like a full sweep quietly measures lineman-vs-lineman only.
- **Seed count dominates the result.** `CatchOfTheDay` fires **21 times in 30 seeds** of wood_elf
  bb2025 and **zero** times in seeds 1-2. Going from 2 to 10 seeds moved 11 steps out of "dead".
  10 seeds is the floor used here and is still not proof of death for a rare mechanic - see
  category E.

**A "dead" list is only as good as its sampling.** Verify per-mechanic on the roster that carries
it before concluding anything is unreachable.

## The 33, classified

### A. Vestigial in BOTH engines — not parity gaps (6)
`EndPlayerAction` `NoOp` `RevertEndTurn` `Bombardier2` `SelectGazeTarget` `SelectGazeTargetEnd`

No Java **generator** pushes any of these (checked against
`ffb-server/.../step/generator/`). In Rust, `StepId::EndPlayerAction` is referenced only from
`step/sequences.rs` and `step/engine.rs` - the pre-driver legacy modules; the live
`EndPlayerAction` generator pushes a sequence of *other* steps and never itself. The gaze twins
were closed on 2026-08-18: `canGazeDuringMove` is registered only by `skill/bb2016/HypnoticGaze`,
so no BB2020 player can declare GAZE.

These are dead code in Java too. Deleting them is a cleanup question, not a fidelity one.

### B. Unreachable by DATA (6)
`InitKickTeamMate` `KickTeamMate` `EndKickTeamMate` `KickTeamMateDoubleRolled`
`PileDriver` `DropActingPlayer`

- The KTM sequence generator is **BB2016-only** in Java (`generator/bb2016/KickTeamMate.java`);
  BB2020/BB2025 route kicks through the shared TTM steps. **No bb2016 roster carries
  "Kick Team-Mate"** (only the bb2020/bb2025 ogre Runt Punter does), so the BB2016 twins cannot
  run. The MECHANIC is live: ogre bb2025 dispatches `ThrowTeamMate` 364 times over 30 seeds and
  the agent declares KICK_TEAM_MATE. This is **not** a regression of commit `60131597`.
- `PileDriver` appears nowhere in `data/` under either name ("Pile Driver" or the older
  "Piling On") - no roster, no star, no drafted team. `DropActingPlayer` is pushed by exactly one
  Java generator, `mixed/PileDriver.java`, so it is the same item.

Reaching either means changing drafted team data - a separate and larger decision (same category
as `DauntlessMultiple`, closed 2026-08-18).

### C. Scoring-gated — BLOCKED on a user tier decision (5)
`AssignTouchdowns` `InitPunt` `EndPunt` `PuntDirection` `PuntDistance`

Out of scope until the agent is allowed to score. Unchanged.

### D. Inducements — parity teams purchase none (6)
`MasterChef` `WeatherMage` `Wizard` `PlayCard` `FanFactor` `PrayerRoll`

The existing "inducement purchasing (7 ids)" backlog item. Needs both harnesses taught to buy
inducements in lockstep - the same shape as the TTM/KTM/interception/bomb campaigns.

### E. Star specials — live in §§9-11, not sampled at 10 seeds (6)
`Treacherous` `LookIntoMyEyes` `InitLookIntoMyEyes` `Swoop` `ThrowARock` `QuickBite`

Recorded LIVE 100/100 in the star campaigns. `CatchOfTheDay` was in this list at 2 seeds and left
it at 10, which is direct evidence the category is a sampling artifact rather than a set of gaps.
**Verify each on its own star's roster with 30+ seeds before treating any as dead.**

### F. Was "unclassified" — now RESOLVED (4), measured 2026-08-25

`EatTeamMate` `KickoffScatterRollAskAfter` `StateMultipleRolls` `ReportStabInjury`

- **`EatTeamMate` is LIVE — not dead.** Measured **10 fires in 315 `AlwaysHungry` rolls** across
  all 17 Always-Hungry roster/edition pairs at 100 seeds (3.2%, against the ~1/36 the rules
  predict: fail the Always Hungry roll AND fail the thrown player's escape roll). It never
  appeared at 10 seeds. `FFB_RNG_STEPS` confirms every `AlwaysHungry` dispatch consumes dice, so
  the mechanic was working the whole time. **Third false "dead" caused by sampling.**
- **`KickoffScatterRollAskAfter` is option-gated, not dead.** `bb2025/Kickoff.java:39-43` picks it
  over `KICKOFF_SCATTER_ROLL` only when the game option `ASK_FOR_KICK_AFTER_ROLL` is enabled.
  Parity games leave it off, so the sibling runs instead - and the sibling IS reached. Reaching
  this means changing a game option, not fixing code.
- **`StateMultipleRolls` is a Rust modelling artifact.** In Java `StepStateMultipleRolls` is not a
  sequence step at all: it is the hook *state* class of
  `AbstractStepModifierMultipleBlock<StepFoulAppearanceMultiple, StepStateMultipleRolls>`. No Java
  code pushes it, because it is not pushable. Rust gave it a `StepId` and a driver entry, so it
  can never dispatch. Cleanup, not fidelity.
- **`ReportStabInjury` — ✅ DRIVEN AND LIVE 2026-08-25** (34 dispatches / 100 seeds of dark_elf bb2020, gate 30/30/30; Rust `24b39a81f`, harness `niels/ffb 14ccfb123`). It *was* dead by HARNESS LOCKSTEP. This is the fifth
  instance of the TTM/KTM/interception/bomb shape. `StepMultipleBlockFork` (both engines) groups
  multi-block targets by `BlockKind` and gives the STAB group its own sequence ending
  `STAB -> HANDLE_DROP_PLAYER_CONTEXT -> REPORT_STAB_INJURY`.

  **CORRECTION (same day):** the first version of this entry credited Rust with building that
  sequence correctly. That is true only of `step/bb2020/multiblock/step_multiple_block_fork.rs`,
  which is a DEAD TWIN - `driver.rs:61` globs `bb2025::mutliblock::*`, so every edition runs the
  BB2025 fork. Reading the bb2020 file to describe live behaviour is exactly the mistake
  `feedback_bb2020_reason_from_live_path` warns about, and it was made here.

  The stab group is **BB2020-only in Java too**: `bb2020/.../StepMultipleBlockFork.java` groups a
  `BlockKind.STAB` bucket, `bb2025/.../StepMultipleBlockFork.java` has none (the live Rust file
  even says so at line 9: "no UsingStab - stab not in multiple block"). So Rust's shared fork
  correctly mirrors BB2025 and **silently drops BB2020's stab path**.

  It never runs because of TWO independent blockers, both of which must be fixed:
  - `ParityRunner:1940-1941` hard-codes `BlockKind.BLOCK` for both targets.
  - Rust's `StepParameter::BlockTargets` carries only player IDs and reconstructs every target as
    `BlockKind::BLOCK` (`step_multiple_block_fork.rs:212`), so the kind cannot survive the
    parameter even if the harness sent it.

  Only ONE drafted team can produce it: bb2020 `dark_elf`, the sole hirer of **Horkon
  Heartripper**, the only star with both Multiple Block and Stab. Measured there at 100 seeds: 33
  `MultipleBlockFork`, 99 `BlockRollMultiple`, 66 `ApothecaryMultiple`, 39 `FoulAppearanceMultiple`,
  1014 `Stab` - and 0 `ReportStabInjury`.

  **To drive it, in this order:**
  1. **Model.** `StepParameter::BlockTargets` is `Vec<String>`; Java's `BLOCK_TARGETS` is
     `List<BlockTarget>`, and `BlockTarget` (already ported faithfully, with `kind`) is discarded.
     Until the parameter carries the kind, any stab group is vacuous by construction.
  2. **Routing.** Edition-gate BB2020's stab group INTO the live shared fork - never by routing to
     the dead bb2020 twin (the rule that blocker 1 of the gaze twins established).
  3. **Harness.** Teach BOTH harnesses in lockstep to choose STAB for a stab-capable blocker
     (`ParityRunner:1940-1941` hard-codes `BlockKind.BLOCK`). Needs a jar rebuild.

  Steps 1-2 are Rust-only and behaviour-neutral (they cannot change a stream while every target is
  BLOCK); step 3 makes it live. Expect real engine bugs then, as the previous four did.

## Summary

Of the 33: **21 are closed** (6 vestigial in both engines + 6 unreachable by data + 5 blocked on
the scoring tier decision + 2 option/modelling artifacts + `StateMultipleRolls`,
`KickoffScatterRollAskAfter`), **7 are sampling artifacts of live mechanics** (the 6 star specials
+ `EatTeamMate`, now proven), **6 are the known inducement campaign**, and **1 is drivable now**:
`ReportStabInjury`.

**`ReportStabInjury` is now LIVE (2026-08-25), and with it the frontier has no actionable items
left.** Everything remaining is closed, blocked on a user decision, or already alive.

It took three separately-gated steps, and splitting them is why it landed clean:
  1. `1284d943f` - BLOCK_TARGETS carries `BlockTarget` (the kind survives the parameter),
  2. `864910484` - BB2020's fork keeps its STAB group and drops BB2025's PICK_UP,
  3. `24b39a81f` + `niels/ffb 14ccfb123` - both harnesses offer the STAB alternative in lockstep.

The previous four campaigns of this shape (TTM, KTM, interception, bomb) each surfaced 6-12 Rust
engine bugs. This one surfaced none, because each prerequisite was measured on its own before the
next was added.

**What remains on the frontier is one GOAL-level decision, not a bug list:** both agents move
exactly ONE SQUARE PER ACTIVATION (measured 1:1, `player_moved_events == activations.Move`), which
alone makes GFI and touchdowns - and therefore the whole scoring/Punt family - structurally
unreachable. See BACKLOG.
