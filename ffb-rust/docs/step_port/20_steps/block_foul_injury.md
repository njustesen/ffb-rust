# 20_steps/block_foul_injury.md — Block + Foul + Injury/Apothecary steps (BB2025, skill-less lineman)

Per-step porting spec. Framework (driver/stack/params) is in `00_framework.md`; sequence
ordering + GOTO routing is in `10_sequences.md` (Block §, BlitzBlock §, Foul §) — **not repeated
here**. Read those first. This file pins, per step: exact dice (type+order), GameEvent order,
prompts, params read/published, control flow, Rust target, and test plan.

Scope = BB2025, plain linemen with no skills. Anything gated on a skill the lineman lacks
(Dauntless, Dodge, Fend, Grab, Side Step, Stand Firm, Pile Driver, Hit&Run, Pro, Brawler,
Hatred, SneakyGit, Stunty, Thick Skull, Regeneration, Block, Wrestle, Juggernaut, Frenzy,
Tackle, etc.) is **pass-through** — listed as such, no dice. The dice that actually fire for a
skill-less lineman block/foul are: **block dice → (per knocked-down player) armour 2d6 →
injury 2d6 when broken → casualty [d16,d6] when injury ≥10**, plus foul's **referee/argue 1d6**.

Resolution of "BB2025-effective class" (bb2025/ → else bb2020/ → else bb2016/ → else action/mixed):
INIT_BLOCKING=bb2020, BLOCK_ROLL/CHOICE/FOLLOWUP/END_BLOCKING=bb2025, BLOCK_DODGE/BOTH_DOWN=mixed,
PUSHBACK=bb2016, BLOCK_STATISTICS/DAUNTLESS=action(COMMON); INIT_FOULING/BRIBES/END_FOULING=bb2025,
FOUL/EJECT_PLAYER=mixed, REFEREE=action(COMMON) + bb2025 SneakyGitBehaviour hook; APOTHECARY/
DROP_FALLING_PLAYERS/PLACE_BALL=bb2025/shared, FALL_DOWN=bb2025/move.

Dice helper facts (cite once, reused below):
- `DiceRoller.rollBlockDice(n)` (`DiceRoller.java:163`) = `n` block dice (`Math.abs(n)`), each a
  `BlockDiceCategory` (d6 mapped to Skull/BothDown/Pushback/Pushback/Pow-Pushback/Pow via
  `BlockResultFactory.forRoll`). One Fortuna draw per die.
- `DiceRoller.rollArmour()` (`:167`) = 2d6. `rollInjury()` (`:195`) = 2d6.
- `RollMechanic(bb2025).rollCasualty(diceRoller)` (`bb2025/RollMechanic.java:92`) =
  `[rollDice(16), rollDice(6)]` — **d16 then d6**, in that order.
- `rollArgueTheCall()` / `rollBribes()` / `rollSkill()` (`:135,131,95`) = 1d6 each.
- Armour broken: `DiceInterpreter.isArmourBroken` (`DiceInterpreter.java:214`) → `armourRoll`
  sum + mods > armour. Injury interpret: `RollMechanic(bb2025).interpretInjuryRoll`
  (`:97`) total=2d6+mods: **2-7=Stunned, 8-9=KO, ≥10=Casualty(null→casualty roll)**; Thick
  Skull `convertKOToStunOn8` turns total 8 KO→Stunned (lineman lacks it). Casualty:
  `interpretCasualtyRollAndAddModifiers` maps `casRoll[0]` (the d16) +mods: ≥15 RIP, ≥9
  SERIOUS_INJURY (=BADLY_HURT cas), else BADLY_HURT. `isArgueTheCallSuccessful` >5,
  `isCoachBanned` <2, `isBribesSuccessful` >1 (`:226-236`).

Injury-roll **ordering** (the parity contract): `ModificationAwareInjuryTypeServer.handleInjury`
(`ModificationAwareInjuryTypeServer.java:32`) = **armourRoll first** (rolls 2d6 if `roll==true`
and armour not pre-broken), then `injury(...)` which **only rolls injury 2d6 if armour broken**,
then `setInjury` rolls casualty `[d16,d6]` only when injury total produced `null` (≥10). For a
lineman there is no injury-modification skill so the single (unmodified) context path runs once.
Crowd push (`InjuryTypeCrowd.handleInjury`, `InjuryTypeCrowd.java:22`): **no armour roll** — sets
armorBroken=true, rolls injury 2d6, casualty if ≥10, then forces RESERVE if not cas/KO.

---

## BLOCK

### StepInitBlocking  [ ]
- Java: `bb2020/StepInitBlocking.java:48` (no bb2025 override; COMMON-ish bb2020 class)
- Sequence(s): Block step 1 (`INIT_BLOCKING`, ON_END=END_BLOCKING, +BLOCK_DEFENDER_ID/USING_*),
  BlitzBlock step 1, Foul step (reused: INIT_FOULING publishes BLOCK_DEFENDER_ID into it).
- Purpose: receive the `Block`/`Foul`/`ActingPlayer`/`EndTurn` command, resolve the defender,
  flip defender to BLOCKED, snapshot OLD_DEFENDER_STATE, publish the per-block params, advance.
- Dice: none.
- Events: none (state change: defender base→BLOCKED; attacker strength set).
- Prompts/dialogs: none for lineman (askForBlockKind only when special-attack kinds offered;
  lineman path: defender already chosen via Block command → straight NEXT_STEP).
- Params: reads init GOTO_LABEL_ON_END(mandatory), BLOCK_DEFENDER_ID/USING_STAB/CHAINSAW/VOMIT/
  BREATHE_FIRE/MULTI_BLOCK_DEFENDER_ID/ASK_FOR_BLOCK_KIND/PUBLISH_DEFENDER (optional). Command
  `Block(defenderId,…)` sets fBlockDefenderId. Publishes (broadcast): OLD_DEFENDER_STATE,
  DEFENDER_POSITION, USING_STAB, USING_CHAINSAW, USING_VOMIT, USING_BREATHE_FIRE; END_TURN or
  END_PLAYER_ACTION on those branches; BLOCK_DEFENDER_ID iff publishDefender. Sets
  `game.defenderId`, `actingPlayer.strength = strengthWithModifiers`.
- Control: NEXT_STEP on success; GOTO_LABEL END (=END_BLOCKING) on EndTurn/EndPlayerAction/
  suffering-bloodlust-while-MOVE. BLITZ_MOVE→changePlayerAction(BLITZ).
- Rust target: `crates/ffb-engine/src/step/block/init_blocking.rs`
- Test: char test (fixture lineman vs adjacent opponent, Block cmd → asserts defender BLOCKED,
  OLD_DEFENDER_STATE published, NEXT_STEP, 0 dice). Java pin: no (mechanical).
- Parity: (none yet)

### StepBlockStatistics  [ ]
- Java: `action/block/StepBlockStatistics.java:29` (COMMON)
- Sequence(s): Block step 7, BlitzBlock step 6.
- Purpose: first-block bookkeeping — mark `hasBlocked`, `turnStarted=true`,
  `concessionPossible=false`, `playerResult.blocks += increment` (default 1; −1 per
  PLAYER_ID_TO_REMOVE).
- Dice: none. Events: none. Prompts: none.
- Params: init INCREMENT (opt); setParameter PLAYER_ID_TO_REMOVE decrements.
- Control: NEXT_STEP always.
- Rust target: `crates/ffb-engine/src/step/block/block_statistics.rs`
- Test: char test (asserts blocks++ once, idempotent on second visit). Java pin: no.
- Parity: (none yet)

### StepDauntless  [ ]  (skill-less = PASS-THROUGH)
- Java: `action/block/StepDauntless.java:28` (COMMON, AbstractStepWithReRoll)
- Sequence(s): Block step 8, BlitzBlock step 7.
- Purpose: resolve Dauntless. Body is entirely `executeStepHooks(this,state)` — no behaviour
  registered for a skill-less lineman → **no hook runs, step falls through with default
  CONTINUE→ the un-overridden REPEAT runs once → effectively NEXT via the sequence**. NOTE: for
  the lineman this publishes nothing and rolls nothing; `successfulDauntless` stays false so
  `ServerUtilBlock.findNrOfBlockDice(...,false,...)` is the dice count.
- Dice: none (lineman). (With Dauntless skill: 1× `rollDauntless()` d6 — out of scope.)
- Events: none. Prompts: none. Params: reads USING_STAB/VOMIT/CHAINSAW/BREATHE_FIRE.
- Control: pass-through (no nextAction set by hook → driver default).
- Rust target: `crates/ffb-engine/src/step/block/dauntless.rs` (no-op for lineman)
- Test: char test asserting 0 dice, 0 events, state_hash unchanged. Java pin: no.
- Parity: (none yet)

### StepBlockRoll  [ ]  ← PRIMARY DICE SITE
- Java: `bb2025/block/StepBlockRoll.java:74` (AbstractStepWithReRoll)
- Sequence(s): Block step 24, BlitzBlock step 21.
- Purpose: compute number of block dice, roll them, show the block-roll dialog (offers reroll +
  for a chooser sets the choosing team). On the follow-up command (BlockChoice or reroll), set
  `fBlockResult` then publish results and advance.
- Dice (in order):
  1. **block dice**: `getDiceRoller().rollBlockDice(fNrOfDice)` (`:276`) — `|fNrOfDice|` draws.
     `fNrOfDice` from `ServerUtilBlock.findNrOfBlockDice(gameState, attacker, defender, false,
     successfulDauntless=false, doubleTargetStrength=false, addBlockDie=false)`
     (`ServerUtilBlock.java:131`): 1 die base; 2 if atkStr>defStr; 3 if atkStr>2·defStr; −2 if
     atkStr<defStr; −3 if 2·atkStr<defStr. Strength via `RollMechanic.getTotalAttackerStrength`
     (`bb2025/RollMechanic.java:542`, +assists/blitz-Horns/Dauntless — lineman: just assists).
     Negative = defender chooses. (Reroll branches roll extra block dice — lineman declines TRR
     → no extra dice.)
- Events (in order): `ReportBlock(defenderId)` (sound BLOCK), `ReportBlockRoll(teamId, roll)`.
  Map to GameEvent `BlockRoll { nr_of_dice, roll, chooser_team }`. (No `ReportBlockReRoll` for
  lineman — TRR declined per AGENT_CONTRACT §7.)
- Prompts/dialogs: `DialogBlockRollPropertiesParameter` — the **BlockChoice** dialog. For a
  multi-die positive block the *acting* coach chooses the die; for `fNrOfDice<0` the *opponent*
  coach chooses (teamId flips, `:349-355`). Reroll options (TRR/Pro/Brawler/Hatred/Mascot) are
  added but the agent **declines reroll** and picks **die index 0** (AGENT_CONTRACT §7, "Block
  die choice = index 0"). Single-die block: dialog still shown, choice forced.
- Params: reads SUCCESSFUL_DAUNTLESS (consumed), DOUBLE_TARGET_STRENGTH (consumed); command
  `BlockChoice(diceIndex)` sets fDiceIndex + `fBlockResult = forRoll(fBlockRoll[diceIndex])`.
  Publishes (broadcast): NR_OF_DICE, BLOCK_ROLL, DICE_INDEX, BLOCK_RESULT.
- Control: CONTINUE while dialog open (`fBlockResult==null`); NEXT_STEP once result chosen.
  `removeAdditionalAssist(actingTeam)` on resolve.
- Rust target: `crates/ffb-engine/src/step/block/block_roll.rs`
- Test: char test — fixture forces a known nr-of-dice (e.g. equal ST → 1 die), seed fixes the
  block die; assert exactly N block-die draws in order, `BlockRoll` event, BlockChoice prompt,
  index-0 response, published BLOCK_RESULT. **Java pin: yes (subtle — block-result branching &
  chooser-team flip on negative dice).**
- Parity: (none yet)

### StepBlockChoice  [ ]
- Java: `bb2025/block/StepBlockChoice.java:53`
- Sequence(s): Block step 25 (ON_DODGE=DODGE_BLOCK, ON_JUGGERNAUT=JUGGERNAUT, ON_PUSHBACK=
  PUSHBACK), BlitzBlock step 22.
- Purpose: apply the chosen `BLOCK_RESULT` to player states and route control. Lineman branches:
  - **SKULL**: defender→OLD_DEFENDER_STATE (restore), attacker→FALLING. NEXT_STEP (falls through
    to DROP_FALLING_PLAYERS — attacker down = turnover).
  - **BOTH_DOWN**: GOTO_LABEL JUGGERNAUT (→ StepJuggernaut pass-through → StepBothDown).
  - **POW_PUSHBACK**: lineman defender has no Dodge/Watch-Out → defender→FALLING,
    `publishParameters(UtilBlockSequence.initPushback)`, GOTO_LABEL PUSHBACK.
  - **POW**: defender→FALLING, initPushback, GOTO_LABEL PUSHBACK.
  - **PUSHBACK**: defender→OLD_DEFENDER_STATE, initPushback, GOTO_LABEL PUSHBACK.
- Dice: none. Events: `ReportBlockChoice(nrOfDice, roll, diceIndex, result, defenderId, …)` →
  GameEvent already covered by BlockRoll/choice; emit a `BlockChoice`/result marker if the
  trace shows one. Prompts: none (hides dialog).
- Params: reads (broadcast, non-consuming) DICE_INDEX, BLOCK_RESULT, BLOCK_ROLL, NR_OF_DICE,
  OLD_DEFENDER_STATE. Publishes STARTING_PUSHBACK_SQUARE via `initPushback` on pow/pushback.
- Control: per result above (NEXT_STEP for SKULL; GOTO JUGGERNAUT/PUSHBACK otherwise).
- Rust target: `crates/ffb-engine/src/step/block/block_choice.rs`
- Test: char test per result variant (SKULL→attacker FALLING+NEXT; POW→defender FALLING+GOTO
  PUSHBACK; BOTH_DOWN→GOTO JUGGERNAUT). **Java pin: yes (subtle — result→label routing).**
- Parity: (none yet)

### StepBothDown  [ ]
- Java: `mixed/block/StepBothDown.java:31` (BB2020+BB2025)
- Sequence(s): Block step 28 (after JUGGERNAUT pass-through), BlitzBlock step 25.
- Purpose: both-down resolution. Lineman (no Block/Wrestle): defender→FALLING and
  attacker→FALLING. (preventFallOnBothDown = Block skill; lineman lacks it.)
- Dice: none. Events: none (two FALLING state changes). Prompts: none.
- Params: reads OLD_DEFENDER_STATE.
- Control: NEXT_STEP. Falls through WRESTLE (pass-through) → GOTO DROP_FALLING_PLAYERS.
- Rust target: `crates/ffb-engine/src/step/block/both_down.rs`
- Test: char test asserting both players FALLING, 0 dice. Java pin: no.
- Parity: (none yet)

### StepPushback  [ ]
- Java: `bb2016/StepPushback.java:58` (no bb2020/bb2025 override)
- Sequence(s): Block step 32 (label PUSHBACK), BlitzBlock step 29.
- Purpose: offer/resolve the pushback square(s), then push the defender (and chained players),
  or crowd-surf. Loops via the Pushback command stack until `doPush`.
- Dice: none directly. (A crowd-push routes through `UtilServerInjury.handleInjury(InjuryType-
  CrowdPush)` → that *does* roll injury 2d6 [+casualty] — see Injury section.)
- Events: `ReportPushback(defenderId, mode)`; on crowd-push the injury reports. Prompts:
  **Pushback** dialog — agent picks the **min-(x,y) on-pitch square** (AGENT_CONTRACT §7).
- Square selection — `UtilServerPushback.findPushbackSquares(game, startingSquare, REGULAR)`
  (`UtilServerPushback.java:60`): the 3 cone squares for the push direction (`findStartingSquare`
  gives direction from attacker→defender delta). Filter to in-bounds; if any is free+valid →
  offer only the **free in-bounds** ones; else (no free square) **crowd-push only when fewer than
  3 in-bounds** (squares.size()<3 → clear → crowd surf); if all 3 in-bounds & occupied →
  **chain push** (offer all 3, pushing the chain). This is the parity-critical rule from
  AUDIT/AGENT notes.
- Params: reads OLD_DEFENDER_STATE, STARTING_PUSHBACK_SQUARE. Publishes (broadcast):
  DEFENDER_PUSHED=true, STARTING_PUSHBACK_SQUARE (selected then null), CATCH_SCATTER_THROW_IN_MODE
  (THROW_IN on crowd-push of carrier; SCATTER_BALL when pushed onto moving ball), INJURY_RESULT
  (crowd-push), THROW_IN_COORDINATE, PLAYER_ENTERING_SQUARE.
- Control: CONTINUE while awaiting Pushback command; NEXT_STEP once pushed. Chain: pops the whole
  pushback stack pushing each player.
- Rust target: `crates/ffb-engine/src/step/block/pushback.rs`
- Test: char tests — (a) single free square auto/min-pick; (b) crowd-push at sideline (no free,
  <3 in-bounds) → injury 2d6 + RESERVE; (c) chain push (all 3 occupied). **Java pin: yes (subtle
  — free/crowd/chain selection + min-(x,y) pick).**
- Parity: (none yet)

### StepBlockDodge  [ ]  (skill-less = effectively PASS-THROUGH on the dodge offer)
- Java: `mixed/StepBlockDodge.java:31` (BB2020+BB2025)
- Sequence(s): Block step 31 (label DODGE_BLOCK), BlitzBlock step 28. Reached only via
  POW_PUSHBACK when defender has Dodge — **lineman defender never has Dodge**, so this label is
  not hit for plain linemen. Documented for completeness.
- Purpose: Dodge-vs-Pow handling. `findDodgeChoice()` decides whether to *ask* (chain/sideline/
  attacker-half push). Hook (`executeStepHooks`) resolves the Dodge skill; with no skill the
  player just falls (defender→FALLING) then `initPushback` + NEXT_STEP.
- Dice: none. Events: none. Prompts: Dodge-use dialog (only when defender has Dodge → N/A).
- Params: reads OLD_DEFENDER_STATE. Publishes STARTING_PUSHBACK_SQUARE via initPushback.
- Control: NEXT_STEP (after fall) → PUSHBACK.
- Rust target: `crates/ffb-engine/src/step/block/block_dodge.rs` (lineman: defender→FALLING, NEXT)
- Test: char test (unreachable for lineman seeds; assert defender FALLING + 0 dice if forced).
  Java pin: no.
- Parity: (none yet)

### StepFollowup  [ ]
- Java: `bb2025/block/StepFollowup.java:38`
- Sequence(s): Block step 34, BlitzBlock step 32 — after PUSHBACK.
- Purpose: ask attacker whether to follow up into the vacated square; on yes move attacker
  (update ball pos, move squares, blitz track number).
- Dice: none. Events: `SoundId.STEP` on follow-up; (skill reports only if Fend/Taunt — N/A).
  Prompts: **FollowUp** dialog → agent **declines** (AGENT_CONTRACT §7 "Follow-up after push =
  decline"). Pinned/MultipleBlock auto-publish FOLLOWUP_CHOICE=false.
- Params: reads COORDINATE_FROM, DEFENDER_POSITION, FOLLOWUP_CHOICE, OLD_DEFENDER_STATE.
  Publishes (broadcast) FOLLOWUP_CHOICE, COORDINATE_FROM (attacker old coord or null),
  DEFENDER_POSITION, PLAYER_ENTERING_SQUARE (on follow).
- Control: CONTINUE while dialog open; NEXT_STEP once choice resolved.
- Rust target: `crates/ffb-engine/src/step/block/followup.rs`
- Test: char test — push then decline-follow: attacker stays, COORDINATE_FROM=null, NEXT; and a
  follow=true variant moves the attacker. **Java pin: yes (subtle — followup dialog gating).**
- Parity: (none yet)

### StepEndBlocking  [ ]
- Java: `bb2025/block/StepEndBlocking.java:52`
- Sequence(s): Block step 46 (label END_BLOCKING), BlitzBlock step 51.
- Purpose: terminal block bookkeeping + continuation routing. Lineman path: revert
  Horns/Dauntless strength (none), `removePlayerBlockStates(oldDefenderState)`, clear dice
  decorations; then route: end-turn/end-player-action → EndPlayerAction sequence; blitz with move
  left & tacklezones → change to BLITZ_MOVE + push Move; plain Move-after-block continuation; else
  EndPlayerAction. (Pile Driver / Hit&Run / Putrid / second-block / chainsaw branches all gated
  on skills the lineman lacks → skipped, `usePileDriver/useHitAndRun/usePutridRegurgitation`
  forced false → no dialogs.)
- Dice: none. Events: none. Prompts: none for lineman (skill-use dialogs all bypassed).
- Params: reads+consumes DEFENDER_PUSHED, END_PLAYER_ACTION, END_TURN, USING_STAB/CHAINSAW/VOMIT/
  BREATHE_FIRE/CHOMP, INJURY_RESULT (collects knockedDownPlayers), TARGET_PLAYER_ID, CHECK_FORGO;
  reads (non-consume) ALLOW_SECOND_BLOCK_ACTION, OLD_DEFENDER_STATE, BLOOD_LUST_ACTION. Publishes
  ALLOW_SECOND_BLOCK_ACTION on the second-block branch (N/A lineman). `fEndTurn |=
  checkTouchdown`.
- Control: NEXT_STEP; pushes EndPlayerAction or Move sequence onto the stack as continuation.
- Rust target: `crates/ffb-engine/src/step/block/end_blocking.rs`
- Test: char test — after a resolved block, asserts block states cleared, dice decorations
  cleared, correct continuation (EndPlayerAction on turnover; Move-continuation on blitz with
  move left), 0 dice. Java pin: no (mechanical, but verify continuation against trace).
- Parity: (none yet)

---

## FOUL

(Foul always ends the turn — `INIT_FOULING` sets `turnData.foulUsed`; the turnover is delivered
via END_TURN published by EJECT_PLAYER/BRIBES, or by the foul itself only when the ref spots it /
argue fails. The foul *action* itself does not auto-set END_TURN for a successful unseen foul —
the turn just continues to its normal end; the "foul ends the turn" effect is the single-foul cap
+ ref/argue turnover. Capture per trace.)

### StepInitFouling  [ ]
- Java: `bb2025/foul/StepInitFouling.java:41`
- Sequence(s): Foul step 1 (ON_END=END_FOULING, FOUL_DEFENDER_ID, USING_CHAINSAW).
- Purpose: receive `Foul`/`ActingPlayer`/`EndTurn`; resolve foul defender; set hasFouled,
  turnStarted, concessionPossible=false, `playerResult.fouls++`, `turnData.foulUsed=true`
  (unless allowsAdditionalFoul), `game.defenderId`.
- Dice: none. Events: none. Prompts: none (lineman issues Foul command directly).
- Params: init GOTO_LABEL_ON_END(mand), FOUL_DEFENDER_ID/USING_CHAINSAW(opt). Command
  `Foul(defenderId, usingChainsaw)`. Publishes (broadcast) USING_CHAINSAW, BLOCK_DEFENDER_ID
  (= fouled id, consumed by reused INIT step pieces); END_TURN+CHECK_FORGO on EndTurn branch;
  END_PLAYER_ACTION on deselect.
- Control: NEXT_STEP on foul; GOTO_LABEL END on EndTurn/EndPlayerAction. preventBeingFouled
  defender aborts (no-op).
- Rust target: `crates/ffb-engine/src/step/foul/init_fouling.rs`
- Test: char test (Foul cmd → fouls++, foulUsed, defenderId set, NEXT). Java pin: no.
- Parity: (none yet)

### StepFoul  [ ]  ← FOUL DICE SITE
- Java: `mixed/foul/StepFoul.java:43` (BB2020+BB2025)
- Sequence(s): Foul step 5.
- Purpose: roll the foul injury on the prone/stunned defender and publish the drop context.
- Dice (in order), via `UtilServerInjury.handleInjury(InjuryTypeFoul(useChainsaw=false), …,
  ApothecaryMode.DEFENDER)` → `InjuryTypeFoul` (`InjuryTypeFoul.java:25`):
  1. **armour 2d6** `rollArmour()` (`:56`), with foul-assist + DirtyPlayer modifiers added
     (`getFoulAssist` from `UtilPlayer.findFoulAssists`). Broken if sum+mods > AV.
  2. **injury 2d6** `rollInjury()` (`:36`) — **only if armour broken**; modifiers via
     `findInjuryModifiers`. Total → Stunned/KO/Casualty per interpret rules.
  3. **casualty [d16,d6]** — only if injury total ≥10.
  (`InjuryTypeFoulForSpp` only differs in SPP attribution when prayer FoulingFrenzy — same dice.)
- Events (in order): `ReportFoul(defenderId)` (sound FOUL), then the injury reports
  (armour/injury/casualty) → GameEvents `Foul`, `Injury { armour_roll, injury_roll?, casualty? }`,
  `PlayerFellDown`/state change. `UtilServerGame.syncGameModel(this)` flushes mid-step.
- Prompts: none here.
- Params: reads+consumes USING_CHAINSAW. Publishes (broadcast) DROP_PLAYER_CONTEXT
  (`new DropPlayerContext(injuryResultDefender, defenderId, DEFENDER, true)`); END_TURN only when
  the CHAINSAW_TURNOVER_ALL_AV_BREAKS option + armour broken (chainsaw — N/A for lineman foul).
- Control: NEXT_STEP.
- Rust target: `crates/ffb-engine/src/step/foul/foul.rs`
- Test: char test — seed fixes armour (broken) then injury(=Stunned 2-7); assert exactly
  armour 2d6 then injury 2d6 (no casualty), defender STUNNED, DROP_PLAYER_CONTEXT published.
  A second seed: armour broken + injury ≥10 → +casualty [d16,d6]. **Java pin: yes (subtle —
  armour-then-injury ordering + foul-assist modifiers).**
- Parity: (none yet)

### StepReferee  [ ]  ← REFEREE DICE SITE (skill-less still rolls/checks)
- Java: `action/foul/StepReferee.java:32` (COMMON) + behaviour
  `bb2025/SneakyGitBehaviour.java:78` registers the StepReferee `StepModifier` (runs for **every**
  player, skill or not — `UtilCards.hasSkill(actingPlayer, skill)` is just a branch inside).
- Sequence(s): Foul step 8 (ON_END=END_FOULING).
- Purpose: decide whether the ref spots the foul.
- Dice (in order):
  1. **none from the doubles check** — `refereeSpotsFoul` is computed from existing rolls:
     `armorRoll[0]==armorRoll[1]` (armour doubles), OR (armour broken &&) `injuryRoll[0]==
     injuryRoll[1]` (injury doubles), OR (under-scrutiny prayer && armour broken). No new dice.
  2. **Biased-ref spot rolls**: only if `opponentInducementSet.value(SPOT_FOUL)>0` — `rollSkill()`
     1d6 each, spots on >4. Lineman games have no SPOT_FOUL inducement → **0 dice**.
- Events: `ReportReferee(spotsFoul, underScrutiny)` → GameEvent `Referee{spotted}`;
  `ReportBiasedRef` only on the inducement path.
- Prompts: none. Params: reads INJURY_RESULT (defender, mode=DEFENDER) into state.
- Control: spotted → NEXT_STEP (sound WHISTLE) → BRIBES/EJECT; not spotted → GOTO_LABEL
  END (=END_FOULING).
- Rust target: `crates/ffb-engine/src/step/foul/referee.rs`
- Test: char tests — (a) armour roll was doubles → spotted, WHISTLE, NEXT, 0 dice; (b) armour
  not-double + broken + injury doubles → spotted; (c) neither → GOTO END_FOULING. **Java pin:
  yes (subtle — doubles-on-armour-or-injury spotting logic).**
- Parity: (none yet)

### StepBribes  [ ]  ← ARGUE DICE SITE
- Java: `bb2025/foul/StepBribes.java:58` (AbstractStepWithReRoll)
- Sequence(s): Foul step 9 (ON_END=END_FOULING).
- Purpose: offer Argue-the-Call (and bribes inducements) to keep the spotted fouler.
- Dice (in order):
  1. **argue 1d6** `rollArgueTheCall()` (`:208`) — only on `fArgueTheCallChoice==true`. Modified
     by FriendsWithRef (+1 if roll>1) and BiasedRef bonus (lineman: none). Success >5 (keeps
     player); coach banned <2. Per AGENT_CONTRACT §7 the agent **ALWAYS argues** (both sides).
  2. **bribes 1d6** `rollBribes()` (`:177`) — only if a bribe inducement is used (none for
     lineman) → **0 dice**.
- Events: `ReportArgueTheCallRoll(playerId, successful, coachBanned, roll, …)` → GameEvent
  `ArgueTheCall{roll, successful, coach_banned}`; `ReportBribesRoll` only on bribe path.
- Prompts: **ArgueTheCall** dialog (DialogArgueTheCallParameter) — agent argues;
  **Bribes** dialog (declined, none available); BriberyReRoll dialog (N/A).
- Params: command `ArgueTheCall(playerId)` sets choice; publishes (broadcast) FOULER_HAS_BALL,
  ARGUE_THE_CALL_SUCCESSFUL, END_TURN=true. `turnData.coachBanned` on coach-ban roll.
- Control: argue success → GOTO_LABEL END (player stays, but END_TURN already published →
  turnover); argue fail → NEXT_STEP → EJECT_PLAYER. (bribe success → GOTO END.)
- Rust target: `crates/ffb-engine/src/step/foul/bribes.rs`
- Test: char tests — argue roll 6 → successful, player kept, END_TURN, GOTO END_FOULING, exactly
  1 d6; argue roll 1 → not successful + coachBanned, NEXT, 1 d6; argue roll 3 → fail, NEXT.
  **Java pin: yes (subtle — argue thresholds >5/<2 + END_TURN always).**
- Parity: (none yet)

### StepEjectPlayer  [ ]
- Java: `mixed/foul/StepEjectPlayer.java:41` (BB2020+BB2025) + SneakyGitBehaviour eject hook
  (`bb2025/SneakyGitBehaviour.java:35`) — sets the box reason/BANNED base.
- Sequence(s): Foul step 10 (ON_END=END_FOULING). Reached only when ref spotted & argue failed.
- Purpose: send the fouler off — state→BANNED (lineman; SneakyGit-ban-to-KO N/A), into the box,
  refresh boxes, END_TURN.
- Dice: none. Events: player-ejected/sent-to-box → GameEvent `PlayerEjected{playerId}`.
  Prompts: none (agent already argued, declined skill).
- Params: reads FOULER_HAS_BALL, ARGUE_THE_CALL_SUCCESSFUL; init GOTO_LABEL_ON_END, OFFICIOUS_REF.
  Publishes (broadcast) END_TURN=true; CATCH_SCATTER_THROW_IN_MODE=SCATTER_BALL iff foulerHasBall.
- Control: foulerHasBall → NEXT_STEP (to scatter the dropped ball); else GOTO_LABEL END.
- Rust target: `crates/ffb-engine/src/step/foul/eject_player.rs`
- Test: char test — argue-failed fouler → BANNED + off-pitch + END_TURN, 0 dice; ball-carrier
  variant publishes SCATTER_BALL. Java pin: no (mechanical given the hook outcome).
- Parity: (none yet)

### StepEndFouling  [ ]
- Java: `bb2025/foul/StepEndFouling.java:38`
- Sequence(s): Foul step 14 (label END_FOULING).
- Purpose: terminal foul routing → push EndPlayerAction (with END_TURN/CHECK_FORGO) for the
  lineman (canMoveAfterFoul = Sneaky Git skill → N/A; bloodlust → N/A).
- Dice: none. Events: none. Prompts: none.
- Params: reads+consumes END_PLAYER_ACTION, END_TURN, BLOOD_LUST_ACTION, CHECK_FORGO.
- Control: NEXT_STEP; pushes EndPlayerAction sequence.
- Rust target: `crates/ffb-engine/src/step/foul/end_fouling.rs`
- Test: char test asserting EndPlayerAction pushed with the right end-turn flag, 0 dice. Java pin: no.
- Parity: (none yet)

---

## INJURY / APOTHECARY (shared — used by block, foul, move, crowd-push)

### StepFallDown (injury portion)  [ ]
- Java: `bb2025/move/StepFallDown.java:37`
- Sequence(s): Move step 25, BlitzMove step 21, BlitzBlock step 35 (label FALL_DOWN). (Block's
  attacker-down goes through DROP_FALLING_PLAYERS, not this step.)
- Purpose: drop the *acting* player with a typed injury (GFI/dodge/etc. via INJURY_TYPE) and
  publish the result + turnover.
- Dice (in order), via `UtilServerInjury.handleInjury(fInjuryType, attacker=null, defender=
  actingPlayer, …, ApothecaryMode.ATTACKER)` → same armour-then-injury(-then-casualty) ordering
  as a block (the InjuryType decides modifiers; for a plain fall it's InjuryTypeBlock-like):
  1. **armour 2d6**; 2. **injury 2d6** if broken; 3. **casualty [d16,d6]** if ≥10.
- Events: injury reports → `PlayerFellDown`/`Injury`. Prompts: none here (apothecary handled by
  the following APOTHECARY step).
- Params: reads INJURY_TYPE, COORDINATE_FROM. Publishes (via `dropPlayer(...,ATTACKER,true)`):
  DROPPED_BALL_CARRIER, CATCH_SCATTER_THROW_IN_MODE=SCATTER_BALL (if on ball), END_TURN (turnover
  if carrier/own-team), and INJURY_RESULT (broadcast). END_TURN also if
  `fInjuryType.fallingDownCausesTurnover()` and not PASS_BLOCK.
- Control: NEXT_STEP.
- Rust target: `crates/ffb-engine/src/step/move/fall_down.rs` (injury delegates to shared injury
  helper in `crates/ffb-engine/src/engine/injury.rs`)
- Test: char test — forced GFI fail fall: armour 2d6 (+injury if broken), END_TURN, INJURY_RESULT
  published. **Java pin: yes (subtle — armour/injury ordering shared with block).**
- Parity: (none yet)

### StepDropFallingPlayers  [ ]  ← BLOCK KNOCKDOWN INJURY SITE
- Java: `bb2025/shared/StepDropFallingPlayers.java:59`
- Sequence(s): Block step 35 (label DROP_FALLING_PLAYERS), BlitzBlock step 37.
- Purpose: roll injuries for everyone left FALLING after a block — **defender first, then
  attacker** — and publish a SteadyFootingContext / drop context per player.
- Dice (in order) — for each FALLING player, via `UtilServerInjury.handleInjury(InjuryTypeBlock
  variant, …)`:
  - **Defender** (if FALLING): InjuryTypeBlock REGULAR (or BlockProne/BlockStunned if
    oldDefenderState prone/stunned — same dice, different modifier set). ApothecaryMode.DEFENDER.
    1. armour 2d6 → 2. injury 2d6 if broken → 3. casualty [d16,d6] if ≥10.
  - **Attacker** (if FALLING): InjuryTypeBlock (or InjuryTypeDropGFI if fellFromRush).
    ApothecaryMode.ATTACKER. Same 1/2/3 dice ordering. Attacker-down also publishes END_TURN
    (turnover).
  Ordering is **defender block first, then attacker** (matches AGENT_CONTRACT §8 Block: "attacker
  first" refers to which side is the *acting* attacker — here Java rolls the *defender's* injury
  first in this step, then the attacker's; pin from the Java trace).
- Events: per-player armour/injury/casualty reports → `Injury`, `PlayerFellDown`. Prompts: none
  (apothecary declined later in APOTHECARY step).
- Params: reads OLD_DEFENDER_STATE. Publishes (broadcast) INJURY_RESULT (per player), END_TURN
  (attacker down / own-team drop), STEADY_FOOTING_CONTEXT / DROP_PLAYER_CONTEXT.
- Control: NEXT_STEP. (Saboteur paths bypass Steady Footing — N/A lineman.)
- Rust target: `crates/ffb-engine/src/step/block/drop_falling_players.rs`
- Test: char test — both knocked down: assert defender armour 2d6 then attacker armour 2d6 in
  that order (each +injury when broken), INJURY_RESULT per player, END_TURN on attacker down.
  **Java pin: yes (subtle — per-player ordering + which injury type variant by oldDefenderState).**
- Parity: (none yet)

### StepApothecary  [ ]  (skill-less = DECLINE, usually NO_APOTHECARY)
- Java: `bb2025/shared/StepApothecary.java:91`
- Sequence(s): Block steps 33/39/42 (CROWD_PUSH/DEFENDER/ATTACKER), Foul steps 7/12, Move steps
  27/28/30, plus ATTACKER/TRAP_DOOR variants — keyed by APOTHECARY_MODE.
- Purpose: optionally use apothecary/regeneration on the matching injury, then **apply** the
  injury (`fInjuryResult.applyTo(this)`).
- Dice (in order) — lineman has no apothecary and no Regeneration, so the lineman path is:
  - pre-regeneration: skipped (canRollToSaveFromInjury = Regeneration → N/A).
  - apothecaryStatus is NO_APOTHECARY (set in `evaluateInjuryContext`) → **no roll, no dialog** →
    `fInjuryResult.applyTo`. **0 dice for a lineman.** (With apothecary: `rollApothecary` re-rolls
    casualty `[d16,d6]` — out of scope.) Raise-dead/getting-even gated on opponent skills — N/A.
- Events: `ReportApothecaryRoll`/`ReportApothecaryChoice` only on the apothecary path (N/A);
  the injury report itself is emitted (if not already) → `Injury`. Prompts: **ApothecaryChoice**/
  UseApothecary dialog only when an apothecary is available — agent **declines** (AGENT_CONTRACT
  §7). Lineman: no prompt.
- Params: init APOTHECARY_MODE (mand); reads INJURY_RESULT (only the matching mode is captured).
- Control: NEXT_STEP after applying; CONTINUE while a dialog is up (apothecary path only).
- Rust target: `crates/ffb-engine/src/step/shared/apothecary.rs`
- Test: char test — injury result with NO_APOTHECARY → applies state, 0 dice, NEXT. Java pin: no
  (lineman path is a straight apply; the apothecary roll path is out of scope).
- Parity: (none yet)

### StepPlaceBall  [ ]  (skill-less = PASS-THROUGH)
- Java: `bb2025/shared/StepPlaceBall.java:46`
- Sequence(s): Block steps 38/41, Foul (via drop), Move steps 26, EndPlayerAction/EndTurn — paired
  with each drop/apothecary.
- Purpose: Safe Pair of Hands ball placement when a carrier is knocked down. **Lineman lacks the
  skill** → `setup()` finds `skill==null` → immediate NEXT_STEP (the ball just scatters normally
  via the CATCH_SCATTER_THROW_IN path). Only acts when `playerId!=null && mode==SCATTER_BALL`.
- Dice: none. Events: none for lineman (ReportPlaceBallDirection only on the skill path).
  Prompts: SkillUse / field-coordinate selection only with the skill — N/A.
- Params: reads CATCH_SCATTER_THROW_IN_MODE, DROPPED_BALL_CARRIER, REVERT_END_TURN.
- Control: NEXT_STEP (immediate for lineman).
- Rust target: `crates/ffb-engine/src/step/shared/place_ball.rs` (no-op for lineman)
- Test: char test asserting immediate NEXT, 0 dice. Java pin: no.
- Parity: (none yet)

---

## Dice-order summary (skill-less lineman — what parity actually checks)

| Step | Dice in order |
|---|---|
| StepBlockRoll | `rollBlockDice(nrOfDice)` = `|n|` block dice |
| StepDropFallingPlayers | defender: armour 2d6 → [injury 2d6 if broken → casualty d16,d6 if ≥10]; then attacker: same |
| StepBlockChoice (SKULL/BothDown) | none (attacker falls → DROP_FALLING_PLAYERS rolls) |
| StepPushback (crowd) | injury 2d6 → [casualty d16,d6 if ≥10] (no armour) |
| StepFoul | armour 2d6 → [injury 2d6 if broken → casualty d16,d6 if ≥10] |
| StepReferee | none (doubles read off existing rolls); biased-ref 1d6 only if SPOT_FOUL inducement |
| StepBribes | argue 1d6 (always, agent argues); bribe 1d6 only if bribe used |
| StepApothecary / StepPlaceBall / StepFallDown(no fall) / Init*/EndBlocking/EndFouling/Dauntless/BothDown/Followup | none for lineman |

`rollCasualty = [rollDice(16), rollDice(6)]` — **d16 then d6**, only when injury total ≥ 10
(`bb2025/RollMechanic.java:92`). Injury 2-7 Stunned / 8-9 KO / ≥10 Casualty
(`bb2025/RollMechanic.java:97`). Thick Skull / Stunty / Regeneration / Dodge / Block / Wrestle /
Juggernaut / SneakyGit / Dauntless / Safe-Pair-of-Hands all absent on a plain lineman → their
branches are pass-throughs with zero dice.
