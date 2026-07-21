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
5. **Dice are truly random** (`Fortuna`). For deterministic rolls, tests must
   either assert on all outcomes or install a seeded delegate
   (see `Fortuna` API / ffb-ai's `Xoshiro256StarStar` pattern).
6. **No real sessions/spectators**: anything iterating
   `sessionManager.getSessionsForGameId(...)` sees an empty list
   (e.g. `updatePlayerMarkings` is a no-op).
7. **DB-backed queries** (`GameCache.queryFromDb`, `findOpenGamesForCoach`,
   `closeGame`'s delete path) NPE on `getDbQueryFactory()` — steps never call
   these directly, but utility code reached from a few admin flows does.
