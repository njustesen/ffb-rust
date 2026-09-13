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

Note what the exception does NOT do: it is swallowed upstream, so nothing is reported and the
step's `StepResult` keeps the `CONTINUE` set by its own constructor (`StepResult:28`). That is why
the step looks "parked at CONTINUE with no dialog outstanding" — BACKLOG §H.34 spent an iteration
looking for the line that set it, and there is none.

### What Rust does

Default (flag `false`): Rust reproduces the stale flag and publishes the same throw-in from the same
interior square. Rust's `StepCatchScatterThrowIn` then makes no progress, mirroring Java's swallowed
exception, and the driver's no-progress guard ends the game — the same observable outcome as Java's
`STUCK_STEP`, reached without a panic.

Flag `true`: `StepPuntDistance` clears `out_of_bounds` on an in-bounds landing, the punt resolves as
a normal catch, and the game continues. This is the behaviour the rules describe, and it is NOT
parity-comparable against stock Java.

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
