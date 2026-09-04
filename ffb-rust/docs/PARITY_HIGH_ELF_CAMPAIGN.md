# High Elf — heuristic-agent parity campaign — 🏁 **CLOSED 2026-09-04, nine gates 100/100**

**Goal**: high_elf v high_elf, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with
`MATCHUP=high_elf`. Started 2026-09-04, after goblin (`fc58b0e1c`) and halfling (`e6571447c`) closed.

## Surface

Roster in all three editions, but **bb2025 is a different roster**, not a re-skin — that is where
the work is:

- bb2016 (11): Blitzer ×2, Catcher ×4, Thrower ×2, Lineman ×3.
- bb2020 (12): Thrower ×1, Blitzer ×1, Catcher ×1, Lineman ×9.
- bb2025 (11): **Dragon Prince ×2, White Lion ×2, Phoenix Warrior ×2**, Lineman ×5.
  - Phoenix Warrior: **Cloud Burster**, Pass, **Safe Pass**
  - Dragon Prince: Block, **My Ball**, **Steady Footing**
  - White Lion: **Claws**, Wrestle

Cloud Burster / My Ball / Safe Pass / Steady Footing are largely unexercised surface for this sweep
(amazon's campaign touched a "Safe Pass reroll order"; the rest are new here).

## Baseline (2026-09-04, measured on `e6571447c`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 | 99 | 98 |
| bb2020 | **100** ✅ | 99 | **100** ✅ |
| bb2025 | **0** | 86 | **0** |

221 reds across the nine gates; two gates (bb2020 @1.0 and @1e6) already green.

**READ THE bb2025 SIGNATURE CAREFULLY — it is the whole race.** bb2025 is `0/100` at BOTH sampled
scales yet `86/100` at argmax. Argmax consumes zero sampler draws, so:
- the engine's *resolution* is mostly right (86/100 at @0), and
- something makes **every single game** diverge the moment sampling is switched on.

That is the fingerprint of a **prompt/draw-count** divergence, not a dice or resolution bug: a
dialog Rust offers that Java never does (or vice versa), fired so early and so often that no seed
survives. The first move is therefore NOT `first_state_divergence.sh` on a seed — it is
`FFB_DRAWS=1` on ONE bb2025 @1.0 seed, comparing the `RDRAW`/`JDRAW` prompt-class sequences and
their running totals to find the first prompt whose class or draw count differs. Then
`FFB_CANDSUM` (`RSUM`/`JSUM` `n=`/`draws=`) at that activation. The new bb2025 skills above
(Cloud Burster, Safe Pass, My Ball, Steady Footing) are the obvious suspects for an extra dialog.

Fix that one prompt and both bb2025 sampled gates should move together; the 14 reds at @0 are a
separate, smaller family to clean up after. bb2016 (6 reds) and bb2020 (1 red) are nearly green.

## ITER1 (2026-09-04) — the Steady Footing dialog and its re-roll offer

**bb2025: 0 / 86 / 0 → 95 / 95 / 96.** Two bugs, both in
`crates/ffb-engine/src/step/bb2025/shared/step_steady_footing.rs`, both about PROMPTS rather than
resolution — exactly the fingerprint the baseline predicted.

### How they were named

`FFB_DRAWS=1` on bb2025 @1.0 seed 1, comparing cumulative totals at the classes both engines emit:

```
RDRAW cls=followup total=14                                   JDRAW cls=FOLLOWUP_CHOICE total=14
RDRAW cls=move     total=16     <- no Rust prompt              JDRAW cls=SKILL_USE total=16 skill=SteadyFooting pid=teamHighElfParity25Home1
```

The totals agree up to the follow-up choice and part company at the very first Steady Footing
fall. Java raises `DialogSkillUseParameter`, `ParityRunner`'s `SKILL_USE` arm routes it through
`heuristic.useSkill("SteadyFooting")` (`HeuristicDriver.java:144`, `_ -> 0.50`), and that spends
**two sampler draws**. Rust's `execute_step` had a placeholder:

```rust
if self.use_skill.is_none() { self.use_skill = Some(true); }   // "auto-accept so the random agent rolls"
```

so Rust never asked, never spent the draws, and the two random streams split on the first Dragon
Prince fall of every game. That is why **every** bb2025 sampled seed was red while argmax (which
consumes no draws) was 86/100.

**Fix 1** — raise the prompt, 1:1 with Java's `showDialog(...); setNextAction(CONTINUE)`; the
answer already came back through `handle_command`'s `Action::UseSkill` arm. The offered skill is
now RESOLVED (`getSkillWithProperty(canAvoidFallingDown)`) rather than the `SteadyFooting`
constant, for the dialog and for `ReportSkillUse`, and the eligibility guard is now Java's
`!skill.isPresent()` instead of the property union (bb2025 `BallAndChain` lists
`canAvoidFallingDown` only as a `registerConflictingProperty`, so SteadyFooting is in fact the
only skill that registers it).

That alone took bb2025 @1.0 from **0 → 30** and @1e6 from **0 → 49**. `FFB_DRAWS` on the new
lowest red (seed 4) then showed the second fault:

```
RDRAW cls=skill  total=26 skill=SteadyFooting pid=away_02      JDRAW cls=SKILL_USE total=26 skill=SteadyFooting pid=teamHighElfParity25Away2
RDRAW cls=reroll total=28 src=TRR action=STEADY_FOOTING        (no JDRAW - Java raises nothing)
```

**Fix 2** — Java's failure branch calls the **PLAYER** overload,
`UtilServerReRoll.askForReRollIfAvailable(gameState, player, STEADY_FOOTING, 6, false)`, and
`RollMechanic.isTeamReRollAvailable` gates the TRR on `actingTeam.hasPlayer(pPlayer)`. `away_02`
was a blocked DEFENDER while home was playing, so Java offers no re-roll dialog at all. Rust
called the ACTING-PLAYER overload (`ask_for_reroll_if_available`), which skips that membership
gate and additionally resolves a re-roll source from the *acting* player's skills by action — two
extra sampler draws on every defender's failed Steady Footing. Switched to
`ask_for_reroll_if_available_for(game, Some(player_id), ...)`, the existing 1:1 player overload.

bb2025 @1.0 **30 → 95**, @0 **86 → 95**, @1e6 **49 → 96**.

### Gates measured this iteration (all nine, seeds 1-100 tier 3, fresh JVM, no `--reuse-java`)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 (=) | 99 (=) | 98 (=) |
| bb2020 | **100** ✅ (=) | 99 (=) | **100** ✅ (=) |
| bb2025 | **95** (was 0) | **95** (was 86) | **96** (was 0) |

221 reds → **21**. bb2016 and bb2020 are unchanged, as expected: the file is bb2025-only.

Tests: `cargo test -p ffb-engine` **7411 / 0**, `ffb-model` **2802 / 0**. Three new colocated
tests written from the Java: the undecided offer must ASK and draw no die, the answer arrives as
`Action::UseSkill` and a decline fails the step, and a non-acting-team faller gets NO re-roll
offer while the same faller on the acting team does.

Closed-roster regressions, bb2025 (the only edition this file serves), seeds 1-100 tier 3 @1.0:
goblin **100**, halfling **100**, amazon **100**, lineman **100**, dwarf **100**, chaos **100**,
chaos_dwarf **100**, chaos_pact **100**, dark_elf **100**, dark_elf_league_fumbbl **100**, elf
**100**. goblin and halfling additionally re-run at @0 and @1e6: **100 / 100** each.

### Frontier for ITER2

- bb2025 @1.0 reds: **16, 37, 56, 62, 80**
- bb2025 @0 reds: **22, 73, 86, 91, 99**
- bb2025 @1e6 reds: **54, 72, 88, 100**
- bb2016 reds: @1.0 **49, 91, 94**; @0 **77**; @1e6 **2, 80** — untouched all iteration
- bb2020 reds: @0 **71** only

Next step: `frontier.sh bb2025` over the 14 remaining bb2025 reds and read the families by
declared action; the bb2025 sampled and argmax red sets are now the same size, so the remaining
faults are most likely resolution, not draw counts. The still-unexercised bb2025 surface is
**Cloud Burster**, **My Ball** and **Safe Pass** (Steady Footing is now driven live).

## ITER2 (2026-09-04) — Cloud Burster, and two bb2016 pass faults

**bb2025 95/95/96 → 100/100/100. bb2016 97/99/98 → 100/100/100. bb2020 100/99/100 unchanged.**
21 reds → **0**. Four engine bugs, all in the pass sequence. The first three were committed as
`175ce3939` (which left one red); the fourth, below, closed the race.

### 1. Cloud Burster: a pass that cannot be intercepted

`frontier.sh bb2025` over ITER1's 14 reds returned one family, not many:

```
seed 16   idx 54    R t7 away Activate(away_06,PassMove)
seed 37   idx  9    R t2 away Activate(away_05,PassMove)
seed 56   idx 67    R t7 home Activate(home_02,Move)
seed 62   idx 18    R t2 away Activate(away_06,PassMove)
seed 80   idx 134   R t6 away Activate(away_06,PassMove)
```

Seed 37 named it. `FFB_DICE_TRACE` diffs clean to `pos=52` and then parts on the SIDES of a single
die — Rust rolls a d8 where Java rolls a d6:

```
Rust  pos=53 sides=8 result=7
Java  pos=53 sides=6 result=5  caller=… DiceRoller.rollSkill StepPass.executeStep:221 StepPass.start:132
```

`FFB_BALLCHG` completed the picture: `step=ResolvePass (13,6) -> (12,5)` — away_05's pass to its own
teammate at (13,6) ended on **home_00** at (12,5), so Rust turned the ball over and flipped to the
home turn while Java played on with `Away2 HAND_OVER_MOVE`. Rust had run an **interception** Java
never offered.

`UtilPassing.findInterceptors` (ffb-common, shared by every edition):

```java
boolean passesNotIntercepted = pThrower.hasSkillProperty(NamedProperties.passesAreNotIntercepted);
for (Player<?> otherPlayer : otherPlayers) {
  if (passesNotIntercepted && !UtilCards.hasSkillToCancelProperty(otherPlayer,
          NamedProperties.passesAreNotIntercepted)) {
    continue;   // Cloud Burster 2025: no interception unless cancelled
  }
  ...
}
```

The thrower is a **Phoenix Warrior** — BB2025 Cloud Burster registers `passesAreNotIntercepted`, so
Java finds NO candidates at all. `bb2025/pass/step_intercept.rs::find_interceptors` had no such
gate (the bb2016 twin has one, but `driver.rs` has a **single** `StepId::Intercept` arm, so the
bb2025 file is the one that runs for every edition and the bb2016/bb2020 twins are dead). Added
the gate, asked **edition-aware** (`has_skill_property_in` / `has_skill_to_cancel_property_in`)
because BB2020's Cloud Burster registers `canForceInterceptionRerollOfLongPasses` instead and must
NOT suppress interceptions.

That single gate closed **all fourteen** remaining bb2025 reds across the three scales.

### 2. bb2016 has no saved-fumble label

bb2016 @1.0's three reds were not hash diffs at all — `frontier.sh` reported
`no hash diff but LENGTH differs: rust 19 java 164`. `FFB_DRIVE_TRACE` ended mid-game on
`DRIVE step=Pass`, and a temporary probe on `StepPass`'s outcome printed
`result=Some(SAVED_FUMBLE) action=GotoLabel label=Some("")`: the driver had jumped to the empty
label and stopped dispatching for the rest of the game.

`bb2016/StepPass.handleFailedPass` ends a SAVED_FUMBLE with
`setNextAction(GOTO_LABEL, state.goToLabelOnEnd)`, and bb2016's `GOTO_LABEL_ON_END` *is*
`IStepLabel.END_PASSING`; its `init` does not even accept `GOTO_LABEL_ON_SAVED_FUMBLE` (bb2020 and
bb2025 both throw without it). Rust's bb2016 generator deliberately re-points `GOTO_LABEL_ON_END`
at `RESOLVE_PASS` (so an accurate pass reaches the catch), so the saved-fumble target has to be
named separately — `generator/bb2016/pass.rs` now supplies
`GotoLabelOnSavedFumble(END_PASSING)`. The shared step is untouched: it still routes every edition
off the same field.

### 3. bb2016's `bombAction` flag is INVERTED

With the stall gone, seed 94 became a real divergence: Rust kept the home turn, Java turned over.
`bb2016/StepPass:172` calls

```java
state.result = mechanic.evaluatePass(game.getThrower(), roll, passingDistance, passModifiers,
    PlayerAction.THROW_BOMB != game.getThrowerAction());
```

— the `bombAction` argument is the **inverse** of `isBomb`, and `bb2016/PassMechanic` is the only
mechanic that reads it:

```java
} else if (isModifiedFumble(roll, distance, modifiers)) {
    if (thrower.hasSkillProperty(NamedProperties.dontDropFumbles) && !bombAction) {
        return PassResult.SAVED_FUMBLE;
    } else { return PassResult.FUMBLE; }
}
```

So on a REGULAR bb2016 pass the flag is TRUE and **Safe Throw never saves a modified fumble** —
only a `THROW_BOMB` (never `HAIL_MARY_BOMB`) makes it false. bb2020/bb2025 pass `isBomb` and then
ignore it. Rust passed `is_bomb` at the shared call site, inverting the meaning, so every bb2016
Safe Throw thrower turned a modified fumble into a SAVED_FUMBLE that Java scores as a plain FUMBLE:
Java scattered the ball off the thrower and turned the drive over, Rust kept it and played on.
Edition-gated the flag at the call site. All three bb2016 reds (49, 91, 94 — each a `PassMove` by a
Thrower) were this one bug.

### Gates measured this iteration (all nine, seeds 1-100 tier 3, fresh JVM, no `--reuse-java`)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ (was 97) | **100** ✅ (was 99) | **100** ✅ (was 98) |
| bb2020 | **100** ✅ (=) | 99 (=) | **100** ✅ (=) |
| bb2025 | **100** ✅ (was 95) | **100** ✅ (was 95) | **100** ✅ (was 96) |

Tests: `cargo test -p ffb-engine` **7414 / 0**, `ffb-model` **2802 / 0**. Three new colocated tests written from the Java: a
BB2025 Cloud Burster thrower has no interceptors while a BB2020 one does; a bb2016 saved fumble is
routed to END_PASSING by the generator; a bb2016 REGULAR pass's modified fumble is not saved by
Safe Throw while a THROW_BOMB's is.

### 4. The last red: a BB2020 PASS_MOVE built its sequence as BB2025

bb2020 @0 seed 71 — `idx 200 R t7 away Activate(away_01,PassMove)`, at argmax, so a resolution
fault. Java's dice callers name every die in the window:

```
pos=162 d6=1  StepPass.executeStep:214  StepPass.start:132          <- the pass roll
pos=163 d6=4  StepPass.executeStep:214  StepPass.handleCommand:149  <- its re-roll
pos=164 d6=6  StepIntercept.intercept:200 … StepIntercept.handleCommand:113
pos=165 d6=1  StepIntercept.intercept:200 … StepIntercept.start:95   <- a SECOND, re-pushed intercept
pos=166 d8=3  StepCatchScatterThrowIn.bounceBall:680 … executeStep:369   <- case SCATTER_BALL
pos=167 d6    StepPickUp.pickUp:188
```

That second `StepIntercept` entered from a fresh `start`, not from `handleCommand` — it is
**BB2020 Cloud Burster**, the other half of the same skill. `CloudBursterBehaviour` registers a
whole standalone step at `@StepHook(HookPoint.PASS_INTERCEPT)`; on a LONG_PASS/LONG_BOMB it
reports, sets `deflectionSuccessful = false` and **re-pushes a fresh INTERCEPT**, forcing the
interception to be re-rolled. away_01 is a Phoenix Warrior; BB2020's Cloud Burster registers
`canForceInterceptionRerollOfLongPasses` where BB2025's registers `passesAreNotIntercepted` —
the same skill, two entirely different mechanics. So in Java the natural 6 at `pos=164` deflects,
Cloud Burster forces the re-roll, `pos=165` comes back a 1, the deflection is **undone**, and the
ball — never having moved onto the interceptor — takes `case SCATTER_BALL`, bounces once and is
picked up. Rust kept the first deflection (`FFB_BALLCHG step=ResolvePass (13,8) -> (9,4)
(rng=164)`, i.e. between dice 164 and 165, so nothing ran in between), spent `pos=165` on the
deflected CATCH, failed it and took `FAILED_DEFLECTION_CONVERSION -> THREE_SQUARE_SCATTER` — the
three d8 at 166-168 — plus a further bounce at 169.

The cause is the **edition-gated-generator-default trap**, one push site over from the
`MoveParams::default()` case the campaign rules already warn about:

```rust
// crates/ffb-engine/src/step/bb2025/move_/step_end_moving.rs
PlayerAction::HandOver | PlayerAction::HandOverMove | PlayerAction::Pass
| PlayerAction::PassMove | PlayerAction::HailMaryPass => {
    Some(Pass::build_sequence(&PassParams::default()))     // <- rules = Bb2025, ALWAYS
}
```

`generator/bb2025/pass.rs` uses `params.rules` for
`seq.insert_hooks(params.rules, HookPoint::PassIntercept, …)`, and the hook set is per-edition
(BB2016 → `StepSafeThrow`, BB2020 → `StepCloudBurster`, BB2025 → nothing, matching Java's own
`StepFactory`). `PassParams::default()` hard-codes `rules: Rules::Bb2025`, so the insertion was an
unconditional **no-op for every pass that reaches the sequence through `StepEndMoving`** — every
PASS_MOVE and hand-over move. Its sibling `bb2025/shared/step_end_selecting.rs:346` already passes
`game.rules`, which is why a plain PASS was fine and a PASS_MOVE was not, and why `StepCloudBurster`
had never dispatched in any BB2020 game.

Fix: `PassParams { target_coordinate: None, rules }` — `rules` is already threaded into
`push_sequence_for_player_action` for exactly this reason. Colocated test
`a_bb2020_pass_move_sequence_contains_the_cloud_burster_hook` asserts the hook is present for
BB2020, `StepSafeThrow` for BB2016 and neither for BB2025, across PASS_MOVE, PASS and
HAND_OVER_MOVE.

## Final result — high_elf CLOSED

**Nine gates, seeds 1-100 tier 3, fresh JVM, no `--reuse-java`, all re-measured on the final
binary:**

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2025 | **100** ✅ | **100** ✅ | **100** ✅ |

Baseline was 97/99/98, 100/99/100, 0/86/0 — **221 reds → 0**.

Tests: `cargo test -p ffb-engine` **7416 / 0**, `ffb-model` **2802 / 0**; four new colocated tests
written from the Java.

**Closed-roster regressions** (all re-run on the final binary, seeds 1-100 tier 3): goblin and
halfling × bb2016/bb2020/bb2025 × @1.0/@0/@1e6, plus amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact, dark_elf, dark_elf_league_fumbbl and elf × three editions @1.0 — **45 runs, every one
100/100**. (The three fixes of `175ce3939` were separately gated against the same 45-run set before
that commit; both sweeps are clean.)

**Random controls** (`FFB_PARITY_ROOT=parity_random`, `--agent random`, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **99/100** (seed 72).

That bb2025 random red is **pre-existing and not caused by this campaign** — verified, not assumed:
seed 72 fails identically on the committed `175ce3939` binary AND with the Cloud Burster
interception gate temporarily forced off (`passes_not_intercepted = false`, rebuilt and re-run),
producing the same `step 297 … rust=None` early end each time. It is a *random-agent* divergence in
bb2025 high_elf that predates ITER1; the heuristic gate is 100/100 on the same matchup at all three
scales. It should be picked up as its own item rather than folded into this race.

**Coverage harvested** for all three editions (`scripts/harvest_coverage.sh`, run alone):
`docs/EVENT_COVERAGE_high_elf_bb2016.md`, `_bb2020.md`, `_bb2025.md`, each from a 100/100 run.

### What generalises from this race

- **One skill, two mechanics.** BB2025 Cloud Burster (`passesAreNotIntercepted`, suppresses the
  interception entirely) and BB2020 Cloud Burster (`canForceInterceptionRerollOfLongPasses`, forces
  a re-roll of a successful one) share a name and nothing else. Both halves were dead in Rust and
  each cost a gate. Check `SkillId::properties_for` per edition before assuming a skill is ported.
- **The dead-file trap bit again.** `driver.rs` has ONE `StepId::Intercept` arm, so
  `bb2016/pass/step_intercept.rs` — which already had the `passesAreNotIntercepted` gate — is a
  dead twin and its gate never ran anywhere.
- **`..Default::default()` on an edition-gated params struct is still the highest-yield bug class
  in this codebase.** This is at least the third instance. Any `XxxParams::default()` at a push
  site where `rules` is in scope is a defect until proven otherwise.
- **A `LENGTH differs` row in `frontier.sh` is a STALL, not a divergence** — chase it with
  `FFB_DRIVE_TRACE` (which simply stops on the offending step) rather than the state hash, and
  suspect an unset goto label.
