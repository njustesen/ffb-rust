# Step-test fixture (`com.fumbbl.ffb.server.fixture`)

Foundation for porting the Rust step tests (`ffb-rust/crates/ffb-engine/src/step/**`)
to Java. A step test needs a `GameState` wired to a `FantasyFootballServer`;
this package provides both without a database, network, Swing, or file-system
dependencies (no `rosters/`/`teams/` XML is read).

Proof of concept: `com.fumbbl.ffb.server.step.game.start.StepInitStartGameFixtureTest`
(mirrors the 11 Rust tests for `StepInitStartGame`).

## Recipe

```java
GameState gameState = GameFixture.createGameState(3);   // 2 teams x 3 linemen
Game game = gameState.getGame();

// direct construction (preferred: gives access to the concrete type)
StepInitStartGame step = new StepInitStartGame(gameState);
// or via the factory: (IStep) GameFixture.createStep(gameState, StepId.INIT_START_GAME)

StepAction a = GameFixture.startStep(step);              // runs start()
// StepAction.CONTINUE == Rust StepOutcome::cont() (waiting for a command)

StepCommandStatus s = GameFixture.handleCommand(step, new ClientCommandStartGame(), true);
// boolean flag = from home coach (sentinel HOME_SESSION) or away (AWAY_SESSION)

StepAction after = GameFixture.nextAction(step);         // step.getResult().getNextAction()
```

Deterministic dice (`GameFixture.installScriptedDice`) and generator-sequence
inspection (`GeneratorTestSupport`) are documented in their own sections below.

Situation helpers:

- `GameFixture.addPlayer(gameState, homeTeam, "p1", 12, "Block", "Dodge")` —
  Java analogue of Rust `add_player_with_skills`; overload with explicit
  MV/ST/AG/PA/AV stats exists. Default stats mirror Rust: 6/3/3/4/8.
- `GameFixture.placePlayer(gameState, "home1", 5, 5)` — standing on pitch.
- `GameFixture.setTurnMode(...)`, `GameFixture.setHalf(...)`,
  `GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK)`.
- `GameFixture.skill(game, "Dodge")` — resolve a `Skill` from the active ruleset.

Fixture invariants after `createGameState(n)`:

- Ruleset **BB2025** (from `UtilServerStartGame.addDefaultGameOptions`, STANDALONE).
- Teams `home` / `away`, player ids `home1..homeN` / `away1..awayN`,
  jersey numbers 1..N, all RESERVE in the box, position `lineman`.
- `game.setTesting(true)`, `TurnMode.START_GAME`, home playing, weather NICE,
  `gameState.getStatus() == GameStatus.STARTING` (Rust `GameStatus::Starting`).
- `gameState.getStepFactory()` initialized; skill behaviours registered.

## Deterministic dice (`installScriptedDice`)

By default the fixture rolls real random dice (`Fortuna`). To port a Rust step
test that asserts a **specific** rolled outcome, install a scripted roll
sequence — the Java analogue of seeding the Rust RNG:

```java
GameState gameState = GameFixture.createGameState(2);
GameFixture.installScriptedDice(gameState, 6, 5);   // next two die faces
// StepWeather rolls 2d6 = 11 -> POURING_RAIN
```

How it works: every game die resolves through
`gameState.getDiceRoller()`, whose `rollDice(...)` methods bottom out at
`gameState.getServer().getFortuna().getDieRoll(sides)`. The fixture server hands
out a `ScriptedFortuna` (a `Fortuna` subclass); `installScriptedDice(...)` loads
its face queue, so **all** categories become deterministic in one place:

| Roll | Draws | Script faces |
|---|---|---|
| d3/d4/d6/d8/d16, coin | one `getDieRoll(sides)` | 1..sides |
| 2d6 (`rollWeather`/`rollArmour`/`rollInjury`/`rollKickoff`) | two d6 | two faces |
| block dice (`rollBlockDice(n)`) | n × d6 | faces 1..6 = skull,bothdown,pushback,pushback,stumble,pow |
| scatter/throw-in direction (`rollScatterDirection`) | one d8 | 1..8 (1=N..8=NW, home team) |

- **Ordering is sequential**, one flat queue for all dice. Script faces in the
  exact order the step rolls them.
- **Out-of-range faces throw** (`IllegalStateException`): a scripted 7 requested
  as a d6 is a test bug, surfaced loudly (unlike the engine's own
  `DiceRoller.addTestRoll(int)`, which silently drops it).
- **Exhausted script falls back to real random**, so incidental background rolls
  a test does not care about still work.
- `GameFixture.clearScriptedDice(gameState)` resets; repeated
  `installScriptedDice` calls append.

Constraints that limit determinism (see `ScriptedFortuna` javadoc):

1. **Re-rolls draw again** — a re-rolled dodge/GFI/block consumes the next
   scripted face(s); provide faces for both the initial roll and the re-roll.
2. **Unexpected earlier rolls** (Pro/Loner/Dauntless, leader draws) consume
   faces you intended for a later roll — script exactly what the step rolls.
3. **Direction away-team mirroring** happens above the RNG (in
   `DirectionDiceCategory`), so script the raw home-team d8 face.

Self-test: `ScriptedDiceFixtureTest` proves scripted `6,5`→POURING_RAIN,
`1,1`→SWELTERING_HEAT, `6,6`→BLIZZARD through the real `StepWeather`, plus raw
d6 / block-dice / fallback behaviour.

## Generator sequence inspection (`GeneratorTestSupport`)

For porting the Rust generator push-order tests
(`ffb-rust/crates/ffb-engine/src/step/generator/**`). Lives in **this fixture
package** (`com.fumbbl.ffb.server.fixture.GeneratorTestSupport`) so any
`step/generator/**` test package can reuse it. A Java generator pushes `IStep`s
onto `gameState.getStepStack()`; `GeneratorTestSupport.sequence(gameState)`
returns them in authored (Rust `Vec<SequenceStep>`) order, plus
`indexOf`/`contains`/`count`/`find`/`findLabelled`/`indexOfInstance` and
`booleanField`/`readField` reflection helpers for asserting step params.

```java
new EndGame().pushSequence(new EndGame.SequenceParams(gameState, false));
IStep[] steps = GeneratorTestSupport.sequence(gameState);
assertEquals(StepId.INIT_END_GAME, steps[0].getId());
```

## How to port a step-logic test

1. **Set up game state** — `GameFixture.createGameState(n)`, then
   `addPlayer` / `placePlayer` / `setActingPlayer` / `setTurnMode` to reproduce
   the Rust test's board.
2. **Install scripted dice** — `GameFixture.installScriptedDice(gameState, ...)`
   with the exact faces the step will roll (mirrors seeding the Rust RNG).
3. **Run the step** — construct it (`new StepXxx(gameState)` or
   `GameFixture.createStep`) and `GameFixture.startStep(step)`; feed client
   commands with `GameFixture.handleCommand(step, cmd, fromHome)` where needed.
4. **Assert the outcome** — inspect the model (`game.getFieldModel()`,
   `game.getDialogParameter()`, player/turn state) and/or the pushed sequence
   via `GeneratorTestSupport`. Assert the *behavioural* result, matching the Rust
   test's assertion.

## What is real vs stubbed (`TestFantasyFootballServer`)

Hand-written subclass (same pattern as ffb-ai's `HeadlessFantasyFootballServer`);
**Mockito was not needed** — everything steps dereference is a real object:

| Member | Status |
|---|---|
| `FactoryManager` + application factories | real (super constructor) |
| `getFortuna()` / `gameState.getDiceRoller()` | real — dice rolls work |
| `getSessionManager()` | real maps; `getSessionOfHomeCoach/AwayCoach` return sentinel proxy `HOME_SESSION`/`AWAY_SESSION` |
| `getGameCache()` | real, `init()` skipped, `queueDbUpdate` no-op |
| `getCommunication()` | real object, both `send(...)` overloads no-op |
| `getDebugLog()` | real `DebugLog`, `isLogging()` always false |
| `getDbUpdater()` | real queue, never drained (nothing enqueues it in practice) |
| `getRequestProcessor()`, `getDbQueryFactory()`, `getReplayCache()`, `getCommandHandlerFactory()`, `getReplayer()` | **null** — only set by `run()`, never called |

## Semantics notes for porters

- A fresh `StepResult` defaults to `StepAction.CONTINUE`, so "step waits" is
  asserted as `CONTINUE`, matching Rust's `Continue`.
- Rust keeps `GameStatus` on the `Game`; Java keeps it on `GameState`
  (`gameState.getStatus()`).
- Where Rust steps gate on their own booleans, Java often gates on model state
  (e.g. `StepInitStartGame.executeStep()` checks `game.getStarted() != null`,
  not `fStartedHome && fStartedAway`). Port the *behavioral* assertion; use
  reflection for private step fields only when the Rust test asserts flags.
- Steps run **in isolation**: `startStep()` calls `step.start()` directly, so
  `NEXT_STEP` does not pop a next step, published `StepParameter`s only reach
  steps already on the stack, and sequences pushed by generators land on
  `gameState.getStepStack()` (inspectable, not executed). For integration-style
  flows push steps on the stack and use `gameState.startNextStep()` /
  `gameState.handleCommand(GameFixture.receivedCommand(cmd, true))`.
- Dialogs: `UtilServerDialog.showDialog` only mutates the model — assert via
  `game.getDialogParameter()`.

## Known limitations (will constrain some step families)

1. **FUMBBL-mode branches** (`server.getMode() == ServerMode.FUMBBL`) NPE on
   `getRequestProcessor()` — fixture is STANDALONE; those branches are
   untestable here (they call the FUMBBL web API anyway).
2. **Skills added via `addPlayer(...)`** do not re-apply `PlayerModifier`s
   (stat-increase/decrease skills). Pass explicit stats instead.
3. **Synthetic roster** has a single `lineman` position and no
   `raisedPositionId` / `riotousPositionId` — steps around raising the dead
   (necromancer), Riotous Rookies, or position-specific logic need the roster
   enriched (extend `createLinemanRoster`, positions added via the reflection
   helper since `Roster.addPosition` is private).
4. **Timers/entropy**: turn timers, session timeout, network entropy tasks
   never run; `getTurnTimeStarted()` is whatever the test sets.
5. **Dice default to truly random** (`Fortuna`). For a specific outcome, call
   `GameFixture.installScriptedDice(gameState, ...)` (see "Deterministic dice"
   above); otherwise assert over all outcomes. Determinism is limited by
   re-rolls and incidental earlier rolls (documented there).
6. **No real sessions/spectators**: anything iterating
   `sessionManager.getSessionsForGameId(...)` sees an empty list
   (e.g. `updatePlayerMarkings` is a no-op).
7. **DB-backed queries** (`GameCache.queryFromDb`, `findOpenGamesForCoach`,
   `closeGame`'s delete path) NPE on `getDbQueryFactory()` — steps never call
   these directly, but utility code reached from a few admin flows does.
