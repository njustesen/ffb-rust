# Halfling — heuristic-agent parity campaign

**Goal**: halfling v halfling, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with
`MATCHUP=halfling`. Started 2026-09-04, immediately after goblin closed (`fc58b0e1c`).

## Surface

Roster in ALL THREE editions. Two Treemen every edition — this race is dominated by the
**Throw Team-Mate / Right Stuff** machinery plus the Treeman negatrait:

- bb2016: Treeman ×2, Halfling ×12.
- bb2020: Treeman ×2, Halfling Catcher ×2, Halfling Hefty ×2, Halfling ×7.
- bb2025: Treeman ×2, Halfling Catcher ×2, Halfling Hefty ×2, Halfling ×8.
- bb2025 Treeman: Mighty Blow, Stand Firm, Strong Arm, **Take Root**, Thick Skull,
  **Throw Team-mate**, **Timmm-ber!**. The halflings are Stunty/Dodge/Right Stuff.

Prior art that matters: goblin ITER19/ITER21 (just closed) both landed in exactly this machinery —
the bb2020 blitz-block's second `GO_FOR_IT` carrying `BALL_AND_CHAIN_GFI`, and BB2020's TTM landing
resolving `dropPlayer` INLINE (so the chain injury REPLACES the hit injury in Java's parameter map,
where bb2025 defers it through a `SteadyFootingContext`). Halfling has no Ball & Chain, but it does
have TTM landings on both editions' paths, so the BB2020-vs-BB2025 landing shape is the first place
to look if bb2020 is weak. `Take Root` is a Select-sequence negatrait (like Really Stupid / Bone
Head) and `Timmm-ber!` is a stand-up assist — both are new surface for this sweep.

## Baseline (2026-09-04, measured on `fc58b0e1c`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 | 99 | 99 |
| bb2020 | 98 | **100** ✅ | 95 |
| bb2025 | 84 | 93 | 90 |

45 reds across the nine gates; one gate (bb2020 @0) already green. **bb2025 is the weak edition**
(84/93/90 = 33 of the 45). Note bb2025 is red even at @0 (argmax), so at least part of the bb2025
fault is candidate CONTENT/eligibility or a genuine engine divergence — not only a sampled
draw-count split. bb2016 and bb2020 are within a few seeds of green at every scale.

Starting far healthier than goblin did (goblin's baseline was 0/0/2, 17/85/15, 0/69/3).

## ITER1 — the Catch hook's `stopProcessing` was dropped in bb2025 (7 of 7 bb2025 @0 reds, one unit)

**Frontier.** bb2025 @0 had 7 reds: 14, 33, 37, 44, 49, 86, 92. `first_state_divergence.sh` named
the SAME resolving activation for **all seven**: `Activate(*_02, ThrowBomb)` — jersey 2 is the star
**Cindy Piewhistle** (Bombardier), fielded by the bb2025 halfling team spec. One family, one unit.

**Mechanism** (seed 92, step 24, home Cindy throws at home_04 (7,1)).

* `FFB_DICE_TRACE` first divergence at `pos=57`: Java `rollInjury … InjuryTypeBombWithModifier`,
  Rust `sides=8`. The two engines agreed on `pos=53` (StepPass) and `pos=54` (a d6) and then split.
* `FFB_IDSTATE` at the next step: Java changed home4 → box, home9 → prone, home11 → casualty
  (a bomb exploding at (7,1)); Rust changed only home5 → box.
* `FFB_TRACE` gave the decisive pair. Java: `JAVA_CATCH catcher=…Home4 roll=2 min=3 ok=false
  mode=CATCH_ACCURATE_BOMB` and **no re-roll dialog** — `JSTATE i=25` goes straight from
  `INIT_SELECTING mode=REGULAR` to `APOTHECARY mode=BOMB_HOME`. Rust: `LOOP applied=Activate(
  home_02,ThrowBomb) prompt_after=ReRollOffer{CATCH}` → `applied=UseReRoll` → `BombRethrow{home_04}`
  → `Pass(7,3)`. Rust spent a team re-roll, home_04 caught the bomb and re-threw it.
* `RREROLLW`/`JREROLLW` confirm the asymmetry as a general one, not a one-off: Rust raised a CATCH
  offer at turn 2 and turn 4 that Java never raised. Java's only bomb-catch re-roll in the game was
  the one whose catcher was the **Treeman** home_01 — a player with **no Catch skill**.

**The Java rule.** `bb2025/shared/StepCatchScatterThrowIn.catchBall()` failure path:

```java
boolean stopProcessing = getGameState().executeStepHooks(this, state);
GameOptionBoolean catchForBombs = ...getOptionWithDefault(GameOptionId.CATCH_WORKS_FOR_BOMBS);
if (state.rerollCatch && (!fCatchScatterThrowInMode.isBomb() || catchForBombs.isEnabled())) {
    return catchBall();                       // skill re-roll — EXCLUDED for bombs by default
}
if (!stopProcessing) {                        // <-- the gate Rust dropped
    if (... UtilServerReRoll.askForReRollIfAvailable(gameState, state.catcher, CATCH, minimumRoll, false)) { ... }
}
```

`bb2025/CatchBehaviour.handleExecuteStepHook` returns **true** for any catcher holding **Catch**.
So a Catch player who fluffs a BOMB catch is offered *nothing*: the skill re-roll is excluded for
bombs, and `stopProcessing` suppresses the team re-roll. A catcher **without** Catch (the Treeman)
gets `stopProcessing == false` and is offered the TRR. That is exactly the pattern in the traces.

**The fix** (`crates/ffb-engine/src/step/bb2025/shared/step_catch_scatter_throw_in.rs`): capture the
`bool` that `dispatch::execute_step_hooks` already returns and wrap the team-re-roll offer in
`if !stop_processing { … }`. **The bb2020 twin already had this gate** — bb2025 was a drifted copy.
bb2016's Java has no bomb exclusion, so its `if (state.rerollCatch) return catchBall()` always fires
when the hook stopped processing; its Rust is equivalent and was left alone. Regression test
`failed_bomb_catch_by_a_catch_skill_player_offers_no_team_reroll`, written from the Java above.

**Gates (seeds 1-100, tier 3), before → after:**

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 → **97** | 99 → **99** | 99 → **99** |
| bb2020 | 98 → **98** | 100 → **100** ✅ | 95 → **95** |
| bb2025 | 84 → **96** | 93 → **100** ✅ | 90 → **95** |

45 reds → **21**. Two gates green. No edition regressed (the change is bb2025-only code).
`cargo test -p ffb-engine` 7399/0. `TIMING java_total=60.9s rust_total=37.3s` (bb2025 @0).

**Next.** bb2025 @1.0 (4 reds) and @1e6 (5 reds) — both still red where @0 is green, so those are
now draw-count/sampling splits rather than content. Then bb2020 @1e6 (5) and bb2016 (3/1/1). Run
`frontier.sh bb2025` against `g25_1b.log` first: with the argmax gate green, a family that survives
only under sampling usually points at a candidate-count difference, not a resolution difference.

**Verification (all measured this iteration, all with the positive `PARITY:` line):**
`cargo test -p ffb-engine` 7399/0, `cargo test -p ffb-model` 2802/0.
Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): halfling bb2016 / bb2020 /
bb2025 **100/100 each**.
Closed-roster regressions @1.0: bb2025 × {goblin, amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact, dark_elf, dark_elf_league_fumbbl, elf} all **100/100**; goblin and elf also 100/100 on
bb2016 and bb2020. Nothing regressed.

## ITER2 — Java has TWO `askForReRollIfAvailable` families; Rust had collapsed them into one

**Frontier.** Nine gates re-measured on `e99b7e0a2` first (they reproduced ITER1 exactly: bb2016
97/99/99, bb2020 98/100/95, bb2025 96/100/95 — 21 reds). `frontier.sh` over the three most populated
red gates named one dominant family:

```
bb2020 @1e6 (5 reds)          bb2025 @1.0 (4 reds)                bb2016 @1.0 (3 reds)
seed 24  Activate(away_03,PassMove)   seed 44  Activate(away_02,AllYouCanEat)   seed 1   Activate(home_01,HandOffMove)
seed 34  Activate(home_03,PassMove)   seed 62  Activate(home_01,PassMove)       seed 21  Activate(away_01,Move)
seed 50  Activate(home_01,PassMove)   seed 68  Activate(away_01,HandOffMove)    seed 58  Activate(home_01,Move)
seed 85  Activate(home_03,PassMove)   seed 94  Activate(away_02,AllYouCanEat)
seed 90  Activate(away_02,PassMove)
```

**ALL FIVE** bb2020 @1e6 rows are `PassMove`, and every declaration is identical on both sides — so
the split is in the RESOLUTION of a pass, not in what the agent declared.

**Mechanism** (bb2020 seed 24 @1e6, i=41, away_03 throws to away_08 at (14,0)).

* `FFB_DICE_TRACE` first divergence at `pos=104`. Java's `caller=` stack is decisive:
  `pos=102 StepPass.executeStep:214` (the pass), `pos=103 StepCatchScatterThrowIn.catchBall:527`
  (d6=2, the catch, failed), `pos=104 StepCatchScatterThrowIn.bounceBall:680` **sides=8** — Java
  goes straight from the failed catch to the bounce. Rust's `pos=104` is a **d6**.
* `FFB_TRACE`'s Rust `LOOP` chain says what that d6 was:
  `Activate(away_03,PassMove)` → `Move→(11,2)` → `Pass(14,0)` →
  `ReRollOffer { source: ReRollSource { name: "Catch" }, action: "CATCH" }` → `UseReRoll`.
  Rust **offered the coach a `Catch` re-roll** for the failed catch, and the agent took it. The
  `RUST_STEP` state strings bracket the cost: `r0,0` at i=41 becomes `r0,-1` at i=42 — a re-roll
  spent out of an empty bank.
* The catcher (away_08) has no Catch. The offer's source is the **thrower's** Catch: away_03 is a
  bb2020 Halfling Catcher.

**The Java rule.** `askForReRollIfAvailable` exists in two families and they are NOT one contract:

```java
// ACTING-PLAYER overload — UtilServerReRoll:43-53
ReRollSource reRollSource = UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction, ignoreSkills);
Skill reRollSkill = reRollSource != null ? reRollSource.getSkill(game) : null;
return askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), …, reRollSkill);

// PLAYER overload — bb2020/RollMechanic:239-269. NO action-keyed lookup at all:
if (reRollSkill == null) {
    Optional<Skill> reRollOnce = UtilCards.getUnusedSkillWithProperty(player, canRerollSingleDieOncePerPeriod);
    if (reRollOnce.isPresent()) { reRollSkill = reRollOnce.get(); }
}
dialogShown = (teamReRollOption || proOption || singleUseReRollOption || reRollSkill != null || modificationSkill != null);
```

`StepCatchScatterThrowIn` calls the **PLAYER** overload with `state.catcher`
(bb2016:412, bb2020:583-585, bb2025:590-592). So the catcher's own Catch never appears here — it is
consumed earlier by `CatchBehaviour.handleExecuteStepHook`, which is exactly what `stopProcessing`
(ITER1) exists to report — and the thrower's Catch is invisible to this call entirely. With the bank
empty Java shows no dialog, and the ball bounces.

Rust had ONE function. `ask_for_reroll_if_available_for(game, player_id, …)` did the acting-player
overload's action-keyed lookup via `find_skill_reroll_source`, and **that helper reads
`game.acting_player` regardless of the `player_id` argument** — so passing the catcher fixed the
team-re-roll gate (an earlier fix) but left the SKILL term reading the thrower.

**The unit** (`crates/ffb-engine/src/step/util_server_re_roll.rs`): split the two families.
`ask_for_reroll_if_available` (acting-player entry) resolves `find_skill_reroll_source` itself and
passes it down; `ask_for_reroll_if_available_for` (player entry) passes `None` and the shared inner
function then applies Java's single own term, `canRerollSingleDieOncePerPeriod` **on the player it
was given**. Regression test `the_player_overload_does_not_read_the_acting_players_skill_reroll`,
written from the Java above, pins both halves (the acting-player overload must still find its own
Catch). Also switched the bb2016 and bb2020 `StepCatchScatterThrowIn` twins to the player entry with
the catcher, matching bb2025 and matching Java — **note the bb2020 file is a DEAD TWIN** (`driver.rs`
`make_step` routes `StepId::CatchScatterThrowIn` to the bb2025 step for BB2020; only bb2016 has its
own arm), so that half is alignment, not behaviour.

**Gates (seeds 1-100, tier 3), before → after:**

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 → **97** | 99 → **99** | 99 → **99** |
| bb2020 | 98 → **98** | 100 → **100** ✅ | 95 → **98** |
| bb2025 | 96 → **96** | 100 → **100** ✅ | 95 → **95** |

21 reds → **18**. bb2020 @1e6 seeds 24, 34 and 85 are green; 50 and 90 remain. No gate moved down,
and the red SEED SETS at every other gate are byte-identical to before the change
(bb2016 1/21/58, 63, 6; bb2020 4/69; bb2025 44/62/68/94, 44/66/70/72/90).

**Remaining frontier.** bb2025 @1e6 (44 66 70 72 90) and @1.0 (44 62 68 94) are now the biggest;
seed 44 is red at both sampled scales and green at argmax. The bb2025 @1.0 table has its own family
worth a unit: TWO of its four rows are `Activate(away_02, AllYouCanEat)` — jersey 2 is the bb2025
halfling team spec's star, and `AllYouCanEat` is a declared player action neither the goblin nor the
earlier campaigns ever drove. Attack that family next.

**Ruled out this iteration:** the ITER1 hand-off's prior ("bb2025 @0 is green, so the sampled reds
are draw-count splits — read RSUM/JSUM `n=`/`draws=` before tracing dice") was NOT how the biggest
family fell. bb2020 @1e6's five `PassMove` rows were a resolution divergence with identical
declarations, and the dice trace found it in one pass. Do not skip `FFB_DICE_TRACE` on a sampled-only
red.

**Verification (all measured this iteration; every parity line quoted is the positive `PARITY:`):**
`cargo test -p ffb-engine` **7400/0** (+1 new test), `ffb-model` **2802/0**, `ffb-mechanics`
**1165/0**.
Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): halfling bb2016 / bb2020 /
bb2025 **100/100 each**.
Closed-roster regressions @1.0: bb2025 × {goblin, amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact, dark_elf, dark_elf_league_fumbbl, elf} all **100/100**; goblin and elf also 100/100 on
bb2016 and bb2020. Nothing regressed. `TIMING java_total=63.6s rust_total=24.9s` (bb2020 @1e6).

## ITER3 — `StepAllYouCanEat` never remembered the re-roll it offered, so an ACCEPTED re-roll ejected the star

**Frontier.** `frontier.sh bb2025` over the five `@1e6` reds named one dominant family:

```
seed 44   idx 64   R t3 away Activate(away_15,HandOffMove)
seed 66   idx 24   R t4 home Activate(home_02,AllYouCanEat)   [after side/turn flip]
seed 70   idx 89   R t4 home Activate(home_01,PassMove)
seed 72   idx 6    R t2 home Activate(home_02,AllYouCanEat)
seed 90   idx 6    R t1 home Activate(home_02,AllYouCanEat)
```

Three of five rows are `Activate(home_02, AllYouCanEat)`, and the `@1.0` table's rows 44 and 94 were
the same declaration — five rows across two gates, one family. Declarations identical on both sides.

**Mechanism** (bb2025 seed 90 `@1e6`, parity idx 5 → the hash splits at idx 6).

* `FFB_IDSTATE`: the boards are byte-identical at `i=6` and Java's `i=7` is byte-identical again —
  Java's whole ALL_YOU_CAN_EAT activation changes nothing. Rust's `i=7` has
  `home_02=-1,-1/Reserve`: **the star was sent off**, and the home turn ended (`R t2 away` against
  `J t1 home`).
* `FFB_DICE_TRACE`, first divergence `pos=61`; Java's `caller=` stack prices the three dice before
  it: `pos=47 rollSkill … StepAllYouCanEat.executeStep:64` (the 4+ eat roll, **2 — a miss**),
  `pos=48 RollMechanic.checkForLoner … useReRoll … StepAllYouCanEat.executeStep:57
  StepAllYouCanEat.handleCommand:39` (the Loner check for the team re-roll the coach ACCEPTED),
  `pos=49 rollSkill … StepAllYouCanEat.executeStep:64 … handleCommand:39` (**4 — the re-roll
  succeeds**). Java plays on.
* Rust's `FFB_TRACE` `LOOP` chain over the same activation ends
  `… → ReRollOffer { source: TRR, action: "ALL_YOU_CAN_EAT" } → UseReRoll →
  ActivatePlayer{away…}` — the coach accepted, and the very next prompt is the OPPONENT's turn.
  `FFB_DIE_AT=61` puts Rust's `pos=61` in `StepCatchScatterThrowIn` (a d8 bounce) where Java is
  still rolling bomb armour: by then the two games are different games.

**The Java rule.** `StepAllYouCanEat.executeStep`:

```java
if (getReRolledAction() == ReRolledActions.ALL_YOU_CAN_EAT) {
    if (getReRollSource() == null || !UtilServerReRoll.useReRoll(this, getReRollSource(), player)) {
        doRoll = false;
    }
}
…
if (!success) { push EJECT_PLAYER; push BRIBES; }
```

`getReRollSource()` is filled by `AbstractStepWithReRoll.handleCommand`, which on
`CLIENT_USE_RE_ROLL` does `setReRolledAction(cmd.getReRolledAction())` **and**
`reRollSourceSuccessfully(cmd.getReRollSource())` — non-null when the coach accepts, null when they
decline (`ParityRunner`'s decline is `sendUseReRoll(action, null)`).

Rust has no such base class: every step carries the pair itself, setting `re_roll_source` when it
raises the dialog and clearing it in `handle_command` on `UseReRoll { use_reroll: false }` (see
`bb2025/move_/step_pick_up.rs:274,285` and its `handle_command`). **`StepAllYouCanEat` did
neither.** It set only `re_rolled_action`, and its `handle_command` ignored the action entirely. So
on the re-entry `re_roll_source` was always `None` → `do_roll = false` → the "declined" tail →
Bribes + EjectPlayer. An *accepted* re-roll ejected the bombardier, every time, in both editions
that have the step. The AllYouCanEat 4+ misses half the time, and the bb2025 halfling team spec
fields Cindy Piewhistle at jersey 2 in every game — which is why one bug carried five reds.

**The unit** (`crates/ffb-engine/src/step/mixed/pass/step_all_you_can_eat.rs`), all three parts of
Java's contract for this step:

1. Remember the offered source: `re_roll_state.re_roll_source = Some(prompt_re_roll_source(&prompt))`
   (read off the `ReRollOffer` the helper returned, as Java reads it off the returning command).
2. Clear it in `handle_command` on `UseReRoll { use_reroll: false }`.
3. Ask through the **PLAYER** overload with the original bombardier —
   `askForReRollIfAvailable(getGameState(), player, ALL_YOU_CAN_EAT, 4, false)`, `player` resolved
   from `passState.getOriginalBombardier()`. Rust had called the acting-player entry, which skips
   the `actingTeam.hasPlayer(player)` gate; this is the last caller of the pair ITER2 split.

Colocated test `an_accepted_re_roll_rolls_again_and_a_declined_one_ejects`, written from the Java
above, pins both halves (accept ⇒ a fresh die + one team re-roll spent + no eject; decline ⇒ no die
and the Bribes→EjectPlayer push). The pre-existing prompt test had leaned on the missing membership
gate (its bombardier was on no team) and was corrected to field a real home player.

**Gates (seeds 1-100, tier 3), before → after:**

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 97 → **97** | 99 → **99** | 99 → **99** |
| bb2020 | 98 → **98** | 100 → **100** ✅ | 98 → **98** |
| bb2025 | 96 → **98** | 100 → **100** ✅ | 95 → **98** |

18 reds → **13**. bb2025 `@1.0` seeds 44 and 94 green; `@1e6` seeds 66, 72 and 90 green. No gate
moved down; bb2016's and bb2020's red seed SETS are byte-identical (bb2016 1/21/58, 63, 6;
bb2020 4/69, —, 50/90). Remaining bb2025 reds: `@1.0` 62, 68; `@1e6` 44, 70.
`TIMING java_total=52.6s rust_total=25.7s` (bb2025 @1e6).

**Next.** The bb2025 frontier is now four rows over two gates and its two biggest neighbours are
bb2016 `@1.0` (1, 21, 58) and bb2020 (4, 69, 50, 90). Re-run `frontier.sh` per edition before
picking: seed 44 is red at `@1e6` only (green at `@1.0` and `@0`) and resolved on
`Activate(away_15, HandOffMove)` — **jersey 15**, a player the state hash cannot see (only `nr<=11`
are hashed), so read `FFB_IDSTATE` there rather than the hash. bb2016's three `@1.0` rows were
`HandOffMove` / `Move` / `Move` on the TREEMEN (`*_01`), which is Take Root surface and has never
been examined in this campaign.

**Verification (all measured this iteration; every parity line quoted is the positive `PARITY:`):**
`cargo test -p ffb-engine` **7401/0** (+1 new test), `ffb-model` **2802/0**.
Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`): halfling bb2016 / bb2020 /
bb2025 **100/100 each**.
Closed-roster regressions @1.0: bb2025 × {goblin, amazon, lineman, dwarf, chaos, chaos_dwarf,
chaos_pact, dark_elf, dark_elf_league_fumbbl, elf} all **100/100**; goblin and elf also 100/100 on
bb2016 and bb2020. Nothing regressed.
