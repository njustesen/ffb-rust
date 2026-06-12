# 00_framework.md — Java step framework → Rust (port spec)

Source base: `C:/Users/Admin/niels/ffb/ffb/ffb-server/src/main/java/com/fumbbl/ffb/server`
Generic step lifecycle / driver loop, for a 1:1 Rust reimplementation. Individual step rules
live in `20_steps/`; this is only the framework that runs them.

## 1. Lifecycle

Actors: `GameState` (owns `fStepStack: StepStack` + `fCurrentStep: IStep`, the driver);
`IStep` (concrete steps extend `AbstractStep`/`AbstractStepWithReRoll`); `StepResult` (per
step; carries `StepAction nextAction` + optional goto-label param + reports/animation/sound +
`synchronize`).

IStep contract (`IStep.java:15-46`): `getId`, `setLabel/getLabel`, `start`, `repeat`,
`init(StepParameterSet)`, `handleCommand(ReceivedCommand) -> StepCommandStatus`, `getResult`,
`setParameter(StepParameter) -> bool`, `publishParameter(s)`.

AbstractStep defaults: `init`/`start` no-op; **`repeat()` forces `nextAction=CONTINUE`** to
prevent endless loops (un-overridden REPEAT runs exactly once); `handleCommand` handles only
CONCEDE_GAME/ILLEGAL_PROCEDURE/SETUP_PLAYER else UNHANDLED; `setParameter` returns false.

### Driver loop — `GameState.executeStep(mode, cmd)` (`GameState.java:275-294`)
```
do {
  if mode==Start: current.start()
  else if cmd!=null: current.handleCommand(cmd)
  while current.result.nextAction.triggerRepeat(): current.repeat()
  syncGameModel(current)            // flush+reset reports/anim/sound; NOT nextAction
  forward = processStepResult()
  if forward: mode = HandleCommand  // forwarded steps never get start()
} while forward
```
`processStepResult()` (`:296-322`): if `triggerGoto()` → `handleStepResultGotoLabel(param)`;
if `triggerNextStep()` → if `forwardCommand()` `progressToNextStep()` (pop, no start) + return
true (loop, forward same cmd) else `startNextStep()` (pop + executeStep(Start), recursive).
Entry points: `startNextStep()` (`:246`, pop+Start) and `handleCommand(cmd)` (`:234`, start a
step if none then HandleCommand). `progressToNextStep()` = `current = stack.pop()`.

**Only `nextAction` advances the game.** The driver discards the `StepCommandStatus` return.

## 2. StepAction (`StepAction.java`) — flags (triggerNextStep, forwardCommand, triggerGoto, triggerRepeat)

| Value | nextStep | forward | goto | repeat | Driver |
|---|---|---|---|---|---|
| CONTINUE | F|F|F|F | wait for next external command |
| NEXT_STEP | T|F|F|F | startNextStep (pop+start, recursive) |
| REPEAT | F|F|F|**T** | call repeat() until !triggerRepeat |
| GOTO_LABEL | T|F|**T**|F | pop-to-label, then start that step |
| NEXT_STEP_AND_REPEAT | T|**T**|F|F | progress (no start) + forward same cmd |
| GOTO_LABEL_AND_REPEAT | T|**T**|**T**|F | pop-to-label + forward same cmd |

`setNextAction(action[, label])`. Default result = CONTINUE + synchronize=true.

## 3. StepCommandStatus (`StepCommandStatus.java`) — UNHANDLED / EXECUTE_STEP / SKIP_STEP
Step-internal control flow only (driver ignores it). Convention: UNHANDLED → leave CONTINUE;
EXECUTE_STEP → run body (sets real nextAction); SKIP_STEP → consumed but skip body (often sets
GOTO_LABEL_AND_REPEAT). `AbstractStepWithReRoll` adds CLIENT_USE_RE_ROLL / PLAYER_CHOICE /
USE_SKILL handling.

## 4. StepStack (`StepStack.java`) — LIFO, top at index 0
`push(step)`=add(0); `push(list)` iterates back-to-front so `list[0]` ends on top / runs first
(this is how a whole `Sequence` is pushed); `pop`/`peek`=index 0.
`handleStepResultGotoLabel(label)` (`:345-359`): `current=null`; while `peek()!=null`: if
`peek().label==label` return (leave it on top), else `pop()` (discard); throw if not found.
(`cleanupStepStack` `:330` is the non-destructive variant for inducement injection.)

## 5. StepParameter / publishParameter / consume
`StepParameter{key, value, consumed}`; equals/hashCode **by key only**; `StepParameterSet` =
map keyed by key (overwrite). Two delivery paths:
1. construction-time `init(StepParameterSet)` — the params listed in `Sequence.add(id, params…)`;
   each step reads its keys, throws on missing mandatory.
2. run-time `publishParameter(p)` (`AbstractStep.java:177`): calls `setParameter(p)` on the
   **current step itself**, then `stack.publishStepParameter(p)` which walks the stack
   **top→bottom**, calling `setParameter`, and **`break`s on first `consumed`**. Consume order
   is observable (e.g. USE_ALTERNATE_LABEL claimed by nearest StepGotoLabel). Most params are
   broadcast (never consumed). A step overrides `setParameter` to capture keys (return true) or
   defer to super (false).

Lineman-relevant keys: MOVE_STACK, MOVE_START, COORDINATE_FROM/TO, DODGE_ROLL,
DISPATCH_PLAYER_ACTION, END_PLAYER_ACTION, END_TURN; BLOCK_DEFENDER_ID, BLOCK_ROLL,
BLOCK_RESULT, NR_OF_DICE, DICE_INDEX, DEFENDER_PUSHED, STARTING_PUSHBACK_SQUARE,
FOLLOWUP_CHOICE; FOUL_DEFENDER_ID, FOULER_HAS_BALL; PASS_RESULT, CATCHER_ID, INTERCEPTOR_ID;
CATCH_SCATTER_THROW_IN_MODE, INJURY_RESULT, APOTHECARY_MODE; GOTO_LABEL[_ON_*],
ALTERNATE_GOTO_LABEL, USE_ALTERNATE_LABEL (consumed), KICKOFF_RESULT, TOUCHBACK.
Standard labels (`IStepLabel.java`): END_MOVING/BLOCKING/FOULING/PASSING/SELECTING, NEXT,
FALL_DOWN, PUSHBACK, BOTH_DOWN, DODGE_BLOCK, RETRY_DODGE, SHADOWING, SCATTER_BALL, END_KICKOFF,
DEFENDER_DROPPED, ATTACKER_DROPPED, DROP_FALLING_PLAYERS, END.

## 6. Command dispatch
`GameState.handleCommand(ReceivedCommand)` routes to `fCurrentStep` (starting one if needed).
Headless: `MatchRunner.inject/injectForTeam` (`MatchRunner.java:1438`) wrap a ClientCommand in
`ReceivedCommand(cmd, HOME|AWAY session)` and call `gameState.handleCommand` directly; session
chosen by `isHomePlaying()`. Steps gate on session (`UtilServerSteps.checkCommandIsFrom*`) and
acting-player id (`checkCommandWithActingPlayer`) — a dropped command → UNHANDLED → CONTINUE →
hang. Lineman ClientCommands: StartGame, SetupPlayer, EndTurn, CoinChoice, ReceiveChoice,
Kickoff, Touchback, ActingPlayer(playerId|null, action, jumping), Move, BlitzMove, Block,
Foul, HandOver, Pass, BlockChoice, Pushback, UseReRoll, UseSkill, PlayerChoice, Confirm.

## 7. Rust mapping (behaviour-identical, idiomatic/fast)
- **Step = enum + match dispatch** (not `Box<dyn>`): one variant per StepId holding that step's
  persistent fields; `start/handle_command/repeat/set_parameter/result/id/label` are `match`.
  No vtable/alloc; exhaustive coverage. A `StepCtx { &mut Game, &mut StepStack-ish, rng, reports }`
  is handed to hooks (driver owns stack+current to avoid borrow conflicts).
- **StepStack = `Vec<Step>`, top = last.** `push_sequence` pushes authored order **reversed**
  (first-authored ends on top). `publish_step_parameter` iterates `iter_mut().rev()` (top→bottom),
  `break` on consume — **order MUST stay identical**.
- **StepParameter = typed enum** (variant = key), with a separate `consumed: bool`. Construction
  set = small overwrite-by-key map.
- **StepResult struct** `{ next_action, next_action_param: Option<String>, reports, animation,
  sound, synchronize }`; default `{Continue,None,…,true}`; `reset()` clears reports/anim/sound,
  NOT next_action.
- **StepAction** Copy enum with const flag accessors mirroring §2.
- **Flatten the recursion** into an explicit loop (Java recurses + needs a depth guard; the Rust
  iterative driver is naturally bounded). `goto_label` mirrors `handleStepResultGotoLabel` exactly.

### MUST stay semantically identical (parity-critical)
1. goto pop-to-label (discard top-down, leave labelled step on top, error if absent).
2. sequence push order (first authored runs first).
3. publishStepParameter top→bottom + break-on-first-consume; publisher (current step) sees it
   first via its own setParameter, but its consume does NOT stop the stack walk.
4. forwardCommand skips start() and re-delivers the same command.
5. REPEAT drains in a loop; default repeat() resets to CONTINUE.
6. syncGameModel resets only reports/anim/sound (never nextAction), after the action, before
   processStepResult.
7. driver ignores StepCommandStatus.
8. command session/acting-player gating must match (else silent drop → hang).
9. init() mandatory-param validation is a hard error.

### Key refs
`GameState.java:234-359`; `AbstractStep.java:95,99,104,172,177,198`;
`AbstractStepWithReRoll.java:43-74`; `StepAction.java`; `StepStack.java:35-90`;
`StepParameter[Set/Key].java`; `Sequence.java`; `SequenceGenerator.java`; `StepFactory.java`;
`ParityRunner.java:143-332,1438`; `MatchRunner.java:1438`; `UtilServerSteps.java:40-74`;
`UtilServerGame.java:42-68`.
