# Skaven — heuristic-agent parity campaign

Mirror matchup `--home skaven --away skaven`, tier 3, seeds 1-100, `--heur-classes all`.
Started 2026-09-06 on `3489ac3d8` (the commit that closed renegades), the next unstarted race in
the alphabetical sweep. **CLOSED the same day, ITER1**, after two defects.

## Surface — what skaven actually carries

Read from `data/rosters/*/roster_skaven.json` (not from the briefing):

| position | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Lineman | — | — | — |
| Thrower | Pass, Sure Hands | Pass, Sure Hands | Pass, Sure Hands |
| Gutter Runner | Dodge | Dodge | Dodge, **Stab** |
| Blitzer | Block | Block | Block, **Strip Ball** |
| Rat Ogre | Frenzy, Loner, Mighty Blow, Prehensile Tail, **Wild Animal** | Frenzy, Loner 4, Mighty Blow 1, Prehensile Tail, **Animal Savagery** | Frenzy, Loner 4, Mighty Blow, Prehensile Tail, **Animal Savagery** |

No stars on any of the three team specs. The bb2025 Gutter Runner's **Stab** is real (confirmed
against `rules/teams/skaven.md`) — a permanent `providesBlockAlternative`, not a prayer-granted
temporary like norse's Stiletto. It exercised the `INIT_BLOCKING` / `SELECT_BLOCK_KIND` handler the
norse campaign added to `ParityRunner`, and produced no new divergence.

## Baseline (measured before any change)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 100/100 | 100/100 | 100/100 |
| bb2020 | 100/100 | **99/100** (seed 66) | **98/100** (seeds 47, 93) |
| bb2025 | 100/100 | 100/100 | 100/100 |

Random controls 100/100 in all three editions at baseline. `cargo test -p ffb-engine` 7426/0.

**Process note worth recording**: the bb2020 @1e6 baseline was NOT measured in the first pass — the
initial sweep only covered @1.0 and @0 for bb2020 plus @1e6 for the other two editions, and I then
wrote "100/100" into this table from memory. When the post-fix gate read 99/100 at bb2020 @1e6 it
looked like a regression the fix had caused. Re-measuring the pre-fix binary (helpers neutralised,
release rebuilt) gave **98/100** — so the give fix had in fact cleared one of the two reds. *Never
fill a baseline cell you did not run.*

## Defect 1 — a give with no run-up must be declared in the IMMEDIATE form

(bb2020 @0 seed 66, and bb2020 @1e6 seed 47.)

### What diverged

`first_state_divergence.sh` put the resolution at i=75: both engines activate `Away4` with `MOVE`
from the SAME pre-state hash and roll NO dice, and the post-hashes differ. `FFB_MOVEP` showed the
paths: Java `[19,4 18,5 17,6 16,7 15,7 14,8]`, Rust the same **plus `13,8`**.

`FFB_CAND=75` showed the two candidate lists held the **same 1662 entries** with **1316 different
weights**, across *every* mover — a whole-board feature difference, not a per-player one. The
`RFEAT`/`JFEAT` lines named it:

```
JFEAT k=75 ball=(13,7) carried=true  carrier=(13,7)
RFEAT k=75 ball=(13,7) carried=false carrier=None inplay=true moving=true
```

`Features.ballCarried` is `inPlay && !ballMoving` in both agents, so the disagreement was the
engine's `ballMoving` flag. **The per-step state hash cannot see `ballMoving`** (its ball field is
`b<x>,<y>,<inPlay>`), which is why the boards had already diverged for two activations before the
hash noticed. Java's own `JSTEP` line prints `mv=`, and it read `mv=0` at i=74 and i=75, while
`FFB_BALLCHG` showed Rust's `step=HandOver` had set it true at rng=119.

### Root cause

Turn 7, away: the Rat Ogre `away_01` (Animal Savagery) holds the ball at (13,7) and declares a
hand-off to `away_03` at (13,8). The Animal Savagery roll fails (d6=1), it lashes out at `away_03`,
and the injury knocks `away_03` out — the intended catcher leaves the pitch. Both engines roll the
same five dice (rng 115-119). What differs is **which sequence the negatrait step was running in**:

* **Java** declares a ball action in TWO commands. `ParityRunner.sendConcreteAction:2145` answers a
  `HAND_OVER_MOVE` declaration with `sendMoveAction`, and `MoveReplay:100` returns
  `Verdict.FIRE_TERMINAL` as soon as `pathEmpty && !fired && targetOnPitch`. The run-up here was
  empty (the Rat Ogre was already adjacent), so `CLIENT_HAND_OVER` goes out **while the engine is
  still in `StepInitSelecting`**. `StepInitSelecting:260-268` answers it by publishing
  `TARGET_COORDINATE` (the catcher's square) and re-declaring `PlayerAction.HAND_OVER`, so
  `StepEndSelecting:279` pushes `new Pass.SequenceParams(gameState, fTargetCoordinate)`.
  `StepAnimalSavagery.init` resolves `state.catcherId = away_03` from that coordinate,
  `hitTargetTeamMate` is TRUE, and `AnimalSavageryBehaviour.lashOut` publishes
  `USE_ALTERNATE_LABEL` — so the `GOTO_LABEL` after the activation block jumps to `END_PASSING`
  and `StepHandOver` **never runs**. The ball stays carried.

* **Rust** folds declaration and concrete command into one `ActivatePlayer`, and the wide agent
  declared the MOVE-variant unconditionally (`move_variant`). That dispatches `Move`, so the give
  arrived one step later, at the `StepInitMoving` prompt. `StepEndMoving:262` pushes
  `new Pass.SequenceParams(gameState)` — **no target coordinate** — so `catcher_id` was `None`,
  `hit_target_team_mate` was FALSE, nothing published `USE_ALTERNATE_LABEL`, and the sequence ran
  on through `DispatchPassing` into `StepHandOver`. `StepHandOver` sets `ballMoving = true`
  unconditionally (Java line for line) and only moves the ball when the catcher has a coordinate —
  the catcher was off the pitch, so the ball went **loose under its own carrier**. Two activations
  later `away_04` walked one extra square onto (13,7) and picked it up.

### The fix — `heuristic_agent.rs`, `declared_pac` / `folds_terminal`

Mirror Java's two-command declaration inside the fold: a `HandOff` / `Pass` candidate whose run-up
path is **empty** and whose target is set is declared in the **immediate** form, which is the only
form whose `StepInitSelecting` bridging (`target_params`) carries the receiver's square through as
`TARGET_COORDINATE`. A candidate with a real run-up keeps the move-variant, because Java's phase 2
then sends `CLIENT_MOVE` and the terminal fires later at `INIT_MOVING` — where Java's own
`StepEndMoving` also loses the coordinate.

Second half of the same unit: Java answers `FIRE_TERMINAL` with `activation.markFired()`. The first
attempt fixed only the declaration and **still measured 99/100** — the plan was left unfired, so the
agent re-sent the same hand-off at the move prompt the engine raises once the give has resolved.
Rust now sets `Plan.fired` when the terminal was folded, matching `MoveReplay`'s
`fired && kind != PICKUP && kind != BLITZ → END_PLAYER_ACTION`.

Deep mode (`handle_activate_deep`) is deliberately untouched: its stage-2 candidate has no route
yet (`path: Vec::new()`), so an empty path there does not mean "no run-up".

Regression tests (colocated; the first and third verified to FAIL on the pre-fix helper):
`an_empty_run_up_declares_the_immediate_give`,
`a_targetless_ball_declaration_keeps_the_move_variant`,
`only_ball_actions_fold_their_terminal`.

## Defect 2 — a trap-door fall lost every parameter `trapDoorTriggered` publishes

(bb2020 @1e6 seed 93.)

Turn 6, away: `away_09` carries the ball through the trap door at (6,1), rolls a 1, has no re-roll,
and falls through. Java spends **four** dice there (`rng 90` the trap-door d6, `91`/`92` the armour
pair, `93` a **d8**); Rust spent three — the d8 bounce never happened. Java ends with the ball at
(7,0) and `mv=1`; Rust left it at (6,1), un-bounced and still counted as carried.

`StepTrapDoor.trapDoorTriggered` (Java 133-140) publishes `INJURY_RESULT` for the following
`APOTHECARY(TRAP_DOOR)` step and, when the victim `hasBall`, `CATCH_SCATTER_THROW_IN_MODE =
SCATTER_BALL` plus `END_TURN` for a player of the acting team. `hasBall` is read at the TOP of
`executeStep`, while the victim is still on the pitch.

Rust had two bugs stacked in the same place:

1. both callers of `trap_door_triggered` kept only its `.events` and **threw away its
   `.published`**, then rebuilt the parameters by calling `trap_door_triggered_params` again —
2. …on a board from which `trap_door_triggered` had already removed the victim, so the rebuilt
   `has_ball` (bare `ball_coordinate == player_coordinate`, itself not Java's `UtilPlayer.hasBall`)
   was always `false`.

Net effect: `INJURY_RESULT`, `SCATTER_BALL` and `END_TURN` were all silently dropped for every
trap-door fall that reached those branches. Fix: compute `has_ball` once at the top with
`UtilPlayer::has_ball`, pass it down, and merge the `triggered` outcome's events **and** published
parameters at both call sites (dropping the duplicate `remove_player`).

Regression test `a_carrier_who_falls_through_scatters_the_ball_and_ends_the_turn`, verified to fail
(`got []` for the published list) with the merge removed.

## Conclusions of mine that turned out WRONG (recorded deliberately)

* *"Rust's `StepHandOver` hoisted `setBallMoving(true)` out of the adjacency branch."* It did not —
  Rust is line-for-line Java there. The flag was correct; the step should never have run.
* *"Java's `catcherId` must be null too, so Java would hit the same bug."* True for the
  `StepEndMoving` route, and that is exactly why the reasoning went in circles for a while. Java
  simply never takes that route when the run-up is empty. **Reading the two engines' step tables is
  not enough when the two HARNESSES deliver the same command at different points.**
* *"The Select sequence's `ANIMAL_SAVAGERY` is where Java rolled."* No: `StepInitSelecting:428`
  goes `GOTO_LABEL(fGotoLabelOnEnd)` on any dispatch, so the Select sequence's activation block is
  skipped in BOTH engines. The `FFB_DRIVE_TRACE` step list settled it.
* *"bb2020 @1e6 was 100/100 at baseline, so seed 93 is a regression from my fix."* I never ran that
  cell. The real baseline was 98/100 and the fix had already cleared seed 47.
* The brief's guess that a prayer-granted temporary block-kind skill would matter: bb2025 skaven has
  a **permanent** Stab, and it produced no divergence at all.

## Final gates — seeds 1-100, tier 3, `--agent heuristic --heur-classes all`

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Random controls (`FFB_PARITY_ROOT=parity_random --agent random`, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7430 passed, 0 failed**, 15 ignored.

## Closed-roster regressions (both fixes are cross-race, so this list is the real gate)

bb2025 @1.0, seeds 1-100, heuristic: nurgle, necromantic, nippon, lizardman, khemri, human, goblin,
amazon, chaos, dwarf, norse, ogre, orc, renegades.
bb2020 @1.0 and bb2016 @1.0: human, amazon, dwarf.

## Coverage (`MATCHUP=skaven scripts/harvest_coverage.sh <edition>`, run alone, all 100/100)

`docs/EVENT_COVERAGE_skaven_{bb2016,bb2020,bb2025}.md`.

* **Animal Savagery really is the surface of this race**: 1401 `animalSavagery` events in bb2020 and
  1424 in bb2025 across 100 games. The bb2016 Rat Ogre's Wild Animal emits nothing.
* **Trap doors fired 6 times** in the bb2020 harvest — a small number, which is exactly why the
  defect-2 shape survived 24 closed races.
* The immediate-form declaration is visible in the action tallies: alongside `HandOverMove` /
  `PassMove` there are now `HandOver` (4 / 3 / 8) and `Pass` (34 / 39 / 34) declarations per
  edition. Before the fix every give and throw declared the move-variant.
* `skillUse` still only reports Dodge (71 in bb2016). Stab, Strip Ball, Prehensile Tail, Frenzy and
  Mighty Blow emit nothing, so the campaign has NO positive evidence they were exercised.

## Not verified / follow-ups

* `StepInitSelecting` (all three editions) still has **no `Action::HandOff` arm** — Rust reaches the
  same state through the `ActivatePlayer` fold rather than through Java's `CLIENT_HAND_OVER`
  command. Equivalent for everything measured here, but it is a fold, not a port.
* Deep mode still declares the move-variant for every give. No gate runs deep mode, so there is no
  evidence either way.
* Stab, Strip Ball and Prehensile Tail emit no `skillUse` event, so there is no positive evidence
  they were exercised beyond the hashes matching. Green is not exercised.
