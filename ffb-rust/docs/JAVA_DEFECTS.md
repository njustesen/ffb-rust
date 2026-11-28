# Java engine defects

Defects found in **stock Java** (`ffb-common` / `ffb-server`) during the parity campaign.

Java is the reference: we port what it does, including where it disagrees with the printed
rulebook. These entries are the exceptions — cases where Java does something no interpretation of
the rules supports, and which we can demonstrate with a probe. They are recorded here so that a
Rust deviation is never silent, and so a future Java update can be checked against the list.

**House rule for every entry.** The defect is ported 1:1 and is the DEFAULT Rust behaviour, so the
two engines stay byte-comparable. The correction ships alongside it behind an explicit
configuration flag on `Game::defect_fixes` (`ffb-model/src/model/java_defect_fixes.rs`), default
`false`. Turning a flag on makes Rust deliberately diverge from Java, and any parity gate run with
it on is not a parity measurement.

---

## JD-001 — a re-rolled punt distance keeps a stale `outOfBounds`, then throws

| | |
|---|---|
| **Where** | `ffb-server/.../step/bb2025/punt/StepPuntDistance.java` (`executeStep`, `leave`) |
| **Surfaces as** | `IllegalStateException: Unable to determine throwInDirection` from `ffb-common/.../mechanics/bb2025/ThrowInMechanic.java:52` |
| **Effect** | The step never sets a next action, so its `StepResult` keeps the `CONTINUE` its constructor set. The game cannot advance and the harness gives up (`END_REASON: stuck_step`). |
| **Found on** | `dark_elf` bb2025 @1.0 seed 28, BACKLOG §H.39 |
| **Rust flag** | `defect_fixes.punt_distance_clears_out_of_bounds` (default `false` = defect) |

### The rules situation is valid

Both halves of the trigger are explicit in the BB2025 rulebook's Punt Special Action
(`rules/core_rules/08_skills_and_traits.md:380-386`):

> "Roll a D6 to determine the direction the ball is kicked, and then a second D6 to determine how
> many squares in that direction the ball will travel. If this player has the Kick Skill, they may
> re-roll either or both of these dice"

and the ball ending up "in the crowd" is a named outcome of the action. So a punt that leaves the
pitch and then has its DISTANCE re-rolled onto the pitch is ordinary play, not a harness artefact —
and Java offers that re-roll itself, through `UtilServerReRoll.askForReRollIfAvailable`.

### The defect

`StepPuntDistance.executeStep` sets the flag but never clears it — there is no `else` branch, and
`setOutOfBounds(false)` appears nowhere in the class:

```java
FieldCoordinate ballIndicatorCoordinate = coordinateFrom.move(direction, distance);
if (!FieldCoordinateBounds.FIELD.isInBounds(ballIndicatorCoordinate)) {
    ballIndicatorCoordinate = findLastSquareOnPitch(distance - 1);
    fieldModel.setOutOfBounds(true);          // set here, cleared nowhere
}
fieldModel.setBallCoordinate(ballIndicatorCoordinate);
```

On the re-roll the method runs again from the top. An in-bounds landing skips the branch entirely,
so `outOfBounds` is still `true` from the FIRST roll, while the ball is now on an ordinary interior
square. `leave()` then reads that stale flag:

```java
if (fieldModel.isOutOfBounds()) {
    publishParameter(new StepParameter(END_TURN, true));
    publishParameter(new StepParameter(CATCH_SCATTER_THROW_IN_MODE, CatchScatterThrowInMode.THROW_IN));
    publishParameter(new StepParameter(THROW_IN_COORDINATE, fieldModel.getBallCoordinate()));
}
```

and publishes a throw-in from that interior square. `ThrowInMechanic.interpretThrowInDirectionRoll`
handles only edge squares (`x < 1`, `x > 24`, `y < 1`, `y > 13`) and throws on anything else.

### Evidence

Stack probe on `dark_elf` bb2025 @1.0 seed 28 (`StepResult.setNextAction` + a `Throwable` catch in
`StepCatchScatterThrowIn.executeStep`, both since reverted):

```
JAVA_DIE rng=46 d6=6 from=...StepPuntDistance.executeStep:108     <- out of bounds, flag set
JNEXTACT set=CONTINUE by: <- StepPuntDistance.java:129(executeStep)  <- re-roll offered
JREROLLA pick=0 use=true                                            <- re-roll accepted
JAVA_DIE rng=47 d6=2 from=...StepPuntDistance.executeStep:108     <- lands ON the pitch
JCSTI enter mode=THROW_IN phase=ASK_HOME throwIn=(2,5) catcher=null
JAVA_DIE rng=48 d6=5 from=...DiceRoller.rollThrowInDirection:231
JCSTI THREW java.lang.IllegalStateException: Unable to determine throwInDirection.
JCSTI   at ...mechanics.bb2025.ThrowInMechanic.interpretThrowInDirectionRoll(ThrowInMechanic.java:52)
JCSTI   at ...step.bb2025.shared.StepCatchScatterThrowIn.throwInBall(StepCatchScatterThrowIn.java:820)
STUCK_STEP: CATCH_SCATTER_THROW_IN unadvanced for 501 iters
```

`(2,5)` is interior on a 26x15 pitch, so no branch of the direction table applies.

Note what the exception does NOT do: nothing reports it, and the step's `StepResult` keeps the
`CONTINUE` set by its own constructor (`StepResult:28`). That is why the step looks "parked at
CONTINUE with no dialog outstanding" — BACKLOG §H.34 spent an iteration looking for the line that
set it, and there is none. The exception is thrown synchronously inside the harness's injection of
the re-roll answer, and `ParityRunner.injectCaptured` (`catch (RuntimeException e)`, ~:5599) clears
the dialog without logging it. That is a harness catch, not a stock-Java one. In stock Java the
exception leaves the step the same way JD-002's does.

### What Rust does

Default (flag `false`): Rust reproduces the stale flag and publishes the same throw-in from the same
interior square. Rust's `StepCatchScatterThrowIn` then makes no progress, mirroring Java's swallowed
exception, and the driver's no-progress guard ends the game — the same observable outcome as Java's
`STUCK_STEP`, reached without a panic.

Flag `true`: `StepPuntDistance` clears `out_of_bounds` on an in-bounds landing, the punt resolves as
a normal catch, and the game continues. This is the behaviour the rules describe, and it is NOT
parity-comparable against stock Java.

Rust also spends the throw-in direction die before stopping (Java's `throwInBall` rolls it before
`interpretThrowInDirectionRoll` throws). The harness names the swallowed exception: a STUCK_STEP at
CATCH_SCATTER_THROW_IN with `isOutOfBounds()` and an interior ball square prints
`JAVA_CRASH ... JD-001` and ends the game as `java_crash`, so the §Q1.2 verdict reports it as a
JAVA-CRASH instead of an unexplained stuck game (2026-11-28; underworld_37844 v dark_elf bb2025 @1.0
seed 3, the BACKLOG §Q1.2b repro).

---

## JD-002 — a Sneaky Git foul on a Ball & Chain player dereferences a null armour roll

| | |
|---|---|
| **Where** | `ffb-server/.../skillbehaviour/bb2025/SneakyGitBehaviour.java:97` (the `StepReferee` hook); the bb2016 (`:88`) and bb2020 (`:97`) hooks have the same unguarded read |
| **Surfaces as** | `NullPointerException: Cannot load from int array because "armorRoll" is null` out of `StepReferee.executeStep` |
| **Effect** | Uncaught in the stock engine: the game ends where the step threw, in the state it had before `StepReferee` ran (no report, no next action). In the headless harness it first escaped `main` and killed the JVM, so the crashed seed AND every seed queued behind it produced no Java log (goblin bb2025 @1.0: seed 67 crashed, 68–100 reported as failures). |
| **Found on** | `goblin` bb2025 @1.0 seed 67, BACKLOG §H.47; closed §H.49 |
| **Rust flag** | `defect_fixes.referee_survives_missing_armour_roll` (default `false` = defect) |

### The rules situation is valid

A Fanatic (Ball & Chain) may be fouled like any other prone player. `UtilServerInjury.handleInjury`
marks a `placedProneCausesInjuryRoll` victim's armour as broken WITHOUT rolling (lines 72–76), so
the injury context of that foul carries `armorRoll == null` and `isArmorBroken() == true`.

### The defect

The hook reads the armour dice whenever the fouler lacks Sneaky Git OR the armour is broken:

```java
if (!game.isActive(NamedProperties.foulBreaksArmourWithoutRoll) && (!UtilCards.hasSkill(actingPlayer, skill)
    || state.injuryResultDefender.injuryContext().isArmorBroken() || ...)) {
    int[] armorRoll = state.injuryResultDefender.injuryContext().getArmorRoll();
    refereeSpotsFoul = (armorRoll[0] == armorRoll[1]);      // null for a Ball & Chain victim
}
```

There is no null guard, and the auto-broken branch is exactly the one where no dice exist.

### What Rust does

The same as JD-001: the defect is ported as its OBSERVABLE outcome and the correction sits behind a
flag. In `step/action/foul/step_referee.rs`, when Java's branch would read the armour dice and there
are none, the step returns `Continue` having touched nothing — no report, no event, no next action —
so the driver's no-progress guard ends the game exactly where Java's exception does. With
`defect_fixes.referee_survives_missing_armour_roll` on, "no armour dice" reads as "no armour
doubles" and the referee goes on to the injury dice as Java's next line would; that is the only
reading the rules support (there were no armour dice to be doubles). Test
`jd002_missing_armour_roll_stalls_by_default_and_reads_injury_dice_when_fixed`.

### What the harness does

`ParityRunner.run` catches a `RuntimeException` thrown by a step or dialog handler, prints
`JAVA_CRASH seed=… step=…` with the stack trace, sets `END_REASON: java_crash`, and finalises the
log like a `stuck_step` game: every step up to the crash is flushed with its post-hash, then
`game_end` carries the pre-crash state. Rust's log ends at the same index with the same hash, so
the seed compares — goblin bb2025 @1.0 seed 67 ends at `i=10`, hash `71be2728309848e2`, on both
sides. `ParityRunner.main` keeps a second catch as a backstop so an exception outside the game loop
still cannot void the seeds queued behind it. Neither engine is patched; the crash still happens in
stock Java on every run.

---

## JD-003 — the bb2016 card deck order is JVM identity-hash order, so a card draw is unreproducible

| | |
|---|---|
| **Where** | `ffb-common/.../inducement/bb2016/Cards.java` (`allCards()` is a `HashSet<Card>`; `Card` has no `hashCode`), `ffb-server/.../CardDeck.java` (`build` adds `allCards()` in that order), `DiceRoller.drawCard` (`draw(rollDice(deckSize) - 1)`) |
| **Surfaces as** | The same die draws a different card from run to run, and between fresh `Cards` instances in one JVM |
| **Effect** | Java cannot replay its own card draw from its dice. Not a rules error: the rule is "a random card of the bought type", and Java does draw one — only WHICH card a die value maps to is undefined. |
| **Found on** | §Q3.3b, 2026-11-27: three fresh `bb2016.Cards` in one JVM printed three different orders (`scratchpad/cardorder/CardOrder.java`) |
| **Rust flag** | none — there is no Java behaviour to reproduce |

### What Rust does

Ports the draw 1:1 (deck per card type, minus the cards already held, die of the deck's size, index
`die - 1`) over a fixed order: the order the cards are declared in `bb2016/Cards.java`, the only
stable order Java's source defines. The decision (user, 2026-11-27) is not to pin the JVM's identity
hash (`-XX:hashCode=2`): it would change every identity-hashed collection in the oracle and tie Rust
to HashMap tree-bin internals, to reproduce an order that is not part of the rules.

### What the gates check

The buy step up to the draw: same dice in the same order, same costs, same card type, same
`cardsBought` report. Which card was drawn is not compared. Card PLAY is gated on fixed hands handed
to both teams by the harness before kick-off (§Q3.3a), so no random draw decides a play gate.

---

## JD-004 — a bb2020 re-rolled leap spends its team re-roll again after the Diving Tackle answer

| | |
|---|---|
| **Where** | `ffb-server/.../step/bb2020/move/StepJump.java` `executeStep` (:165): `if (JUMP == getReRolledAction() && !TRUE.equals(useIgnoreModifierAfterRollSkill)) { if (getReRollSource() == null \|\| !useReRoll(...)) handleFailure }` -- no `usingDivingTackle == null` guard (bb2025's StepJump has one) |
| **Surfaces as** | A failed leap, re-rolled with a team re-roll, succeeds; the opposing coach is asked about Diving Tackle; on the answer `executeStep` runs again and `useReRoll(TEAM_RE_ROLL)` decrements the bank a second time (second ReportReRoll; a Loner player rolls Loner again and can turn the made leap into a failure) |
| **Effect** | One team re-roll paid twice for one re-roll. No rule supports it. The second spend has no availability check: `RollMechanic.updateTurnDataAfterReRollUsage` (bb2020, :346) decrements `reRolls` unconditionally, so a team that spent its last re-roll ends with `-1`. |
| **Found on** | §Q2.10 code audit, 2026-11-27 (bb2020 games run Rust's shared StepJump); not yet demonstrated with a probe |
| **Rust flag** | none yet -- add `defect_fixes` flag with the probe |

### What Rust does

Ports it 1:1 (`step/bb2025/move_/step_jump.rs`, bb2020 branch of `execute_step`): after a Diving
Tackle answer with a JUMP re-roll recorded, the source is spent again and a failed spend fails the
leap; the leap then re-reads the same roll.

---

## JD-005 — a bb2025 dodge re-roll is spent again after the Arm Bar choice

| | |
|---|---|
| **Where** | `ffb-server/.../step/bb2025/move/StepMoveDodge.java`: `executeStep` (~:221-238) spends the re-roll whenever `getReRolledAction() == DODGE`; `failDodge` (~:284-287) shows the `ARM_BAR` player choice when two or more Arm Bar players qualify; the `CLIENT_PLAYER_CHOICE` (ARM_BAR) answer (~:203-206) runs `executeStep` again |
| **Surfaces as** | A dodge that fails after a team re-roll, next to two or more Arm Bar opponents: when the choice is answered, the re-roll is spent a second time (a second `ReportReRoll`) and a fresh dodge die is rolled |
| **Effect** | One failed, re-rolled dodge costs two team re-rolls and gets a third dodge roll. No rule supports it. |
| **Found on** | `chaos` bb2025 seed 28 i=60, BACKLOG §H.46 |
| **Rust flag** | none yet |

### What Rust does

Ports it 1:1 (`step/bb2025/move_/step_move_dodge.rs`, ~:263-269): the Arm Bar answer re-enters
`execute_step` with the DODGE re-roll still recorded, so the source is spent again and the dodge is
re-rolled.

---

## JD-006 — ending the turn during a Furious Outburst dereferences a null target selection

| | |
|---|---|
| **Where** | `ffb-server/.../step/bb2025/StepEndFuriousOutburst.java:71` (bb2020 `:67`): `fieldModel.getTargetSelectionState().getSelectedPlayerId()` with no null check |
| **Surfaces as** | `NullPointerException` out of `StepEndFuriousOutburst.executeStep` |
| **Effect** | `mixed/StepInitFuriousOutburst.executeStep` reaches `END` without setting a `TargetSelectionState` on three paths: the end-turn branch, a prone or stunned player, and no skill or no eligible target. Only the end-player-action branch (`.cancel()`) and the target branch set one. The `CLIENT_END_TURN` handler (~:78-82) makes the first path reachable from a real client, so a coach who ends the turn at the Furious Outburst prompt crashes the step. |
| **Found on** | code reading while driving Furious Outburst live (`5b073d8d1`; BACKLOG.md ~:1113-1127) |
| **Rust flag** | none |

### What Rust does

Rust does **not** reproduce this one. `step/bb2025/step_end_furious_outburst.rs` (~:68) reads the
target selection as an `Option` and treats `None` as "no target". No parity gate reaches the path,
because neither agent ends the turn at that prompt. Open follow-up: port the crash as the default
(the same observable outcome as JD-002) and move the current behaviour behind a flag.

---

## JD-007 — modification-aware injury types interpret the MEMBER context, not the alternate one

| | |
|---|---|
| **Where** | `ffb-server/.../injury/injuryType/ModificationAwareInjuryTypeServer.java`. Its own comment (:24) says subclasses "MUST NOT use the injuryContext member". But eleven subclasses end `injuryRoll` with the three-argument `setInjury(pDefender, gameState, diceRoller)`, which (`InjuryTypeServer.java:84-85`) interprets the **member** context: BlockProne, BlockStunned, BreatheFire, Stab and their `ForSpp` twins, plus ProjectileVomit, QuickBite and ThenIStartedBlastin. (Block, Foul and Chainsaw use the four-argument form and are not affected.) |
| **Surfaces as** | When an armour modification (the Master Assassin re-roll, Old Pro) creates an alternate context, `injury(…, alternateInjuryContext)` runs `injuryRoll` on the alternate, and `setInjury` then interprets the ORIGINAL context. That context has no injury roll yet, so `setInjury` rolls a casualty on it. |
| **Effect** | Two extra dice. The alternate context keeps its dice but gets no interpretation, so the original context's injury is decided by a roll the rules never call for. |
| **Found on** | Skitter Stab-Stab, skaven vs human bb2025 seed 1 (BACKLOG §L, `af63a4ce1`); it also cleared underworld vs dark_elf bb2025 |
| **Rust flag** | none yet |

### What Rust does

Ports it 1:1: a trait flag `interprets_member_injury` on the eleven types, a `defer_interpretation`
context flag, and `set_injury_java_member` (`ffb-engine/src/injury.rs`, ~:682).

---

## JD-008 — a bb2025 Pitch Invasion on a Ball & Chain player loses the chain injury

| | |
|---|---|
| **Where** | `ffb-server/.../step/bb2025/kickoff/StepApplyKickoffResult.java` `stunPlayers` (~:813-825) calls `UtilServerInjury.stunPlayer`. For a `placedProneCausesInjuryRoll` player, `dropPlayer` (~:339-342) does not change the player state: it rolls `InjuryTypeBallAndChain` and only *publishes* `INJURY_RESULT`. The bb2025 `Kickoff` generator has no `APOTHECARY` step to consume it. |
| **Surfaces as** | The Pitch Invasion report lists the Fanatic as stunned, the chain armour/injury dice are spent, and the player stays STANDING. |
| **Effect** | The injury is rolled but never applied. Ball & Chain also converts stun to KO, so the rules outcome is at least a KO. |
| **Found on** | `goblin` bb2025 seed 28, BACKLOG §H.47(a) |
| **Rust flag** | none yet |

### What Rust does

Ports it 1:1 (`step/bb2025/kickoff/step_apply_kickoff_result.rs`). The chain injury is rolled and,
in bb2025, not handed on, so the Fanatic stays STANDING. bb2020's sequence does consume it. Test
`a_pitch_invasion_discards_scatter_parameters_and_bb2025_drops_the_chain_injury`.

---

## JD-009 — the Diving Tackle "would trip" hint ignores the Diving Tackle modifier (cosmetic)

| | |
|---|---|
| **Where** | `ffb-server/.../skillbehaviour/bb2020/DivingTackleBehaviour.java` (~:100-105), bb2025 (~:95-100) |
| **Surfaces as** | The Diving Tackle modifier is added to `dodgeModifiers`, but the required roll is computed from `dodgeModifiersWithBT`, a set that never receives it. |
| **Effect** | Only the explanatory text offered with the Diving Tackle dialog ("This will NOT trip the dodger…") can be wrong. The dice, the state and the dodge result are unaffected. |
| **Found on** | code reading during the Diving Tackle port |
| **Rust flag** | none (text only) |

### What Rust does

Ports the same computation (`step_diving_tackle.rs`, ~:287-303), so the dialog descriptions match.

---

## Open candidates (not yet confirmed)

The following were flagged during the campaign but have not been checked against the Java source.
They are not defects until they are:

- bb2016 Diving Tackle: the tail of the step after the skill is used
- a Badly Hurt player left on the pitch
- bb2020: a kicked player who lands in the crowd
- `StepBlockStatistics` NPE after a Dump Off
- Animal Savagery: `catcherId` handling
