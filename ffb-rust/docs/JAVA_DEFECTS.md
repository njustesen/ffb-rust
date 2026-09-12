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
