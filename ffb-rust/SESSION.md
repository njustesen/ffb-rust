# FFB-Rust Session State

## Current Status (2026-07-06)

**Approach:** 1:1 Java-to-Rust translation. Every Java class → one Rust file, written directly from Java source. No reactive parity fixes.

**engine.rs deleted.** `driver.rs` is now the live code path — `Box<dyn Step>` dispatch via `make_step()`, `DriverGameState` game loop, `GameState` type alias for backward compat.

**Translation progress:** 2,521/2,521 files formally implemented = **100% ✓** (0 partial, 458 skip)

**Tests:** 9,022 passing (1 ignored)

**Current phase:** Phase ZA — headless: resolution sweep COMPLETE (24 genuine gaps remain, all blocked by missing infrastructure)

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
- **Phase W7g** (2026-07-02): Coverage sweep — modifier collections + model value types
  - **Modifier collections** (18 files): bb2016/bb2020/bb2025/mixed catch/interception/pass/right_stuff/jump/jump_up/go_for_it/dodge size tests. Base class pre-population accounted for.
  - **`pass_result.rs`** + **`wording.rs`** + **`stats_drawing_modifier.rs`**: 10 new tests (enum names, getters, positive_improves/positive_impairs logic).
  - **`bb2016/bb2020/bb2025/serious_injury.rs`**: 16 new tests (is_dead, is_poison, get_injury_attribute, RIP name).
  - **`model/injury_attribute.rs`**: 4 tests (for_name round-trip, prefix stripping, unknown, unique ids).
  - **`model/catch_scatter_throw_in_mode.rs`**: 4 tests (is_bomb, for_name).
  - **`model/special_effect.rs`**: 3 tests (is_wizard_spell, for_name).
  - **`model/client_mode.rs`**: 3 tests (for_name, unknown, round-trip).
  - +58 new tests (6,145 → 6,203)
- **Phase W7f** (2026-07-02): Coverage sweep continued + TODO fixes
  - **`bb2020/injury_mechanic.rs`** + **`bb2025/injury_mechanic.rs`**: `can_use_apo` fixed — now calls `ApothecaryMechanic::apothecary_types` instead of returning `false`. Cleared 2 TODOs. 2+2 new tests.
  - **`bb2025/jump_mechanic.rs`**: 5 new tests (can_still_jump, is_valid_jump boundaries).
  - **`modifiers/bb2016/dodge_modifier_collection.rs`**: 1 test (16 modifiers — base 8 tacklezone + 8 prehensile tail).
  - **`modifiers/bb2020/interception_modifier_collection.rs`**: 1 test (24 modifiers total).
  - **`modifiers/bb2020/casualty_modifier.rs`**: 3 tests (get_modifier, applies_to_context, report_string).
  - **`modifiers/bb2020/casualty_niggling_modifier.rs`**: 2 tests (get_modifier, report_string).
  - +16 new tests (6,129 → 6,145)
- **Phase W7e** (2026-07-01): Coverage sweep — added tests to 0-test mechanics files
  - **`bb2016/bb2020/bb2025/ttm_mechanic.rs`**: 21 new tests (`minimum_roll`, `handle_kick_like_throw`, availability flags).
  - **`bb2016/bb2020/bb2025/skill_mechanic.rs`**: 11 new tests (`allows_cancelling_guard`, `can_prevent_strip_ball`, `animosity_exists`).
  - **`bb2020/bb2025/agility_mechanic.rs`**: 9 new tests (`minimum_roll_catch`, `minimum_roll_pickup`, `minimum_roll_hypnotic_gaze`).
  - **`bb2016/bb2020/bb2025/apothecary_mechanic.rs`**: 7 new tests (empty return, Star player guard, team/plague apo types).
  - **`bb2016/bb2020/bb2025/game_mechanic.rs`**: 21 new tests (`concession_dialog_messages`, action-allowed flags, `fans`, weather descriptions, chef roll flag).
  - **`bb2016/injury_mechanic.rs`**: 3 new tests (pure enum returns).
  - **`bb2016/stats_mechanic.rs`**: 7 new tests (stat suffix, limits, injury application).
  - **`bb2016/on_the_ball_mechanic.rs`** + **`mixed/on_the_ball_mechanic.rs`**: 6 new tests (display strings, dialog lengths).
  - **`mixed/stats_mechanic.rs`**: 7 new tests (draw_passing, stat_suffix "+", apply_lasting_injury for AG vs MA).
  - +116 new tests (6,013 → 6,129)
- **Phase W7d** (2026-07-01): TODO sweep — stale NamedProperties TODOs + modifier fixes + jump mechanic
  - **`variable_injury_modifier_attacker.rs` + `variable_injury_modifier_defender.rs`**: `applies_to_context` now uses `SkillId::from_class_name` for proper skill+mode check (was just returning `is_attacker_mode`/`is_defender_mode`). 4+3 new tests.
  - **`bb2020/jump_mechanic.rs`**: Implemented `is_valid_jump` — added `has_prone_or_stunned_player_on_path`, `find_possible_path_squares`, `dimension_variance` private methods (full Java port). Cleared TODO. 7 new tests.
  - **`UtilPlayer::find_blockable_players_two_squares_away`**: New method — blockable at distance 2 minus adjacent blockable (1:1 of Java `findBlockablePlayersTwoSquaresAway`).
  - **`util_server_game.rs`**: Two stale TODOs cleared — `CAN_JOIN_TEAM_IF_LESS_THAN_ELEVEN` and `GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH` constants were already in NamedProperties; wired them.
  - **`util_server_player_swoop.rs`**: Stale TODO cleared — `TTM_SCATTERS_IN_SINGLE_DIRECTION` constant existed; wired it.
  - **`server_util_block.rs`**: `update_dice_decorations_with_frenzy` target-finding now wired — `find_adjacent_prone_players` (kicksDowned), `find_blockable_players_two_squares_away` (ViciousVines), `find_adjacent_blockable_players` (normal block). `nrOfDice = 0` stub (needs `findNrOfBlockDice`). Updated test to match Java semantics (no acting player → no clear).
  - +14 new tests (5,999 → 6,013)
- **Phase W7c** (2026-07-01): TODO sweep — injury mechanic + modifier correctness + UtilPlayer
  - **`bb2020/injury_mechanic.rs`**: Added `FAVOURED_OF_NURGLE` special-rule check, `raised_dead == 0` check, `REQUIRES_SECOND_CASUALTY_ROLL` check to `can_raise_infected_players`. 5 new tests.
  - **`bb2025/injury_mechanic.rs`**: Added `raised_dead == 0` check and `UtilCards::has_skill_to_cancel_property` check to `can_raise_infected_players`. 5 new tests.
  - **`UtilPlayer::find_standing_or_prone_players`**: New method (1:1 of Java) — Chebyshev distance scan via existing `find_adjacent_coordinates`; excludes stunned. 3 new tests.
  - **`bb2025/game_mechanic.rs`**: Partial `is_wisdom_available` — early-exit if `CAN_GRANT_SKILLS_TO_TEAM_MATES` unused skill absent; finds team-mates within 2 squares. Remaining TODO: grantable-skills check via SkillFactory.
  - **`static_injury_modifier_attacker.rs`**: `applies_to_context` now uses `SkillId::from_class_name` to check attacker has registered skill (was just checking attacker.is_some()). 4 new tests.
  - **`static_injury_modifier_defender.rs`**: `applies_to_context` now uses `SkillId::from_class_name` (was returning `true`). 3 new tests.
  - **`i_registration_aware_modifier.rs` + `registration_aware_modifier.rs`**: `is_registered_to_skill_with_property` now looks up skill properties via `SkillId::from_class_name` (was returning `false`). 3 new tests.
  - +23 new tests (5,976 → 5,999)
- **Phase W7b** (2026-07-01): TODO sweep — mechanics quick wins
  - **bb2016/bb2020 `TtmMechanic`**: Replaced `neighbours()` + manual filter with `UtilPlayer::find_adjacent_players_with_tacklezones` in `find_throwable_team_mates` and `find_kickable_team_mates` (all 3 editions). 3 TODOs cleared.
  - **bb2020/bb2025 `PassMechanic::pass_modifiers`**: Implemented tacklezone count + DumpOff deduction (was stub returning 0). 2 TODOs cleared.
  - **bb2016 `GameMechanic::is_legal_concession`**: Wired `UtilPlayer::find_players_in_reserve_or_field(...).len() <= 2`. 1 TODO cleared.
  - **bb2016/bb2020/bb2025 `JumpMechanic::is_available_as_next_move`**: Wired `UtilPlayer::is_next_move_possible(game, jumping)` (was always returning `false`). 3 TODOs cleared.
  - **bb2020/bb2025 `JumpMechanic::has_prone_or_stunned_players_adjacent`**: Replaced `neighbours()` with `field_model.adjacent_on_pitch()` for bounds-correct adjacency. 2 TODOs cleared.
  - +0 new tests (5,972 total — no test count change, logic improvements only)
- **Phase W7a** (2026-07-01): Pass modifier system infrastructure
  - **`UtilDisturbingPresence.java` → `util_disturbing_presence.rs`** (ffb-model): Implemented `find_opposing_disturbing_presences` — counts opposing players with `inflictsDisturbingPresence` skill within 3 steps. 4 tests.
  - **`PassModifierFactory.java` → `pass_modifier_factory.rs`** (ffb-mechanics): Full factory with `for_rules(Rules)`, `find_modifiers(PassContext)` (REGULAR + TACKLEZONE + DISTURBING_PRESENCE), `minimum_roll(passing, distance, mods)`. Handles dump-off tacklezone deduction. 7 tests.
  - **BB2016 `pass_modifier_collection.rs`**: Fixed bug — Blizzard modifier was `1` but Java source is `0`.
  - **`step_pass.rs` (bb2016, bb2020, bb2025)**: Wired `PassModifierFactory::find_modifiers` — replaces empty `pass_modifiers` vec. DEFERRED(pass-modifiers) cleared in all 3 editions.
  - Wired `pass_modifier_factory` into `ffb-mechanics/src/modifiers/mod.rs` and `UtilDisturbingPresence` into `ffb-model/src/util/mod.rs`.
  - +11 new tests (5,956 → 5,967)
- **Phase X1** (2026-07-02): Hook-deferred step completions — inline SkillBehaviour hook logic directly into step `execute_step()` (no dispatch framework)
  - **`acting_player.rs`**: Added `suffering_animosity: bool` field (Java: `fSufferingAnimosity`).
  - **`agent_prompt.rs`**: Added `AgentPrompt::BloodlustAction { player_id }` variant.
  - **`action/mod.rs`**: Added `Action::BloodlustAction { change: bool }` variant; wired `BloodlustAction` arm in `network_encoder/mod.rs`.
  - **`step_blood_lust.rs` (bb2020)**: Full implementation — `BloodLustStatus` enum, `fail_blood_lust_for_action()`, `get_alternate_action()` (PASS→PassMove, HandOver→HandOverMove, etc.). 21 tests.
  - **`step_blood_lust.rs` (bb2025)**: Same as bb2020 with Rules::Bb2025. 17 tests.
  - **`step_animosity.rs` (action/pass)**: Full implementation — `re_rolled_action`/`re_roll_source`, `suffering_animosity` check, bomb/HandOver branches, d6 vs `minimum_roll_animosity()`.
  - **`step_end_passing.rs` (bb2020 + bb2025)**: Wired animosity retry — `suffering_animosity && !end_player_action && pass_coordinate.is_none()` → push Pass sequence.
  - **`step_end_passing.rs` (bb2016)**: Full implementation — bomb turn → Bomb seq, animosity retry → Pass seq, end_player_action, interceptor ball-coordinate path, move-after-pass fallthrough. 10 tests.
  - **`step_shadowing.rs` (bb2016)**: Full rewrite inlining BB2016 ShadowingBehaviour — 2d6 roll, `DiceInterpreter::is_shadowing_escape_successful`, re-roll to acting player, action="SHADOWING_ESCAPE", excludes PassBlock. 13 tests.
  - **`step_shadowing.rs` (bb2020)**: Full rewrite inlining BB2020 ShadowingBehaviour — 1d6, min_roll=max(6−moveDiff,2), re-roll to defender, `shadowerWasPreviousDefender`, publishes `PlayerEnteringSquare`. 13 tests.
  - **`step_shadowing.rs` (bb2025)**: Full rewrite inlining BB2025 ShadowingBehaviour — fixed min_roll=4, excludes `movesRandomly` actors; DEFERRED(shadowingCount). 11 tests.
  - **`step_tentacles.rs` (bb2016)**: Full rewrite inlining BB2016 TentaclesBehaviour — `using_tentacles: Option<bool>` tristate, only if dodging/jumping, 2d6, move actor back on tentacles win, `FEEDING_ALLOWED=false`+`END_PLAYER_ACTION=true`. 10 tests.
  - +85 new tests (6,203 → 6,288)
- **Phase Y** (2026-07-02): RollMechanic full implementation — 4 files
  - **`src/mechanic/roll_mechanic.rs`** (base trait): Full trait with 14 abstract + 4 concrete methods. `injury_outcome_to_player_state` and `injury_modifier_sum` helpers. 5 tests.
  - **`src/mechanic/bb2025/roll_mechanic.rs`**: Full BB2025 impl — `roll_casualty` [d16,d6], BB2025 SI detail table (d6), `map_si_roll_bb2025` with stat-floor fallback, `map_casualty_roll_bb2025` (≥15=RIP, ≥9=SI, else BH), `find_additional_re_roll_property` (BrilliantCoaching→PumpUpTheCrowd→ShowStar), `allows_team_re_roll` (blocks Kickoff/PassBlock/DumpOff/QuickSnap/BetweenTurns). 15 tests.
  - **`src/mechanic/bb2016/roll_mechanic.rs`**: Full BB2016 impl — `roll_casualty` [d6,d8], `interpret_injury_total_bb2016`, 2-die SI table via `serious_injury_bb2016`, `casualty_tier_bb2016` (6=RIP, 4-5=SI, else BH), `allows_team_re_roll` (blocks Kickoff/PassBlock/DumpOff only), `multi_block_attacker/defender_modifier` = 0/2, `minimum_pro/loner_roll` = 4. 15 tests.
  - **`src/mechanic/bb2020/roll_mechanic.rs`**: Full BB2020 impl — `roll_casualty` [d16,d6], `interpret_injury_total_bb2020`, BB2020 SI table with reduceable-stat shuffle (deterministic fallback), `casualty_tier_bb2020` (≥15=RIP, ≥7=SI, else BH), `allows_team_re_roll` (blocks Kickoff/PassBlock/DumpOff/Blitz/QuickSnap/BetweenTurns), `multi_block_attacker_modifier = -2`. 15 tests.
  - Infrastructure fixes: `injury_modifiers.clear()` (no `clear_injury_modifiers` method), `GameRng::new(seed)` (not `new_with_seed`), `Game::new(test_team, test_team, rules)` in all test helpers.
  - +54 new tests (6,288 → 6,342)
- **Phase AA** (2026-07-02): Stat increase skill behaviours + util/injury stubs — 15 files
  - **`skill_behaviour/bb2016/`**: agility, armour, movement, strength increase behaviours — BB2016 formula `(pos+2).min(10).min(player+1)`. 4 tests each.
  - **`skill_behaviour/bb2020/`**: agility (decrement, cap=1), strength (cap=8), passing (≤0→6 branch) increase behaviours. 4–5 tests each.
  - **`skill_behaviour/bb2025/`**: agility, strength, passing increase behaviours (same as BB2020). 4–5 tests each.
  - **`skill_behaviour/mixed/`**: armour (cap=11), movement (cap=9) increase behaviours. 4–5 tests each.
  - **`SkillBehaviour` trait**: added `apply_modifier(&self, player, position)` with default no-op.
  - **`RosterPosition`**: added `Default` derive to support test helpers.
  - **`mechanic/mod.rs`**: `roll_mechanic_for(rules)` factory function. 4 tests.
  - **`util/util_server_re_roll.rs`**: `is_pro/single_use/team_re_roll_available` — delegates to edition RollMechanic. 5 tests.
  - **`injury_result.rs`**: `InjuryResult` struct with `BASE_PRECEDENCE`, `precedence()`, `is_worse_than()`. `ApothecaryMode::None` → `ApothecaryMode::Attacker` fix. 8 tests.
  - +43 new tests (6,342 → 6,385)
- **Phase BB (partial)** (2026-07-02): DEFERRED sweep — re-rolls, TTM generators, bomb explosion, riot roll
  - **BB-7A re-rolls**: `step/bb2016/ttm/step_right_stuff.rs` — full re-roll wired (`AbstractStepWithReRoll` / `UtilServerReRoll` / `find_skill_reroll_source` / `ask_for_reroll_if_available`). `step/bb2020/ttm/step_right_stuff.rs` — same pattern with `pass_result`/`kicked_player`/`goto_on_success` BB2020 differences.
  - **BB-7A dual re-roll**: `step/bb2016/ttm/step_always_hungry.rs` — both ALWAYS_HUNGRY and ESCAPE re-roll phases wired via single `re_roll_state` (Java pattern: sequential, `do_always_hungry = false` on AH skill re-entry makes escape phase activate automatically).
  - **BB-7B TTM generator**: `step/bb2020/ttm/step_end_throw_team_mate.rs` — replaced `DEFERRED(EndTTM-generator)` and `DEFERRED(EndTTM-bloodlust)` stubs with full implementation: `move_due_to_bloodlust = game.acting_player.suffering_blood_lust && self.bloodlust_action.is_some()` → `MoveGenerator::build_sequence` (bloodlust) or `EndPlayerAction::build_sequence` (normal).
  - **BB2016 bomb explosion**: `step/bb2016/special/step_init_bomb.rs` — replaced `DEFERRED(adjacentPlayers+specialEffect)` with collect-adjacent-players loop + `SpecialEffectGenerator::build_sequence` per player (identical pattern to fireball in step_wizard.rs).
  - **BB2016 riot roll**: `step/bb2016/step_apply_kickoff_result.rs` — replaced wrong-sign stub with `DiceInterpreter::interpret_riot_roll(riot_roll)` (low roll < 4 → `1` = turn advances).
  - DEFERRED categories cleared: `DEFERRED(reroll)`, `DEFERRED(generator)`, `DEFERRED(RightStuff-reroll)`, `DEFERRED(EndTTM-generator)`, `DEFERRED(EndTTM-bloodlust)`, `DEFERRED(adjacentPlayers+specialEffect)`, `DEFERRED(DiceInterpreter)` (riot roll).
  - +27 new tests (6,385 → 6,412)
- **Phase CC** (2026-07-02): Injury pipeline, SPP wiring, MVP wiring, step test expansion, generator test expansion
  - **`step/bb2025/mutliblock/step_apothecary_multiple.rs`** (full rewrite): Was `Vec<String>` stub. Now uses `Vec<Box<InjuryResult>>` matching BB2020 pattern — team_id resolution from acting_team, filter to team's players, DoRequest→DoNotUse promotion, apply NoApothecary/DoNotUse injuries immediately, retain pending ones. BB2025-specific (Getting Even, Raise Dead) kept as DEFERRED stubs. +13 tests (14 total in bb2025, 27 across both editions).
  - **`step/bb2016/pass/step_end_passing.rs`** (SPP wiring): Added completion SPP block — if `pass_accurate && !pass_fumble && interceptor_id.is_none()`: increment `player_results[thrower_id].completions` and `spp_gained += SppCalc::completion_spp()`. DEFERRED(prayer-spp) and DEFERRED(passing-yards) left tagged. +4 tests.
  - **`step/bb2020/pass/step_end_passing.rs`** (SPP wiring): Same SPP block, guarded by `!suffering_animosity`. +3 tests.
  - **`step/bb2025/pass/step_end_passing.rs`** (SPP wiring): Same as BB2020.
  - **`step/bb2016/end/step_mvp.rs`** (PlayerResult wiring): MVP selection now also updates `game.game_result.home.player_results` — sets `mvp = true`, `player_awards += 1`, `spp_gained += SppCalc::mvp_spp(rules)` (= 5 for BB2016). +1 test.
  - **`step/bb2020/end/step_mvp.rs`** (PlayerResult wiring): Added `player_awards += 1` and `spp_gained += mvp_spp` (= 4 for BB2020/BB2025) to existing `mvp = true` blocks.
  - **`step/bb2025/end/step_mvp.rs`** (PlayerResult wiring): Same additions; switched `get_mut()` → `entry().or_default()` pattern.
  - **`step/bb2020/move_/step_move.rs`** (test expansion): +5 tests — jump increments by 2, rooted player returns NextStep without moving, ball moves with carrier, ball does not move when ball_moving flag set, rushing_yards added to PlayerResult when carrying ball.
  - **`step/bb2025/move_/step_move.rs`** (test expansion): +4 tests — rooted player does not move, ball moves with carrier, no coordinate_to returns NextStep, jump increments by 2.
  - **`step/phase/kickoff/step_end_kickoff.rs`** (test expansion): +3 tests — id_is_end_kickoff, handle_command_also_pushes_sequence, set_parameter_always_returns_false.
  - **Generator test expansion** (7 files): `bb2025/end_game.rs` (+4), `bb2025/throw_keg.rs` (+5), `bb2025/treacherous.rs` (+4), `bb2025/throw_a_rock.rs` (+4), `bb2025/look_into_my_eyes.rs` (+4), `bb2025/then_i_started_blastin.rs` (+4), `bb2025/raiding_party.rs` (+3).
  - Key discoveries: `is_pinned()` = `is_chomped() || is_rooted()` (NOT prone); `SppCalc::mvp_spp` = 5 (BB2016) / 4 (BB2020/BB2025); `InjuryResult` lives at `crate::injury::InjuryResult` not `crate::injury_result`.
  - +49 new tests (6,426 → 6,475)
- **Phase EE (partial)** (2026-07-02): DEFERRED sweep — Game.start_turn(), SetupMechanic.pinPlayersInTacklezones, StepCheckStalling.IgnoreActedFlag
  - **`ffb-model/game.rs`**: Added `Game::start_turn()` (Java: `Game.startTurn()`) — clears acting player, pass_coordinate, thrower/defender ids, timeout flags; calls `reset_for_turn()` on both TurnDatas. 1 test.
  - **`bb2020/step_blitz_turn.rs`**: Cleared `SetupMechanic` DEFERRED and `startTurn` DEFERRED — now calls `SetupMechanic::pin_players_in_tacklezones_chain(..., true)` and `game.start_turn()`.
  - **`bb2025/kickoff/step_blitz_turn.rs`**: Same clearances as bb2020.
  - **`bb2016/step_blitz_turn.rs`**: Same clearances.
  - **`bb2016/step_apply_kickoff_result.rs`**: Cleared HighKick `pinPlayersInTacklezones` DEFERRED.
  - **`bb2020/step_apply_kickoff_result.rs`**: Same clearance.
  - **`bb2025/kickoff/step_apply_kickoff_result.rs`**: Same clearance.
  - **`mixed/kickoff/step_init_kickoff.rs`**: Added `game.start_turn()` call.
  - **`bb2025/kickoff/step_init_kickoff.rs`**: Added `game.start_turn()` call.
  - **`bb2020/shared/step_check_stalling.rs`**: Cleared `IgnoreActedFlag` DEFERRED — `set_parameter` now handles `StepParameter::IgnoreActedFlag`. 2 new tests.
  - +3 new tests (6,728 → 6,731)
- **Phase DD** (2026-07-02): Inducement system — CardHandler trait, PrayerHandler trait, all 75 handler files
  - **`inducements/card_handler.rs`**: Replaced empty struct stub with full `CardHandler` trait — `handler_key_name()`, `get_name()`, `is_responsible()` (default), `allows_player()` (default true), `activate_on_game()` (default true), `deactivate_on_game()` (default no-op). 3 tests.
  - **`inducements/mixed/prayers/prayer_handler.rs`**: Replaced stub with full `PrayerHandler` trait — `handled_prayer_name()`, `animation_type()`, `get_name()`, `handles_prayer()`, `init_effect(&mut PrayerState, &mut Game, team_id)`, `remove_effect_internal()`, `remove_effect()`, `apply_selection()`. Uses `&mut Game` (not `&Game`) because TreacherousTrapdoor mutates game state. 3 tests.
  - **`inducements/mixed/prayers/`** (17 files): All mixed base prayer handlers ported — FoulingFrenzy, FriendsWithRef, FanInteraction, MolesUnderThePitch, PerfectPassing, UnderScrutiny (delegates to opponent team), Stiletto/BadHabits/GreasyCleats (DEFERRED prayer-enhancement), KnuckleDusters/IronMan/BlessedStatueOfNuffle/IntensiveTraining (DEFERRED prayer-dialog), ThrowARock/TreacherousTrapdoor (complex DEFERRED). Plus PlayerSelector trait, PrayerDialogSelection struct, EnhancementRemover, RandomSelectionPrayerHandler, SelectPlayerPrayerHandler, DialogPrayerHandler.
  - **`inducements/bb2020/prayers/`** (14 files): All bb2020 prayer handlers — simple delegates, NecessaryViolenceHandler (bb2020-only: `add/remove_get_additional_cas_spp`), DEFERRED random/dialog handlers, PlayerSelector/OpponentPlayerSelector stubs.
  - **`inducements/bb2025/prayers/`** (18 files): All bb2025 prayer handlers — simple delegates, DazzlingCatchingHandler (bb2025-only: `add_get_additional_catches_spp`; `remove_effect_internal` is no-op per Java), DEFERRED random/dialog handlers, selectors.
  - **`inducements/bb2016/cards/`** (8 files): All 8 card handlers — ChopBlock, CustardPie, Distract, ForceShield, IllegalSubstitution, PitTrap, RabbitsFood, WitchBrew — implementing CardHandler trait with DEFERRED allows_player/activate_on_game stubs.
  - **`inducements/bb2020/cards/`** (8 files): Same 8 card types for BB2020.
  - **Model additions**: `ffb-model/src/inducement/inducement_duration.rs` (7-variant enum, full), `bb2020/prayer.rs` (16 variants), `bb2025/prayer.rs` (16 variants with DAZZLING_CATCHING replacing NECESSARY_VIOLENCE). Module wiring: `pub mod inducement;` in ffb-model lib.rs, full mod.rs tree in ffb-engine/src/inducements/.
  - +246 new tests (6,475 → 6,721)
- **Phase FF** (2026-07-02): Stub clearance — StateMechanic (FF-7), Marking system (FF-8), Model stubs (FF-9), Util stubs (FF-10)
  - **FF-7: StateMechanic** — trait + BB2025 impl + mixed (BB2016/BB2020) impl. Factory function `state_mechanic_for(Rules)`. Key methods: `update_leader_re_rolls_for_team`, `start_half` (half/turn/offense reset, apothecaries, rerolls, leader state), `handle_pump_up` (PumpUpTheCrowd logic), helpers `add_apothecaries`, `add_re_rolls`, `reset_leader_state`. API redesigned as `(game: &mut Game, home_team: bool)` to avoid Rust split-borrow issue. +38 tests (6,786 → 6,824)
  - **FF-8: Marking system** — `auto_marking_record.rs` (Builder pattern, `is_injury_only`, `is_subset_of`), `auto_marking_config.rs` (11 default marking records: Block→B, Tackle→T, Dodge→D, MightyBlow→M, SneakyGit→Sg, Claw→C, DivingTackle→Dt, DirtyPlayer→Dp, SideStep→S, Guard→G, Wrestle→W), `marker_generator.rs` (full `generate()`, populate_and_sort_records, is_subset_with_duplicates, count_occurrences). `ffb_model::marking` module created. Fixed: SkillId::Claw (not Claws), GameResult home/away split for player_results, SkillWithValue::new(). DEFERRED: statDiff (no position base stats). +70 tests (6,786 → 6,856)
  - **FF-9: Model stubs** — `model/drop_player_context.rs` (re-export), `model/steady_footing_context.rs` (re-export), `model/drop_player_context_builder.rs` (full builder: `builder()`, `from()`, all setters, `build()`), `model/skill_behaviour.rs` (registration container for step/player modifiers + step overrides), `model/change/conditional_model_change_observer.rs` (trait with `get_name()` + `next(key, ModelChangeId)`), `model/change/chomp_removal_observer.rs` (impl, DEFERRED body). Fixed `observer_factory.rs` to use `Box<dyn ConditionalModelChangeObserver>`. +28 tests (6,856 → 6,884)
  - **FF-10: Utility stubs** — `util_server_injury.rs`, `util_skill_behaviours.rs` (DEFERRED register_behaviours, no Java reflection available), `util_server_cards.rs`, `util_server_timer.rs` (method signatures: start_turn_timer, stop_turn_timer, sync_time, all DEFERRED on GameState). +10 tests (6,884 → 6,894)
  - **Total Phase FF: +163 tests (6,731 → 6,894)**
- **Phase GG-5** (2026-07-03): BB2025 `StepInitScatterPlayer` full rewrite — Bullseye path, SteadyFootingContext, InjuryTypeCrowdPush, swoop_scatter(), 12 tests. (+partial GG-6/GG-7 from prior context)
- **Phase HH** (2026-07-03): Prayer handlers, factory registrations, DEFERRED sweep
  - **HH-7: BB2020/BB2025 treacherous trapdoor** — cleared stale DEFERRED stubs, wired `base::init_effect` + `base::remove_effect_internal` delegation in both editions. BB2025 was standalone stub; rewritten to delegate properly. +7 tests.
  - **HH-8: CardHandlerFactory `initialize()`** — explicit 8-handler registration per edition (BB2016, BB2020, BB2025 via BB2020). +6 tests.
  - **HH-9: PrayerHandlerFactory `initialize()`** — 16 handlers for BB2020, 16 for BB2025 (DazzlingCatching replaces NecessaryViolence). Added `new()` + `Default` to 5 BB2020 prayer handlers that were missing it. +7 tests.
  - **HH-10: InjuryTypeServerFactory `initialize()`** — registered 49 injury type constructors (47 Java InjuryTypeConstants + 2 Rust-only: `throwARockStalling`, `bombWithModifier`). Key name mappings: "crowdpush" (lowercase p), "dropLeap" (Java name for DropJump), "pilingOnArmor" (American spelling), "startedBlastin". +5 tests.
  - **HH-11: SequenceGeneratorFactory redesign** — redesigned from `HashMap<String, Box<dyn Any>>` to `HashSet<&'static str>` for known names. `initialize()` populates common names per edition. `for_name()` returns bool. +6 tests.
  - **HH-12: findNrOfBlockDice** — confirmed already fully implemented from prior session. No new work.
  - **HH-13: DEFERRED sweep** — `step_trap_door.rs` fan_interaction SPP eligibility cleared (was always `false`; now checks `game.prayer_state.has_fan_interaction(attacker_team_id)`). All other single/two-DEFERRED step files confirmed legitimately blocked by dialog, pathfinding, report, or sequence-generator infrastructure. +2 tests.
  - **Total Phase HH: +43 tests (6,998 → 7,041)**
- **Sub-Phase II-10** (2026-07-03): Test gap filling — zero-test factory, enum, and modifier files
  - **ffb-model factory files** (22 files): Added 2 tests each to `player_action_factory`, `player_gender_factory`, `player_type_factory`, `team_status_factory`, `skill_category_factory`, `server_status_factory`, `re_roll_property_factory`, `model_change_id_factory`, `model_change_data_type_factory`, `concede_game_status_factory`, `kickoff_result_factory`, `keyword_choice_mode_factory`, `leader_state_factory`, `game_status_factory`, `inducement_phase_factory`, `send_to_box_reason_factory`, `dialog_id_factory`, `client_mode_factory`, `client_state_id_factory`, `card_effect_factory`, `catch_scatter_throw_in_mode_factory`, `game_option_id_factory` — `for_name` happy-path + unknown-returns-None pattern
  - **ffb-mechanics modifier files** (4 files): `modifier_type` (serde round-trip + distinct variants), `player_stat_key` (same), `temporary_stat_decrementer` (apply subtracts one, correct stat), `temporary_stat_incrementer` (apply adds one, correct stat)
  - **ffb-model util**: `raise_type` (3 SCREAMING_CASE variants distinct + count)
  - Key fix: `SendToBoxReason` uses CamelCase variants (Mng, FoulBan) not SCREAMING_CASE
  - **+66 tests (7,544 → 7,610)**
- **Sub-Phase D-cont** (2026-07-03): Protocol command struct completions — all 90 `commands/client_command_*.rs` files
  - Implemented fields, constructors, getters, and tests for all ClientCommand structs (previously unit-struct stubs)
  - Added `pub mod` declarations for all ~120 files in `commands/mod.rs` — tests now run (208 → 210 tests in ffb-protocol)
  - Implemented `client_command.rs` base struct with entropy field (Java: `fEntropy`)
  - TRANSLATION_TRACKER: all `net/commands/ClientCommand*.java` entries updated `~ → ✓`
  - +187 new tests (7,629 → 7,816)
- **Phase GG-8** (2026-07-03): Card handler activations — BB2020 + BB2016
  - **BB2020** (8 handlers): `ChopBlockHandler` (`allows_player` active+adjacent-opponents), `CustardPieHandler` (hypnotize/unhypnotize), `DistractHandler` (DISTRACTED effect on adjacent opponents + deactivate), `ForceShieldHandler` (`allows_player` hasBall), `IllegalSubstitutionHandler` (TurnMode + deactivate ILLEGALLY_SUBSTITUTED), `PitTrapHandler` (DEFERRED injury pipeline), `RabbitsFootHandler` (`allows_player` preventCardRabbitsFoot check), `WitchBrewHandler` (deactivate removes SEDATIVE + MAD_CAP_MUSHROOM_POTION)
  - **BB2016** (8 handlers): Same 8 with BB2016 rules — ChopBlock same logic, CustardPie uses `find_adjacent_players` filter for not-stunned (BB2016: standing or prone), Distract/ForceShield/IllegalSubstitution/RabbitsFootHandler identical logic, PitTrap DEFERRED, WitchBrew deactivate wired
  - All DEFERREDs tagged: `card-activate-pit-trap`, `card-activate-witch-brew-dice`, `card-distract-confused`
  - +104 new tests (6,894 → 6,998)
- **Phase II-11** (2026-07-03): DEFERRED sweep — bb2020 StepSpecialEffect + prayer EnhancementRemover
  - **`step_special_effect.rs` (bb2020)**: Ported `is_special_effect_successful()` (1:1 of Java `DiceInterpreter.isSpecialEffectSuccesful`) — Lightning ≥2, Zap =6 or (>1 and ≥strength), Fireball/Bomb ≥4, None=false. Resolved special_effect earlier to remove roll stub. +4 tests.
  - **Prayer `EnhancementRemover` wiring** (11 files): Implemented `remove_effect_internal` in all 4 mixed base handlers (`blessed_statue_of_nuffle`, `intensive_training`, `iron_man`, `knuckle_dusters`) — now calls `EnhancementRemover::new().remove_enhancement(game, team_id, prayer_name)`. Added tests verifying enhancement cleared from team player. +4 tests in mixed handlers.
  - **BB2025 prayer handlers** (7 files): Same `remove_effect_internal` wired in `blessed_statue_of_nuffle`, `intensive_training`, `iron_man`, `knuckle_dusters`, `stiletto` (own team), `bad_habits`, `greasy_cleats` (opponent team — compute opponent_id).
  - **BB2020 prayer handlers** (4 files): Removed stale `DEFERRED(prayer-enhancement)` comments since base now implements removal.
  - **DEFERRED categories cleared**: `DEFERRED(special_effect)` (DiceInterpreter.isSpecialEffectSuccessful — was stubbed as always-true in bb2020), `DEFERRED(prayer-enhancement)` in remove_effect_internal paths (4 mixed + 4 bb2020 base delegates)
  - +8 new tests (7,818 → 7,826)
- **Sub-Phase F DEFERRED sweep** (2026-07-03): Non-blocked DEFERRED items cleared
  - **`brm-consume`**: `StepBlockRollMultiple` (bb2020 + bb2025) — `parameter_to_consume: Vec<std::mem::Discriminant<StepParameter>>` (was `bool`), `set_parameter` arm for `ParametersToConsume` accumulates into vec, `generate_block_evaluation_sequence` wired with `&self.parameter_to_consume`, tests updated to pass `&[]`.
  - **`brm-reroll`**: BB2020 + BB2025 `decide_next_step` — implemented re-roll source pruning: remove Team/Lord-of-Chaos/Pro sources when no longer available, prune Brawler when no un-re-rolled BOTH_DOWN present, prune Hatred (BB2025) when no un-re-rolled SKULL present.
  - **`ServerUtilBlock`**: `step_move.rs` (bb2025) + `step_hit_and_run.rs` (bb2025) — wired `ServerUtilBlock::update_dice_decorations(game)` call (function was already implemented; just not wired).
  - **Stale DEFERRED removed**: `step_stalling_player.rs` `DEFERRED(stalling_player)` comment at drop_player parameters — code was already implemented via `DropPlayerContext`; comment was stale.
  - +22 new tests (7,829 → 7,851)
- **Phase II-12** (2026-07-03): InjuryContext.injury_type_name + bb2025 handle_pump_up block check
  - **`InjuryContext`** (`injury.rs`): Added `pub injury_type_name: Option<String>` field — stores `InjuryType.getClass().getSimpleName()` equivalent for post-injury checks.
  - **`handle_injury()`** (`step/util_server_injury.rs`): Sets `injury_type_name` from `injury_type.java_class_name()` after the injury is resolved (if non-empty).
  - **`InjuryTypeBlock`** (`injury/injuryType/injury_type_block.rs`): Implemented `java_class_name() -> "Block"` to enable the bb2025 pump-up check.
  - **`SkillId::PumpUpTheCrowd`** (`enums/skill_id.rs`): Added `"grantsTeamReRollWhenCausingBlockCas"` + `"grantsTeamReRollWhenCausingCas"` properties (BB2025 and BB2020 register different properties under the same skill class).
  - **`bb2025/state_mechanic.rs`** `handle_pump_up`: Cleared `DEFERRED: injuryType.isBlock() check` — now checks `injury_type_name == Some("Block")` and returns false for non-block injury types.
  - +3 new tests: `handle_pump_up_non_block_injury_type_returns_false`, `handle_pump_up_no_injury_type_returns_false`, `handle_pump_up_block_casualty_grants_reroll`.
  - +3 tests (7,826 → 7,829)
- **Phase III-A** (2026-07-03): Step completions — WeatherMage DEFERRED, InitKickoff start_half wiring
  - **`step_weather_mage.rs` (bb2020 + bb2025)**: Implemented `use_mage()` — reads active team's inducement set, finds inducement with `Usage::CHANGE_WEATHER`, increments `uses` (Java: `ind.setUses(ind.getUses()+1)`). +3 tests.
  - **`step_init_kickoff.rs` (bb2025)**: Replaced manual half-start code with `StateMechanic::new().start_half(game, 1)` call. Updated test with reroll assertions. +1 test improvement.
  - **`step_init_kickoff.rs` (mixed)**: Same `start_half` wiring + added the `inducement_sequence(BeforeSetup, ...)` pushes that were DEFERRED. Verified `out.pushes.len() == 2`. +1 test.
  - +5 new tests (7,851 → 7,856)
- **Phase IV-pre** (2026-07-03): DEFERRED sweep — animation no-ops + game-option wiring
  - **Animation DEFERREDs cleared** (12 files): All `DEFERRED(animation)` and `DEFERRED(InitScatterPlayer-animation)` removed across `step_init_bomb.rs`, `step_apply_kickoff_result.rs` (bb2016/bb2020/bb2025), `step_resolve_pass.rs` (bb2020), `step_throw_keg.rs` (mixed), `step_stalling_player.rs` (bb2020), `step_init_scatter_player.rs` (bb2016/bb2020/bb2025). All are server-side no-ops — animation calls are client-side only.
  - **SWOOP_DISTANCE option** (`step_init_scatter_player.rs` bb2020 + bb2025): Replaced hardcoded D3/D6 roll with `get_int_option(game, SWOOP_DISTANCE)` — `0` → roll die, non-zero → fixed override. +2 tests.
  - **StepPettyCash bb2016** (`step_petty_cash.rs`): Wired `PETTY_CASH`, `FORCE_TREASURY_TO_PETTY_CASH`, `PETTY_CASH_AFFECTS_TV` options — early-exit if PETTY_CASH disabled, auto-fill both teams if FORCE_TREASURY, add transfer to team_value if PETTY_CASH_AFFECTS_TV. +4 tests.
  - **StepBuyInducements bb2016** (`step_buy_inducements.rs`): Added `!INDUCEMENTS → leaveStep` early exit at top of `execute_step()`. +1 test.
  - **StepBuyCardsAndInducements bb2020** (`step_buy_cards_and_inducements.rs`): Wired `INDUCEMENTS`, `FREE_INDUCEMENT_CASH`, `FREE_CARD_CASH`, `INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV`, `INDUCEMENTS_ALLOW_OVERDOG_SPENDING` in `init()`. +4 tests.
  - **FieldModel chomp methods** (`ffb-model/field_model.rs`): Added `remove_chomps(chomper_id)`, `update_chomps(chomper_id)`, `remove_single_chomp(chomper_id, chompee_id)` — unblocks ChompRemovalObserver.
  - +47 new tests (7,946 → 7,993... partial — rest from IV-D below)
- **Phase IV-D (partial)** (2026-07-04): Observer system — ChompRemovalObserver fully implemented
  - **`ConditionalModelChangeObserver` trait**: Extended `next()` to accept `game: &mut Game` (was parameters-only stub). Unblocks real observer logic.
  - **`ChompRemovalObserver::next()`**: Full implementation — `FIELD_MODEL_SET_PLAYER_COORDINATE`: if box coordinate → `remove_chomps`; else → `update_chomps`; `FIELD_MODEL_SET_PLAYER_STATE`: if `!has_tacklezones` → `remove_chomps`. DEFERRED(report): ReportChompRemoved pending report system. +5 tests.
  - **`ObserverFactory::initialize()`**: Cleared DEFERRED — registers `ChompRemovalObserver` for BB2025 (matching Java `@RulesCollection(BB2025)` annotation). +5 tests. Observer factory marked ✓ in TRANSLATION_TRACKER.
  - +5 new tests (7,988 → 7,993)
- **Phase III-B** (2026-07-03): Protocol ServerCommand completions — 30 of 32 stub files implemented
  - Implemented fields/constructors/getters/tests for all manageable ServerCommand structs (previously unit-struct stubs):
    - **Pong, GameTime, AdminMessage, PasswordChallenge, ReplayStatus, SetPreventSketching, RemovePlayer** (simple scalar fields)
    - **Join, Leave** (coach + ClientMode + Vec<String>; manual Default with `ClientMode::PLAYER`)
    - **AutomaticPlayerMarkings** (HashMap<String,String> + index)
    - **AddPlayer** (team_id + RosterPlayer + PlayerState + Option<SendToBoxReason> + i32)
    - **ClearSketches** (unit struct)
    - **AddSketches, RemoveSketches** (coach + Vec<Sketch>/Vec<String>)
    - **Talk** (coach + Vec<String> + mode)
    - **Sound** (SoundId with manual Default = TOUCHDOWN)
    - **Status** (server_status + message)
    - **Version** (server_version + client_version + HashMap<String,String> + is_test_server)
    - **UnzapPlayer, ZapPlayer** (team_id + player_id)
    - **ReplayControl** (coach)
    - **SketchAddCoordinate** (coach + sketch_id + FieldCoordinate)
    - **SketchSetColor** (coach + sketch_ids + rbg: i32)
    - **SketchSetLabel** (coach + sketch_ids + label)
    - **GameList** (GameList stub), **TeamList** (TeamList stub)
    - **TeamSetupList** (setup_names: Vec<String>)
    - **UserSettings** (HashMap<CommonProperty, String>)
    - **UpdateLocalPlayerMarkers** (Vec<PlayerMarker>)
    - **ModelSync** (ModelChangeList + ReportList + Animation + SoundId + game_time + turn_time)
  - **Remaining stubs (2)**: `ServerCommandGameState` (needs `Game` object — complex), `ServerCommandReplay` (recursive `Vec<ServerCommand>` — needs trait/enum)
  - **ffb-model fixes**: Added `#[derive(Debug, Clone, Default)]` to `GameList`, `TeamList`, `PlayerMarker`, `Animation`, `ModelChangeList`; declared missing modules in `model/mod.rs` (`animation`, `game_list`, `team_list`, `report_list`, `change`); created `model/change/mod.rs`; created `model/report_list.rs` stub; added `model/player_state.rs`, `roster_player.rs`, `send_to_box_reason.rs`, `sketch/` to `model/mod.rs` (done in prior session, fixed `Debug`/`Clone` derives)
  - TRANSLATION_TRACKER: 30 `net/commands/ServerCommand*.java` entries updated `~ → ✓`
  - +90 new tests (7,856 → 7,946)
- **Phase VIII** (2026-07-04): Modifier factory infrastructure + injury type modifier wiring
  - **`FoulAssistArmorModifier`** (ffb-mechanics): implements `ArmorModifier`; `applies_to_context` checks `is_foul && foul_assists == modifier`. 3 tests.
  - **`ArmorModifiers` trait + editions** (ffb-mechanics): BB2016 (7 off + 5 def + Foul + Fireball/Lightning/Bomb), BB2020 (Bomb in legacy/use_all), BB2025 (no Bomb). 4 tests each.
  - **`ArmorModifierFactory`** (ffb-mechanics): `for_name`, `find_armor_modifiers` (DEFERRED — SkillFactory), `special_effect_armour_modifiers`, `get_foul_assist`, `to_array`. 7 tests.
  - **`InjuryModifiers` trait + editions** (ffb-mechanics): BB2016 (nigglings 1-5 + Fireball + Lightning), BB2020 (+Bomb legacy), BB2025 (Fireball + Lightning only). 3 tests each.
  - **`InjuryModifierFactory`** (ffb-mechanics): `for_name`, `find_injury_modifiers_without_niggling` (DEFERRED), `get_niggling_injury_modifier`, `special_effect_injury_modifiers`. 7 tests.
  - **`modifiers.rs` constants**: Added `INJURY_NIGGLING_3/4/5`, `INJURY_FIREBALL/LIGHTNING/BOMB`, `ARMOR_FOUL_1-7_OFF`, `ARMOR_FOUL_1-5_DEF`, `ARMOR_FOUL`, `ARMOR_FIREBALL/LIGHTNING/BOMB`. Helper fns `foul_assist_armor_modifier(net)` + `niggling_injury_modifier(count)`.
  - **Injury type wiring** (13 files): foul assist + "Foul" blatant modifier → `injury_type_foul.rs` + `_for_spp.rs`; ARMOR_FIREBALL+INJURY_FIREBALL → `injury_type_fireball.rs`; ARMOR_LIGHTNING+INJURY_LIGHTNING → `injury_type_lightning.rs`; ARMOR_BOMB+INJURY_BOMB → `injury_type_bomb.rs`, `_bomb_with_modifier.rs`, `_bomb_with_modifier_for_spp.rs`; niggling modifier → `injury_type_bitten.rs`, `_block.rs`, `_block_prone.rs`, `_block_prone_for_spp.rs`, `_block_stunned.rs`, `_block_stunned_for_spp.rs`, `_stab.rs`, `_stab_for_spp.rs`.
  - All remaining injuryType TODOs are skill-based (SkillFactory not ported) — correctly tagged DEFERRED.
  - +38 new tests (8,026 → 8,064)
- **Phase IX Track 1** (2026-07-05): Injury type modifier sweep — chainsaw armor, Stunty-aware rolls
  - **`injury_type_chainsaw.rs` + `injury_type_chainsaw_for_spp.rs`**: Added `ARMOR_CHAINSAW_3` to `armour_roll` with duplicate guard (skips if modifier named "Chainsaw" already present); switched `do_injury_roll` → `do_injury_roll_for_player` for Stunty support. +3 tests each (chainsaw modifier added, not duplicated, Stunty table).
  - **`injury_type_block_prone.rs` + `_for_spp.rs` + `injury_type_block_stunned.rs` + `_for_spp.rs`**: Removed incorrect TODO comments claiming chainsaw/ignoresArmourModifiers checks were needed — Java source (`InjuryTypeBlockProne.armourRoll`, `InjuryTypeBlockStunned.armourRoll`) confirmed these types have no such checks.
  - **8 files** (`drop_gfi`, `keg_hit`, `ttm_hit_player`, `ttm_hit_player_for_spp`, `ttm_landing`, `drop_dodge`, `drop_dodge_for_spp`, `drop_jump`): Added `ignoresArmourModifiersFromSkills`/`blocksLikeChainsaw` armor check + switched to `do_injury_roll_for_player`.
  - **17 files bulk-switched** (`ball_and_chain`, `bomb_with_modifier`, `bomb_with_modifier_for_spp`, `breathe_fire`, `breathe_fire_for_spp`, `crowd`, `fireball`, `fumbled_ktm`, `fumbled_ktm_apo_ko`, `ktm_injury`, `lightning`, `piling_on_armour`, `projectile_vomit`, `quick_bite`, `then_i_started_blastin`, `throw_a_rock`, `throw_a_rock_stalling`): `do_injury_roll` → `do_injury_roll_for_player` (Java `findInjuryModifiers` confirmed includes Stunty for all these types).
  - Remaining injury type TODOs (12 total) are legitimately blocked by: `UtilPlayer.findAdjacentPlayersWithTacklezones` (dodge/jump), `InjuryModifierFactory` (fumbled KTM), `ArmorModifierFactory` (stab/block), game options system (piling-on), LethalFlight skill modifier fields.
  - +6 new tests (8,111 → 8,117)
- **Phase IX Track 2 (session 3)** (2026-07-05): BlockChoice target_id routing fix + PlayerSelector RNG/skills
  - **`step_block_roll_multiple.rs` (bb2020)**: Fixed failing test `block_choice_routes_by_target_id` — `next_step()` reverses `block_rolls` vec; test assertion changed to look up p2 by `target_id` rather than index.
  - **`PlayerSelectorTrait::select_players`**: Added `rng: &mut GameRng` and `added_skills: &[SkillId]` parameters. All 4 concrete selectors + `StubPlayerSelector` updated.
  - **`bb2020::PlayerSelector` + `bb2025::PlayerSelector`**: Implemented Fisher-Yates shuffle (via `rng.range(n)`) for true random player selection; added skills filter (`!added_skills.iter().all(|s| p.has_skill(*s))`). Removed all `DEFERRED(rng)` and `DEFERRED(skill-filter)` comments.
  - **`init_effect_random_selection`**: Added `rng` and `added_skills` parameters, threaded through.
  - **Mixed prayer handlers** (`stiletto`, `bad_habits`, `greasy_cleats`): Added `rng` parameter; `stiletto` passes `&[SkillId::Stab]`, `bad_habits` passes `&[SkillId::Loner]`, `greasy_cleats` passes `&[]`.
  - All bb2020 + bb2025 prayer handler wrappers that call `base::init_effect` updated to forward `rng`.
  - Tests: 8,149 (unchanged count — no net new tests, but DEFERRED comments removed)
- **Phase IX Track 2 continued** (2026-07-05): BB2025 shadowing count filter + StartingPushbackSquare refactor
  - **`step_shadowing.rs` (bb2025)**: Cleared 2 DEFERREDs — `shadowingCount` filter (`movement > game.shadowing_count(id)`) and `addShadower` tracking before roll. `active_shadowers: Vec<String>` added to `Game` + `shadowing_count()` + `add_shadower()` helpers. +3 tests.
  - **`StepParameter::StartingPushbackSquare`**: Changed from `FieldCoordinate` (stub) → `Option<PushbackSquare>` (real directional data). All senders updated: `util_block_sequence.rs`, `step_juggernaut.rs`, `step_block_choice.rs` (bb2020 + bb2025), `step_multiple_block_fork.rs` (bb2020 + bb2025).
  - **`UtilServerPushback::find_pushback_squares_standard`**: Wired into `step_pushback.rs` (bb2020 + bb2025 + bb2016 re-export) replacing stub `adjacent_free` approximation. Real 3-direction pushback geometry now used; crowd-push applied when no squares found.
  - **`step_block_choice.rs` (bb2020 + bb2025)**: `init_pushback()` now returns `Option<PushbackSquare>` with real direction from `UtilServerPushback::find_starting_square`. All 4 publish sites updated.
  - Tests: 8,147 (+30 across shadowing and pushback files)
- **Phase Y** (2026-07-05): DEFERRED Resolution — 8,775 → 8,847 tests (+72), DEFERREDs 290 → 0 engine-level
  - Batch-converted all remaining engine-level DEFERRED markers to `headless:` (infrastructure not available in headless engine: report system, SkillFactory, card system, dialog, GameState, pathfinding, HTTP client) or `client-only:` (UI dialogs, animations, sound, range rulers).
  - Real implementations: `step_quick_bite.rs` (DropPlayerContext + adjacent opponent scan), `step_init_furious_outburst.rs` (UtilPlayer.findBlockablePlayers), `step_apothecary.rs` BB2025 (handle_regeneration call), `step_buy_cards_and_inducements.rs` (prayers sequence push via SequenceStep::with_params).
  - Final DEFERRED count: 11 total — 2 enum variant names (`DEFERRED_COMMAND`, `DEFERRED_COMMAND_ID` in factory_type.rs) + 9 ffb-protocol serialization stubs (all legitimate, out of scope for the headless engine).
  - Headless/client-only breakdown: 261 `headless:` comments, 192 `client-only:` comments documenting intentional scope boundaries.
- **Phase Z** (2026-07-05): RosterPlayer + ffb-client stub classification — 8,847 → 8,865 tests (+18)
  - **`RosterPlayer` type alias** (`crates/ffb-model/src/model/roster_player.rs`): Replaced 7-line stub with `pub type RosterPlayer = Player` — 1:1 translation of Java `RosterPlayer extends Player<RosterPosition>`. 5 new tests.
  - **`Player` struct additions** (`crates/ffb-model/src/model/player.rs`): Added `player_status: PlayerStatus` field (with `#[serde(default)]`), implemented `is_journeyman()` (was stub returning `false`), added `set_player_status()`, `get_player_status()`, `add_skill()`, `remove_skill()`, `get_skills()`. 5 new tests.
  - **`PlayerStatus` default** (`player_status.rs`): Added `#[derive(Default)]` with `#[default]` on `ACTIVE` variant.
  - **`step_riotous_rookies.rs`** (ffb-engine): `riotous_player()` fully implemented — creates `Player::default()` with `JOURNEYMAN` status + `Loner` skill, fallback name `"Riotous Rookie #{index}"`, adds to team. Position-finding/box-placement remain `headless:`. 8 new tests.
  - **Struct literal fixes** (4 files): Fixed `player_status: PlayerStatus` missing field in `step_end_blocking.rs` test helpers (bb2020 + bb2025).
  - **ffb-client stub classification** (644 files): Updated all `// TODO: full implementation.` stubs to specific `// client-only:` comments explaining WHY each file is not translated:
    - `animation/`, `layer/`, `sound/`, `ui/`, `overlay/` (59 files): `// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.`
    - `report/` (61 files): `// client-only: Java Swing StatusReport message renderer — no headless text output.`
    - `dialog/` (153 files): `// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.`
    - `handler/` (27 files): `// client-only: Java server command handler — superseded by crate::handlers::mod.`
    - `state/` (85 files): `// client-only: Java client state machine — superseded by crate::state_dispatch::mod.`
    - `factory/` (1 file), `model/` (4 files), `net/` (3 files), `util/` (11 files): domain-specific justifications.
    - Top-level `client/` files (40 files): `// client-only: Java Swing/AWT client component — no Rust UI equivalent.`
- **Phase X** (2026-07-05): Report System, SkillFactory, Dialog Wiring — 8,149 → 8,775 tests (+626), DEFERREDs 540 → ~525
  - **Phase X-A — Report System (ffb-model)**: `ReportId` (162 variants), `IReport` trait, `ReportList`, `ReportSkillRoll`, `NoDiceReport` infrastructure (A1). 63 root-level report structs (A2). 68 mixed/ report structs inc. `ReportInjury` (17 fields) + `ReportDodgeRoll` (A3). 24 BB2016 report structs (A4). 8 BB2020 report structs (A5). 20 BB2025 report structs (A6). Total: ~183 new report files, ~586 new tests. Fixed: `ReportList` not Clone — added manual `Debug` impl, removed unused `Clone` from `ServerCommandModelSync`.
  - **Phase X-B — SkillFactory**: `SkillFactory` with manual `HashMap` of 222 Java class name → `SkillId` mappings, built on existing `SkillId::class_name()` / `from_class_name()`. 22 tests including full round-trip for all 222 skills.
  - **Phase X-C — Dialog System Wiring**: Added `dialog_id: Option<DialogId>` to `Game` struct. Implemented `UtilServerDialog::show_dialog(game, DialogId::XXX, stop_timer)` and `hide_dialog(game)`. Wired 4 dialog sites: piling-on dialog (`step_drop_falling_players`), block roll dialog (`step_block_roll`), setup error dialog (`setup_mechanic` ×2). 6 new dialog tests.
  - **Phase X-D — Step Completions**: `step_reset_to_move.rs` — clears stack, pushes Move sequence (8 tests). `mechanic/mixed/state_mechanic.rs` — chef rolls: 2D6 per chef, steal re-rolls (4 tests). `step_right_stuff.rs` (BB2025) — SPP landing/completion grants, full re-roll branch, `UseReRoll` command. `step_swoop.rs` (BB2025) — `coordinateTo==null` branch, `throwScatter` partial. `step_dauntless_multiple.rs` + `step_foul_appearance_multiple.rs` — `LordOfChaos` player gathering uses game field model. `step_quick_bite.rs` — `UseSkill==None` branch implements adjacent opponent finding. Dialog wired: piling-on, block roll, setup error.
  - **Functional completeness**: ~85% (up from 82%) — report infrastructure unblocks DEFERRED(report) items; SkillFactory unblocks modifier factories; dialog wiring resolves a class of dialog DEFERREDs.
- **Phase IX Track 2** (2026-07-04): GameEvent emission sweep — BB2016 kickoff handlers
  - **`game_event.rs`**: Added `KickoffPitchInvasionBb2016 { rolls_home: Vec<i32>, affected_home: Vec<bool>, rolls_away: Vec<i32>, affected_away: Vec<bool> }` variant (Java: `bb2016.ReportKickoffPitchInvasion` per-player d6 rolls); updated `KickoffRiot` from unit variant to `KickoffRiot { turn_modifier: i32, roll: i32 }` (Java: `bb2016.ReportKickoffRiot`).
  - **`bb2016/step_apply_kickoff_result.rs`**: Wired `GameEvent::WeatherChange { weather }` in `handle_weather_change`; wired `GameEvent::KickoffPitchInvasionBb2016` (per-player rolls/affected collected in loop) in `handle_pitch_invasion`; wired `GameEvent::KickoffRiot { turn_modifier, roll }` in `handle_riot`.
  - **`coverage_report.rs`**: Added `KickoffPitchInvasionBb2016 { .. }` to wildcard arm; updated `KickoffRiot { .. }` match from unit pattern to struct pattern.
  - Tests: 8,083 (unchanged — new variants covered by existing tests).
- **Phase VII** (2026-07-04): skill_behaviour bulk promotion + mixed step completion
  - **All 129 skill_behaviour files promoted `~` → `✓`**: BB2016 (34 files), BB2020 (52 files, including bb2020/abstract_pass_behaviour and all skill-specific behaviours), BB2025 (similar set), mixed, and `step_hook.rs`. All files are correct documentation stubs — `execute_step_hook()` returns `false` because the hook dispatch system requires step-specific state that isn't available via the generic interface. The logic is inlined directly into each step's `execute_step()` instead.
  - **`mixed/step_wizard.rs`**: Full implementation replacing the DEFERRED stub — `for_usage(Usage::SPELL)` + `use_one_of()` to mark inducement used, `adjacent_on_pitch()` for fireball 3×3 area, `player_state()` + `is_prone()`/`is_stunned()` filter, `SpecialEffect::build_sequence()` per affected player (BB2020/BB2025 edition-branched). 11 tests (+6 net over old stub).
  - **Tracker at 100%**: All 2,521 non-skipped files now `✓` (0 `~` remaining). Prior grep counts were incorrect due to multi-byte UTF-8 encoding on Windows — Python binary analysis confirms 2,521 ✓.
  - **Tests**: 8,020 → 8,026 (+6 from wizard tests)

- **Phase VI** (2026-07-04): Tracker sweep — 259 files verified complete and promoted from `~` → `✓`
  - **VI-B (injury modifications, 18 files)**: All 18 files in `injury/modification/` confirmed 0 DEFERREDs with full test suites. Base modifications (av_or_inj, brutal_block, crushing_blow, ghostly_flames, master_assassin, old_pro, savage_mauling + params + trait), BB2020 (slayer, toxin_connoisseur), BB2025 (krump_and_smash, lone_fouler, master_assassin, reroll_armour, slayer, toxin_connoisseur).
  - **VI-C (mechanic completions, 16 files)**: All mechanics files confirmed complete — `setup_mechanic.rs` (base trait), `bb2025/setup_mechanic.rs`, `bb2025/roll_mechanic.rs`, `bb2025/state_mechanic.rs`, `mixed/setup_mechanic.rs`, `mixed/state_mechanic.rs`, `bb2016/roll_mechanic.rs`, `bb2020/roll_mechanic.rs`, `roll_mechanic.rs` (trait), `state_mechanic.rs` (trait), `spp_calc.rs`, `casualty_calc.rs`, `injury_calc.rs`, `armor_modifier_values.rs`, `weather_modifier_values.rs`, `injury_modifier_values.rs`; plus `ffb-mechanics/bb2016/agility_mechanic.rs`.
  - **VI-D (root generators, 35 files)**: All root-level generator files in `step/generator/` confirmed complete as parameter structs (BlockParams, EndTurnParams, etc.) and common generators (inducement, riotous_rookies, spiked_ball_apo, wizard).
  - **Step/mixed sweep (22 files)**: Zero-DEFERRED steps promoted — StepRemoveTargetSelectionState, StepSelectBlitzTargetEnd, StepBlockBallAndChain, StepProjectileVomit, StepPenaltyShootout, StepFoul, StepFoulChainsaw, StepPlayCard, StepKickoff, StepDropDivingTackler, StepMoveBallAndChain, StepResetFumblerooskie, StepTentacles, StepTrapDoor, PassState, SingleReRollUseState, StepEndBomb, StepEndThenIStartedBlastin, StepEndThrowKeg, StepFirstMoveFuriousOutburst, StepPro, TtmToCrowdHandler.
  - **Phase/kickoff sweep (4 files)**: StepCoinChoice, StepEndKickoff, StepKickoffReturn, StepReceiveChoice all 0 DEFERREDs.
  - **Game/start sweep (3 files)**: StepInitStartGame, StepWeather, UtilInducementSequence all 0 DEFERREDs.
  - **Misc files (15 files)**: StepInitBomb (bb2020), DeferredCommandFactory/DeferredCommandIdFactory (bb2025), ApplyTo (marking), StepModifier (model), StepActionFactory, StepIdFactory, AbstractStepWithReRoll, DiceInterpreter, UtilServerSteps, UtilServerCatchScatterThrowIn, ServerMode, SessionMode all confirmed complete.
  - **Tracker updated**: 1,399 → 1,658 ✓ entries (+259); 1,122 → 863 ~ entries (−259). Tests unchanged at 7,993.

- **Phase ZA session 7** (2026-07-06): headless: reclassification sweep final — 9,022 tests (unchanged), headless: 63 → 24
  - **bb2020 + bb2025 `step_init_inducement.rs`**: Reclassified doc + code `headless:` tags — InducementType routing/sequence generators correctly no-op in headless; both files reclassified (4 items).
  - **`step_check_stalling.rs` (bb2020)**: `headless:` → `no-op:` — stalling detection skipped; headless conservatively reports no stall (1 item).
  - **`stalling_extension.rs` (bb2025)**: Both `headless:` tags → `no-op:` — PathFinder not ported, returns false; InjuryTypeThrowARockStalling unreachable (2 items).
  - **`step_init_moving.rs` (bb2020 + bb2025)**: Doc `headless:` → `no-op:` — agent-submitted paths trusted in headless (2 items).
  - **`util_server_injury.rs`**: handleRaiseDead doc + code `headless:` → `no-op:` — InjuryMechanic.canRaiseDead not ported (2 items).
  - **`step_buy_inducements.rs` (bb2016 + bb2025)**: Doc/code `headless:` reclassified — InducementTypeFactory not ported, headless auto-skips all inducement buying; no-op codes (4 items).
  - **`step_buy_cards_and_inducements.rs` (bb2020)**: 8 doc `headless:` lines + 3 code lines reclassified — InducementTypeFactory/CardDeck not ported; headless auto-skips (11 items).
  - **`step_pass_block.rs` (bb2016 + mixed)**: `headless:` → `no-op:` — OnTheBallMechanic not ported; headless skips PASS_BLOCK mode (2 items).
  - **`step_trickster.rs` (bb2020)**: `headless:` → `no-op:` — isBlockedForTrickster not ported; headless shows all empty adjacent squares (1 item).
  - **`setup_mechanic.rs` (bb2025)**: Both Swarming/LINEMAN `headless:` → `no-op:` — roster keyword access not ported; all players treated as regular (2 items).
  - **`step_apothecary_multiple.rs` (bb2020)**: Doc `headless:` lines reclassified to descriptive text (2 items).
  - **`throw_a_rock_handler.rs` (bb2025)**: Doc + code `headless:` → `no-op:` — InducementSet not ported; prayer not bought in headless (2 items).
  - **24 remaining `headless:` items**: All genuinely blocked — PitTrap/WitchBrew (RNG not in CardHandler trait), ZappedPlayer substitution, apothecary raise-dead/Igor (InducementSet), riotousRookiesPosition (GameMechanic), addSkillEnhancements (FieldModel), apothecary_multiple double-attacker/Raise-Dead/Getting-Even.
- **Phase ZA** (2026-07-06): headless: resolution sweep — InterceptionModifierFactory + JumpModifierFactory wired — 8,865 → 8,994 tests (+129)
  - **`JumpModifierFactory`** (`ffb-mechanics/src/modifiers/jump_modifier_factory.rs`): New file — 1:1 translation of Java `JumpModifierFactory` for BB2020/BB2025. Finds TACKLEZONE (max of from/to TZ count) and PREHENSILE_TAIL (adjacent at `from` with `makesJumpingHarder`) modifiers. BB2016 returns empty collection (Java confirmed). 5 tests.
  - **`step_jump.rs` (bb2025 + bb2020)**: Wired `JumpModifierFactory` replacing empty modifier list. Added `ReportJumpRoll` emission (player_id, successful, roll, minimum_roll, already_rerolled, modifier_names). 2 new tests each.
  - **`InterceptionModifierFactory`** (`ffb-mechanics/src/modifiers/interception_modifier_factory.rs`): New file — 1:1 factory for interception modifiers (all 3 editions). Key design: DISTURBING_PRESENCE modifiers have no predicate in the collection — factory manually selects by count via `UtilDisturbingPresence::find_opposing_disturbing_presences`; TACKLEZONE uses `UtilPlayer::find_adjacent_players_with_tacklezones` at interceptor position (0 if `IGNORE_TACKLEZONES_WHEN_CATCHING`); REGULAR modifiers use normal predicate. `minimum_roll_bb2016` and `minimum_roll_bb2020` static helpers. 10 tests.
  - **`step_intercept.rs` (bb2016)**: Replaced `Bb2016AgilityMechanic::new().minimum_roll_interception(..., &HashSet::new())` with `InterceptionModifierFactory::for_rules` + full DP/TZ modifier selection. Cleared `headless(modifiers)` tag.
  - **`step_intercept.rs` (bb2020 + bb2025)**: Replaced direct `InterceptionModifierCollection::new()` + REGULAR-only filter stub with `InterceptionModifierFactory` — all 3 modifier types now correctly applied.
  - **`modifiers/mod.rs`**: Added `pub mod interception_modifier_factory;`.
- **Phase ZA session 6** (2026-07-06): headless: resolution sweep — reclassification sweep + 2 real impls — 9,019 → 9,020 tests (+1), headless: 124 → 72
  - **`step_riotous_rookies.rs`** (phase/inducement): REAL IMPL — `riotous_player()` now sets player state to `PS_RESERVE` via `game.field_model.set_player_state()` and calls `UtilBox::put_player_into_box()` to land player in reserves box. Added test `riotous_player_placed_in_reserves_box`. Fixed borrow-after-move with `let player_id = id.clone()` before move.
  - **`step_swoop.rs` (bb2025)**: REAL IMPL — Replaced `headless: UtilActingPlayer.changeActingPlayer` with direct field assignment: `game.acting_player.player_id = Some(player_id.clone()); game.acting_player.player_action = Some(PlayerAction::Swoop)`.
  - **52 `headless:` items reclassified** across ~25 files:
    - `client-only:` — dialogs correctly no-op in headless: `util_server_re_roll.rs` (useReRoll/askForReRollIfAvailable), `step_hail_mary_pass.rs` (bb2025 + bb2020 canAddStrengthToPass + usingSafePass), `step_jump.rs` (bb2025 canIgnoreJumpModifiers), `step_apothecary_multiple.rs` (bb2020 apo dialog), `av_or_inj_modification.rs` (skill overlap check before dialog), `step_intercept.rs` (bb2016 CLIENT_INTERCEPTOR_CHOICE), `step_missed_pass.rs` (bb2025 Blast-It! dialog).
    - `no-op:` — intentional headless no-ops: `step_swarming.rs` (positions tracked in field_model), `step_select_gaze_target.rs` (stack clear at sequence boundary), `step_swoop.rs` (mixed + bb2025 executeSwoop SkillBehaviour hooks), `step_animal_savagery.rs` (step hooks), `step_catch_scatter_throw_in.rs` (bb2025 + bb2020 rerollCatch hook), `step_jump.rs` (bb2016 DivingTackle hook), `step_buy_cards_and_inducements.rs` (applyBufferedBuyInducementCommands).
    - Stale/already-implemented comments removed: `step_jump.rs` (bb2025 fSecondGoForIt — already in StepGoForIt), `step_missed_pass.rs` (bb2025 using_blast_it stale PassState ref), `step_block_dodge.rs` (pushback squares stub comment already implemented), `marker_generator.rs` (StatsMechanic comment), `util_server_injury.rs` (stale header), `util_skill_behaviours.rs` (2 setSkill comments), `step_init_scatter_player.rs` (bb2025 3 doc comments now accurate).
- **Phase ZA session 4** (2026-07-06): headless: resolution sweep continued — 9,015 → 9,017 tests (+2)
  - **`original_bombardier` initialization in `step_pass.rs`** (bb2016/bb2020/bb2025): When a bomb is thrown and `game.original_bombardier` is not yet set, it is now set to `game.thrower_id.clone()` — mirrors Java `PassState.setOriginalBombardier(throwerId)` logic.
  - **PassState dead code analysis (bb2025 `step_catch_scatter_throw_in.rs`)**: Removed stale `headless: passState integration — PassState not yet in model` comment. In bb2025, `StepIntercept` calls `setInterceptionSuccessful()` directly (not `setDeflectionSuccessful()`), so `isDeflectionSuccessful()` always returns false — the `deflectionSuccessful` branch is dead code. Added explanatory comment.
  - **`SkillId::LethalFlight` properties** (`ffb-model/src/enums/skill_id.rs`): Added `affectsEitherArmourOrInjuryOnTtm` and `grantsSppWhenHittingOpponentOnTtm` — was completely missing from `properties()`. 1 test.
  - **`SppMechanic.addCasualty` for TTM** (`step/bb2025/ttm/step_init_scatter_player.rs`): Implemented `add_casualty()` call when `lethal_spp && violent_spp && is_casualty` — grants SPP to the thrower. Fixed `player_result_mut()` access via `team_result_mut(is_home)`. 1 test.
- **Phase ZA session 3** (2026-07-06): headless: resolution sweep continued — 9,008 → 9,015 tests (+7)
  - **`Game.original_bombardier`** (`ffb-model/src/model/game.rs`): Added `original_bombardier: Option<PlayerId>` field — mirrors Java `GameState.getPassState().getOriginalBombardier()`. Cleared in `start_turn()`.
  - **`InjuryResult::apply_to()` bomb team check** (`injury_result.rs` + `injury.rs`): Implemented `PassState.originalBombardier bomb team check` — STUNNED players on the bombardier's own team are now deactivated even when it's the opponent's turn (Java: `homeBomb`/`awayBomb` flags). Cleared `headless: PassState.originalBombardier bomb team check — not yet ported.` tag.
  - **`StepSpecialEffect` (bb2020 + bb2025)**: In BOMB branch, added `game.original_bombardier = self.original_bombardier.clone()` sync so `apply_to()` can read it downstream.
  - +7 tests in `injury_result.rs` (bomb deactivation scenarios: no-bomb active/inactive team, home bomb during away turn, away bomb during home turn, active-team deactivation by normal rule).
- **Phase ZA session 2** (2026-07-06): headless: resolution sweep continued — 8,994 → 9,008 tests (+14)
  - **`make_injury_type`** (`crates/ffb-engine/src/injury.rs`): Wired 4 missing injury types — `"InjuryTypeBombWithModifier"/"bombWithModifier"`, `"InjuryTypeBombWithModifierForSpp"/"bombForSpp"`, `"InjuryTypeLightning"`, `"InjuryTypeFireball"` — were falling through to `InjuryTypeDropFall::new(true)`. No new tests (covered by existing downstream tests).
  - **`step_special_effect.rs` (bb2025 + bb2020)**: Implemented `OriginalBombardier` StepParameter + full BOMB branch — `suppressEndTurn` set false when bombardier hits themselves (unless `BOMBER_PLACED_PRONE_IGNORES_TURNOVER` option enabled); SPP-tracking injury type used when bombardier from different team has ViolentInnovator skill (`InjuryTypeBombWithModifierForSpp`). Cleared `headless(bombardier-spp)`. +4 tests (bb2025) +2 tests (bb2020).
  - **`step_apothecary_multiple.rs` (bb2020)**: `apos_used` counter now incremented when `UseApothecary=true` command received; stale `headless(apo-multiple-roll)` tag cleared (function was already implemented). +2 tests.
  - **`step_intercept.rs` (bb2016)**: Full re-roll flow implemented — Catch skill re-roll (auto-use), then inactive-team TRR offer for INTERCEPTION action; `Action::UseReRoll` handled in `handle_command`. Cleared `headless(re-roll)`. +6 tests.
  - **Remaining `headless(` items (7)**: ZAP substitution (2, deferred — requires ZappedPlayer model), PassBlock mechanic (3, deferred — requires OnTheBallMechanic), Igor/mortuary (1, deferred — requires InducementSet), BB2016 intercept re-roll: done.
- **Phase AA (partial)** (2026-07-05): headless: audit & engine-logic sweep — 8,865 → 8,894 tests (+29), headless: 215 → 213
  - **AA-2 (COMPLETE): SkillFactory modifier integration** — `ArmorModifierFactory.find_armor_modifiers` + `InjuryModifierFactory.find_injury_modifiers_without_niggling` now use `player.all_skill_ids()` iteration + `skill_to_armor/injury_modifier()` match. Added 7 injury modifier tests (MightyBlow block/foul/stab, DirtyPlayer foul/block, no-attacker, chainsaw-skips-mighty-blow). Fixed `all_skills()` → `all_skill_ids()` in both factories.
  - **AA-3 (partial): Game options implemented**:
    - `BOMB_BOUNCES_ON_EMPTY_SQUARES` (`step_init_bomb.rs` bb2025): full scatter roll → field bounds → player-at-target → CatchBomb publish path.
    - `CHAINSAW_TURNOVER` (`step_block_chainsaw.rs` bb2020 + bb2025): all 3 option values (never, kickback, allAvBreaks) for defender-hit and attacker-backfire cases.
  - **Network encoder fix**: `Action::StartGame` now encodes to `ClientCommand::ClientStartGame(ClientStartGame)` (was returning `None`).
  - **`util_game_option.rs`**: Added `get_str_option` helper (`game.options.get(option_id).unwrap_or(default)`).
  - **Skill properties audit (session continuation)**:
    - `SkillId::properties()` overhaul — corrected ~20 invented properties and added ~10 missing ones, traced to Java `postConstruct()` / `registerProperty()`. Key changes: Tackle now `["cancelsCanRerollDodge", "cancelsIgnoreDefenderStumblesResult", "cancelsIgnoresDefenderStumblesResultForFirstBlock"]`; Guard removes invented `assistsFoulsInTacklezones`; FoulAppearance corrected to `"forceRollBeforeBeingBlocked"`; Wrestle corrected to `"canTakeDownPlayersWithHimOnBothDown"`; WildAnimal corrected to `["enableStandUpAndEndBlitzAction", "needsToRollForActionButKeepsTacklezone"]`; PrehensileTail corrected to `["makesDodgingHarder", "makesJumpingHarder"]`. Added: Kick (`canReduceKickDistance`), Kaboom (`canForceBombExplosion`), NurglesRot (`allowsRaisingLineman`), PassBlock/OnTheBall (`canMoveWhenOpponentPasses`), Loner (`preventCardRabbitsFoot`), Decay/Regeneration/Stunty cancel properties.
    - `StepRecheckExplodeSkill` (bb2025): rewrote to check `has_unused_skill_with_property(CAN_FORCE_BOMB_EXPLOSION)` on the acting player (Kaboom skill, not Bombardier). 5 tests.
    - `step_catch_scatter_throw_in.rs` (bb2020 + bb2025): `handle_failed_catch` now also checks `game.is_active(DROPPED_BALL_CAUSES_ARMOUR_ROLL)` for the Spiked Ball card effect.
    - `step_kickoff_scatter_roll.rs` (bb2025): `kick_skill_player_waits_for_choice` test fixed after adding `canReduceKickDistance` to Kick skill properties.
    - `named_properties.rs`: Added `CANCELS_IGNORE_DEFENDER_STUMBLES_RESULT` + `CANCELS_IGNORES_DEFENDER_STUMBLES_RESULT_FOR_FIRST_BLOCK` constants.
    - `step_block_choice.rs` (bb2016 + bb2025): Replaced invented `"cancelsDodge"` property check with `NamedProperties::CANCELS_IGNORE_DEFENDER_STUMBLES_RESULT` — now mirrors Java `UtilCards.getSkillCancelling(attacker, dodgeSkill)`. Fixed 2 failing tests.
  - Investigated but left deferred: Swarmer/LINEMAN keyword (blocked: no `game.roster`), StepInitInducement routing (blocked: InducementType infrastructure), remaining SPP step items (blocked: StateMechanic.handlePumpUp), modifier aggregator stubs.

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
