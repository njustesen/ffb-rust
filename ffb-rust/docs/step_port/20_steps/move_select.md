# 20_steps/move_select.md — MOVE + SELECT/activation steps (BB2025, skill-less lineman)

Per-step porting spec. Read `00_framework.md` (lifecycle/driver/StepAction/params), `10_sequences.md`
(Select/Move/BlitzMove step ordering), and `TESTING.md` (Test-line format) first — not duplicated here.

Java base: `ffb-server/src/main/java/com/fumbbl/ffb/server/step`. BB2025-effective class chosen per
step (bb2025/ override, else action/ COMMON). Paths below are relative to that base.

Dice mapping: Java `getDiceRoller().rollSkill()` / `.rollGoingForIt()` are single d6 draws; the
*target* (minimum roll) is computed by `DiceInterpreter` / `AgilityMechanic` and in Rust maps to
`ffb-mechanics` `minimum_roll_dodge` / `minimum_roll_going_for_it` / `minimum_roll_stand_up`, with
success via `is_skill_roll_successful` (`is_stand_up_successful` for stand-up). **The d6 is rolled
once per attempt; a team/skill reroll re-enters the same roll fn → a second d6 of the same type.**
For a skill-less lineman: no skill rerolls, only the team reroll offered via `ReRollOffer` prompts.

GameEvent names refer to `ffb-model/src/events/game_event.rs`. Lineman-relevant variants:
`DodgeRoll{target,roll,success,rerolled}`, `GoForItRoll{…}`, `StandUpRoll{…}`, `Injury{…}`,
`SkillUse{…}` (skill-less ⇒ none), `TurnEnd{team_id,turn_nr}`. Java `ReportFumblerooskie` /
`ReportStallerDetected` / `ReportPlayerEvent` map to their analogues where present (skill-less moves
rarely hit them).

---

### StepInitSelecting  [ ]
- Java: `bb2025/shared/StepInitSelecting.java:51` (class) / `:454` (executeStep) / `:94` (handleCommand)
- Sequence(s): Select step 1, `INIT_SELECTING(GOTO_LABEL_ON_END=END_SELECTING, UPDATE_PERSISTENCE)`.
- Purpose: The activation gate. Waits on the player's command (`CLIENT_ACTING_PLAYER`/MOVE/BLITZ_MOVE/
  BLOCK/FOUL/PASS/HAND_OVER/END_TURN/CONFIRM/…); resolves which `PlayerAction` was chosen and where to
  route. Lineman-reachable commands: ACTING_PLAYER (select/deselect/stand-up), MOVE, BLITZ_MOVE, BLOCK,
  FOUL, PASS, HAND_OVER, END_TURN.
- Dice (in order): none.
- Events (in order): `ReportStallerDetected` if stalling check fires on ACTING_PLAYER (`checkForStaller`
  `:527`); `ReportFumblerooskie(true)` on CLIENT_USE_FUMBLEROOSKIE (`:343`). Skill-less normal play: none.
- Prompts/dialogs: waits at `CONTINUE` for the player's command (this is the per-activation
  `AgentPrompt`); `DialogConfirmEndActionParameter` only on committed blitz/gaze cancel (`:153`, not
  lineman). No reroll prompt.
- Params: reads init `GOTO_LABEL_ON_END`, `UPDATE_PERSISTENCE`. Publishes on the chosen command:
  `MOVE_START`, `MOVE_STACK`, `BALL_AND_CHAIN_RE_ROLL_SETTING` (MOVE/BLITZ_MOVE); `FOUL_DEFENDER_ID`,
  `USING_CHAINSAW` (FOUL); `BLOCK_DEFENDER_ID`, `USING_STAB/CHAINSAW/VOMIT/BREATHE_FIRE/CHOMP`,
  `USE_ALTERNATE_LABEL` (BLOCK/BLITZ); `TARGET_COORDINATE` (PASS/HAND_OVER); `CHECK_FORGO`,
  `END_TURN` (END_TURN); and always `DISPATCH_PLAYER_ACTION` + (`END_PLAYER_ACTION`|`END_TURN`) in
  executeStep. NOTE the home/away coordinate transform on PASS/TTM target (`:265`).
- Control: executeStep (`:454`): EndTurn/timeout → publish END_TURN → `GOTO_LABEL` END_SELECTING.
  EndPlayerAction → publish END_PLAYER_ACTION → `GOTO_LABEL` END_SELECTING. Dispatch with a real action:
  publish DISPATCH_PLAYER_ACTION; if `isStandingUp() && !forceGotoOnDispatch` → `NEXT_STEP` (continue
  to JUMP_UP/STAND_UP in-sequence); else → `GOTO_LABEL` END_SELECTING. Bare stand-up/remove-confusion →
  `NEXT_STEP`. `prepareStandingUp` (`:489`) sets currentMove to MINIMUM_MOVE_TO_STAND_UP and
  recomputes move squares — parity-relevant for the StandUp roll path.
- Rust target: `crates/ffb-engine/src/step/select/init_selecting.rs`
- Test: Rust characterization (fixture + ACTING_PLAYER(MOVE) command → assert DISPATCH_PLAYER_ACTION
  published, GOTO END_SELECTING, no dice, post-hash). Java pin: yes(subtle) — the dispatch routing +
  prepareStandingUp currentMove side-effect.
- Parity: (none yet)

### StepEndSelecting  [ ]
- Java: `bb2025/shared/StepEndSelecting.java:80` (class) / `:242` (executeStep) / `:281` (dispatchPlayerAction)
- Sequence(s): Select step 7 (`END_SELECTING`, the dispatch hub). Entered via `GOTO_LABEL` END_SELECTING
  from InitSelecting; consumes all the broadcast params.
- Purpose: Terminal step of Select. Consumes the published action params and **pushes the next action's
  sequence** (Move/Block/BlitzBlock/Foul/Pass/EndPlayerAction/…) onto the stack. Pure routing.
- Dice (in order): none.
- Events (in order): none.
- Prompts/dialogs: hides any open dialog (`:243`); never waits.
- Params: reads/consumes `BLOCK_DEFENDER_ID`, `DISPATCH_PLAYER_ACTION`, `END_PLAYER_ACTION`, `END_TURN`,
  `FOUL_DEFENDER_ID`, `GAZE_VICTIM_ID`, `HAIL_MARY_PASS`, `MOVE_START`, `MOVE_STACK`, `TARGET_COORDINATE`,
  `THROWN_PLAYER_ID`, `KICKED_PLAYER_ID`, `NR_OF_DICE`, `USING_*`, `BLOCK_TARGETS`, `IS_KICKED_PLAYER`,
  `TARGET_PLAYER_ID`, `BLOOD_LUST_ACTION`, `BALL_AND_CHAIN_RE_ROLL_SETTING`, `CHECK_FORGO`. Publishes none.
- Control: always `NEXT_STEP` after pushing the dispatched sequence. Routing (`dispatchPlayerAction`
  `:281`): null OR (MOVE while pinned & can-gaze) → re-push **Select**; PASS/HMP/THROW_BOMB/HAND_OVER →
  **Pass**; THROW/KICK_TEAM_MATE → ThrowTeamMate; BLITZ → BlitzBlock; BLOCK → Block; MULTIPLE_BLOCK →
  MultiBlock; FOUL → Foul; MOVE (pinned) → EndPlayerAction; MOVE & *_MOVE/GAZE/SECURE_THE_BALL →
  **Move** (`Move.SequenceParams(moveStack, gazeVictimId, moveStart, ballAndChainRr)`); BLITZ_MOVE/
  KICK_EM_BLITZ → BlitzMove; STAND_UP/STAND_UP_BLITZ/REMOVE_CONFUSION/FORGO → EndPlayerAction. The
  bloodlust branch (`:252`) is not reachable for a skill-less lineman.
- Rust target: `crates/ffb-engine/src/step/select/end_selecting.rs`
- Test: Rust characterization (set DISPATCH_PLAYER_ACTION=MOVE + MOVE_STACK, run start → assert a Move
  sequence is pushed in correct order, NEXT_STEP, no dice). Java pin: yes(subtle) — the action→generator
  table is parity-critical for which sequence runs next.
- Parity: (none yet)

### StepInitActivation  [ ]
- Java: `bb2025/shared/StepInitActivation.java:15` (class) / `:26` (start)
- Sequence(s): ActivationSequenceBuilder step 1, pushed after the INIT step of Move/Block/Foul/Pass/Select
  (see 10_sequences §ActivationSequenceBuilder). For a skill-less lineman the whole activation block is
  pass-through but still pushed.
- Purpose: Recover the acting player's tacklezones and clear eye-gouge at the start of activation; cache
  old player state into the target-selection-state if one exists (blitz/gaze, not lineman).
- Dice (in order): none.
- Events (in order): none.
- Prompts/dialogs: none (runs entirely in `start()`).
- Params: reads none; publishes none.
- Control: always `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/select/init_activation.rs`
- Test: Rust characterization (prone/marked player fixture → assert `recoverTacklezones().clearEyeGouge()`
  applied, NEXT_STEP, post-hash). Java pin: no.
- Parity: (none yet)

### StepStandUp  [ ]
- Java: `bb2025/move/StepStandUp.java:40` (class) / `:82` (executeStep)
- Sequence(s): Select step 5, `STAND_UP(ON_FAILURE=END_SELECTING)` (also Block step 6). Runs after JUMP_UP.
- Purpose: Stand a prone acting player at activation. If MA < `MINIMUM_MOVE_TO_STAND_UP` (3) and no
  stand-up-for-free, requires a stand-up roll; otherwise stands for free (costs 3 move).
- Dice (in order): **one d6 stand-up roll** — only when `rollStandUp` (MA<3 & not free) and
  `isStandingUp() && !hasMoved` (or re-entered as STAND_UP reroll). `roll = diceRoller.rollSkill()`
  (`:102`); success = `DiceInterpreter.isStandUpSuccessful(roll, modifier)` → Rust
  `is_stand_up_successful` (target = `minimum_roll_stand_up()` = 4, minus stand-up-assist modifier,
  floored at 2). On failure with team reroll accepted → re-roll a second d6 via the same fn (reroll
  re-enters executeStep with `getReRolledAction()==STAND_UP`). Skill-less lineman with MA≥3: **none**.
- Events (in order): `ReportStandUpRoll(playerId, success, roll, modifier, reRolled)` → `StandUpRoll`,
  emitted once per roll (so twice if rerolled).
- Prompts/dialogs: on failed roll, `askForReRollIfAvailable(…, STAND_UP, max(2,4-modifier), false)`
  (`:123`) → `ReRollOffer` prompt; if declined/none, the stand-up fails.
- Params: reads init `GOTO_LABEL_ON_FAILURE`. Publishes `END_PLAYER_ACTION=true` on failed stand-up
  (`:133`).
- Control: success & not pinned → `NEXT_STEP`; success & pinned → `GOTO_LABEL` ON_FAILURE (`:117`).
  Failed roll (after reroll exhausted) → set player PRONE+inactive, publish END_PLAYER_ACTION, `GOTO_LABEL`
  ON_FAILURE (`:134`). `handleFailedStandUp` (`:145`) marks the per-turn action-used flag for the chosen
  action (blitz/pass/foul/…). Stands-for-free or already-moved/standing → `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/move_/stand_up.rs`
- Test: Rust characterization — two fixtures: (a) MA≥3 lineman ⇒ no dice, hasMoved set, NEXT_STEP;
  (b) MA<3 lineman, seed forcing success then a failure+reroll path ⇒ assert one/two d6, StandUpRoll
  events, END_PLAYER_ACTION on final failure. Java pin: yes(subtle) — roll/modifier and turnover flags.
- Parity: (none yet)

### StepJumpUp  [ ]
- Java: `action/select/StepJumpUp.java:31` (class, COMMON) / `:84` (executeStep → `executeStepHooks`)
- Sequence(s): Select step 4, `JUMP_UP[NEXT](ON_FAILURE=END_SELECTING)` (also Block step 5). Runs before
  STAND_UP.
- Purpose: Resolve the JUMP_UP skill (stand up for free / +roll). The base class body is empty — it just
  calls `getGameState().executeStepHooks(this, state)`; behaviour is entirely the injected skill hooks.
  **For a skill-less lineman there are no hooks, so this step is a pure pass-through** (no dice/events,
  default `repeat()`→CONTINUE then driven on by the sequence).
- Dice (in order): none (skill-less).
- Events (in order): none (skill-less).
- Prompts/dialogs: none (skill-less).
- Params: reads init `GOTO_LABEL_ON_FAILURE`. Publishes none (skill-less). Skill hooks (out of lineman
  scope) may publish DISPATCH_PLAYER_ACTION / END_PLAYER_ACTION.
- Control: with no hooks the result stays default `CONTINUE`; the labelled-`NEXT` sequence position means
  it is effectively transparent for lineman. (When ported, model the no-hook case as `NEXT_STEP`
  pass-through to match the sequence advancing — verify against the runner trace.)
- Rust target: `crates/ffb-engine/src/step/select/jump_up.rs`
- Test: Rust characterization (lineman with no Jump Up ⇒ no dice/events, sequence advances). Java pin: no.
- Parity: (none yet)

### StepResetFumblerooskie  [ ]
- Java: `mixed/move/StepResetFumblerooskie.java:27` (class, BB2020+BB2025) / `:80` (start)
- Sequence(s): Select step 6, `RESET_FUMBLEROOSKIE[END_SELECTING]`; also EndPlayerAction step 2 and
  BlitzBlock step 41 (`RESET_FUMBLEROOSKIE(failed_block)`).
- Purpose: Finalize a pending Fumblerooskie (deliberate ball drop): if the ball is moving on the carrier's
  own square and the action is ending or movement can't continue, settle the ball-down and emit the
  pickup report; otherwise clear the pending flag. Skill-less lineman normally never sets Fumblerooskie,
  so this is a no-op pass-through.
- Dice (in order): none.
- Events (in order): `ReportFumblerooskie(playerId, false)` (sound PICKUP) only when a pending
  fumblerooskie settles (`:112`). Normal lineman: none.
- Prompts/dialogs: none.
- Params: reads init `RESET_FOR_FAILED_BLOCK`, `END_PLAYER_ACTION`, `IN_SELECT`. setParameter also
  captures runtime `END_PLAYER_ACTION`/`END_TURN` (both set the local `endPlayerAction` flag). May publish
  `DROPPED_BALL_CARRIER` on a failed-block reset (`:98`).
- Control: always `NEXT_STEP`. Skips entirely if `actingPlayer.isJumping()` (`:87`).
- Rust target: `crates/ffb-engine/src/step/move_/reset_fumblerooskie.rs`
- Test: Rust characterization (no pending fumblerooskie ⇒ no-op NEXT_STEP). Java pin: no.
- Parity: (none yet)

---

### StepInitMoving  [ ]
- Java: `bb2025/move/StepInitMoving.java:74` (class) / `:324` (executeStep) / `:140` (handleCommand)
- Sequence(s): Move step 1, `INIT_MOVING(GOTO_LABEL_ON_END=END_MOVING, MOVE_STACK, GAZE_VICTIM_ID,
  BALL_AND_CHAIN_RE_ROLL_SETTING)`; BlitzMove step 1 likewise.
- Purpose: Init the move sequence and **pop one square off MOVE_STACK** to drive the current step. Also
  acts as the mid-move command gate: a fresh ACTING_PLAYER/MOVE re-targets; a BLOCK/FOUL/PASS/etc. mid-move
  redirects to that action via `GOTO_LABEL_AND_REPEAT` END_MOVING.
- Dice (in order): none.
- Events (in order): `ReportFumblerooskie(true)` on CLIENT_USE_FUMBLEROOSKIE (`:276`). Normal: none.
- Prompts/dialogs: when reached at start with a MOVE_STACK already set it runs immediately; otherwise waits
  at `CONTINUE` for the next Move/BlitzMove command (the per-step `AgentPrompt`). No reroll.
- Params: reads init `GOTO_LABEL_ON_END`, `GAZE_VICTIM_ID`, `MOVE_STACK`, `BALL_AND_CHAIN_RE_ROLL_SETTING`;
  setParameter captures runtime `MOVE_STACK`. Publishes (executeStep `:324`): `BALL_AND_CHAIN_RE_ROLL_SETTING`,
  the shortened `MOVE_STACK` (drops index 0), `COORDINATE_FROM`, `COORDINATE_TO`; and on the redirect/end
  paths `END_TURN`+`CHECK_FORGO`, `END_PLAYER_ACTION`, `DISPATCH_PLAYER_ACTION`, `THROWN_PLAYER_ID`,
  `USING_CHAINSAW`/`USING_VOMIT`. Side effects: sets `dodging`/`goingForIt` from the destination MoveSquare,
  `hasMoved=true`, `commitTargetSelection`, `turnStarted=true`, per-turn action-used flag, concession off.
- Control: EndTurn → publish END_TURN+CHECK_FORGO → `GOTO_LABEL` END_MOVING. EndPlayerAction (deselect) →
  publish END_PLAYER_ACTION → `GOTO_LABEL` END_MOVING. Gaze victim provided → set GAZE action → `NEXT_STEP`.
  Mid-move action redirect (`dispatchPlayerAction` `:397`) → `GOTO_LABEL_AND_REPEAT` END_MOVING. Normal:
  next square in-bounds → publish COORDINATE_*/MOVE_STACK → `NEXT_STEP`. (Out-of-bounds dest leaves result
  default — guarded by legal-move validation.)
- Rust target: `crates/ffb-engine/src/step/move_/init_moving.rs`
- Test: Rust characterization (MOVE_STACK of 2 squares ⇒ assert COORDINATE_FROM/TO + shortened stack
  published, dodging/goingForIt set from MoveSquare, NEXT_STEP, no dice). Java pin: yes(subtle) — the
  stack-pop + dodging/GFI flag derivation feeds every downstream roll.
- Parity: (none yet)

### StepMove  [ ]
- Java: `bb2025/move/StepMove.java:45` (class) / `:95` (executeStep)
- Sequence(s): Move step 7 (`MOVE`); BlitzMove step 3.
- Purpose: Actually relocate the player to COORDINATE_TO: increment currentMove (by 2 if jumping, else 1),
  add the TrackNumber footprint, update player+ball position, accumulate rushing yards, recompute
  goingForIt/move-squares/dice-decorations. No roll here — GFI/dodge are separate later steps.
- Dice (in order): none.
- Events (in order): `ReportSkillUse(RUSH_ADDITIONAL_SQUARE_ONCE)` only for the canMakeAnExtraGfiOnce skill
  (`:111`) — not lineman. Sound DODGE/STEP. The actual "PlayerMoved"-type state change is the position
  update (no dedicated GameEvent for plain move in this step). Normal lineman: none.
- Prompts/dialogs: none.
- Params: reads/consumes `COORDINATE_FROM`, `COORDINATE_TO`, `MOVE_STACK` (only its length →
  `fMoveStackSize`). Publishes `PLAYER_ENTERING_SQUARE` (`:133`) — triggers PickUp/CatchScatterThrowIn
  later (covered by the pickup/catch agent; this is where Move feeds them the entered square).
- Control: always `NEXT_STEP`. Skips the body entirely if the player is pinned (`:99`).
- Rust target: `crates/ffb-engine/src/step/move_/move_step.rs`
- Test: Rust characterization (move one square ⇒ assert position+rushing updated, currentMove+1,
  PLAYER_ENTERING_SQUARE published, NEXT_STEP, no dice; jumping ⇒ +2). Java pin: yes(subtle) — rushing
  delta sign depends on `isHomePlaying`.
- Parity: (none yet)

### StepGoForIt  [ ]
- Java: `bb2025/move/StepGoForIt.java:64` (class) / `:137` (executeStep) / `:207` (rush)
- Sequence(s): Move steps 8 & 9 (`GO_FOR_IT(ON_FAILURE=STEADY_FOOTING)`, the 2nd carries
  `BALL_AND_CHAIN_GFI`); BlitzMove steps 4-5; BlitzBlock step 2; Block step 21. Two GFI steps run so the
  normal-GFI and ball-and-chain-GFI variants self-select via `fBallandChainGfi` vs `goForItAfterBlock`.
- Purpose: Roll the Rush (Go-For-It) test when the current square exceeds MA. Only the step whose
  `fBallandChainGfi` matches the player's `goForItAfterBlock` actually rolls.
- Dice (in order): **one d6 GFI roll** `roll = diceRoller.rollGoingForIt()` (`:216`), only when
  `isGoingForIt() && currentMove > MA`. Target = `DiceInterpreter.minimumRollGoingForIt(modifiers)` → Rust
  `minimum_roll_going_for_it(modifier_total)` (base 2, +1 per Sure-Feet-cancelling modifier / blizzard /
  moles). Success via `is_skill_roll_successful`. On failed roll with team reroll accepted → re-enter
  `rush()` → a second d6 of the same type (`:251-253`). Jumping players may roll a **second** GFI in the
  same step (`succeedGfi` re-pushes the step, `:185`) — not relevant to a non-jumping lineman.
- Events (in order): `ReportGoForItRoll(playerId, success, roll, minimumRoll, reRolled, modifiers)` →
  `GoForItRoll` (once per roll). Skill-only `ReportSkillUse` paths not lineman.
- Prompts/dialogs: failed roll → `askForReRollIfAvailable(…, RUSH, minimumRoll, false)` (`:256`) →
  `ReRollOffer`; declined/none → FAILURE.
- Params: reads init `GOTO_LABEL_ON_FAILURE`, `BALL_AND_CHAIN_GFI`; setParameter captures `MOVE_START`.
  On failure publishes `END_TURN=true` and `STEADY_FOOTING_CONTEXT(InjuryTypeDropGFI)` (`:198`).
- Control: success → `NEXT_STEP` (`succeedGfi`). Failure → `failGfi` (`:190`): publish END_TURN +
  STEADY_FOOTING_CONTEXT, `GOTO_LABEL` ON_FAILURE (→ STEADY_FOOTING → FALL_DOWN). Not-this-variant or no
  GFI needed → `NEXT_STEP`. Waiting-for-reroll → `CONTINUE`.
- Rust target: `crates/ffb-engine/src/step/move_/go_for_it.rs`
- Test: Rust characterization (currentMove>MA fixture, seed forcing pass; then fail+reroll path ⇒ assert
  one/two d6 GFI, GoForItRoll events, END_TURN + STEADY_FOOTING_CONTEXT + GOTO STEADY_FOOTING on final
  fail). Java pin: yes(subtle) — modifier target + reroll re-roll order.
- Parity: (none yet)

### StepMoveDodge  [ ]
- Java: `bb2025/move/StepMoveDodge.java:92` (class) / `:215` (executeStep) / `:300` (dodge)
- Sequence(s): Move steps 14 & 17 (`MOVE_DODGE(ON_FAILURE=STEADY_FOOTING)`, the 2nd labelled `RETRY_DODGE`
  for the Diving-Tackle retry); BlitzMove steps 10 & 13.
- Purpose: Roll the Dodge test when leaving a tackle zone (set via `actingPlayer.isDodging()`). Skill-less
  lineman: plain AG roll with the dodge modifiers; no Break Tackle / Diving Tackle / Dodge-skill branches.
- Dice (in order): **one d6 dodge roll** `DODGE_ROLL = diceRoller.rollSkill()` (`:309`, published as a
  param), only when `actingPlayer.isDodging()` and a roll is due (`doRoll`). Target =
  `AgilityMechanic.minimumRollDodge(game, player, dodgeModifiers)` → Rust `minimum_roll_dodge(agility,
  modifiers)` (modifiers from `DodgeContext`: +1 per opposing tackle zone on destination, etc.). Success
  via `is_skill_roll_successful`. Failed roll + team reroll accepted → `dodge(true)` recursion → a second
  d6 of the same type (`:476`). (Break-Tackle/Diving-Tackle re-checks recompute the *target* only, no
  extra dice.)
- Events (in order): `ReportDodgeRoll(playerId, success, dodgeRoll, minimumRoll, reRolled, modifiers,
  statMod)` → `DodgeRoll` (once per roll). `ReportModifiedDodgeResultSuccessful` / `ReportSkillUse` are
  skill paths — not lineman.
- Prompts/dialogs: failed roll → `askForReRollIfAvailable(…, DODGE, minimumRoll, false, …)` (`:443`,`:485`)
  → `ReRollOffer`; declined/none → FAILURE. Diving-Tackle prompt + ARM_BAR player-choice dialog
  (`:284`,`:443`) are not lineman.
- Params: reads init `GOTO_LABEL_ON_FAILURE`; reads `COORDINATE_FROM`, `COORDINATE_TO`, `DODGE_ROLL`,
  `USING_BREAK_TACKLE`, `USING_DIVING_TACKLE`, `USING_MODIFYING_SKILL`, `RE_ROLL_USED`. Publishes
  `DODGE_ROLL`, `RE_ROLL_USED` (on success), `STEADY_FOOTING_CONTEXT(InjuryTypeDropDodge)` (on failure),
  `USING_BREAK_TACKLE` (skill), `END_PLAYER_ACTION` (only the STAND_FIRM_NO_DROP option, `:253`).
- Control: not dodging → `NEXT_STEP`. Success → publish RE_ROLL_USED, `NEXT_STEP`. Failure →
  `failDodge` (`:267`): publish STEADY_FOOTING_CONTEXT, `GOTO_LABEL` ON_FAILURE (→ STEADY_FOOTING →
  FALL_DOWN). Waiting-for-reroll → `CONTINUE`.
- Rust target: `crates/ffb-engine/src/step/move_/move_dodge.rs`
- Test: Rust characterization (dodging fixture across 1-2 marked TZ, seed pass; then fail+reroll ⇒ assert
  one/two d6, DodgeRoll events with correct target, STEADY_FOOTING_CONTEXT + GOTO STEADY_FOOTING on final
  fail). Java pin: yes(subtle) — modifier target + reroll order; ignore the BT/DT/skill branches for lineman.
- Parity: (none yet)

### StepFallDown  [ ]
- Java: `bb2025/move/StepFallDown.java:37` (class) / `:82` (executeStep)
- Sequence(s): Move step 25 (`FALL_DOWN`); BlitzMove step 21. Reached via `GOTO_LABEL` FALL_DOWN after a
  failed GFI/dodge (through STEADY_FOOTING) when the player drops.
- Purpose: Drop the acting player and resolve their injury from the supplied INJURY_TYPE (DropGFI /
  DropDodge). Causes a turnover unless in PASS_BLOCK turn mode.
- Dice (in order): **the injury sequence's dice** — armour d6+d6 then injury d6+d6 (and casualty roll if
  injured), via `UtilServerInjury.handleInjury` (`:86`). Exact roll order/targets are owned by the injury
  mechanics (see the injury spec / `ffb-mechanics` injury calc); FallDown only triggers them. No roll of
  its own beyond that delegation.
- Events (in order): `Injury{…}` (armour/injury/casualty results) from `handleInjury`; plus the drop
  reports from `UtilServerInjury.dropPlayer` (player set PRONE, ball handling).
- Prompts/dialogs: none directly (apothecary prompts are later APOTHECARY steps, out of lineman scope).
- Params: reads/consumes `INJURY_TYPE`, `COORDINATE_FROM`. Publishes the drop params from `dropPlayer`,
  `INJURY_RESULT`, and `END_TURN=true` when `fInjuryType.fallingDownCausesTurnover()` and not PASS_BLOCK
  (`:90`).
- Control: always `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/move_/fall_down.rs`
- Test: Rust characterization (InjuryTypeDropDodge fixture, seed → assert armour/injury d6 order, Injury
  event, player PRONE, END_TURN published, INJURY_RESULT published, NEXT_STEP). Java pin: yes(subtle) —
  injury dice order is parity-critical; pin against the injury oracle.
- Parity: (none yet)

### StepEndMoving  [ ]
- Java: `bb2025/move/StepEndMoving.java:65` (class) / `:194` (executeStep) / `:303` (pushSequenceForPlayerAction)
- Sequence(s): Move step 32 (`END_MOVING`, carries BLOOD_LUST_ACTION); BlitzMove step 28. Terminal step of
  Move/BlitzMove; the continuation hub.
- Purpose: Decide what happens after a square resolves: end the action, continue moving (re-push Move/
  BlitzMove with the remaining MOVE_STACK), or dispatch into a new action (Block/Foul/Pass/TTM/Punt/Blitz).
  **This is where Blitz becomes BlitzMove→BlitzBlock** (`pushSequenceForPlayerAction` BLITZ→BlitzBlock).
- Dice (in order): none.
- Events (in order): `ReportPlayerEvent("could not secure the ball…")` on a failed Secure-the-Ball
  turnover (`:206`) — not a plain lineman move. Normal: none.
- Prompts/dialogs: hides any dialog (`:198`); never waits (handleCommand only re-dispatches redirected
  commands).
- Params: reads/consumes `BLOCK_DEFENDER_ID`, `DISPATCH_PLAYER_ACTION`, `END_PLAYER_ACTION`, `END_TURN`,
  `FEEDING_ALLOWED`, `MOVE_START`, `MOVE_STACK`, `USING_CHAINSAW`, `THROWN_PLAYER_ID`, `BLOOD_LUST_ACTION`,
  `CHECK_FORGO`. Publishes none. Side effect: `fEndTurn |= checkTouchdown(...)` (touchdown forces turn end).
- Control: always `NEXT_STEP` after pushing the chosen sequence (or `NEXT_STEP_AND_REPEAT` when a redirected
  command pushes a new action, `:298`). Branches (`:229`): EndTurn/EndPlayerAction → EndPlayerAction
  sequence; BLOCK_DEFENDER_ID set (ball-and-chain) → Block; non-moving action mid-move →
  `pushSequenceForPlayerAction`; MOVE_STACK remaining → re-push Move/BlitzMove (`Move.SequenceParams(moveStack,
  null, moveStart, null, bloodlustAction)`); else `isNextMovePossible`/per-action continuation → re-push
  Move/BlitzMove with a fresh stack; else → EndPlayerAction.
- Rust target: `crates/ffb-engine/src/step/move_/end_moving.rs`
- Test: Rust characterization — three fixtures: (a) MOVE_STACK remaining ⇒ re-push Move, NEXT_STEP;
  (b) END_TURN set ⇒ EndPlayerAction pushed; (c) DISPATCH BLITZ ⇒ BlitzBlock pushed. Assert no dice,
  correct sequence pushed, NEXT_STEP. Java pin: yes(subtle) — the continuation table + touchdown turnover.
- Parity: (none yet)
