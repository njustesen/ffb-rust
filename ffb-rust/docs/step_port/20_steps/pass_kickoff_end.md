# 20_steps/pass_kickoff_end.md — Pass / HandOver · Pickup/Catch-Scatter · Kickoff/Setup · End

BB2025, skill-less lineman. Per-step porting spec. Framework (driver loop, StepAction flags,
publishParameter top→bottom, goto pop-to-label) is in `00_framework.md`; sequence ordering and
GOTO labels are in `10_sequences.md` — not repeated here. Scatter/bounce/throw-in 1:1 findings are
in `AUDIT_scatter.md` (referenced, not duplicated). Agent dialog responses are in `AGENT_CONTRACT.md`.

**Dice TYPE+ORDER are the parity contract.** Every `getGameState().getDiceRoller().rollX()` is one
draw on the single game `Fortuna` stream; the order below is the order the engine consumes it.

## DiceRoller reference (sides per call) — `DiceRoller.java:43-273`
For BB2025 the relevant roll methods map to these dice (one Fortuna draw per `rollDice(N)`):
- `rollSkill()` = d6 (`:95`) — used for pass / intercept / pickup / catch.
- `rollScatterDirection()` = d8 (DirectionDiceCategory, `:207`).
- `rollScatterDistance()` = d6 (`:219`); `rollKickScatterDistance()` = d3 (`:223`, Kick skill only).
- `rollThrowInDirection()` = d6 (`:211`); `rollCornerThrowInDirection()` = d3 (`:215`).
- `rollThrowInDistance()` = **2× d6** (`:227`, array `[d6,d6]`).
- `rollKickoff()` = **2× d6** (`:231`).
- `rollWeather()` = **2× d6** (`:91`).
- `rollFanFactor()` = d3 (`:87`).
- `rollKnockoutRecovery()` = d6 (`:199`).
- `rollDice(3)` = d3; `rollDice(N)` = dN (player-selection draws over a list of size N).
- `rollArgueTheCall()`/`rollBribes()` = d6; `rollSecretWeapon()` = 2× d6.
- `randomPlayer(arr)` / `randomPlayerId(arr)` = one `rollDice(arr.length)` draw (`:259-273`).

---

# PART A — PASS / HAND-OVER

### StepInitPassing  [ ]
- Java: `mixed/pass/StepInitPassing.java:53` (`@RulesCollection BB2020+BB2025` — this *mixed* class
  IS the BB2025 one; there is no bb2025/ override).
- Sequence(s): Pass generator step 1 (`10_sequences.md` Pass).
- Purpose: latch the pass/hand-over command, set `passCoordinate`, `throwerId`, `throwerAction`;
  decide whether this is a hand-over (no roll), a real pass, end-turn, or end-player-action.
- Dice (in order): **none**.
- Events (in order): none.
- Prompts/dialogs: none here (target was already chosen by the agent's Pass/HandOver command).
- Params: reads init `GOTO_LABEL_ON_END` (mandatory), optional `TARGET_COORDINATE` (sets
  passCoordinate + derives `fCatcherId` from player under target). On command latches
  passCoordinate (transform for away, `:127`), catcher, thrower. Publishes `CATCHER_ID` (if catcher
  present), and on end-turn/end-player `END_TURN` / `END_PLAYER_ACTION`.
- Control: HAND_OVER + adjacent catcher → `setHasPassed`, `handOverUsed=true`, `NEXT_STEP` (`:201-208`);
  legal PASS (findPassingDistance != null) → `passUsed=true`, range ruler, `NEXT_STEP` (`:225-239`);
  endTurn/endPlayer/bloodlust → `GOTO_LABEL(GOTO_LABEL_ON_END=END_PASSING)`.
- Rust target: `crates/ffb-engine/src/step/pass/init_passing.rs`
- Test: fixture with a chosen pass target + a chosen hand-over target → assert 0 dice, the published
  CATCHER_ID, and NEXT_STEP vs GOTO(END_PASSING). Java pin: no (no dice; control only).
- Parity: (none yet)

### StepDispatchPassing  [ ]
- Java: `action/pass/StepDispatchPassing.java:35` (`@RulesCollection COMMON`).
- Sequence(s): Pass step 6 (`ON_END=END_PASSING, ON_HAND_OVER=HAND_OVER, ON_HAIL_MARY_PASS=HAIL_MARY_PASS`).
- Purpose: branch by `throwerAction`.
- Dice: none. Events: none. Prompts: none.
- Params: reads `CATCHER_ID` (setParameter). No publishes.
- Control (`:118`): PASS/THROW_BOMB/DUMP_OFF → `NEXT_STEP` (→ StepPass); HAIL_MARY_* →
  `GOTO_LABEL(HAIL_MARY_PASS)`; HAND_OVER → `GOTO_LABEL(HAND_OVER)`; default → `GOTO_LABEL(END_PASSING)`.
- Rust target: `crates/ffb-engine/src/step/pass/dispatch_passing.rs`
- Test: one fixture per action enum → assert the branch label. Java pin: no.
- Parity: (none yet)

### StepPass  [ ]
- Java: `bb2025/pass/StepPass.java:71` (`AbstractStepWithReRoll`).
- Sequence(s): Pass step 7 (`PASS`, `ON_END=INTERCEPT, ON_MISSED_PASS=MISSED_PASS, ON_SAVED_FUMBLE=END_PASSING`).
- Purpose: roll the pass; classify ACCURATE / INACCURATE / FUMBLE (bb2025 PassMechanic); on fail
  offer reroll, else route.
- Dice (in order): **1× `rollSkill()` (d6)** at `:221` (the normal path; `roll==0` guard means exactly
  one draw per attempt). A used team reroll forces `roll=0` then re-enters `executeStep` and draws
  **another d6** (`:194` then `:221`). minimumRoll comes from `mechanic.minimumRoll(...)`
  (bb2025 PassMechanic = range + AG + modifiers); if `minimumRollO` empty (auto-fumble distance) then
  **no d6 drawn** (roll stays 0). Mechanic fn: `mechanic.evaluatePass(thrower, roll, distance, mods, isBomb)`
  → `PassResult`.
- Events (in order): `ReportPassRoll(throwerId, roll, minimumRoll, reRolled, modifiers, distance, isBomb, result)`
  (`:227`). On FUMBLE path publishes mode for scatter (no extra event). → maps to Rust `PassRoll` event.
- Prompts/dialogs: team `ReRollOffer` only on non-ACCURATE when eligible (`askForReRollIfAvailable`,
  `:251`); lineman declines → no reroll. (Skill `canAddStrengthToPass` / safe-pass dialogs do not
  apply to linemen.)
- Params: reads `CATCHER_ID` (into PassState). Publishes: `PASSING_DISTANCE` (`:218`), `PASS_FUMBLE`
  (`:323`), `CATCHER_ID=null` on fail (`:343/351`), and on FUMBLE `CATCH_SCATTER_THROW_IN_MODE=SCATTER_BALL`
  (`:341`). ACCURATE sets ballCoordinate=passCoordinate (`:234`).
- Control: ACCURATE → `GOTO_LABEL(END... )` wait, actually `goToLabelOnEnd` = INTERCEPT (`:230`,
  next is Intercept). FUMBLE → `NEXT_STEP` (`:344`, falls into GOTO→SCATTER_BALL at Pass step 8).
  INACCURATE (else branch, `:352`) → `GOTO_LABEL(goToLabelOnMissedPass=MISSED_PASS)`. SAVED_FUMBLE
  (lineman: never, no Safe Pass) → `GOTO_LABEL(goToLabelOnSavedFumble=END_PASSING)`.
- Rust target: `crates/ffb-engine/src/step/pass/pass.rs`
- Test: 3 fixtures pinning d6 → ACCURATE/INACCURATE/FUMBLE; assert exactly **1 d6**, the PassRoll
  event fields, the published mode, and the branch. Java pin: **yes (subtle)** — pass-result
  branching + the `roll==0` single-draw guard + reroll re-draw.
- Parity: (none yet)

### StepIntercept  [ ]
- Java: `bb2025/pass/StepIntercept.java:64` (`AbstractStepWithReRoll`).
- Sequence(s): Pass step 11 (`INTERCEPT`, `ON_FAILURE=RESOLVE_PASS`).
- Purpose: optional interception by an opponent under the flight path.
- Dice (in order): **0 draws when the agent declines** (lineman always declines interference — see
  AGENT_CONTRACT §7). If an interceptor is chosen: **1× `rollSkill()` (d6)** in `intercept()` `:200`;
  a skill reroll path can re-enter `intercept()` for **another d6** (`:241`) — not reachable for
  linemen. min roll = `mechanic.minimumRollInterception` (or 2 for easyIntercept, not for lineman).
- Events: `ReportInterceptionRoll(interceptorId, successful, roll, minimumRoll, …)` (`:218`) → Rust
  `InterceptionRoll`. None when declined.
- Prompts/dialogs: `Interception` dialog (`DialogInterceptionParameter`, `:151`) shown only when
  `possibleInterceptors.length>0` and not yet chosen; sets `TurnMode.INTERCEPTION`, `CONTINUE`.
  Lineman agent → decline → `doIntercept=false`.
- Params: reads PassState (interceptorId/chosen). Publishes `INTERCEPTOR_ID` on bomb success only
  (`:226`). Sets `state.interceptionSuccessful`.
- Control: no interceptors OR declined OR failed → `GOTO_LABEL(RESOLVE_PASS)` (`:183`); successful
  intercept → `NEXT_STEP` (`:181`, into insertHooks PASS_INTERCEPT [none] → RESOLVE_PASS).
- Rust target: `crates/ffb-engine/src/step/pass/intercept.rs`
- Test: (a) no eligible interceptor → 0 dice, GOTO RESOLVE_PASS; (b) eligible + agent-decline → 0 dice
  (dialog then decline), GOTO RESOLVE_PASS. Java pin: **yes (subtle)** — the dialog/decline path
  must draw 0 dice and not advance turn mode permanently.
- Parity: (none yet)

### StepResolvePass  [ ]
- Java: `bb2025/pass/StepResolvePass.java:23` (default nextAction NEXT_STEP).
- Sequence(s): Pass step 13 (`RESOLVE_PASS`).
- Purpose: convert pass outcome into a CatchScatterThrowIn mode; place ball/bomb; set the
  catch-scatter mode for the downstream shared step.
- Dice: **none** (only sets coordinates + animation + publishes mode).
- Events: animation only (`Animation(PASS/HAIL_MARY/…)`).
- Prompts: none.
- Params: reads PassState (result, catcherId, interceptionSuccessful, throwerCoordinate). Publishes
  `CATCH_SCATTER_THROW_IN_MODE` (one of: CATCH_ACCURATE_PASS / CATCH_ACCURATE_PASS_EMPTY_SQUARE /
  CATCH_MISSED_PASS / THROW_IN + `THROW_IN_COORDINATE`), and `PASS_ACCURATE=true` on accurate-into-tacklezone
  (`:69`). Branch logic `:43-100`: interception successful → ball at interceptor; ACCURATE + catcher
  with tacklezones → CATCH_ACCURATE_PASS; ACCURATE + empty/no-tz → CATCH_ACCURATE_PASS_EMPTY_SQUARE;
  inaccurate out-of-bounds → THROW_IN; inaccurate in-bounds → CATCH_MISSED_PASS.
- Control: always `NEXT_STEP` (→ GOTO→SCATTER_BALL placeholder then CatchScatterThrowIn step 16).
- Rust target: `crates/ffb-engine/src/step/pass/resolve_pass.rs`
- Test: per-branch fixture → assert published mode + ball coordinate + 0 dice. Java pin: **yes (subtle)**
  — the mode mapping drives the +1 CATCH_SCATTER catch modifier downstream (AUDIT #5).
- Parity: (none yet)

### StepHandOver  [ ]
- Java: `bb2025/pass/StepHandOver.java:35`.
- Sequence(s): Pass step 15 (`HAND_OVER`).
- Purpose: place the ball on an adjacent teammate; set mode CATCH_HAND_OFF; end player action.
- Dice: **none here** (the catch d6 happens later in CatchScatterThrowIn).
- Events: `ReportHandOver(catcherId)` (`:86`) → Rust `HandOver`.
- Prompts: none.
- Params: reads `CATCHER_ID`. Publishes `CATCH_SCATTER_THROW_IN_MODE=CATCH_HAND_OFF` (only if thrower
  adjacent to catcher, `:84`), and `END_PLAYER_ACTION=true` unless `canMoveAfterHandOff` (lineman: always)
  (`:92`). Sets ballMoving, ballCoordinate=catcherCoordinate.
- Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/pass/hand_over.rs`
- Test: adjacent receiver fixture → assert HandOver event, mode CATCH_HAND_OFF, END_PLAYER_ACTION, 0 dice.
  Java pin: no.
- Parity: (none yet)

### StepMissedPass  [ ]
- Java: `bb2025/pass/StepMissedPass.java:53`.
- Sequence(s): Pass step 10 (`MISSED_PASS`).
- Purpose: scatter an inaccurate pass up to 3 squares (one d8 per square), tracking lastValid;
  out-of-bounds detection.
- Dice (in order): **up to 3× `rollScatterDirection()` (d8)** — loop `:123` `while inBounds && rollList.size()<3`,
  one d8 per iteration (`:125`). Stops early only when the ball would leave the field (lastValid stays).
  For a lineman (no Blast-It/HMP reroll) exactly `min(3, until-OOB)` draws.
- Events: `ReportScatterBall(directions[], rolls[], false)` once at the end (`:174`) → Rust
  `ScatterBall` / `BallScattered`. (Per-roll `ReportScatterBall` only on the HMP reroll path, not lineman.)
- Prompts: Blast-It reroll dialog only for HAIL_MARY_PASS with the skill (`:146`) — not lineman.
- Params: no inbound reads. Sets `passCoordinate=lastValid`, `outOfBounds`, ballCoordinate=lastValid,
  range ruler.
- Control: `NEXT_STEP` (→ GOTO→SCATTER_BALL then CatchScatterThrowIn).
- Rust target: `crates/ffb-engine/src/step/pass/missed_pass.rs`
- Test: fixture where d8s keep ball in bounds → 3 d8; fixture going OOB on 2nd → 2 d8 then OOB.
  Java pin: **yes (subtle)** — 3-iteration loop bound + lastValid/OOB.
- Parity: (none yet)

### StepEndPassing  [ ]
- Java: `bb2025/pass/StepEndPassing.java:56`.
- Sequence(s): Pass step 18 (`END_PASSING`).
- Purpose: finalize the pass: SPP/completion stats, decide end-turn vs end-player vs continue-move,
  push EndPlayerAction or Move sequence.
- Dice: **none**.
- Events: none (stat/result mutations only).
- Prompts: `hideDialog`.
- Params: **consumes** `CATCHER_ID, END_PLAYER_ACTION, END_TURN, INTERCEPTOR_ID, PASS_ACCURATE,
  PASS_FUMBLE, DONT_DROP_FUMBLE, BOMB_OUT_OF_BOUNDS, PASSING_DISTANCE, BLOOD_LUST_ACTION,
  REVERT_END_TURN, PLAYER_ID` (`:79-132`, each `consume()`). Turnover logic at `:236-288`: end-turn if
  touchdown OR catcher null OR caught by other team OR (fumble & !dontDropFumble).
- Control: pushes EndPlayerAction or Move sequence then `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/pass/end_passing.rs`
- Test: fixtures: completed-to-teammate (no turnover, may continue move), caught-by-opponent
  (turnover), fumble (turnover). Assert which sequence is pushed + 0 dice. Java pin: **yes (subtle)** —
  turnover semantics per result; consume-order of params.
- Parity: (none yet)

---

# PART B — PICKUP / CATCH-SCATTER / THROW-IN (shared)

### StepPickUp  [ ]
- Java: `bb2025/move/StepPickUp.java:57` (`AbstractStepWithReRoll`).
- Sequence(s): Move step 21, BlitzMove step 17, Block step 10, BlitzBlock steps 10/33
  (`ON_FAILURE=SCATTER_BALL` or `DROP_FALLING_PLAYERS`) — see `10_sequences.md`.
- Purpose: pick up the ball the player stands on; reroll on fail (SureHands etc.); on fail → turnover
  + scatter.
- Dice (in order): **1× `rollSkill()` (d6)** in `pickUp()` `:236`. min roll =
  `mechanic.minimumRollPickup(player, mods)` (AG + Pouring-Rain +1 + tacklezone/DP mods). A skill
  reroll (SureHands, `getRerollSource`, `:250`) re-enters `pickUp()` → **another d6** (`:254`).
  Lineman has no SureHands → team `ReRollOffer` (declined) → 0 extra dice. (`secureTheBall` path
  uses min 2+, not reachable for plain MOVE.)
- Events: `ReportPickupRoll(playerId, successful, roll, minimumRoll, reRolled, mods, secureTheBall)`
  (`:243`) → Rust `PickupRoll`. (Java emits a `System.err JPICKUP` debug line at `:242` — not state,
  ignore.) Success sets sound PICKUP.
- Prompts: team `ReRollOffer` on fail when available (`:256`) — lineman declines.
- Params: reads init `GOTO_LABEL_ON_FAILURE` (mandatory), optional `THROWN_PLAYER_ID`; runtime
  `FOLLOWUP_CHOICE, PICK_UP_OPTIONAL, PLAYER_ON_BALL_ID, ATTEMPT_PICK_UP`. On fail publishes
  `FEEDING_ALLOWED=false`, `END_TURN=true` (non-optional), `CATCH_SCATTER_THROW_IN_MODE=FAILED_PICK_UP`
  (`:189`).
- Control: SUCCESS → ballMoving=false, `NEXT_STEP`; FAILURE (non-optional) → `END_TURN` +
  `GOTO_LABEL(ON_FAILURE=SCATTER_BALL)` (`:185`). No-tacklezone-on-ball edge (`:196`) → mode
  FAILED_PICK_UP + GOTO failure.
- Rust target: `crates/ffb-engine/src/step/move/pickup.rs`
- Test: success fixture → 1 d6, PickupRoll, NEXT_STEP, ball held; fail fixture → 1 d6, END_TURN +
  FAILED_PICK_UP + GOTO(SCATTER_BALL). Java pin: **yes (subtle)** — min-roll modifiers (rain +1) and
  the single-draw guard.
- Parity: (none yet)

### StepCatchScatterThrowIn  [ ]
- Java: `bb2025/shared/StepCatchScatterThrowIn.java:99` (`AbstractStepWithReRoll`; this is the BB2025
  shared owner). Helpers: `UtilServerCatchScatterThrowIn` (`findScatterCoordinate`, divingCatchers).
  **The 1:1 Rust-vs-Java findings for this whole subsystem are in `AUDIT_scatter.md` — read it; do
  not re-derive.** This entry pins the dice TYPE+ORDER only.
- Sequence(s): trailing step of nearly every sequence (Move 31, BlitzMove 27, Block 45, BlitzBlock 49,
  Foul 13, Pass 16, Kickoff 17/19, EndPlayerAction 5/10, EndTurn 5). Consumes
  `CATCH_SCATTER_THROW_IN_MODE` + `THROW_IN_COORDINATE`.
- Purpose: the recursive ball-resolution engine: catch at a target, bounce, 3-square scatter, throw-in;
  re-pushes itself (`pushCurrentStepOnStack`, `:399`) until the mode resolves to null.
- Dice (in order) — per mode, in the order the recursion consumes them:
  - **catch** (`catchBall()` `:530`): **1× `rollSkill()` (d6)** when `doRoll`. min =
    `mechanic.minimumRollCatch(catcher, mods)`. BB2025 adds **+1 (harder)** for any
    CATCH_SCATTER/scatter/bounce/throw-in/inaccurate mode via `CatchModifierCollection`
    (AUDIT #5 — Rust `resolve_ball_landing` must add this for `Bb2025`). Failed catch + team reroll
    declined = no extra d6 for lineman.
  - **bounce** (`bounceBall()` `:684`): **exactly 1× `rollScatterDirection()` (d8)** + resolve
    one square. SoundId.BOUNCE.
  - **3-square scatter** (`scatterBall()` `:638`): **up to 3× d8**, one per in-bounds square
    (`while inBounds && size<3`); used only for deflected-pass THREE_SQUARE_SCATTER (not plain lineman
    bounce).
  - **throw-in** (`throwInBall()` `:790`): **1 direction die** = `rollCornerThrowInDirection()` (d3)
    if `isCornerThrowIn(start)` else `rollThrowInDirection()` (d6), **then `rollThrowInDistance()` =
    2× d6** (`:792`). Distance walk moves `distance` squares from start tracking lastValid; AUDIT #2/#3/#4
    note the corner-d3 table, the off-by-one (Java walks `i in 0..distance` via findScatterCoordinate),
    and the lastValid-start fixes.
- Events (in order): `ReportCatchRoll` (`:533`) → Rust `CatchRoll`; `ReportScatterBall(dirs,rolls,false)`
  for bounce/scatter (`:691`/`:652`) → `BallScattered`/`ScatterBall`; `ReportThrowIn(direction,
  directionRoll, distanceRoll)` (`:803`) → `ThrowIn`. Sounds CATCH/BOUNCE.
- Prompts/dialogs: DivingCatch player-choice + Catch reroll — not reachable for skill-less linemen
  (no Diving Catch, no Catch skill); team reroll on failed catch (declined). So lineman path is all dice,
  no prompts.
- Params: **consumes** `CATCH_SCATTER_THROW_IN_MODE`, `THROW_IN_COORDINATE`. Publishes `CATCHER_ID`
  (final resting player or null, `:418`), `TOUCHBACK=true` if Diving Catch carries OOB during kickoff
  (`:424`), `INJURY_RESULT` only on Spiked-Ball option (off by default).
- Control: re-pushes self (REPEAT-like via pushCurrentStepOnStack) while mode != null; once resolved →
  `NEXT_STEP`. Recursion is the parity-critical part: each mode transition that draws dice MUST draw
  the same count/type in the same order as Java.
- Rust target: `crates/ffb-engine/src/step/shared/catch_scatter_throw_in.rs` (currently the logic lives
  in `engine/mod.rs`: `resolve_ball_landing`, `resolve_throw_in`, `scatter_ball_if_carried` — see AUDIT).
- Test: one fixture per mode pinning the exact dice — catch (1 d6 w/ +1 mod), bounce (1 d8), throw-in
  edge (d6 dir + 2d6 dist), throw-in corner (d3 dir + 2d6 dist), bounce-onto-player → recurse to catch
  (1 d8 then 1 d6). Java pin: **yes (subtle)** — catch-scatter recursion + corner-vs-edge die + the
  BB2025 +1 catch modifier + distance walk.
- Parity: (none yet)

---

# PART C — KICKOFF / SETUP

### StepInitKickoff  [ ]
- Java: `bb2025/kickoff/StepInitKickoff.java:31`.
- Sequence(s): Kickoff step 3.
- Purpose: on the very first kickoff (`TurnMode.START_GAME`) start half 1, set up; push the
  before-setup inducement sequences for both teams.
- Dice: **none** here. (`stateMechanic.startHalf` is bookkeeping; KO recovery dice are in EndTurn.)
- Events: `StartHalf` (via `stateMechanic.startHalf(this,1)`) on the first kickoff only.
- Prompts: hideDialog.
- Params: none published except via pushed sequences.
- Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/kickoff/init_kickoff.rs`
- Test: first-kickoff fixture → StartHalf emitted, turnMode SETUP, 0 dice. Java pin: no.
- Parity: (none yet)

### StepSetup  [ ]
- Java: `bb2025/kickoff/StepSetup.java:27`.
- Sequence(s): Kickoff steps 4 & 5 (two SETUP steps — kicking then receiving team).
- Purpose: accept SetupPlayer/EndTurn commands; validate setup via SetupMechanic; flip
  `homePlaying` once the acting team finishes; push before-setup inducements for the offense.
- Dice: **none**.
- Events: SoundId.DING on valid end.
- Prompts: none (SetupPlayer/TeamSetupLoad commands are SKIP_STEP, EndTurn is EXECUTE_STEP).
- Params: none published (inducement sequences pushed on offense setup).
- Control: valid setup → flip homePlaying, `NEXT_STEP`; invalid → stay (`fEndSetup=false`, CONTINUE).
- Rust target: `crates/ffb-engine/src/step/kickoff/setup.rs`
- Test: full valid 11-player setup + EndTurn → flip + NEXT_STEP + 0 dice; invalid → no advance.
  Java pin: no (no dice; SetupMechanic.checkSetup is its own unit).
- Parity: (none yet)

### StepKickoff  [ ]
- Java: `bb2025/kickoff/StepKickoff.java:32`.
- Sequence(s): Kickoff step 9.
- Purpose: latch the KickBall coordinate; publish KICKOFF_START_COORDINATE; push before-scatter
  inducements.
- Dice: **none**.
- Events: none.
- Prompts: KickBall is the agent command (`ClientCommandKickoff`); coord transformed for away (`:60`).
- Params: publishes `KICKOFF_START_COORDINATE` (`:79`).
- Control: with coord → `NEXT_STEP`; without (initial start) sets `TurnMode.KICKOFF` and waits.
- Rust target: `crates/ffb-engine/src/step/kickoff/kickoff.rs`
- Test: KickBall fixture (home & away) → published start coord (transform for away), 0 dice. Java pin: no.
- Parity: (none yet)

### StepKickoffScatterRoll  [ ]
- Java: `bb2025/kickoff/StepKickoffScatterRoll.java:41`.
- Sequence(s): Kickoff step 10 (`_ASK_AFTER` variant if the "ask kick distance after" option is set —
  not default).
- Purpose: scatter the kick: direction d8 + distance d6 (or d3 if Kick skill); walk back into bounds;
  determine touchback.
- Dice (in order): **1× `rollScatterDirection()` (d8)** (`:132`) then **1× `rollScatterDistance()` (d6)**
  (`:135`) — OR `rollKickScatterDistance()` (d3) when the kicking player has Kick and chose to halve
  (lineman: no Kick → always d6). Order = direction first, distance second. (Java prints
  `System.err JAVA_SCATTER_*` debug — ignore.)
- Events: `ReportKickoffScatter(end, direction, dirRoll, distance)` (`:142`) → Rust `KickoffScatter`;
  `ReportEvent("...TOUCHBACK")` on OOB (`:164`).
- Prompts: Kick "reduce distance" skill dialog (`:124`) — not lineman.
- Params: reads `KICKOFF_START_COORDINATE`. Publishes `KICKING_PLAYER_COORDINATE`, `KICKOFF_BOUNDS`,
  `TOUCHBACK` (`:167-169`). Ball placed at lastValid (walk `--distance` until in bounds, `:147`).
- Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/kickoff/kickoff_scatter_roll.rs`
- Test: in-bounds fixture → d8 then d6, KickoffScatter event, TOUCHBACK=false; OOB fixture → d8+d6 +
  TOUCHBACK=true. Java pin: **yes (subtle)** — dir-then-dist order + the lastValid back-walk +
  touchback-half test.
- Parity: (none yet)

### StepKickoffResultRoll  [ ]
- Java: `bb2025/kickoff/StepKickoffResultRoll.java:36`.
- Sequence(s): Kickoff step 12.
- Purpose: roll the 2d6 kickoff-event table; publish the result.
- Dice (in order): **`rollKickoff()` = 2× d6** (`:80`) in the normal (half < 3 or overtime-all) case.
  Overtime variants may draw 1 d6 (`rollDice(6)`, `:84`) or none — but for a normal lineman game it's
  **2× d6**. Interpreted by `DiceInterpreter.interpretRollKickoff` → bb2025 `KickoffResult`.
- Events: `ReportKickoffResult(result, rollKickoff[])` (`:99`) → Rust `KickoffResultEvent`.
- Prompts: none (the result-choice dialog is only for overtime "ask", not default).
- Params: publishes `KICKOFF_RESULT` (`:100`).
- Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/kickoff/kickoff_result_roll.rs`
- Test: pin 2d6 → assert exactly 2 d6, the mapped KickoffResult, KickoffResultEvent. Java pin: **yes
  (subtle)** — 2d6 table mapping.
- Parity: (none yet)

### StepApplyKickoffResult  [ ]
- Java: `bb2025/kickoff/StepApplyKickoffResult.java:88` — the per-event dice fan-out.
- Sequence(s): Kickoff step 13 (`ON_END=END_KICKOFF, ON_BLITZ=BLITZ_TURN`).
- Purpose: apply the rolled `KickoffResult`. Each event consumes its own extra dice **in this order**:
- Dice (in order), by `fKickoffResult`:
  - `GET_THE_REF` (`:260`): **0 dice** (grants Bribes).
  - `TIME_OUT` (`:378`): **0 dice**.
  - `SOLID_DEFENCE` (`:282`): **1× `rollDice(3)` (d3)** → `nrOfPlayersAllowed = min(roll+3, eligible)`
    (`:339`); `ReportSolidDefenceRoll`. Then player-choice dialog (re-setup). No dice on the
    confirm pass.
  - `HIGH_KICK` (`:397`): **0 dice** (setup flip only).
  - `CHEERING_FANS` (`:473`): **`rollDice(6)` (d6) home, then d6 away** (`:478`/`:484`); each may
    **re-roll once (another d6)** only if that team has the Cheering-Fans reroll inducement
    (lineman parity teams: none → exactly 2× d6, home then away). `ReportCheeringFans`.
  - `WEATHER_CHANGE` (`:513`): **`rollWeather()` = 2× d6** (`:517`) → new Weather; **then if
    Weather==NICE and not touchback: up to 3× `rollScatterDirection()` (d8)** gust scatter
    (`:552-567`, one d8 per in-bounds square, stop on OOB). `ReportWeather` then `ReportScatterBall(...,true)`.
    Publishes `TOUCHBACK`.
  - `BRILLIANT_COACHING` (`:428`): **`rollDice(6)` home + `rollDice(6)` away** (`:432`/`:434`) =
    2× d6; `ReportKickoffExtraReRoll`.
  - `QUICK_SNAP` (`:577`): **1× `rollDice(3)` (d3)** → `nrOfPlayersAllowed = roll+3` (`:651`);
    `ReportQuickSnapRoll`. (Subsequent moves are SetupPlayer commands, TRAP_DOOR/APOTHECARY sub-steps
    pushed; no kickoff-table dice.)
  - `CHARGE` (`:679`): **1× `rollDice(3)` (d3)** → `min(roll+3, eligible)` (`:697`); `ReportBlitzRoll`.
    On confirm → `GOTO_LABEL(ON_BLITZ=BLITZ_TURN)`.
  - `DODGY_SNACK` (`:728`): **`rollDice(6)` home + `rollDice(6)` away** (`:735`/`:736`) to pick which
    team(s) are affected; **then `randomPlayer(playersOnField)` = 1× `rollDice(N)`** per affected team
    (`:743`/`:749`); **then 1× `rollDice(6)` per affected player** in `insertSteps` (`:771`).
    `ReportKickoffDodgySnack` + `ReportDodgySnackRoll`. Order: rollHome, rollAway, then home-player-pick
    (if away≥home), away-player-pick (if home≥away), then per selected player a d6.
  - `PITCH_INVASION` (`:782`): **`rollDice(6)` home + `rollDice(6)` away** (`:787`/`:788`), **then
    1× `rollDice(3)` (d3)** = `stunned` count (`:795`), **then per stunned player a `rollDice(N)`**
    selection over the standing list (`stunPlayers`, `:819`, N shrinks each pick). Order: rollHome,
    rollAway, d3, then `min(stunned, standing)` selection draws **per affected team** (home team first
    if totalHome≤totalAway, then away if totalHome≥totalAway). `ReportKickoffPitchInvasion`.
- Events: per above (`KickoffResultEvent` already emitted by ResultRoll; here: WeatherChange,
  ScatterBall, CheeringFans, BrilliantCoaching, SolidDefence/Blitz/QuickSnap rolls, PitchInvasion,
  DodgySnack, Injury/Stun via `UtilServerInjury.stunPlayer`). Animations per event.
- Prompts: SOLID_DEFENCE / CHARGE / QUICK_SNAP drive player-choice + setup dialogs (agent: see
  AGENT_CONTRACT — kickoff Blitz!/QuickSnap = activate-then-deselect).
- Params: reads `KICKOFF_BOUNDS, KICKOFF_RESULT, TOUCHBACK` (setParameter). Publishes `TOUCHBACK`
  (WeatherChange), `PLAYER_ENTERING_SQUARE` (QuickSnap trap-door).
- Control: most → `NEXT_STEP`; CHARGE confirm → `GOTO_LABEL(BLITZ_TURN)`; dialogs → CONTINUE.
- Rust target: `crates/ffb-engine/src/step/kickoff/apply_kickoff_result.rs`
- Test: one fixture per KickoffResult pinning the exact dice sequence above. Java pin: **yes (subtle)** —
  this is the highest-risk dice-order step; each event's extra-dice count/type/order is a separate pin
  (CheeringFans 2× d6, WeatherChange 2d6 [+ NICE gust 3× d8 only if NICE & !touchback], PitchInvasion
  d6+d6+d3+per-player dN, DodgySnack d6+d6+pick+per-player d6).
- Parity: (none yet)

### StepEndKickoff  [ ]
- Java: `phase/kickoff/StepEndKickoff.java:29` (`COMMON`).
- Sequence(s): Kickoff step 20 (`END_KICKOFF`).
- Purpose: push EndTurn(false) + after-kickoff inducement sequence; closes the kickoff.
- Dice: **none**. Events: none. Prompts: none.
- Params: none published. Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/kickoff/end_kickoff.rs`
- Test: assert EndTurn + Inducement sequences pushed, 0 dice. Java pin: no.
- Parity: (none yet)

### StepTouchback  [ ]
- Java: `phase/kickoff/StepTouchback.java:40` (`COMMON`).
- Sequence(s): Kickoff step 18 (between two CatchScatterThrowIn steps).
- Purpose: on touchback, let the receiving team pick a player to hold the ball; place ball; else set
  CATCH_KICKOFF for the trailing CatchScatterThrowIn.
- Dice: **none** (the Touchback choice is an agent prompt, AGENT_CONTRACT §7 = nearest-center player).
- Events: SoundId.CATCH if the chosen player simply holds it.
- Prompts: `Touchback` dialog (`DialogTouchbackParameter`, `:107`) when `fTouchback && coord==null`.
- Params: reads `TOUCHBACK`. Publishes `CATCH_SCATTER_THROW_IN_MODE=CATCH_KICKOFF` (`:121`) when the
  target square has no holdable player.
- Control: no touchback → `NEXT_STEP`; touchback awaiting choice → CONTINUE; chosen → place + NEXT_STEP.
- Rust target: `crates/ffb-engine/src/step/kickoff/touchback.rs`
- Test: touchback → agent picks center player → ball placed, 0 dice; non-touchback → pass-through.
  Java pin: no (dice-free; the catch on CATCH_KICKOFF happens in CatchScatterThrowIn).
- Parity: (none yet)

---

# PART D — START-OF-GAME ROLLS (Spectators / Weather)

### StepSpectators  [ ]
- Java: `mixed/start/StepSpectators.java:28` (`@RulesCollection BB2020+BB2025` — the mixed class is the
  BB2025 one).
- Sequence(s): StartGame step 2 (`10_sequences.md` StartGame).
- Purpose: roll fan factor for both teams.
- Dice (in order): **`rollFanFactor()` = d3 home, then d3 away** (`:66`/`:70`). (Java prints
  `System.err SPECTATORS` debug — ignore.) fanFactor = dedicatedFans + roll.
- Events: `ReportFanFactor(home, rollHome, dedFansHome)` then `ReportFanFactor(away, …)` (`:74`/`:75`).
- Prompts: none.
- Params: none. Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/start/spectators.rs`
- Test: pin 2× d3 → assert home-then-away order + fanFactor math + 2 events. Java pin: no (simple, but
  the home-before-away draw order matters → capture from runner trace).
- Parity: (none yet)

### StepWeather  [ ]
- Java: `game/start/StepWeather.java:23` (`COMMON`).
- Sequence(s): StartGame step 3.
- Purpose: roll initial weather.
- Dice (in order): **`rollWeather()` = 2× d6** (`:54`). Mapped by `DiceInterpreter.interpretRollWeather`.
- Events: `ReportWeather(weather, roll[])` (`:57`) → Rust `WeatherChange`.
- Prompts: none. Params: none. Control: `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/start/weather.rs`
- Test: pin 2d6 → assert 2 d6 + weather mapping + WeatherChange. Java pin: **yes (subtle)** — 2d6→Weather
  table (shared with WeatherChange kickoff event).
- Parity: (none yet)

---

# PART E — END TURN / END GAME

### StepEndTurn  [ ]
- Java: `bb2025/StepEndTurn.java:113`.
- Sequence(s): EndTurn generator step 6 (`10_sequences.md` EndTurn); also reached after touchdown /
  end of half. Pushes next Kickoff / EndGame / start-of-turn inducement.
- Purpose: advance turn/half/drive; at a drive boundary (`fNewHalf || fTouchdown`) run KO recovery
  rolls and Sweltering-Heat fainting; handle secret-weapon/argue plumbing.
- Dice (in order) — only on the drive-boundary branch (`fNewHalf || fTouchdown`), inside `getFaintingCount`
  (`:576`):
  1. **KO recovery: 1× `rollKnockoutRecovery()` (d6) per KNOCKED_OUT player** (`:749`), iterating
     `game.getPlayers()` in player order; a natural 1 with the Bloodweiser/RerollKOs inducement triggers
     **one re-roll (another d6)** (`:761`) — lineman parity teams: no inducement → exactly 1 d6 per KO'd
     player. `isRecoveringFromKnockout(roll, keg)` decides recovery.
  2. **Sweltering Heat (only if Weather==SWELTERING_HEAT)** (`:610`): **1× `rollDice(3)` (d3)** =
     `faintingCount` (`:611`), **then per-team selection draws**: for home then away, `min(faintingCount,
     onPitch)` draws of **`rollDice(onPitch.size())` (dN, N shrinks each pick)** (`:618`) to pick who
     faints. Order: d3 first, then all home picks, then all away picks.
  - The argue-the-call / secret-weapon-ban path can draw `rollArgueTheCall()`/`rollSecretWeapon()`/
    `rollBribes()` (d6 / 2d6 / d6) but only when a player carries a used secret weapon AND
    ARGUE_THE_CALL is enabled — **not reachable for plain skill-less linemen** (no secret weapons).
    `useStarOfTheShow` TD-reroll dialog is skill-gated (not lineman).
- Events: `ReportTurnEnd(touchdownPlayerId, knockoutRecoveries[], heatExhaustions[], unzapped, faintingCount)`
  (`:485`) → carries the KO + heat results; `ReportSecretWeaponBan` (not lineman); SoundId WHISTLE/DING.
  KO recovery and fainting do not emit separate per-player dice events — the rolls are summarized in
  ReportTurnEnd, so the Rust side must still consume the dice in the same order even though the event is
  aggregate.
- Prompts: argue/bribes/star-of-show dialogs — not lineman.
- Params: consumes nothing critical for lineman; publishes `END_TURN=true` for non-regular turn modes
  (`:263`). Pushes Kickoff/EndGame/Inducement sequences (`:370-407`).
- Control: drive boundary → run dice then `NEXT_STEP` (after pushing next sequence); mid-game regular
  turn → flip homePlaying, bump turnNr, push start-of-turn inducement, `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/end/end_turn.rs`
- Test: (a) plain turn end (no TD/half) → **0 dice**, turn flip; (b) end-of-half with 2 KO'd players →
  **2× d6** in player order, ReportTurnEnd; (c) Sweltering-Heat half boundary → KO d6s **then** d3 **then**
  per-team faint selection dN. Java pin: **yes (subtle)** — KO-recovery-per-player order + the
  d3-then-selection sweltering-heat sequence (both feed an aggregate event, so order is invisible in the
  event and must be pinned by the dice trace).
- Parity: (none yet)

### StepInitEndGame  [ ]
- Java: `bb2025/end/StepInitEndGame.java:32`.
- Sequence(s): EndGame generator step 1 (`generator/bb2025/EndGame.java`: INIT_END_GAME · PENALTY_SHOOTOUT ·
  MVP · WINNINGS · DEDICATED_FANS · PLAYER_LOSS · END_GAME[END_GAME]).
- Purpose: enter END_GAME turn mode; apply concession score adjustment.
- Dice: **none**. Events: none.
- Params: reads init `GOTO_LABEL_ON_END` (mandatory), optional `ADMIN_MODE`.
- Control: already-finished → `GOTO_LABEL(END_GAME)`; else `NEXT_STEP`.
- Rust target: `crates/ffb-engine/src/step/end/init_end_game.rs`
- Test: fresh game → NEXT_STEP, turnMode END_GAME, 0 dice. Java pin: no.
- Parity: (none yet)

### StepMvp  [ ]
- Java: `bb2025/end/StepMvp.java:43`.
- Sequence(s): EndGame step 3.
- Purpose: award MVP(s). In non-nomination / admin mode each MVP is a random pick.
- Dice (in order): when `MVP_NOMINATIONS == 0` or admin (`:170`): **`randomPlayerId(homeNominees)` =
  1× `rollDice(N)` per home MVP**, then **per away MVP** the same (`:175`/`:180`). Number of MVPs:
  1 each (or +1 with EXTRA_MVP option; 0 for an illegally-conceding team). With nominations enabled the
  agent picks via dialog then 1 `rollDice(N)` per nominated set (`:122`/`:128`).
- Events: `ReportMostValuablePlayers` (`:186`) with home/away player ids.
- Prompts: MVP player-choice dialog only when `MVP_NOMINATIONS>0` and >1 candidate.
- Params: none published. Control: `NEXT_STEP` once all MVPs assigned.
- Rust target: `crates/ffb-engine/src/step/end/mvp.rs`
- Test: default-options fixture (no nominations) → **1× dN per team** (home then away), ReportMVP.
  Java pin: **yes (subtle)** — `randomPlayerId` draw count = nr-of-MVPs, home-before-away order, and the
  candidate list (excludes stars/mercs/killed/missing).
- Parity: (none yet)

### StepEndGame  [ ]
- Java: `game/end/StepEndGame.java:31` (`COMMON`).
- Sequence(s): EndGame final step (`END_GAME` label).
- Purpose: finalize: set finished date, FINISHED status, clear the step stack, show stats dialog,
  queue replay/result upload.
- Dice: **none**. Events: GAME_STATISTICS dialog.
- Params: none. Control: `NEXT_STEP` (stack already cleared).
- Rust target: `crates/ffb-engine/src/step/end/end_game.rs`
- Test: assert finished set, status FINISHED, stack cleared, 0 dice. Java pin: no.
- Parity: (none yet)

---

## Cross-cutting notes
- **BB2025-effective class resolution** for these steps: most live in `bb2025/...`; `StepInitPassing`
  and `StepSpectators` are the `mixed/...` classes annotated `@RulesCollection(BB2020)+(BB2025)`;
  `StepDispatchPassing`, `StepEndKickoff`, `StepTouchback`, `StepWeather`, `StepEndGame` are
  `@RulesCollection(COMMON)`. No bb2020/bb2016 sibling is used for BB2025.
- **The single hot path for a lineman game** through this file is: StartGame(Spectators d3×2,
  Weather 2d6) → Kickoff(ScatterRoll d8+d6, ResultRoll 2d6, ApplyResult per-event dice, CatchScatterThrowIn
  catch/bounce/throw-in) → … → Pass(StepPass d6, Intercept 0, Resolve, CatchScatterThrowIn catch d6+1) /
  HandOver(catch d6+1) → EndTurn(KO d6 per KO + sweltering d3+picks at drive boundaries).
- **Aggregate-event traps:** ReportPassRoll/PickupRoll/CatchRoll carry their own roll, but
  ReportTurnEnd, ReportKickoffPitchInvasion and ReportKickoffDodgySnack summarize *multiple* dice — the
  Rust side must consume the same dice in the same order even though the event hides the order. Pin
  these via the dice trace (TESTING.md Layer 2), not via event equality alone.
