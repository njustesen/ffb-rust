# FFB-Rust Session State

## Current Status (2026-07-01)

**Approach:** 1:1 Java-to-Rust translation. Every Java class → one Rust file, written directly from Java source. No reactive parity fixes.

**engine.rs deleted.** `driver.rs` is now the live code path — `Box<dyn Step>` dispatch via `make_step()`, `DriverGameState` game loop, `GameState` type alias for backward compat.

**Translation progress:** ~1,695/2,521 files formally implemented = **~67%**

**Tests:** 5,815 passing (1 ignored)

**Current phase:** Phase V (in progress)

---

## Completed Phases

- **Phases 1–4** (2026-06-24): Step trait + framework.rs, UtilServerSteps, driver.rs, engine.rs deleted → 2,557 tests
- **Phase A** (2026-06-24): TRANSLATION_TRACKER reconciled — 952 ✓, 1569 ~, 458 — → 2,686 tests
- **Phase B** (2026-06-24): NamedProperties/SkillId.properties() (B1), UtilPlayer full impl (B2), GoForItModifierFactory (B3), BlockResultFactory (B4) → 2,746 tests
- **Phase C session 1** (2026-06-24): DiceInterpreter, PickupModifierFactory, DodgeModifierFactory, StepWeather, StepGoForIt, StepStandUp, StepPickUp, StepMoveDodge → 2,786 tests
- **Phase C session 2** (2026-06-25): StepCoinChoice, StepReceiveChoice, StepSetup, StepKickoff, StepEndKickoff, StepEndTurn (inducement push), StepInitInducement, StepEndInducement, StepInitSelecting (full rewrite), StepInitEndGame, StepWeatherMage, ActingPlayer.{suffering_blood_lust, forgone}, inducement_sequence() → 2,805 tests
- **Phase C session 3** (2026-06-25): ActingPlayer.{strength, fell_from_rush}, FieldModel.chomped, step_trickster home_playing flip, step_pushback same-team EndTurn, step_breathe_fire remove_blitz_target, step_move updatePlayerAndBallPosition, step_pick_up ball_moving=false, step_stand_up MA/free-standup/PRONE, step_init_moving defender_id, step_jump fumble reposition, step_resolve_pass out_of_bounds → 3,002 tests
- **Phase C session 4** (2026-06-25): step_pick_up no-TZ scatter branch, step_stand_up turn_started+concession_possible, step_init_bomb bomb_used, step_init_moving turn_started+concession_possible+has_moved+per-action TurnData flags, step_end_turn new_half h2_kickoff_sequence, step_followup updatePlayerAndBallPosition+PlayerEnteringSquare → 3,009 tests
- **Phase D** (2026-06-25): AbstractStepWithReRoll + ReRollState, UtilServerReRoll (ask_for_reroll_if_available, use_reroll), UtilCards.has_unused_skill_with_property, end_turn_sequence fix, end_game_sequence new, StepEndTurn end-game paths, re-roll branches for StepGoForIt/StepPickUp/StepMoveDodge/StepStandUp/StepJump → 3,038 tests
- **Phase F** (2026-06-26): All 50 concrete injury type files in `injury/injuryType/`, ModificationAwareInjuryType trait + free function, `step::framework` made pub(crate) → 3,206 tests
- **Phase G** (2026-06-26): UtilServerInjury.handleInjury() + evaluateInjuryContext() (full injury pipeline), StepBlockRoll multi-die re-roll (Brawler/Hatred/Pro/ConsummateProfessional/SavageBlow/single-die/multi-die), 6 new Action variants for block re-rolls → 3,233 tests
- **Phase H** (2026-06-26): StepCatchScatterThrowIn full implementation — all 7 private methods (bounce_ball, scatter_ball, scatter_bomb, throw_in_ball, deactivate_cards, diving_catch, catch_ball) + execute_step dispatch wiring; CatchModifierFactory (H0, ffb-mechanics); UtilServerCatchScatterThrowIn (H1, ffb-engine); framework CatchScatterThrowInMode.is_bomb() → 3,250 tests
- **Phase J** (2026-06-26): All BB2016 generators (15 files), BB2020 generators (26 files via agent), mixed generators (end_turn, kickoff, card, pile_driver, quick_bite), common generators (riotous_rookies, spiked_ball_apo, wizard) — all with build_sequence() + unit tests. Added labels: SKIP_PILE_DRIVER, END_KICK_TEAM_MATE, FUMBLE_TTM_PASS, APOTHECARY_DEFENDER, KICK_TM_DOUBLE_ROLLED. Added StepParameter variants: AllowMoveAfterPass, CardId. Added StepId::RiotousRookies → 3,494 tests
- **Phase K** (2026-06-26 → 2026-06-29): BB2016 StepMissedPass (passDeviates two-path scatter), BB2020 StepTreacherous (InjuryTypeStab + DropPlayerContext), BB2020 StepBalefulHex (change_hypnotized), BB2020 StepBlackInk (no distracted filter), BB2020 StepCatchOfTheDay, BB2020 StepEndFuriousOutburst (bb2020 generator, no check_forgo), BB2020 StepLookIntoMyEyes, BB2020 StepPrayer, BB2020 StepRaidingParty, BB2020 StepWisdomOfTheWhiteDwarf — 10 files, 43 new tests → 3,669 tests
- **Phase I** (2026-06-29): Infrastructure sweep — FieldModel (multi_block_targets, dice_decorations fields/methods), ServerUtilBlock::update_dice_decorations, UtilServerPlayerMove::update_move_squares, UtilBox::put_player_into_box, InjuryResult::apply_to. BB2025: step_end_blocking (all 8 TODOs cleared), step_end_moving (updateMoveSquares/updateDiceDecorations), step_apothecary (apply_to wired) → 3,704 tests
- **Phase L** (2026-06-29): 11 BB2020 steps — step_breathe_fire, step_blitz_turn, step_stalling_player, step_prayers, step_select_blitz_target, step_end_turn, step_apply_kickoff_result, step_kickoff_scatter_roll, step_special_effect, step_state_multiple_rolls, step_then_i_started_blastin → 3,798 tests
- **Phase M** (2026-06-29): 63 BB2016 step files implemented across all subdirs (block/, move_/, pass/, foul/, start/, end/, ttm/, special/ + top-level) — full field/method translation with ≥3 tests per file → 4,183 tests
- **Phase N** (2026-06-30): 129 skillbehaviour files (bb2025×40, bb2020×39, bb2016×34, mixed×14, common×1) — `execute_step_hook` method + 2 tests per file. SkillBehaviour trait extended with default `execute_step_hook(&self, game) -> bool` → 4,439 tests
- **Phase O** (2026-06-30): BB2020 block/ (9 files), move_/ (11 files), foul/ (3), gaze/ (2), inducements/ (3), kickoff/ (2), start/ (1), end/ (5), multiblock/ (4) — 40 files total, full Java translation with ≥3 tests per file. StepOutcome extended with `events`/`prompt` fields + `with_event`/`with_prompt`/`with_events` methods. StepParameter variants typed (ApothecaryMode, BlockResult, InjuryResult, SteadyFootingContext, DropPlayerContext, InducementPhase, DispatchPlayerAction). CatchScatterThrowInMode PascalCase→SCREAMING_SNAKE_CASE fixed across all files. StepId::BloodLust/PlayCard added. → 4,828 tests
- **Phase P** (2026-06-30): BB2020 pass/ (6 files: StepPass, StepEndPassing, StepIntercept, StepHailMaryPass, StepMissedPass, StepResolvePass), BB2020 shared/StepCatchScatterThrowIn (~700 lines, 14 tests), BB2020 ttm/StepAlwaysHungry (17 tests, 4 blockers resolved), mixed/block/ (StepBlockBallAndChain, StepProjectileVomit), mixed/blitz/ (StepRemoveTargetSelectionState, StepSelectBlitzTargetEnd), mixed/inducements/StepPlayCard, mixed/end/StepPenaltyShootout, mixed/special/StepEndBomb, mixed/ttm/ (StepSwoop, TtmToCrowdHandler), mixed/kickoff/ (StepInitKickoff, StepKickoff, StepSwarming), mixed/move_/ (StepMoveBallAndChain, StepResetFumblerooskie, StepTentacles, StepTrapDoor), mixed/pass/ (StepAllYouCanEat, StepInitPassing, StepPassBlock), mixed/multiblock/ (AbstractStepMultiple, StepDauntlessMultiple, StepFoulAppearanceMultiple), mixed/shared/ (StepAnimalSavagery, StepPickMeUp). ffb-model: `penalty_score: i32` added to TeamResult. Action variants added: UseReRollForTarget, LordOfChaosChoice, IndomitableChoice, PlayerChoice. Network encoder updated for new action variants. → 5,045 tests
- **Phase Q** (2026-06-30): Generator completions + server utility wave
  - Completed all 15 BB2016 generators + 26 BB2020 generators (unit tests pass, marked ✓)
  - Implemented 32 root abstract generator param structs
  - Added 10 calc utility files: agility_calc, block_dice, block_result, catch, foul, movement, pass, passing_distance, roll, scatter
  - Added 7 more calc utils: kickoff_event, post_match, special_roll, stat, throw_in, weather, marker_loading
  - Added 6 game/dialog/setup server utils: util_server_game, util_server_dialog, util_server_setup, util_server_start_game, util_server_inducement_use, util_server_player_swoop
  - Added 4 block/player/pushback utils: server_util_block, server_util_player, util_server_pushback, util_server_player_move
  - Filled test gaps in mixed steps (3A) and phase steps + StepInitBomb (3B)
  - +428 new tests (5100 → 5528)
- **Phase T** (2026-07-01): Long-tail DEFERRED resolution sweep
  - **`skill_id.rs`**: Added 3 missing `SkillId::properties()` entries — `PutridRegurgitation` (3 props), `ViolentInnovator` (`grantsSppFromSpecialActionsCas`), `MaximumCarnage` (`canPerformSecondChainsawAttack`). 3 new tests.
  - **`acting_player.rs`**: Added `has_passed: bool` field (Java: `fHasPassed`) to `ActingPlayer`. No tests needed (covered by downstream step tests).
  - **`step_pass.rs` (bb2016)**: Full implementation of the BB2016 pass step — resolves thrower/bomb, calls `PassMechanic::evaluate_pass_simple`, branches on `PassResult` (ACCURATE/FUMBLE/SAVED_FUMBLE/INACCURATE/WILDLY_INACCURATE). Added `mech_result: Option<PassResult>` field. 4 new tests.
  - **`step_init_passing.rs` (bb2016)**: Implemented `has_passed = true`, `concession_possible = false`, `turn_started = true`, `pass_used`/`hand_over_used` TurnData flags. 2 new tests.
  - **`step_special_effect.rs` (bb2016)**: Extracted `is_special_effect_successful()` function (Java: `DiceInterpreter.isSpecialEffectSuccesful`) — Lightning ≥2, Zap =6 or (>1 and ≥strength), Fireball/Bomb ≥4, None=false. Replaced stub. 4 new tests.
  - **`step_mvp.rs` (bb2016)**: Wired `player_state.is_killed()` filter to exclude dead players from MVP pool. 1 new test.
  - **`step_end_passing.rs` (bb2020)**: Fixed misplaced `has_passed = false` / `pass_coordinate = None` — moved inside the `suffering_blood_lust && bloodlust_action.is_some()` if-block, removed duplicate `pass_coordinate = None`. 2 new tests.
  - **`step_end_passing.rs` (bb2025)**: Implemented the bloodlust if-block that was only a comment — `has_passed = false`, `pass_coordinate = None`, change player action, push Move sequence. 2 new tests.
  - +87 new tests (5,655 → 5,742)
- **Phase S** (2026-07-01): DEFERRED resolution sweep
  - **`step_right_stuff.rs` (bb2016)**: Implemented minimum-roll calculation using `Bb2016RightStuffModifiers`, filtering out TACKLEZONE modifiers (predicates not wired), matching KTM range string via `get_name()`. 2 new tests.
  - **`step_right_stuff.rs` (bb2020)**: Implemented `RightStuffModifierFactory::for_rules` + `RightStuffContext`, mapped `ModelPassResult` → `MechanicPassResult`. 2 new tests.
  - **`step_move_ball_and_chain.rs` (bb2016)**: Wired D8 scatter via `Direction::for_roll` + `FieldCoordinate::step`. 1 new test.
  - **`named_properties.rs`**: Added 3 new Juggernaut cancel constants (`CANCELS_CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN`, `CANCELS_CAN_REFUSE_TO_BE_PUSHED`, `CANCELS_PREVENT_OPPONENT_FOLLOWING_UP`).
  - **`skill_id.rs`**: Added all 3 `CancelSkillProperty` strings to `SkillId::Juggernaut`; added `canBeKicked` to `SkillId::RightStuff` (was missing from Java parity).
  - **`step_followup.rs` (bb2020 + bb2025)**: Implemented Juggernaut/Fend auto-cancel logic — when attacker has `cancelsPreventOpponentFollowingUp` and action is BLITZ (or MOVE + `blocksDuringMove`), Fend is auto-cancelled. 1 new test each.
  - **`step_jump.rs` (bb2016)**: Confirmed BB2016 `JumpModifierCollection` is empty (from Java source); changed to `agility_with_modifiers()`.
  - **`can_kick_team_mate` / `can_throw_team_mate` (bb2016 + bb2020 + bb2025 `step_end_moving.rs`)**: Implemented `UtilPlayer.canKickTeamMate` / `canThrowTeamMate` as free functions using edition-specific `TtmMechanic`. Wired into `can_make_next_move` branch in all 3 editions. 3 new tests in bb2016.
  - +57 new tests (5,598 → 5,655)
- **Phase R** (2026-07-01): Step body completions + bulk TODO→DEFERRED sweep
  - **`step_always_hungry.rs` (bb2016)**: Full implementation — always-hungry roll (2+), skill-usage tracking via `used_skills.insert(SkillId::AlwaysHungry)`, escape roll (2+), publishes `PassResult::Fumble` on escape success, goes to failure label on escape failure. 14 new tests (both `DiceInterpreter::is_always_hungry_successful` and `is_escape_from_always_hungry_successful` were already ported; both return `roll >= 2`).
  - **`skill_id.rs`**: Added `SkillId::BallAndChain => &["movesRandomly", "blocksLikeChainsaw"]` so `has_skill_property(MOVES_RANDOMLY)` returns true for BallAndChain carriers.
  - **`step_move_ball_and_chain.rs`**: Fixed 3 broken tests by adding `add_ball_and_chain_player` test helper; all 16 tests pass.
  - **`step_init_feeding.rs`**: Implemented feed-on-player and bite-spectator paths. 18 tests.
  - **`step_apothecary.rs`**: Implemented `InjuryResult::apply_to` wiring, cured state computation (KO→Stunned, else Reserve). 39 tests.
  - **`step_kickoff_scatter_roll.rs`**: Implemented `game.field_model.out_of_bounds = self.touchback`.
  - **`step_apply_kickoff_result.rs`**: Implemented cheerleaders/coaches bonus in extra-reroll calculation.
  - **Bulk TODO→DEFERRED sweep**: Converted all `// TODO(...)`, `// TODO:`, and `/// TODO` inline comments to `// DEFERRED(...)` across all step directories (bb2016, bb2020, bb2025, mixed, action, game, phase, generator). Stub placeholder files (`// TODO: full implementation.`) intentionally left unchanged.
  - +70 new tests (5528 → 5598)
- **Phase U** (2026-07-01): Event emission, infrastructure stubs, game lifecycle steps
  - **Sub-Phase U2**: DEFERRED(events) → wired `GameEvent::PassDeviate`/`ScatterBall` in `step_missed_pass.rs` (bb2020); `GameEvent::ApothecaryRoll` in `step_apothecary.rs` (bb2016 + bb2025); `GameEvent::KickoffRiot` in `step_apply_kickoff_result.rs` (bb2016).
  - **Sub-Phase U2 (gaze/blitz targets)**: `GameEvent::SelectBlitzTarget` in `step_select_blitz_target.rs` (bb2020); `GameEvent::SelectGazeTarget` in `step_select_gaze_target.rs` (bb2020). Added 4 new GameEvent variants: `Block`, `ApothecaryRoll`, `SelectBlitzTarget`, `SelectGazeTarget`.
  - **Sub-Phase U4**: Infrastructure stubs — `StepIdFactory` full impl (130 name↔id mappings, 6 tests), `StepActionFactory` full impl (6 action mappings, 7 tests), `StepModifierTrait` + `StepCommandStatus` + `sort_by_priority` (4 tests), `HookPoint` enum + `StepHookHandler` trait (3 tests). Created `factory/mod.rs` and `model/mod.rs`.
  - **Sub-Phase U5**: Game lifecycle steps — `StepInitStartGame` full impl (standalone fast-path: set `GameStatus::Active` on `start()`, handle `Action::StartGame` in `handle_command()`, 8 tests); `StepEndGame` full impl (set `GameStatus::Finished`, 5 tests). Added `Action::StartGame { home: bool }` variant.
  - +42 new tests (5,742 → 5,784)
- **Phase V** (2026-07-01): Root mixed steps, phase/game step audit, model additions
  - **`step_throw_keg.rs` (mixed)**: Full implementation — `execute_step`, `hit_player`, `fail()` with fumble path, re-roll cycle. 10 tests.
  - **`SkillId::BeerBarrelBash`**: Added `canThrowKeg` property (was missing from Java parity).
  - **`step_riotous_rookies.rs` (phase/inducement)**: Implemented from stub — `start()`, `hire_riotous_rookies_for_team`, `roll_riotous_rookies`; core player-creation DEFERRED on InducementSet/RosterPlayer. 7 tests.
  - **`util_inducement_sequence.rs` (game/start)**: Implemented `calculate_inducement_gold` (TV-diff + petty-cash logic). 7 tests.
  - **`TeamResult`**: Added `petty_cash_transferred` + `petty_cash_used` fields (Java: `fPettyCashTransferred`/`fPettyCashUsed`).
  - **`GameRng`**: Added `roll_riotous_rookies()` method (Java: `DiceRoller.rollRiotousRookies`).
  - **`step_first_move_furious_outburst.rs`**: Added `.remove_selected_blitz_target()` to state chain (Java parity fix).
  - Phase R-U uncommitted work committed as single commit.
  - +31 new tests (5,784 → 5,815)

---

## Phase D — Completed (2026-06-25)

Re-roll & injury infrastructure implemented:

| Item | Status | Notes |
|---|---|---|
| `AbstractStepWithReRoll` (`ReRollState` + `find_skill_reroll_source`) | ✓ | `abstract_step_with_re_roll.rs`, 4 tests |
| `UtilServerReRoll` (`ask_for_reroll_if_available`, `use_reroll`) | ✓ | `util_server_re_roll.rs`, 5 tests |
| `UtilCards.has_unused_skill_with_property` | ✓ | `util_cards.rs`, 3 tests |
| `end_turn_sequence(check_forgo)` fix | ✓ | `sequences.rs` — was NoOp-filled |
| `end_game_sequence(admin_mode)` | ✓ | `sequences.rs` — new function |
| StepEndTurn end-game paths | ✓ | `step_end_turn.rs` — calls end_game/h2_kickoff |
| StepGoForIt re-roll (GFI → skill auto-use → TRR) | ✓ | 10 tests |
| StepPickUp re-roll (PICKUP → skill auto-use → TRR) | ✓ | 9 tests |
| StepMoveDodge re-roll (DODGE → skill auto-use → TRR) | ✓ | 8 tests |
| StepStandUp re-roll (STAND_UP → TRR) | ✓ | 8 tests |
| StepJump re-roll (JUMP → TRR) | ✓ | 13 tests |

## Phase D — Remaining blockers (now cleared by Phase G)

| Infra | Status |
|---|---|
| `InjuryResult` / `UtilServerInjury.handleInjury()` | ✓ Cleared (Phase G) |
| StepBlockRoll re-roll (complex multi-die path, Brawler/Hatred) | ✓ Cleared (Phase G) |

## Phase H — Planned (2026-06-26): Ball Resolution

**Goal:** Full 1:1 implementation of `StepCatchScatterThrowIn` — the single biggest gap in the codebase. Every game action involving ball movement (catch, scatter, throw-in, kickoff, hand-off, deflection, bomb) routes through this step. Currently a stub with 8 TODO sections.

### Key architectural note

Java `getGameState().pushCurrentStepOnStack() + setNextAction(NEXT_STEP)` → Rust `StepAction::Repeat`.
The driver re-calls `start()` on the same step instance (it keeps the step in `self.current`), preserving all mutable fields. This lets StepCatchScatterThrowIn loop through multiple `CatchScatterThrowInMode` values in a single game tick.
`StepAction::Continue` = waiting for user dialog.

### H1: UtilServerCatchScatterThrowIn (65-line Java → currently empty stub)

| Method | Java | Notes |
|--------|------|-------|
| `find_scatter_coordinate(start, dir, dist)` | `findScatterCoordinate` | Delegates to `mechanics::scatter::scatter_coordinate` |
| `find_diving_catchers(game, team, coord)` | `findDivingCatchers` | Adjacent players with `canAttemptCatchInAdjacentSquares` |

Unit tests: 4 (coordinate offsets all 8 directions, no divers if no skill, finds divers adjacent)

### H2: StepCatchScatterThrowIn private methods (in dependency order)

| Method | Java lines | Description |
|--------|-----------|-------------|
| `bounce_ball()` | ~50 | Roll d8 direction, compute end square, in-bounds: player with TZ → CATCH_SCATTER; no TZ → FAILED_CATCH; OOB → THROW_IN or TOUCHBACK (kickoff) |
| `scatter_ball()` | ~55 | Loop ≤3 squares: roll d8 each step; stop at OOB → THROW_IN; stop at player with TZ → CATCH_SCATTER; else SCATTER_BALL |
| `scatter_bomb()` | ~55 | Same as scatter_ball but on bomb_coordinate; OOB → sets bomb_coordinate=null, bomb_moving=false, returns null |
| `throw_in_ball()` | ~40 | Corner-check, roll direction D3 or D6, roll 2D6 distance; advance step-by-step; if end OOB → loop (THROW_IN again), else CATCH_THROW_IN |
| `deactivate_cards()` | ~10 | For every player: deactivate WHILE_HOLDING_THE_BALL cards if player ≠ ball carrier |
| `diving_catch(coord)` | ~60 | ASK_HOME → find divers (show dialog or skip); ASK_AWAY → same; PROCESS → attempt catch_ball for each declared catcher; if none → SCATTER_BALL |
| `catch_ball()` | ~115 | AG roll using `agility::minimum_roll` + `CatchModifierFactory`; success → place ball/bomb, return null; fail → ask for re-roll if available; second fail → FAILED_CATCH |

### H3: Outer execute_step() wiring

- After each private-method call: if mode still set → `StepAction::Repeat`; if dialog shown → `StepAction::Continue`; if null → terminal path
- Terminal: publish `CatcherId` (ball/bomb carrier at final coord); if QuickBite adjacency found → push `quick_bite_sequence()`; if kickoff and ball OOB → publish `Touchback(true)`
- `deactivate_cards()` call at terminal

### H4: Secondary sweep — step stubs with easy TODOs

After H2-H3 are green, sweep these partial files that are close to done:

| File | Java lines | Gap |
|------|-----------|-----|
| `step_end_blocking.rs` | 506 | TODO: `updateDiceDecorations`, `removePlayerBlockStates`, `clear_multi_block_targets`, bloodlust after block |
| `step_apothecary.rs` | 557 | Check how complete; may need Igor/MortuaryAssistant paths |
| `step_end_moving.rs` | 391 | Check remaining TODOs; check_touchdown wiring added in Phase E |
| `step_drop_falling_players.rs` | 252 | Check remaining TODOs |
| `step_handle_drop_player_context.rs` | 189 | Check remaining TODOs |

### Unit test targets

| Batch | Tests |
|-------|-------|
| UtilServerCatchScatterThrowIn | +4 |
| bounce_ball | +3 |
| scatter_ball | +4 |
| scatter_bomb | +3 |
| throw_in_ball | +3 |
| deactivate_cards | +2 |
| diving_catch | +3 |
| catch_ball | +6 |
| execute_step integration | +8 |
| **Total target** | **+36** |

Expected workspace total after Phase H: **≥ 3,269 tests**

---

## Phase E — In Progress (2026-06-25)

Re-roll cycle implementations + NamedProperties constant sweep:

| Item | Status | Notes |
|---|---|---|
| StepBlockChainsaw re-roll | ✓ | consume + offer TRR after backfire-triggering roll=1 |
| StepBlockRoll re-roll offer | ✓ | `ask_for_reroll_if_available` → prompt (was auto-applying) |
| StepBoneHead re-roll | ✓ | full cycle: consume + offer + decline→fail; 2 new tests |
| StepReallyStupid re-roll | ✓ | full cycle: consume + offer + decline→fail; 2 new tests |
| StepJumpUp re-roll | ✓ | full cycle: consume at top + offer after fail; 2 new tests |
| StepDauntless re-roll | ✓ | added re-roll fields + full cycle; 2 new tests |
| StepPass re-roll | ✓ | offer after INACCURATE/FUMBLE; decline path; 2 new tests |
| skill markings (canTeleportBefore/AfterAvRollAttack, WatchOut, FlashingBlade, ForceSecondBlock) | ✓ | 4 files |
| check_touchdown wiring (EndFeed, EndInducement, EndPunt, EndMoving, EndPassing) | ✓ | 5 files |
| UseSkill routing by property (StepPass, StepHailMaryPass, StepRightStuff) | ✓ | 3 files |
| StepInitBomb explode_skill_used auto-skip | ✓ | test updated |
| scattersSingleDirection = hasSwoop (StepDispatchScatterPlayer) | ✓ | key insight |
| SkillId::Swoop properties (ttmScattersInSingleDirection) | ✓ | skill_id.rs |
| NamedProperties constant sweep (ffb-engine, ffb-mechanics, ffb-model) | ✓ | all `has_skill_property("string")` → constants; ~60 replacements across 35+ files |
| out_of_bounds field wiring (StepMissedPass, StepEndPassing) | ✓ | FieldModel.out_of_bounds already existed |

Blocked (executeStepHooks or other unported infra):
- StepShadowing — entirely `executeStepHooks` delegated
- StepAlwaysHungry — requires `UtilCards.hasUnusedSkillWithProperty(mightEatPlayerToThrow)`
- StepRightStuff — requires RightStuffModifierFactory, SteadyFootingContext (complex)
- StepBlockDodge, StepFoulAppearance, StepTentacles, StepAnimalSavagery — executeStepHooks
- game.last_defender_id — field not yet on Game (used by MaximumCarnage path)
- push_player inner-fn publish — pushback ball-scatter params require publish-from-inner-fn pattern

---

## Architecture

- `framework.rs` — `Step` trait, `StepOutcome`, `StepId`, `StepParameter`, `SequenceStep`, `test_team`
- `driver.rs` — `make_step()` dispatch, `DriverGameState` loop, `GameState` alias, `new_game()` test helper
- `sequences.rs` — sequence functions (`start_game_sequence`, `h2_kickoff_sequence`, `inducement_sequence`, etc.)
- `bb2025/**` — 142 step files (many still have stub bodies)

## Java Source Location

`C:\Users\Admin\niels\ffb\ffb\ffb-server\src\main\java\com\fumbbl\ffb\server\step\`
