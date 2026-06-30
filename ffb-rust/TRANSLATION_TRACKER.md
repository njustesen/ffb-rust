# FFB Java to Rust 1:1 Translation Tracker

<!-- Auto-generated skeleton from Java source. Update status cells manually as files are translated. -->
<!-- To regenerate: python scripts/gen_translation_tracker.py -->

## How to Use

This file tracks every Java class in ffb-common, ffb-server, and ffb-client-logic against its target Rust file.

1. When you start translating a file: change its status to `~`
2. When it matches the Java source 1:1 and parity is confirmed: change to `✓`
3. When a race passes T3b 100/100, all files exercised by that race should be `✓`

**Workflow per Java file:**
- Read the Java source in `ffb-java/<module>/src/main/java/<path>.java`
- Find or create the corresponding Rust file at the listed Rust Target path
- Translate method by method, matching dice consumption order, conditions, and state transitions exactly
- Run `cargo test` after each file
- Run `python scripts/parity_run.py --home amazon --away amazon --seeds 1-10` to catch regressions

## Status Legend

- `○` Not started -- no Rust equivalent exists
- `~` Partial -- Rust equivalent exists but not yet 1:1 with Java
- `✓` Done -- Rust matches Java line-by-line, parity confirmed
- `—` Not translating (GUI, DB, WebSocket layer, serialization utils)

---

## Progress Summary

| Metric | Count |
|--------|-------|
| Total Java files in scope | 2521 |
| Not started (○) | 0 |
| Partial (~) | 1201 |
| Done (✓) | 1320 |
| Not translating (—) | 458 |

---

## Module: ffb-common

### bb2016/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2016/SeriousInjury.java` | `ffb-model` | `src/bb2016/serious_injury.rs` | ✓ |

### bb2020/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2020/InjuryDescription.java` | `ffb-model` | `src/bb2020/injury_description.rs` | ✓ |
| `bb2020/SeriousInjury.java` | `ffb-model` | `src/bb2020/serious_injury.rs` | ✓ |

### bb2025/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2025/SeriousInjury.java` | `ffb-model` | `src/bb2025/serious_injury.rs` | ✓ |

### dialog/ (70 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `dialog/DialogApothecaryChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_apothecary_choice_parameter.rs` | ✓ |
| `dialog/DialogArgueTheCallParameter.java` | `ffb-model` | `src/dialog/dialog_argue_the_call_parameter.rs` | ✓ |
| `dialog/DialogBlockRollParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_parameter.rs` | ✓ |
| `dialog/DialogBlockRollPartialReRollParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_partial_re_roll_parameter.rs` | ✓ |
| `dialog/DialogBlockRollPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_properties_parameter.rs` | ✓ |
| `dialog/DialogBloodlustActionParameter.java` | `ffb-model` | `src/dialog/dialog_bloodlust_action_parameter.rs` | ✓ |
| `dialog/DialogBriberyAndCorruptionParameter.java` | `ffb-model` | `src/dialog/dialog_bribery_and_corruption_parameter.rs` | ✓ |
| `dialog/DialogBribesParameter.java` | `ffb-model` | `src/dialog/dialog_bribes_parameter.rs` | ✓ |
| `dialog/DialogBuyCardsAndInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_cards_and_inducements_parameter.rs` | ✓ |
| `dialog/DialogBuyCardsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_cards_parameter.rs` | ✓ |
| `dialog/DialogBuyInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_inducements_parameter.rs` | ✓ |
| `dialog/DialogBuyPrayersAndInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_prayers_and_inducements_parameter.rs` | ✓ |
| `dialog/DialogCoinChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_coin_choice_parameter.rs` | ✓ |
| `dialog/DialogConcedeGameParameter.java` | `ffb-model` | `src/dialog/dialog_concede_game_parameter.rs` | ✓ |
| `dialog/DialogConfirmEndActionParameter.java` | `ffb-model` | `src/dialog/dialog_confirm_end_action_parameter.rs` | ✓ |
| `dialog/DialogDefenderActionParameter.java` | `ffb-model` | `src/dialog/dialog_defender_action_parameter.rs` | ✓ |
| `dialog/DialogFollowupChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_followup_choice_parameter.rs` | ✓ |
| `dialog/DialogGameStatisticsParameter.java` | `ffb-model` | `src/dialog/dialog_game_statistics_parameter.rs` | ✓ |
| `dialog/DialogId.java` | `ffb-model` | `src/dialog/dialog_id.rs` | ✓ |
| `dialog/DialogInformationOkayParameter.java` | `ffb-model` | `src/dialog/dialog_information_okay_parameter.rs` | ✓ |
| `dialog/DialogInterceptionParameter.java` | `ffb-model` | `src/dialog/dialog_interception_parameter.rs` | ✓ |
| `dialog/DialogInvalidSolidDefenceParameter.java` | `ffb-model` | `src/dialog/dialog_invalid_solid_defence_parameter.rs` | ✓ |
| `dialog/DialogJoinParameter.java` | `ffb-model` | `src/dialog/dialog_join_parameter.rs` | ✓ |
| `dialog/DialogJourneymenParameter.java` | `ffb-model` | `src/dialog/dialog_journeymen_parameter.rs` | ✓ |
| `dialog/DialogKickOffResultParameter.java` | `ffb-model` | `src/dialog/dialog_kick_off_result_parameter.rs` | ✓ |
| `dialog/DialogKickoffReturnParameter.java` | `ffb-model` | `src/dialog/dialog_kickoff_return_parameter.rs` | ✓ |
| `dialog/DialogKickSkillParameter.java` | `ffb-model` | `src/dialog/dialog_kick_skill_parameter.rs` | ✓ |
| `dialog/DialogOpponentBlockSelectionParameter.java` | `ffb-model` | `src/dialog/dialog_opponent_block_selection_parameter.rs` | ✓ |
| `dialog/DialogOpponentBlockSelectionPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_opponent_block_selection_properties_parameter.rs` | ✓ |
| `dialog/DialogParameterFactory.java` | `ffb-model` | `src/dialog/dialog_parameter_factory.rs` | ✓ |
| `dialog/DialogPassBlockParameter.java` | `ffb-model` | `src/dialog/dialog_pass_block_parameter.rs` | ✓ |
| `dialog/DialogPenaltyShootoutParameter.java` | `ffb-model` | `src/dialog/dialog_penalty_shootout_parameter.rs` | ✓ |
| `dialog/DialogPettyCashParameter.java` | `ffb-model` | `src/dialog/dialog_petty_cash_parameter.rs` | ✓ |
| `dialog/DialogPickUpChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_pick_up_choice_parameter.rs` | ✓ |
| `dialog/DialogPileDriverParameter.java` | `ffb-model` | `src/dialog/dialog_pile_driver_parameter.rs` | ✓ |
| `dialog/DialogPilingOnParameter.java` | `ffb-model` | `src/dialog/dialog_piling_on_parameter.rs` | ✓ |
| `dialog/DialogPlayerChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_player_choice_parameter.rs` | ✓ |
| `dialog/DialogPuntToCrowdParameter.java` | `ffb-model` | `src/dialog/dialog_punt_to_crowd_parameter.rs` | ✓ |
| `dialog/DialogReceiveChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_receive_choice_parameter.rs` | ✓ |
| `dialog/DialogReRollBlockForTargetsParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_block_for_targets_parameter.rs` | ✓ |
| `dialog/DialogReRollBlockForTargetsPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_block_for_targets_properties_parameter.rs` | ✓ |
| `dialog/DialogReRollForTargetsParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_for_targets_parameter.rs` | ✓ |
| `dialog/DialogReRollParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_parameter.rs` | ✓ |
| `dialog/DialogReRollPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_properties_parameter.rs` | ✓ |
| `dialog/DialogReRollRegenerationMultipleParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_regeneration_multiple_parameter.rs` | ✓ |
| `dialog/DialogSelectBlitzTargetParameter.java` | `ffb-model` | `src/dialog/dialog_select_blitz_target_parameter.rs` | ✓ |
| `dialog/DialogSelectGazeTargetParameter.java` | `ffb-model` | `src/dialog/dialog_select_gaze_target_parameter.rs` | ✓ |
| `dialog/DialogSelectKeywordParameter.java` | `ffb-model` | `src/dialog/dialog_select_keyword_parameter.rs` | ✓ |
| `dialog/DialogSelectPositionParameter.java` | `ffb-model` | `src/dialog/dialog_select_position_parameter.rs` | ✓ |
| `dialog/DialogSelectSkillParameter.java` | `ffb-model` | `src/dialog/dialog_select_skill_parameter.rs` | ✓ |
| `dialog/DialogSelectWeatherParameter.java` | `ffb-model` | `src/dialog/dialog_select_weather_parameter.rs` | ✓ |
| `dialog/DialogSetupErrorParameter.java` | `ffb-model` | `src/dialog/dialog_setup_error_parameter.rs` | ✓ |
| `dialog/DialogSkillUseParameter.java` | `ffb-model` | `src/dialog/dialog_skill_use_parameter.rs` | ✓ |
| `dialog/DialogStartGameParameter.java` | `ffb-model` | `src/dialog/dialog_start_game_parameter.rs` | ✓ |
| `dialog/DialogSwarmingErrorParameter.java` | `ffb-model` | `src/dialog/dialog_swarming_error_parameter.rs` | ✓ |
| `dialog/DialogSwarmingPlayersParameter.java` | `ffb-model` | `src/dialog/dialog_swarming_players_parameter.rs` | ✓ |
| `dialog/DialogTeamSetupParameter.java` | `ffb-model` | `src/dialog/dialog_team_setup_parameter.rs` | ✓ |
| `dialog/DialogTouchbackParameter.java` | `ffb-model` | `src/dialog/dialog_touchback_parameter.rs` | ✓ |
| `dialog/DialogUseApothecariesParameter.java` | `ffb-model` | `src/dialog/dialog_use_apothecaries_parameter.rs` | ✓ |
| `dialog/DialogUseApothecaryParameter.java` | `ffb-model` | `src/dialog/dialog_use_apothecary_parameter.rs` | ✓ |
| `dialog/DialogUseChainsawParameter.java` | `ffb-model` | `src/dialog/dialog_use_chainsaw_parameter.rs` | ✓ |
| `dialog/DialogUseIgorParameter.java` | `ffb-model` | `src/dialog/dialog_use_igor_parameter.rs` | ✓ |
| `dialog/DialogUseIgorsParameter.java` | `ffb-model` | `src/dialog/dialog_use_igors_parameter.rs` | ✓ |
| `dialog/DialogUseInducementParameter.java` | `ffb-model` | `src/dialog/dialog_use_inducement_parameter.rs` | ✓ |
| `dialog/DialogUseMortuaryAssistantParameter.java` | `ffb-model` | `src/dialog/dialog_use_mortuary_assistant_parameter.rs` | ✓ |
| `dialog/DialogUseMortuaryAssistantsParameter.java` | `ffb-model` | `src/dialog/dialog_use_mortuary_assistants_parameter.rs` | ✓ |
| `dialog/DialogWinningsReRollParameter.java` | `ffb-model` | `src/dialog/dialog_winnings_re_roll_parameter.rs` | ✓ |
| `dialog/DialogWithoutParameter.java` | `ffb-model` | `src/dialog/dialog_without_parameter.rs` | ✓ |
| `dialog/DialogWizardSpellParameter.java` | `ffb-model` | `src/dialog/dialog_wizard_spell_parameter.rs` | ✓ |
| `dialog/UtilDialogParameter.java` | `ffb-model` | `src/dialog/util_dialog_parameter.rs` | ✓ |

### factory/ (82 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `factory/AnimationTypeFactory.java` | `ffb-model` | `src/factory/animation_type_factory.rs` | ✓ |
| `factory/ApothecaryModeFactory.java` | `ffb-model` | `src/factory/apothecary_mode_factory.rs` | ✓ |
| `factory/ApothecaryStatusFactory.java` | `ffb-model` | `src/factory/apothecary_status_factory.rs` | ✓ |
| `factory/application/NetCommandIdFactory.java` | `ffb-model` | `src/factory/application/net_command_id_factory.rs` | ✓ |
| `factory/ArmorModifierFactory.java` | `ffb-model` | `src/factory/armor_modifier_factory.rs` | ✓ |
| `factory/ArmorModifiers.java` | `ffb-model` | `src/factory/armor_modifiers.rs` | ✓ |
| `factory/bb2016/ArmorModifiers.java` | `ffb-model` | `src/factory/bb2016/armor_modifiers.rs` | ✓ |
| `factory/bb2016/InjuryModifiers.java` | `ffb-model` | `src/factory/bb2016/injury_modifiers.rs` | ✓ |
| `factory/bb2016/JumpModifierFactory.java` | `ffb-model` | `src/factory/bb2016/jump_modifier_factory.rs` | ✓ |
| `factory/bb2020/ArmorModifiers.java` | `ffb-model` | `src/factory/bb2020/armor_modifiers.rs` | ✓ |
| `factory/bb2020/InjuryModifiers.java` | `ffb-model` | `src/factory/bb2020/injury_modifiers.rs` | ✓ |
| `factory/bb2020/PrayerFactory.java` | `ffb-model` | `src/factory/bb2020/prayer_factory.rs` | ✓ |
| `factory/bb2025/ArmorModifiers.java` | `ffb-model` | `src/factory/bb2025/armor_modifiers.rs` | ✓ |
| `factory/bb2025/InjuryModifiers.java` | `ffb-model` | `src/factory/bb2025/injury_modifiers.rs` | ✓ |
| `factory/bb2025/PrayerFactory.java` | `ffb-model` | `src/factory/bb2025/prayer_factory.rs` | ✓ |
| `factory/BlockResultFactory.java` | `ffb-model` | `src/factory/block_result_factory.rs` | ✓ |
| `factory/CardEffectFactory.java` | `ffb-model` | `src/factory/card_effect_factory.rs` | ✓ |
| `factory/CardFactory.java` | `ffb-model` | `src/factory/card_factory.rs` | ✓ |
| `factory/CardTypeFactory.java` | `ffb-model` | `src/factory/card_type_factory.rs` | ✓ |
| `factory/CatchModifierFactory.java` | `ffb-model` | `src/factory/catch_modifier_factory.rs` | ✓ |
| `factory/CatchScatterThrowInModeFactory.java` | `ffb-model` | `src/factory/catch_scatter_throw_in_mode_factory.rs` | ✓ |
| `factory/ClientModeFactory.java` | `ffb-model` | `src/factory/client_mode_factory.rs` | ✓ |
| `factory/ClientStateIdFactory.java` | `ffb-model` | `src/factory/client_state_id_factory.rs` | ✓ |
| `factory/common/GoForItModifierFactory.java` | `ffb-model` | `src/factory/common/go_for_it_modifier_factory.rs` | ✓ |
| `factory/ConcedeGameStatusFactory.java` | `ffb-model` | `src/factory/concede_game_status_factory.rs` | ✓ |
| `factory/DialogIdFactory.java` | `ffb-model` | `src/factory/dialog_id_factory.rs` | ✓ |
| `factory/DirectionFactory.java` | `ffb-model` | `src/factory/direction_factory.rs` | ✓ |
| `factory/DodgeModifierFactory.java` | `ffb-model` | `src/factory/dodge_modifier_factory.rs` | ✓ |
| `factory/FoulAssistArmorModifier.java` | `ffb-model` | `src/factory/foul_assist_armor_modifier.rs` | ✓ |
| `factory/GameOptionFactory.java` | `ffb-model` | `src/factory/game_option_factory.rs` | ✓ |
| `factory/GameOptionIdFactory.java` | `ffb-model` | `src/factory/game_option_id_factory.rs` | ✓ |
| `factory/GameStatusFactory.java` | `ffb-model` | `src/factory/game_status_factory.rs` | ✓ |
| `factory/GazeModifierFactory.java` | `ffb-model` | `src/factory/gaze_modifier_factory.rs` | ✓ |
| `factory/GenerifiedModifierFactory.java` | `ffb-model` | `src/factory/generified_modifier_factory.rs` | ✓ |
| `factory/IFactorySource.java` | `ffb-model` | `src/factory/i_factory_source.rs` | ✓ |
| `factory/ILoggingFacade.java` | `ffb-model` | `src/factory/i_logging_facade.rs` | ✓ |
| `factory/INamedObjectFactory.java` | `ffb-model` | `src/factory/i_named_object_factory.rs` | ✓ |
| `factory/InducementPhaseFactory.java` | `ffb-model` | `src/factory/inducement_phase_factory.rs` | ✓ |
| `factory/InducementTypeFactory.java` | `ffb-model` | `src/factory/inducement_type_factory.rs` | ✓ |
| `factory/InjuryModifierFactory.java` | `ffb-model` | `src/factory/injury_modifier_factory.rs` | ✓ |
| `factory/InjuryModifiers.java` | `ffb-model` | `src/factory/injury_modifiers.rs` | ✓ |
| `factory/InjuryTypeFactory.java` | `ffb-model` | `src/factory/injury_type_factory.rs` | ✓ |
| `factory/InterceptionModifierFactory.java` | `ffb-model` | `src/factory/interception_modifier_factory.rs` | ✓ |
| `factory/IRollModifierFactory.java` | `ffb-model` | `src/factory/i_roll_modifier_factory.rs` | ✓ |
| `factory/JumpModifierFactory.java` | `ffb-model` | `src/factory/jump_modifier_factory.rs` | ✓ |
| `factory/JumpUpModifierFactory.java` | `ffb-model` | `src/factory/jump_up_modifier_factory.rs` | ✓ |
| `factory/KickoffResultFactory.java` | `ffb-model` | `src/factory/kickoff_result_factory.rs` | ✓ |
| `factory/LeaderStateFactory.java` | `ffb-model` | `src/factory/leader_state_factory.rs` | ✓ |
| `factory/MechanicsFactory.java` | `ffb-model` | `src/factory/mechanics_factory.rs` | ✓ |
| `factory/mixed/CasualtyModifierFactory.java` | `ffb-model` | `src/factory/mixed/casualty_modifier_factory.rs` | ✓ |
| `factory/mixed/JumpModifierFactory.java` | `ffb-model` | `src/factory/mixed/jump_modifier_factory.rs` | ✓ |
| `factory/ModelChangeDataTypeFactory.java` | `ffb-model` | `src/factory/model_change_data_type_factory.rs` | ✓ |
| `factory/ModelChangeIdFactory.java` | `ffb-model` | `src/factory/model_change_id_factory.rs` | ✓ |
| `factory/PassingDistanceFactory.java` | `ffb-model` | `src/factory/passing_distance_factory.rs` | ✓ |
| `factory/PassModifierFactory.java` | `ffb-model` | `src/factory/pass_modifier_factory.rs` | ✓ |
| `factory/PassResultFactory.java` | `ffb-model` | `src/factory/pass_result_factory.rs` | ✓ |
| `factory/PickupModifierFactory.java` | `ffb-model` | `src/factory/pickup_modifier_factory.rs` | ✓ |
| `factory/PlayerActionFactory.java` | `ffb-model` | `src/factory/player_action_factory.rs` | ✓ |
| `factory/PlayerChoiceModeFactory.java` | `ffb-model` | `src/factory/player_choice_mode_factory.rs` | ✓ |
| `factory/PlayerGenderFactory.java` | `ffb-model` | `src/factory/player_gender_factory.rs` | ✓ |
| `factory/PlayerTypeFactory.java` | `ffb-model` | `src/factory/player_type_factory.rs` | ✓ |
| `factory/PrayerFactory.java` | `ffb-model` | `src/factory/prayer_factory.rs` | ✓ |
| `factory/PushbackModeFactory.java` | `ffb-model` | `src/factory/pushback_mode_factory.rs` | ✓ |
| `factory/ReportFactory.java` | `ffb-model` | `src/factory/report_factory.rs` | ✓ |
| `factory/ReportIdFactory.java` | `ffb-model` | `src/factory/report_id_factory.rs` | ✓ |
| `factory/ReRolledActionFactory.java` | `ffb-model` | `src/factory/re_rolled_action_factory.rs` | ✓ |
| `factory/ReRollPropertyFactory.java` | `ffb-model` | `src/factory/re_roll_property_factory.rs` | ✓ |
| `factory/ReRollSourceFactory.java` | `ffb-model` | `src/factory/re_roll_source_factory.rs` | ✓ |
| `factory/RightStuffModifierFactory.java` | `ffb-model` | `src/factory/right_stuff_modifier_factory.rs` | ✓ |
| `factory/SendToBoxReasonFactory.java` | `ffb-model` | `src/factory/send_to_box_reason_factory.rs` | ✓ |
| `factory/SeriousInjuryFactory.java` | `ffb-model` | `src/factory/serious_injury_factory.rs` | ✓ |
| `factory/ServerStatusFactory.java` | `ffb-model` | `src/factory/server_status_factory.rs` | ✓ |
| `factory/SkillCategoryFactory.java` | `ffb-model` | `src/factory/skill_category_factory.rs` | ✓ |
| `factory/SkillFactory.java` | `ffb-model` | `src/factory/skill_factory.rs` | ✓ |
| `factory/SkillPropertiesFactory.java` | `ffb-model` | `src/factory/skill_properties_factory.rs` | ✓ |
| `factory/SkillUseFactory.java` | `ffb-model` | `src/factory/skill_use_factory.rs` | ✓ |
| `factory/SoundIdFactory.java` | `ffb-model` | `src/factory/sound_id_factory.rs` | ✓ |
| `factory/SpecialEffectFactory.java` | `ffb-model` | `src/factory/special_effect_factory.rs` | ✓ |
| `factory/TeamStatusFactory.java` | `ffb-model` | `src/factory/team_status_factory.rs` | ✓ |
| `factory/TemporaryStatModifierFactory.java` | `ffb-model` | `src/factory/temporary_stat_modifier_factory.rs` | ✓ |
| `factory/TurnModeFactory.java` | `ffb-model` | `src/factory/turn_mode_factory.rs` | ✓ |
| `factory/WeatherFactory.java` | `ffb-model` | `src/factory/weather_factory.rs` | ✓ |

### inducement/ (29 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `inducement/bb2016/CardHandlerKey.java` | `ffb-model` | `src/inducement/bb2016/card_handler_key.rs` | ✓ |
| `inducement/bb2016/Cards.java` | `ffb-model` | `src/inducement/bb2016/cards.rs` | ✓ |
| `inducement/bb2016/CardType.java` | `ffb-model` | `src/inducement/bb2016/card_type.rs` | ✓ |
| `inducement/bb2016/InducementCollection.java` | `ffb-model` | `src/inducement/bb2016/inducement_collection.rs` | ✓ |
| `inducement/bb2020/CardHandlerKey.java` | `ffb-model` | `src/inducement/bb2020/card_handler_key.rs` | ✓ |
| `inducement/bb2020/Cards.java` | `ffb-model` | `src/inducement/bb2020/cards.rs` | ✓ |
| `inducement/bb2020/CardType.java` | `ffb-model` | `src/inducement/bb2020/card_type.rs` | ✓ |
| `inducement/bb2020/InducementCollection.java` | `ffb-model` | `src/inducement/bb2020/inducement_collection.rs` | ✓ |
| `inducement/bb2020/Prayer.java` | `ffb-model` | `src/inducement/bb2020/prayer.rs` | ✓ |
| `inducement/bb2020/Prayers.java` | `ffb-model` | `src/inducement/bb2020/prayers.rs` | ✓ |
| `inducement/bb2025/InducementCollection.java` | `ffb-model` | `src/inducement/bb2025/inducement_collection.rs` | ✓ |
| `inducement/bb2025/Prayer.java` | `ffb-model` | `src/inducement/bb2025/prayer.rs` | ✓ |
| `inducement/bb2025/Prayers.java` | `ffb-model` | `src/inducement/bb2025/prayers.rs` | ✓ |
| `inducement/BriberyAndCorruptionAction.java` | `ffb-model` | `src/inducement/bribery_and_corruption_action.rs` | ✓ |
| `inducement/Card.java` | `ffb-model` | `src/inducement/card.rs` | ✓ |
| `inducement/CardChoice.java` | `ffb-model` | `src/inducement/card_choice.rs` | ✓ |
| `inducement/CardChoices.java` | `ffb-model` | `src/inducement/card_choices.rs` | ✓ |
| `inducement/CardHandlerKey.java` | `ffb-model` | `src/inducement/card_handler_key.rs` | ✓ |
| `inducement/CardReport.java` | `ffb-model` | `src/inducement/card_report.rs` | ✓ |
| `inducement/Cards.java` | `ffb-model` | `src/inducement/cards.rs` | ✓ |
| `inducement/CardType.java` | `ffb-model` | `src/inducement/card_type.rs` | ✓ |
| `inducement/EnhancementProvider.java` | `ffb-model` | `src/inducement/enhancement_provider.rs` | ✓ |
| `inducement/Inducement.java` | `ffb-model` | `src/inducement/inducement.rs` | ✓ |
| `inducement/InducementCollection.java` | `ffb-model` | `src/inducement/inducement_collection.rs` | ✓ |
| `inducement/InducementDuration.java` | `ffb-model` | `src/inducement/inducement_duration.rs` | ✓ |
| `inducement/InducementPhase.java` | `ffb-model` | `src/inducement/inducement_phase.rs` | ✓ |
| `inducement/InducementType.java` | `ffb-model` | `src/inducement/inducement_type.rs` | ✓ |
| `inducement/Prayer.java` | `ffb-model` | `src/inducement/prayer.rs` | ✓ |
| `inducement/Usage.java` | `ffb-model` | `src/inducement/usage.rs` | ✓ |

### injury/ (52 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `injury/BallAndChain.java` | `ffb-model` | `src/injury/ball_and_chain.rs` | ✓ |
| `injury/Bitten.java` | `ffb-model` | `src/injury/bitten.rs` | ✓ |
| `injury/Block.java` | `ffb-model` | `src/injury/block.rs` | ✓ |
| `injury/BlockProne.java` | `ffb-model` | `src/injury/block_prone.rs` | ✓ |
| `injury/BlockProneForSpp.java` | `ffb-model` | `src/injury/block_prone_for_spp.rs` | ✓ |
| `injury/BlockStunned.java` | `ffb-model` | `src/injury/block_stunned.rs` | ✓ |
| `injury/BlockStunnedForSpp.java` | `ffb-model` | `src/injury/block_stunned_for_spp.rs` | ✓ |
| `injury/Bomb.java` | `ffb-model` | `src/injury/bomb.rs` | ✓ |
| `injury/BombForSpp.java` | `ffb-model` | `src/injury/bomb_for_spp.rs` | ✓ |
| `injury/BreatheFire.java` | `ffb-model` | `src/injury/breathe_fire.rs` | ✓ |
| `injury/BreatheFireForSpp.java` | `ffb-model` | `src/injury/breathe_fire_for_spp.rs` | ✓ |
| `injury/Chainsaw.java` | `ffb-model` | `src/injury/chainsaw.rs` | ✓ |
| `injury/ChainsawForSpp.java` | `ffb-model` | `src/injury/chainsaw_for_spp.rs` | ✓ |
| `injury/context/IInjuryContextModification.java` | `ffb-model` | `src/injury/context/i_injury_context_modification.rs` | ✓ |
| `injury/context/InjuryContext.java` | `ffb-model` | `src/injury/context/injury_context.rs` | ✓ |
| `injury/context/InjuryModification.java` | `ffb-model` | `src/injury/context/injury_modification.rs` | ✓ |
| `injury/context/ModifiedInjuryContext.java` | `ffb-model` | `src/injury/context/modified_injury_context.rs` | ✓ |
| `injury/CrowdPush.java` | `ffb-model` | `src/injury/crowd_push.rs` | ✓ |
| `injury/CrowdPushForSpp.java` | `ffb-model` | `src/injury/crowd_push_for_spp.rs` | ✓ |
| `injury/DropDodge.java` | `ffb-model` | `src/injury/drop_dodge.rs` | ✓ |
| `injury/DropDodgeForSpp.java` | `ffb-model` | `src/injury/drop_dodge_for_spp.rs` | ✓ |
| `injury/DropGFI.java` | `ffb-model` | `src/injury/drop_gfi.rs` | ✓ |
| `injury/DropJump.java` | `ffb-model` | `src/injury/drop_jump.rs` | ✓ |
| `injury/EatPlayer.java` | `ffb-model` | `src/injury/eat_player.rs` | ✓ |
| `injury/Fireball.java` | `ffb-model` | `src/injury/fireball.rs` | ✓ |
| `injury/Foul.java` | `ffb-model` | `src/injury/foul.rs` | ✓ |
| `injury/FoulForSpp.java` | `ffb-model` | `src/injury/foul_for_spp.rs` | ✓ |
| `injury/FoulForSppWithChainsaw.java` | `ffb-model` | `src/injury/foul_for_spp_with_chainsaw.rs` | ✓ |
| `injury/FoulWithChainsaw.java` | `ffb-model` | `src/injury/foul_with_chainsaw.rs` | ✓ |
| `injury/InjuryType.java` | `ffb-model` | `src/injury/injury_type.rs` | ✓ |
| `injury/KegHit.java` | `ffb-model` | `src/injury/keg_hit.rs` | ✓ |
| `injury/KTMCrowd.java` | `ffb-model` | `src/injury/ktm_crowd.rs` | ✓ |
| `injury/KTMFumbleApoKoInjury.java` | `ffb-model` | `src/injury/ktm_fumble_apo_ko_injury.rs` | ✓ |
| `injury/KTMFumbleInjury.java` | `ffb-model` | `src/injury/ktm_fumble_injury.rs` | ✓ |
| `injury/KTMInjury.java` | `ffb-model` | `src/injury/ktm_injury.rs` | ✓ |
| `injury/Lightning.java` | `ffb-model` | `src/injury/lightning.rs` | ✓ |
| `injury/PilingOnArmour.java` | `ffb-model` | `src/injury/piling_on_armour.rs` | ✓ |
| `injury/PilingOnInjury.java` | `ffb-model` | `src/injury/piling_on_injury.rs` | ✓ |
| `injury/PilingOnKnockedOut.java` | `ffb-model` | `src/injury/piling_on_knocked_out.rs` | ✓ |
| `injury/ProjectileVomit.java` | `ffb-model` | `src/injury/projectile_vomit.rs` | ✓ |
| `injury/QuickBite.java` | `ffb-model` | `src/injury/quick_bite.rs` | ✓ |
| `injury/Sabotaged.java` | `ffb-model` | `src/injury/sabotaged.rs` | ✓ |
| `injury/Saboteur.java` | `ffb-model` | `src/injury/saboteur.rs` | ✓ |
| `injury/Stab.java` | `ffb-model` | `src/injury/stab.rs` | ✓ |
| `injury/StabForSpp.java` | `ffb-model` | `src/injury/stab_for_spp.rs` | ✓ |
| `injury/ThenIStartedBlastin.java` | `ffb-model` | `src/injury/then_i_started_blastin.rs` | ✓ |
| `injury/ThrowARock.java` | `ffb-model` | `src/injury/throw_a_rock.rs` | ✓ |
| `injury/TrapDoorFall.java` | `ffb-model` | `src/injury/trap_door_fall.rs` | ✓ |
| `injury/TrapDoorFallForSpp.java` | `ffb-model` | `src/injury/trap_door_fall_for_spp.rs` | ✓ |
| `injury/TTMHitPlayer.java` | `ffb-model` | `src/injury/ttm_hit_player.rs` | ✓ |
| `injury/TTMHitPlayerForSpp.java` | `ffb-model` | `src/injury/ttm_hit_player_for_spp.rs` | ✓ |
| `injury/TTMLanding.java` | `ffb-model` | `src/injury/ttm_landing.rs` | ✓ |

### json/ (35 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `json/IJsonOption.java` | `—` | `—` | — |
| `json/IJsonReadable.java` | `—` | `—` | — |
| `json/IJsonSerializable.java` | `—` | `—` | — |
| `json/IJsonWriteable.java` | `—` | `—` | — |
| `json/JsonAbstractOption.java` | `—` | `—` | — |
| `json/JsonArrayOption.java` | `—` | `—` | — |
| `json/JsonBooleanArrayOption.java` | `—` | `—` | — |
| `json/JsonBooleanMapOption.java` | `—` | `—` | — |
| `json/JsonBooleanOption.java` | `—` | `—` | — |
| `json/JsonDateOption.java` | `—` | `—` | — |
| `json/JsonEnumWithNameOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateArrayOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateMapOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateOption.java` | `—` | `—` | — |
| `json/JsonIntArrayOption.java` | `—` | `—` | — |
| `json/JsonIntegerListMapOption.java` | `—` | `—` | — |
| `json/JsonIntegerMapOption.java` | `—` | `—` | — |
| `json/JsonIntOption.java` | `—` | `—` | — |
| `json/JsonLegacySkillValuesOption.java` | `—` | `—` | — |
| `json/JsonLongOption.java` | `—` | `—` | — |
| `json/JsonObjectOption.java` | `—` | `—` | — |
| `json/JsonPlayerStateOption.java` | `—` | `—` | — |
| `json/JsonSkillPropertiesMapOption.java` | `—` | `—` | — |
| `json/JsonSkillValuesMapOption.java` | `—` | `—` | — |
| `json/JsonSkillWithValuesMapOption.java` | `—` | `—` | — |
| `json/JsonStringArrayOption.java` | `—` | `—` | — |
| `json/JsonStringListMapOption.java` | `—` | `—` | — |
| `json/JsonStringMapListOption.java` | `—` | `—` | — |
| `json/JsonStringMapOption.java` | `—` | `—` | — |
| `json/JsonStringOption.java` | `—` | `—` | — |
| `json/JsonTemporaryModifiersMapOption.java` | `—` | `—` | — |
| `json/JsonValueOption.java` | `—` | `—` | — |
| `json/LZString.java` | `—` | `—` | — |
| `json/MissingKeyException.java` | `—` | `—` | — |
| `json/UtilJson.java` | `—` | `—` | — |

### kickoff/ (8 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `kickoff/bb2016/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2016/kickoff_result.rs` | ✓ |
| `kickoff/bb2016/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2016/kickoff_result_mapping.rs` | ✓ |
| `kickoff/bb2020/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2020/kickoff_result.rs` | ✓ |
| `kickoff/bb2020/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2020/kickoff_result_mapping.rs` | ✓ |
| `kickoff/bb2025/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2025/kickoff_result.rs` | ✓ |
| `kickoff/bb2025/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2025/kickoff_result_mapping.rs` | ✓ |
| `kickoff/KickoffResult.java` | `ffb-model` | `src/kickoff/kickoff_result.rs` | ✓ |
| `kickoff/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/kickoff_result_mapping.rs` | ✓ |

### marking/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `marking/FieldMarker.java` | `ffb-model` | `src/marking/field_marker.rs` | ✓ |
| `marking/PlayerMarker.java` | `ffb-model` | `src/marking/player_marker.rs` | ✓ |
| `marking/SortMode.java` | `ffb-model` | `src/marking/sort_mode.rs` | ✓ |
| `marking/TransientPlayerMarker.java` | `ffb-model` | `src/marking/transient_player_marker.rs` | ✓ |

### mechanics/ (50 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `mechanics/AgilityMechanic.java` | `ffb-mechanics` | `src/agility_mechanic.rs` | ✓ |
| `mechanics/ApothecaryMechanic.java` | `ffb-mechanics` | `src/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2016/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2016/agility_mechanic.rs` | ~ |
| `mechanics/bb2016/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2016/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2016/GameMechanic.java` | `ffb-mechanics` | `src/bb2016/game_mechanic.rs` | ✓ |
| `mechanics/bb2016/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2016/injury_mechanic.rs` | ✓ |
| `mechanics/bb2016/JumpMechanic.java` | `ffb-mechanics` | `src/bb2016/jump_mechanic.rs` | ✓ |
| `mechanics/bb2016/OnTheBallMechanic.java` | `ffb-mechanics` | `src/bb2016/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/bb2016/PassMechanic.java` | `ffb-mechanics` | `src/bb2016/pass_mechanic.rs` | ✓ |
| `mechanics/bb2016/SkillMechanic.java` | `ffb-mechanics` | `src/bb2016/skill_mechanic.rs` | ✓ |
| `mechanics/bb2016/SppMechanic.java` | `ffb-mechanics` | `src/bb2016/spp_mechanic.rs` | ✓ |
| `mechanics/bb2016/StatsMechanic.java` | `ffb-mechanics` | `src/bb2016/stats_mechanic.rs` | ✓ |
| `mechanics/bb2016/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2016/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2016/TtmMechanic.java` | `ffb-mechanics` | `src/bb2016/ttm_mechanic.rs` | ✓ |
| `mechanics/bb2020/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2020/agility_mechanic.rs` | ✓ |
| `mechanics/bb2020/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2020/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2020/GameMechanic.java` | `ffb-mechanics` | `src/bb2020/game_mechanic.rs` | ✓ |
| `mechanics/bb2020/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2020/injury_mechanic.rs` | ✓ |
| `mechanics/bb2020/JumpMechanic.java` | `ffb-mechanics` | `src/bb2020/jump_mechanic.rs` | ✓ |
| `mechanics/bb2020/PassMechanic.java` | `ffb-mechanics` | `src/bb2020/pass_mechanic.rs` | ✓ |
| `mechanics/bb2020/SkillMechanic.java` | `ffb-mechanics` | `src/bb2020/skill_mechanic.rs` | ✓ |
| `mechanics/bb2020/SppMechanic.java` | `ffb-mechanics` | `src/bb2020/spp_mechanic.rs` | ✓ |
| `mechanics/bb2020/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2020/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2020/TtmMechanic.java` | `ffb-mechanics` | `src/bb2020/ttm_mechanic.rs` | ✓ |
| `mechanics/bb2025/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2025/agility_mechanic.rs` | ✓ |
| `mechanics/bb2025/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2025/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2025/GameMechanic.java` | `ffb-mechanics` | `src/bb2025/game_mechanic.rs` | ✓ |
| `mechanics/bb2025/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2025/injury_mechanic.rs` | ✓ |
| `mechanics/bb2025/JumpMechanic.java` | `ffb-mechanics` | `src/bb2025/jump_mechanic.rs` | ✓ |
| `mechanics/bb2025/PassMechanic.java` | `ffb-mechanics` | `src/bb2025/pass_mechanic.rs` | ✓ |
| `mechanics/bb2025/SkillMechanic.java` | `ffb-mechanics` | `src/bb2025/skill_mechanic.rs` | ✓ |
| `mechanics/bb2025/SppMechanic.java` | `ffb-mechanics` | `src/bb2025/spp_mechanic.rs` | ✓ |
| `mechanics/bb2025/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2025/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2025/TtmMechanic.java` | `ffb-mechanics` | `src/bb2025/ttm_mechanic.rs` | ✓ |
| `mechanics/GameMechanic.java` | `ffb-mechanics` | `src/game_mechanic.rs` | ✓ |
| `mechanics/InjuryMechanic.java` | `ffb-mechanics` | `src/injury_mechanic.rs` | ✓ |
| `mechanics/JumpMechanic.java` | `ffb-mechanics` | `src/jump_mechanic.rs` | ✓ |
| `mechanics/Mechanic.java` | `ffb-mechanics` | `src/mechanic.rs` | ✓ |
| `mechanics/mixed/OnTheBallMechanic.java` | `ffb-mechanics` | `src/mixed/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/mixed/StatsMechanic.java` | `ffb-mechanics` | `src/mixed/stats_mechanic.rs` | ✓ |
| `mechanics/OnTheBallMechanic.java` | `ffb-mechanics` | `src/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/PassMechanic.java` | `ffb-mechanics` | `src/pass_mechanic.rs` | ✓ |
| `mechanics/PassResult.java` | `ffb-mechanics` | `src/pass_result.rs` | ✓ |
| `mechanics/SkillMechanic.java` | `ffb-mechanics` | `src/skill_mechanic.rs` | ✓ |
| `mechanics/SppMechanic.java` | `ffb-mechanics` | `src/spp_mechanic.rs` | ✓ |
| `mechanics/StatsDrawingModifier.java` | `ffb-mechanics` | `src/stats_drawing_modifier.rs` | ✓ |
| `mechanics/StatsMechanic.java` | `ffb-mechanics` | `src/stats_mechanic.rs` | ✓ |
| `mechanics/ThrowInMechanic.java` | `ffb-mechanics` | `src/throw_in_mechanic.rs` | ✓ |
| `mechanics/TtmMechanic.java` | `ffb-mechanics` | `src/ttm_mechanic.rs` | ✓ |
| `mechanics/Wording.java` | `ffb-mechanics` | `src/wording.rs` | ✓ |

### model/ (61 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `model/ActingPlayer.java` | `ffb-model` | `src/model/acting_player.rs` | ✓ |
| `model/Animation.java` | `ffb-model` | `src/model/animation.rs` | ✓ |
| `model/AnimationType.java` | `ffb-model` | `src/model/animation_type.rs` | ✓ |
| `model/BlitzTurnState.java` | `ffb-model` | `src/model/blitz_turn_state.rs` | ✓ |
| `model/BlockKind.java` | `ffb-model` | `src/model/block_kind.rs` | ✓ |
| `model/BlockRoll.java` | `ffb-model` | `src/model/block_roll.rs` | ✓ |
| `model/BlockRollProperties.java` | `ffb-model` | `src/model/block_roll_properties.rs` | ✓ |
| `model/BlockTarget.java` | `ffb-model` | `src/model/block_target.rs` | ✓ |
| `model/change/IModelChangeObserver.java` | `ffb-model` | `src/model/change/i_model_change_observer.rs` | ✓ |
| `model/change/ModelChange.java` | `ffb-model` | `src/model/change/model_change.rs` | ✓ |
| `model/change/ModelChangeDataType.java` | `ffb-model` | `src/model/change/model_change_data_type.rs` | ✓ |
| `model/change/ModelChangeId.java` | `ffb-model` | `src/model/change/model_change_id.rs` | ✓ |
| `model/change/ModelChangeList.java` | `ffb-model` | `src/model/change/model_change_list.rs` | ✓ |
| `model/change/ModelChangeObservable.java` | `ffb-model` | `src/model/change/model_change_observable.rs` | ✓ |
| `model/change/ModelChangeProcessor.java` | `ffb-model` | `src/model/change/model_change_processor.rs` | ✓ |
| `model/EnhancementRegistry.java` | `ffb-model` | `src/model/enhancement_registry.rs` | ✓ |
| `model/FieldModel.java` | `ffb-model` | `src/model/field_model.rs` | ✓ |
| `model/Game.java` | `ffb-model` | `src/model/game.rs` | ✓ |
| `model/GameOptions.java` | `ffb-model` | `src/model/game_options.rs` | ✓ |
| `model/GameResult.java` | `ffb-model` | `src/model/game_result.rs` | ✓ |
| `model/GameRules.java` | `ffb-model` | `src/model/game_rules.rs` | ✓ |
| `model/InducementSet.java` | `ffb-model` | `src/model/inducement_set.rs` | ✓ |
| `model/InjuryTypeConstants.java` | `ffb-model` | `src/model/injury_type_constants.rs` | ✓ |
| `model/ISkillBehaviour.java` | `ffb-model` | `src/model/i_skill_behaviour.rs` | ✓ |
| `model/Keyword.java` | `ffb-model` | `src/model/keyword.rs` | ✓ |
| `model/KickTeamMateRange.java` | `ffb-model` | `src/model/kick_team_mate_range.rs` | ✓ |
| `model/Player.java` | `ffb-model` | `src/model/player.rs` | ✓ |
| `model/PlayerModifier.java` | `ffb-model` | `src/model/player_modifier.rs` | ✓ |
| `model/PlayerResult.java` | `ffb-model` | `src/model/player_result.rs` | ✓ |
| `model/PlayerStats.java` | `ffb-model` | `src/model/player_stats.rs` | ✓ |
| `model/PlayerStatus.java` | `ffb-model` | `src/model/player_status.rs` | ✓ |
| `model/Position.java` | `ffb-model` | `src/model/position.rs` | ✓ |
| `model/property/CancelSkillProperty.java` | `ffb-model` | `src/model/property/cancel_skill_property.rs` | ✓ |
| `model/property/ISkillProperty.java` | `ffb-model` | `src/model/property/i_skill_property.rs` | ✓ |
| `model/property/NamedProperties.java` | `ffb-model` | `src/model/property/named_properties.rs` | ✓ |
| `model/property/NamedProperty.java` | `ffb-model` | `src/model/property/named_property.rs` | ✓ |
| `model/property/PassingProperty.java` | `ffb-model` | `src/model/property/passing_property.rs` | ✓ |
| `model/Roster.java` | `ffb-model` | `src/model/roster.rs` | ✓ |
| `model/RosterPlayer.java` | `ffb-model` | `src/model/roster_player.rs` | ✓ |
| `model/RosterPosition.java` | `ffb-model` | `src/model/roster_position.rs` | ✓ |
| `model/RosterSkeleton.java` | `ffb-model` | `src/model/roster_skeleton.rs` | ✓ |
| `model/sketch/Sketch.java` | `ffb-model` | `src/model/sketch/sketch.rs` | ✓ |
| `model/sketch/SketchState.java` | `ffb-model` | `src/model/sketch/sketch_state.rs` | ✓ |
| `model/skill/AnimosityValueEvaluator.java` | `ffb-model` | `src/model/skill/animosity_value_evaluator.rs` | ✓ |
| `model/skill/DeclareCondition.java` | `ffb-model` | `src/model/skill/declare_condition.rs` | ✓ |
| `model/skill/Skill.java` | `ffb-model` | `src/model/skill/skill.rs` | ✓ |
| `model/skill/SkillClassWithValue.java` | `ffb-model` | `src/model/skill/skill_class_with_value.rs` | ✓ |
| `model/skill/SkillDisplayInfo.java` | `ffb-model` | `src/model/skill/skill_display_info.rs` | ✓ |
| `model/skill/SkillUsageType.java` | `ffb-model` | `src/model/skill/skill_usage_type.rs` | ✓ |
| `model/skill/SkillValueEvaluator.java` | `ffb-model` | `src/model/skill/skill_value_evaluator.rs` | ✓ |
| `model/skill/SkillWithValue.java` | `ffb-model` | `src/model/skill/skill_with_value.rs` | ✓ |
| `model/SpecialRule.java` | `ffb-model` | `src/model/special_rule.rs` | ✓ |
| `model/stadium/OnPitchEnhancement.java` | `ffb-model` | `src/model/stadium/on_pitch_enhancement.rs` | ✓ |
| `model/stadium/TrapDoor.java` | `ffb-model` | `src/model/stadium/trap_door.rs` | ✓ |
| `model/TargetSelectionState.java` | `ffb-model` | `src/model/target_selection_state.rs` | ✓ |
| `model/Team.java` | `ffb-model` | `src/model/team.rs` | ✓ |
| `model/TeamResult.java` | `ffb-model` | `src/model/team_result.rs` | ✓ |
| `model/TeamSkeleton.java` | `ffb-model` | `src/model/team_skeleton.rs` | ✓ |
| `model/TurnData.java` | `ffb-model` | `src/model/turn_data.rs` | ✓ |
| `model/ZappedPlayer.java` | `ffb-model` | `src/model/zapped_player.rs` | ✓ |
| `model/ZappedPosition.java` | `ffb-model` | `src/model/zapped_position.rs` | ✓ |

### modifiers/ (82 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `modifiers/ArmorModifier.java` | `ffb-mechanics` | `src/modifiers/armor_modifier.rs` | ✓ |
| `modifiers/ArmorModifierContext.java` | `ffb-mechanics` | `src/modifiers/armor_modifier_context.rs` | ✓ |
| `modifiers/bb2016/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/dodge_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/gaze_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/jump_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/pass_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/CasualtyModifier.java` | `ffb-mechanics` | `src/modifiers/bb2020/casualty_modifier.rs` | ✓ |
| `modifiers/bb2020/CasualtyNigglingModifier.java` | `ffb-mechanics` | `src/modifiers/bb2020/casualty_niggling_modifier.rs` | ✓ |
| `modifiers/bb2020/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/gaze_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/CatchContext.java` | `ffb-mechanics` | `src/modifiers/catch_context.rs` | ✓ |
| `modifiers/CatchModifier.java` | `ffb-mechanics` | `src/modifiers/catch_modifier.rs` | ✓ |
| `modifiers/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/catch_modifier_collection.rs` | ✓ |
| `modifiers/DodgeContext.java` | `ffb-mechanics` | `src/modifiers/dodge_context.rs` | ✓ |
| `modifiers/DodgeModifier.java` | `ffb-mechanics` | `src/modifiers/dodge_modifier.rs` | ✓ |
| `modifiers/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/dodge_modifier_collection.rs` | ✓ |
| `modifiers/GazeModifier.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier.rs` | ✓ |
| `modifiers/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_collection.rs` | ✓ |
| `modifiers/GazeModifierContext.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_context.rs` | ✓ |
| `modifiers/GoForItContext.java` | `ffb-mechanics` | `src/modifiers/go_for_it_context.rs` | ✓ |
| `modifiers/GoForItModifier.java` | `ffb-mechanics` | `src/modifiers/go_for_it_modifier.rs` | ✓ |
| `modifiers/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/InjuryModifier.java` | `ffb-mechanics` | `src/modifiers/injury_modifier.rs` | ✓ |
| `modifiers/InjuryModifierContext.java` | `ffb-mechanics` | `src/modifiers/injury_modifier_context.rs` | ✓ |
| `modifiers/InterceptionContext.java` | `ffb-mechanics` | `src/modifiers/interception_context.rs` | ✓ |
| `modifiers/InterceptionModifier.java` | `ffb-mechanics` | `src/modifiers/interception_modifier.rs` | ✓ |
| `modifiers/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/interception_modifier_collection.rs` | ✓ |
| `modifiers/IRegistrationAwareModifier.java` | `ffb-mechanics` | `src/modifiers/i_registration_aware_modifier.rs` | ✓ |
| `modifiers/JumpContext.java` | `ffb-mechanics` | `src/modifiers/jump_context.rs` | ✓ |
| `modifiers/JumpModifier.java` | `ffb-mechanics` | `src/modifiers/jump_modifier.rs` | ✓ |
| `modifiers/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/jump_modifier_collection.rs` | ✓ |
| `modifiers/JumpUpContext.java` | `ffb-mechanics` | `src/modifiers/jump_up_context.rs` | ✓ |
| `modifiers/JumpUpModifier.java` | `ffb-mechanics` | `src/modifiers/jump_up_modifier.rs` | ✓ |
| `modifiers/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/mixed/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/dodge_modifier_collection.rs` | ✓ |
| `modifiers/mixed/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/mixed/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/jump_modifier_collection.rs` | ✓ |
| `modifiers/mixed/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/mixed/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/pass_modifier_collection.rs` | ✓ |
| `modifiers/ModifierAggregator.java` | `ffb-mechanics` | `src/modifiers/modifier_aggregator.rs` | ✓ |
| `modifiers/ModifierCollection.java` | `ffb-mechanics` | `src/modifiers/modifier_collection.rs` | ✓ |
| `modifiers/ModifierContext.java` | `ffb-mechanics` | `src/modifiers/modifier_context.rs` | ✓ |
| `modifiers/ModifierType.java` | `ffb-mechanics` | `src/modifiers/modifier_type.rs` | ✓ |
| `modifiers/PassContext.java` | `ffb-mechanics` | `src/modifiers/pass_context.rs` | ✓ |
| `modifiers/PassModifier.java` | `ffb-mechanics` | `src/modifiers/pass_modifier.rs` | ✓ |
| `modifiers/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/pass_modifier_collection.rs` | ✓ |
| `modifiers/PickupContext.java` | `ffb-mechanics` | `src/modifiers/pickup_context.rs` | ✓ |
| `modifiers/PickupModifier.java` | `ffb-mechanics` | `src/modifiers/pickup_modifier.rs` | ✓ |
| `modifiers/PickupModifierCollection.java` | `ffb-mechanics` | `src/modifiers/pickup_modifier_collection.rs` | ✓ |
| `modifiers/PlayerStatKey.java` | `ffb-mechanics` | `src/modifiers/player_stat_key.rs` | ✓ |
| `modifiers/PlayerStatLimit.java` | `ffb-mechanics` | `src/modifiers/player_stat_limit.rs` | ✓ |
| `modifiers/RegistrationAwareModifier.java` | `ffb-mechanics` | `src/modifiers/registration_aware_modifier.rs` | ✓ |
| `modifiers/RightStuffContext.java` | `ffb-mechanics` | `src/modifiers/right_stuff_context.rs` | ✓ |
| `modifiers/RightStuffModifier.java` | `ffb-mechanics` | `src/modifiers/right_stuff_modifier.rs` | ✓ |
| `modifiers/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/RollModifier.java` | `ffb-mechanics` | `src/modifiers/roll_modifier.rs` | ✓ |
| `modifiers/SpecialEffectArmourModifier.java` | `ffb-mechanics` | `src/modifiers/special_effect_armour_modifier.rs` | ✓ |
| `modifiers/SpecialEffectInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/special_effect_injury_modifier.rs` | ✓ |
| `modifiers/StatBasedRollModifier.java` | `ffb-mechanics` | `src/modifiers/stat_based_roll_modifier.rs` | ✓ |
| `modifiers/StatBasedRollModifierFactory.java` | `ffb-mechanics` | `src/modifiers/stat_based_roll_modifier_factory.rs` | ✓ |
| `modifiers/StaticArmourModifier.java` | `ffb-mechanics` | `src/modifiers/static_armour_modifier.rs` | ✓ |
| `modifiers/StaticInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier.rs` | ✓ |
| `modifiers/StaticInjuryModifierAttacker.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier_attacker.rs` | ✓ |
| `modifiers/StaticInjuryModifierDefender.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier_defender.rs` | ✓ |
| `modifiers/TemporaryEnhancements.java` | `ffb-mechanics` | `src/modifiers/temporary_enhancements.rs` | ✓ |
| `modifiers/TemporaryStatDecrementer.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_decrementer.rs` | ✓ |
| `modifiers/TemporaryStatIncrementer.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_incrementer.rs` | ✓ |
| `modifiers/TemporaryStatModifier.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_modifier.rs` | ✓ |
| `modifiers/VariableArmourModifier.java` | `ffb-mechanics` | `src/modifiers/variable_armour_modifier.rs` | ✓ |
| `modifiers/VariableInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier.rs` | ✓ |
| `modifiers/VariableInjuryModifierAttacker.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier_attacker.rs` | ✓ |
| `modifiers/VariableInjuryModifierDefender.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier_defender.rs` | ✓ |

### net/ (137 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `net/commands/ClientCommand.java` | `ffb-protocol` | `src/commands/client_command.rs` | ~ |
| `net/commands/ClientCommandActingPlayer.java` | `ffb-protocol` | `src/commands/client_command_acting_player.rs` | ~ |
| `net/commands/ClientCommandAddSketch.java` | `ffb-protocol` | `src/commands/client_command_add_sketch.rs` | ~ |
| `net/commands/ClientCommandApothecaryChoice.java` | `ffb-protocol` | `src/commands/client_command_apothecary_choice.rs` | ~ |
| `net/commands/ClientCommandArgueTheCall.java` | `ffb-protocol` | `src/commands/client_command_argue_the_call.rs` | ~ |
| `net/commands/ClientCommandBlitzMove.java` | `ffb-protocol` | `src/commands/client_command_blitz_move.rs` | ~ |
| `net/commands/ClientCommandBlock.java` | `ffb-protocol` | `src/commands/client_command_block.rs` | ~ |
| `net/commands/ClientCommandBlockChoice.java` | `ffb-protocol` | `src/commands/client_command_block_choice.rs` | ~ |
| `net/commands/ClientCommandBlockOrReRollChoiceForTarget.java` | `ffb-protocol` | `src/commands/client_command_block_or_re_roll_choice_for_target.rs` | ~ |
| `net/commands/ClientCommandBloodlustAction.java` | `ffb-protocol` | `src/commands/client_command_bloodlust_action.rs` | ~ |
| `net/commands/ClientCommandBuyCard.java` | `ffb-protocol` | `src/commands/client_command_buy_card.rs` | ~ |
| `net/commands/ClientCommandBuyInducements.java` | `ffb-protocol` | `src/commands/client_command_buy_inducements.rs` | ~ |
| `net/commands/ClientCommandClearSketches.java` | `ffb-protocol` | `src/commands/client_command_clear_sketches.rs` | ~ |
| `net/commands/ClientCommandCloseSession.java` | `ffb-protocol` | `src/commands/client_command_close_session.rs` | ~ |
| `net/commands/ClientCommandCoinChoice.java` | `ffb-protocol` | `src/commands/client_command_coin_choice.rs` | ~ |
| `net/commands/ClientCommandConcedeGame.java` | `ffb-protocol` | `src/commands/client_command_concede_game.rs` | ~ |
| `net/commands/ClientCommandConfirm.java` | `ffb-protocol` | `src/commands/client_command_confirm.rs` | ~ |
| `net/commands/ClientCommandDebugClientState.java` | `ffb-protocol` | `src/commands/client_command_debug_client_state.rs` | ~ |
| `net/commands/ClientCommandEndTurn.java` | `ffb-protocol` | `src/commands/client_command_end_turn.rs` | ~ |
| `net/commands/ClientCommandFieldCoordinate.java` | `ffb-protocol` | `src/commands/client_command_field_coordinate.rs` | ~ |
| `net/commands/ClientCommandFollowupChoice.java` | `ffb-protocol` | `src/commands/client_command_followup_choice.rs` | ~ |
| `net/commands/ClientCommandFoul.java` | `ffb-protocol` | `src/commands/client_command_foul.rs` | ~ |
| `net/commands/ClientCommandGaze.java` | `ffb-protocol` | `src/commands/client_command_gaze.rs` | ~ |
| `net/commands/ClientCommandHandOver.java` | `ffb-protocol` | `src/commands/client_command_hand_over.rs` | ~ |
| `net/commands/ClientCommandIllegalProcedure.java` | `ffb-protocol` | `src/commands/client_command_illegal_procedure.rs` | ~ |
| `net/commands/ClientCommandInterceptorChoice.java` | `ffb-protocol` | `src/commands/client_command_interceptor_choice.rs` | ~ |
| `net/commands/ClientCommandJoin.java` | `ffb-protocol` | `src/commands/client_command_join.rs` | ~ |
| `net/commands/ClientCommandJoinReplay.java` | `ffb-protocol` | `src/commands/client_command_join_replay.rs` | ~ |
| `net/commands/ClientCommandJourneymen.java` | `ffb-protocol` | `src/commands/client_command_journeymen.rs` | ~ |
| `net/commands/ClientCommandKeywordSelection.java` | `ffb-protocol` | `src/commands/client_command_keyword_selection.rs` | ~ |
| `net/commands/ClientCommandKickoff.java` | `ffb-protocol` | `src/commands/client_command_kickoff.rs` | ~ |
| `net/commands/ClientCommandKickOffResultChoice.java` | `ffb-protocol` | `src/commands/client_command_kick_off_result_choice.rs` | ~ |
| `net/commands/ClientCommandKickTeamMate.java` | `ffb-protocol` | `src/commands/client_command_kick_team_mate.rs` | ~ |
| `net/commands/ClientCommandLoadAutomaticPlayerMarkings.java` | `ffb-protocol` | `src/commands/client_command_load_automatic_player_markings.rs` | ~ |
| `net/commands/ClientCommandMove.java` | `ffb-protocol` | `src/commands/client_command_move.rs` | ~ |
| `net/commands/ClientCommandPass.java` | `ffb-protocol` | `src/commands/client_command_pass.rs` | ~ |
| `net/commands/ClientCommandPasswordChallenge.java` | `ffb-protocol` | `src/commands/client_command_password_challenge.rs` | ~ |
| `net/commands/ClientCommandPettyCash.java` | `ffb-protocol` | `src/commands/client_command_petty_cash.rs` | ~ |
| `net/commands/ClientCommandPickUpChoice.java` | `ffb-protocol` | `src/commands/client_command_pick_up_choice.rs` | ~ |
| `net/commands/ClientCommandPileDriver.java` | `ffb-protocol` | `src/commands/client_command_pile_driver.rs` | ~ |
| `net/commands/ClientCommandPing.java` | `ffb-protocol` | `src/commands/client_command_ping.rs` | ~ |
| `net/commands/ClientCommandPlayerChoice.java` | `ffb-protocol` | `src/commands/client_command_player_choice.rs` | ~ |
| `net/commands/ClientCommandPositionSelection.java` | `ffb-protocol` | `src/commands/client_command_position_selection.rs` | ~ |
| `net/commands/ClientCommandPuntToCrowd.java` | `ffb-protocol` | `src/commands/client_command_punt_to_crowd.rs` | ~ |
| `net/commands/ClientCommandPushback.java` | `ffb-protocol` | `src/commands/client_command_pushback.rs` | ~ |
| `net/commands/ClientCommandReceiveChoice.java` | `ffb-protocol` | `src/commands/client_command_receive_choice.rs` | ~ |
| `net/commands/ClientCommandRemoveSketches.java` | `ffb-protocol` | `src/commands/client_command_remove_sketches.rs` | ~ |
| `net/commands/ClientCommandReplay.java` | `ffb-protocol` | `src/commands/client_command_replay.rs` | ~ |
| `net/commands/ClientCommandReplayStatus.java` | `ffb-protocol` | `src/commands/client_command_replay_status.rs` | ~ |
| `net/commands/ClientCommandRequestVersion.java` | `ffb-protocol` | `src/commands/client_command_request_version.rs` | ~ |
| `net/commands/ClientCommandSelectCardToBuy.java` | `ffb-protocol` | `src/commands/client_command_select_card_to_buy.rs` | ~ |
| `net/commands/ClientCommandSelectWeather.java` | `ffb-protocol` | `src/commands/client_command_select_weather.rs` | ~ |
| `net/commands/ClientCommandSetBlockTargetSelection.java` | `ffb-protocol` | `src/commands/client_command_set_block_target_selection.rs` | ~ |
| `net/commands/ClientCommandSetMarker.java` | `ffb-protocol` | `src/commands/client_command_set_marker.rs` | ~ |
| `net/commands/ClientCommandSetPreventSketching.java` | `ffb-protocol` | `src/commands/client_command_set_prevent_sketching.rs` | ~ |
| `net/commands/ClientCommandSetupPlayer.java` | `ffb-protocol` | `src/commands/client_command_setup_player.rs` | ~ |
| `net/commands/ClientCommandSketchAddCoordinate.java` | `ffb-protocol` | `src/commands/client_command_sketch_add_coordinate.rs` | ~ |
| `net/commands/ClientCommandSketchSetColor.java` | `ffb-protocol` | `src/commands/client_command_sketch_set_color.rs` | ~ |
| `net/commands/ClientCommandSketchSetLabel.java` | `ffb-protocol` | `src/commands/client_command_sketch_set_label.rs` | ~ |
| `net/commands/ClientCommandSkillSelection.java` | `ffb-protocol` | `src/commands/client_command_skill_selection.rs` | ~ |
| `net/commands/ClientCommandStartGame.java` | `ffb-protocol` | `src/commands/client_command_start_game.rs` | ~ |
| `net/commands/ClientCommandSwoop.java` | `ffb-protocol` | `src/commands/client_command_swoop.rs` | ~ |
| `net/commands/ClientCommandSynchronousMultiBlock.java` | `ffb-protocol` | `src/commands/client_command_synchronous_multi_block.rs` | ~ |
| `net/commands/ClientCommandTalk.java` | `ffb-protocol` | `src/commands/client_command_talk.rs` | ~ |
| `net/commands/ClientCommandTargetSelected.java` | `ffb-protocol` | `src/commands/client_command_target_selected.rs` | ~ |
| `net/commands/ClientCommandTeamSetupDelete.java` | `ffb-protocol` | `src/commands/client_command_team_setup_delete.rs` | ~ |
| `net/commands/ClientCommandTeamSetupLoad.java` | `ffb-protocol` | `src/commands/client_command_team_setup_load.rs` | ~ |
| `net/commands/ClientCommandTeamSetupSave.java` | `ffb-protocol` | `src/commands/client_command_team_setup_save.rs` | ~ |
| `net/commands/ClientCommandThrowKeg.java` | `ffb-protocol` | `src/commands/client_command_throw_keg.rs` | ~ |
| `net/commands/ClientCommandThrowTeamMate.java` | `ffb-protocol` | `src/commands/client_command_throw_team_mate.rs` | ~ |
| `net/commands/ClientCommandTouchback.java` | `ffb-protocol` | `src/commands/client_command_touchback.rs` | ~ |
| `net/commands/ClientCommandTransferReplayControl.java` | `ffb-protocol` | `src/commands/client_command_transfer_replay_control.rs` | ~ |
| `net/commands/ClientCommandUnsetBlockTargetSelection.java` | `ffb-protocol` | `src/commands/client_command_unset_block_target_selection.rs` | ~ |
| `net/commands/ClientCommandUpdatePlayerMarkings.java` | `ffb-protocol` | `src/commands/client_command_update_player_markings.rs` | ~ |
| `net/commands/ClientCommandUseApothecaries.java` | `ffb-protocol` | `src/commands/client_command_use_apothecaries.rs` | ~ |
| `net/commands/ClientCommandUseApothecary.java` | `ffb-protocol` | `src/commands/client_command_use_apothecary.rs` | ~ |
| `net/commands/ClientCommandUseBrawler.java` | `ffb-protocol` | `src/commands/client_command_use_brawler.rs` | ~ |
| `net/commands/ClientCommandUseChainsaw.java` | `ffb-protocol` | `src/commands/client_command_use_chainsaw.rs` | ~ |
| `net/commands/ClientCommandUseConsummateReRollForBlock.java` | `ffb-protocol` | `src/commands/client_command_use_consummate_re_roll_for_block.rs` | ~ |
| `net/commands/ClientCommandUseFumblerooskie.java` | `ffb-protocol` | `src/commands/client_command_use_fumblerooskie.rs` | ~ |
| `net/commands/ClientCommandUseHatred.java` | `ffb-protocol` | `src/commands/client_command_use_hatred.rs` | ~ |
| `net/commands/ClientCommandUseIgors.java` | `ffb-protocol` | `src/commands/client_command_use_igors.rs` | ~ |
| `net/commands/ClientCommandUseInducement.java` | `ffb-protocol` | `src/commands/client_command_use_inducement.rs` | ~ |
| `net/commands/ClientCommandUseMultiBlockDiceReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_multi_block_dice_re_roll.rs` | ~ |
| `net/commands/ClientCommandUseProReRollForBlock.java` | `ffb-protocol` | `src/commands/client_command_use_pro_re_roll_for_block.rs` | ~ |
| `net/commands/ClientCommandUseReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_re_roll.rs` | ~ |
| `net/commands/ClientCommandUseReRollForTarget.java` | `ffb-protocol` | `src/commands/client_command_use_re_roll_for_target.rs` | ~ |
| `net/commands/ClientCommandUserSettings.java` | `ffb-protocol` | `src/commands/client_command_user_settings.rs` | ~ |
| `net/commands/ClientCommandUseSingleBlockDieReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_single_block_die_re_roll.rs` | ~ |
| `net/commands/ClientCommandUseSkill.java` | `ffb-protocol` | `src/commands/client_command_use_skill.rs` | ~ |
| `net/commands/ClientCommandUseTeamMatesWisdom.java` | `ffb-protocol` | `src/commands/client_command_use_team_mates_wisdom.rs` | ~ |
| `net/commands/ClientCommandWizardSpell.java` | `ffb-protocol` | `src/commands/client_command_wizard_spell.rs` | ~ |
| `net/commands/ClientSketchCommand.java` | `ffb-protocol` | `src/commands/client_sketch_command.rs` | ~ |
| `net/commands/ICommandWithActingPlayer.java` | `ffb-protocol` | `src/commands/i_command_with_acting_player.rs` | ~ |
| `net/commands/ServerCommand.java` | `ffb-protocol` | `src/commands/server_command.rs` | ~ |
| `net/commands/ServerCommandAddPlayer.java` | `ffb-protocol` | `src/commands/server_command_add_player.rs` | ~ |
| `net/commands/ServerCommandAddSketches.java` | `ffb-protocol` | `src/commands/server_command_add_sketches.rs` | ~ |
| `net/commands/ServerCommandAdminMessage.java` | `ffb-protocol` | `src/commands/server_command_admin_message.rs` | ~ |
| `net/commands/ServerCommandAutomaticPlayerMarkings.java` | `ffb-protocol` | `src/commands/server_command_automatic_player_markings.rs` | ~ |
| `net/commands/ServerCommandClearSketches.java` | `ffb-protocol` | `src/commands/server_command_clear_sketches.rs` | ~ |
| `net/commands/ServerCommandGameList.java` | `ffb-protocol` | `src/commands/server_command_game_list.rs` | ~ |
| `net/commands/ServerCommandGameState.java` | `ffb-protocol` | `src/commands/server_command_game_state.rs` | ~ |
| `net/commands/ServerCommandGameTime.java` | `ffb-protocol` | `src/commands/server_command_game_time.rs` | ~ |
| `net/commands/ServerCommandJoin.java` | `ffb-protocol` | `src/commands/server_command_join.rs` | ~ |
| `net/commands/ServerCommandLeave.java` | `ffb-protocol` | `src/commands/server_command_leave.rs` | ~ |
| `net/commands/ServerCommandModelSync.java` | `ffb-protocol` | `src/commands/server_command_model_sync.rs` | ~ |
| `net/commands/ServerCommandPasswordChallenge.java` | `ffb-protocol` | `src/commands/server_command_password_challenge.rs` | ~ |
| `net/commands/ServerCommandPong.java` | `ffb-protocol` | `src/commands/server_command_pong.rs` | ~ |
| `net/commands/ServerCommandRemovePlayer.java` | `ffb-protocol` | `src/commands/server_command_remove_player.rs` | ~ |
| `net/commands/ServerCommandRemoveSketches.java` | `ffb-protocol` | `src/commands/server_command_remove_sketches.rs` | ~ |
| `net/commands/ServerCommandReplay.java` | `ffb-protocol` | `src/commands/server_command_replay.rs` | ~ |
| `net/commands/ServerCommandReplayControl.java` | `ffb-protocol` | `src/commands/server_command_replay_control.rs` | ~ |
| `net/commands/ServerCommandReplayStatus.java` | `ffb-protocol` | `src/commands/server_command_replay_status.rs` | ~ |
| `net/commands/ServerCommandSetPreventSketching.java` | `ffb-protocol` | `src/commands/server_command_set_prevent_sketching.rs` | ~ |
| `net/commands/ServerCommandSketchAddCoordinate.java` | `ffb-protocol` | `src/commands/server_command_sketch_add_coordinate.rs` | ~ |
| `net/commands/ServerCommandSketchSetColor.java` | `ffb-protocol` | `src/commands/server_command_sketch_set_color.rs` | ~ |
| `net/commands/ServerCommandSketchSetLabel.java` | `ffb-protocol` | `src/commands/server_command_sketch_set_label.rs` | ~ |
| `net/commands/ServerCommandSound.java` | `ffb-protocol` | `src/commands/server_command_sound.rs` | ~ |
| `net/commands/ServerCommandStatus.java` | `ffb-protocol` | `src/commands/server_command_status.rs` | ~ |
| `net/commands/ServerCommandTalk.java` | `ffb-protocol` | `src/commands/server_command_talk.rs` | ~ |
| `net/commands/ServerCommandTeamList.java` | `ffb-protocol` | `src/commands/server_command_team_list.rs` | ~ |
| `net/commands/ServerCommandTeamSetupList.java` | `ffb-protocol` | `src/commands/server_command_team_setup_list.rs` | ~ |
| `net/commands/ServerCommandUnzapPlayer.java` | `ffb-protocol` | `src/commands/server_command_unzap_player.rs` | ~ |
| `net/commands/ServerCommandUpdateLocalPlayerMarkers.java` | `ffb-protocol` | `src/commands/server_command_update_local_player_markers.rs` | ~ |
| `net/commands/ServerCommandUserSettings.java` | `ffb-protocol` | `src/commands/server_command_user_settings.rs` | ~ |
| `net/commands/ServerCommandVersion.java` | `ffb-protocol` | `src/commands/server_command_version.rs` | ~ |
| `net/commands/ServerCommandZapPlayer.java` | `ffb-protocol` | `src/commands/server_command_zap_player.rs` | ~ |
| `net/commands/UtilNetCommand.java` | `ffb-protocol` | `src/commands/util_net_command.rs` | ~ |
| `net/GameCoach.java` | `ffb-protocol` | `src/game_coach.rs` | ~ |
| `net/IConnectionListener.java` | `ffb-protocol` | `src/i_connection_listener.rs` | ~ |
| `net/INetCommandHandler.java` | `ffb-protocol` | `src/i_net_command_handler.rs` | ~ |
| `net/NetCommand.java` | `ffb-protocol` | `src/net_command.rs` | ~ |
| `net/NetCommandFactory.java` | `ffb-protocol` | `src/net_command_factory.rs` | ~ |
| `net/NetCommandId.java` | `ffb-protocol` | `src/net_command_id.rs` | ~ |
| `net/NetCommandLog.java` | `ffb-protocol` | `src/net_command_log.rs` | ~ |
| `net/ServerStatus.java` | `ffb-protocol` | `src/server_status.rs` | ~ |
| `net/SocketChangeRequest.java` | `ffb-protocol` | `src/socket_change_request.rs` | ~ |

### option/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `option/GameOptionAbstract.java` | `ffb-model` | `src/option/game_option_abstract.rs` | ✓ |
| `option/GameOptionBoolean.java` | `ffb-model` | `src/option/game_option_boolean.rs` | ✓ |
| `option/GameOptionId.java` | `ffb-model` | `src/option/game_option_id.rs` | ✓ |
| `option/GameOptionInt.java` | `ffb-model` | `src/option/game_option_int.rs` | ✓ |
| `option/GameOptionString.java` | `ffb-model` | `src/option/game_option_string.rs` | ✓ |
| `option/IGameOption.java` | `ffb-model` | `src/option/i_game_option.rs` | ✓ |
| `option/UtilGameOption.java` | `ffb-model` | `src/option/util_game_option.rs` | ✓ |

### report/ (191 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `report/bb2016/ReportApothecaryRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportArgueTheCallRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportCardsBought.java` | `—` | `—` | — |
| `report/bb2016/ReportDodgeRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportFanFactorRollPostMatch.java` | `—` | `—` | — |
| `report/bb2016/ReportHypnoticGazeRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportInducementsBought.java` | `—` | `—` | — |
| `report/bb2016/ReportInjury.java` | `—` | `—` | — |
| `report/bb2016/ReportKickoffExtraReRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportKickoffPitchInvasion.java` | `—` | `—` | — |
| `report/bb2016/ReportKickoffRiot.java` | `—` | `—` | — |
| `report/bb2016/ReportKickoffThrowARock.java` | `—` | `—` | — |
| `report/bb2016/ReportKickTeamMateRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportNervesOfSteel.java` | `—` | `—` | — |
| `report/bb2016/ReportNoPlayersToField.java` | `—` | `—` | — |
| `report/bb2016/ReportPassRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportPenaltyShootout.java` | `—` | `—` | — |
| `report/bb2016/ReportReferee.java` | `—` | `—` | — |
| `report/bb2016/ReportSpectators.java` | `—` | `—` | — |
| `report/bb2016/ReportSwoopPlayer.java` | `—` | `—` | — |
| `report/bb2016/ReportTentaclesShadowingRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportThrowTeamMateRoll.java` | `—` | `—` | — |
| `report/bb2016/ReportTurnEnd.java` | `—` | `—` | — |
| `report/bb2016/ReportWinningsRoll.java` | `—` | `—` | — |
| `report/bb2020/ReportCardsAndInducementsBought.java` | `—` | `—` | — |
| `report/bb2020/ReportCheeringFans.java` | `—` | `—` | — |
| `report/bb2020/ReportKickoffOfficiousRef.java` | `—` | `—` | — |
| `report/bb2020/ReportOfficiousRefRoll.java` | `—` | `—` | — |
| `report/bb2020/ReportPrayerRoll.java` | `—` | `—` | — |
| `report/bb2020/ReportSkillUseOtherPlayer.java` | `—` | `—` | — |
| `report/bb2020/ReportSwoopPlayer.java` | `—` | `—` | — |
| `report/bb2020/ReportTwoForOne.java` | `—` | `—` | — |
| `report/bb2025/ReportCheeringFans.java` | `—` | `—` | — |
| `report/bb2025/ReportChompRemoved.java` | `—` | `—` | — |
| `report/bb2025/ReportChompRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportDodgySnackRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportGettingEvenRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportKickoffDodgySnack.java` | `—` | `—` | — |
| `report/bb2025/ReportMascotUsed.java` | `—` | `—` | — |
| `report/bb2025/ReportPickupRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportPrayerRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportPrayersAndInducementsBought.java` | `—` | `—` | — |
| `report/bb2025/ReportPuntDirection.java` | `—` | `—` | — |
| `report/bb2025/ReportPuntDistance.java` | `—` | `—` | — |
| `report/bb2025/ReportSaboteurRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportSteadyFootingRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportSwarmingRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportSwoopDirection.java` | `—` | `—` | — |
| `report/bb2025/ReportSwoopPlayer.java` | `—` | `—` | — |
| `report/bb2025/ReportTeamCaptainRoll.java` | `—` | `—` | — |
| `report/bb2025/ReportTeamEvent.java` | `—` | `—` | — |
| `report/bb2025/ReportThrowAtPlayer.java` | `—` | `—` | — |
| `report/IReport.java` | `—` | `—` | — |
| `report/logcontrol/SkipInjuryParts.java` | `—` | `—` | — |
| `report/mixed/ReportAllYouCanEatRoll.java` | `—` | `—` | — |
| `report/mixed/ReportAnimalSavagery.java` | `—` | `—` | — |
| `report/mixed/ReportApothecaryRoll.java` | `—` | `—` | — |
| `report/mixed/ReportArgueTheCallRoll.java` | `—` | `—` | — |
| `report/mixed/ReportBalefulHexRoll.java` | `—` | `—` | — |
| `report/mixed/ReportBiasedRef.java` | `—` | `—` | — |
| `report/mixed/ReportBlitzRoll.java` | `—` | `—` | — |
| `report/mixed/ReportBlockReRoll.java` | `—` | `—` | — |
| `report/mixed/ReportBreatheFire.java` | `—` | `—` | — |
| `report/mixed/ReportBriberyAndCorruptionReRoll.java` | `—` | `—` | — |
| `report/mixed/ReportBrilliantCoachingReRollsLost.java` | `—` | `—` | — |
| `report/mixed/ReportCatchOfTheDayRoll.java` | `—` | `—` | — |
| `report/mixed/ReportCloudBurster.java` | `—` | `—` | — |
| `report/mixed/ReportDedicatedFans.java` | `—` | `—` | — |
| `report/mixed/ReportDodgeRoll.java` | `—` | `—` | — |
| `report/mixed/ReportDoubleHiredStaff.java` | `—` | `—` | — |
| `report/mixed/ReportEvent.java` | `—` | `—` | — |
| `report/mixed/ReportFanFactor.java` | `—` | `—` | — |
| `report/mixed/ReportFreePettyCash.java` | `—` | `—` | — |
| `report/mixed/ReportFumblerooskie.java` | `—` | `—` | — |
| `report/mixed/ReportHitAndRun.java` | `—` | `—` | — |
| `report/mixed/ReportHypnoticGazeRoll.java` | `—` | `—` | — |
| `report/mixed/ReportIndomitable.java` | `—` | `—` | — |
| `report/mixed/ReportInjury.java` | `—` | `—` | — |
| `report/mixed/ReportKickoffExtraReRoll.java` | `—` | `—` | — |
| `report/mixed/ReportKickoffPitchInvasion.java` | `—` | `—` | — |
| `report/mixed/ReportKickoffSequenceActivationsCount.java` | `—` | `—` | — |
| `report/mixed/ReportKickoffSequenceActivationsExhausted.java` | `—` | `—` | — |
| `report/mixed/ReportKickoffTimeout.java` | `—` | `—` | — |
| `report/mixed/ReportKickTeamMateFumble.java` | `—` | `—` | — |
| `report/mixed/ReportLookIntoMyEyesRoll.java` | `—` | `—` | — |
| `report/mixed/ReportModifiedDodgeResultSuccessful.java` | `—` | `—` | — |
| `report/mixed/ReportModifiedPassResult.java` | `—` | `—` | — |
| `report/mixed/ReportNervesOfSteel.java` | `—` | `—` | — |
| `report/mixed/ReportOldPro.java` | `—` | `—` | — |
| `report/mixed/ReportPassRoll.java` | `—` | `—` | — |
| `report/mixed/ReportPenaltyShootout.java` | `—` | `—` | — |
| `report/mixed/ReportPickMeUp.java` | `—` | `—` | — |
| `report/mixed/ReportPickupRoll.java` | `—` | `—` | — |
| `report/mixed/ReportPlaceBallDirection.java` | `—` | `—` | — |
| `report/mixed/ReportPlayerEvent.java` | `—` | `—` | — |
| `report/mixed/ReportPrayerAmount.java` | `—` | `—` | — |
| `report/mixed/ReportPrayerEnd.java` | `—` | `—` | — |
| `report/mixed/ReportPrayerWasted.java` | `—` | `—` | — |
| `report/mixed/ReportProjectileVomit.java` | `—` | `—` | — |
| `report/mixed/ReportPumpUpTheCrowdReRoll.java` | `—` | `—` | — |
| `report/mixed/ReportPumpUpTheCrowdReRollsLost.java` | `—` | `—` | — |
| `report/mixed/ReportQuickSnapRoll.java` | `—` | `—` | — |
| `report/mixed/ReportRaidingParty.java` | `—` | `—` | — |
| `report/mixed/ReportReferee.java` | `—` | `—` | — |
| `report/mixed/ReportSelectBlitzTarget.java` | `—` | `—` | — |
| `report/mixed/ReportSelectGazeTarget.java` | `—` | `—` | — |
| `report/mixed/ReportShowStarReRoll.java` | `—` | `—` | — |
| `report/mixed/ReportShowStarReRollsLost.java` | `—` | `—` | — |
| `report/mixed/ReportSkillWasted.java` | `—` | `—` | — |
| `report/mixed/ReportSolidDefenceRoll.java` | `—` | `—` | — |
| `report/mixed/ReportStallerDetected.java` | `—` | `—` | — |
| `report/mixed/ReportSwarmingRoll.java` | `—` | `—` | — |
| `report/mixed/ReportTentaclesShadowingRoll.java` | `—` | `—` | — |
| `report/mixed/ReportThenIStartedBlastin.java` | `—` | `—` | — |
| `report/mixed/ReportThrowAtStallingPlayer.java` | `—` | `—` | — |
| `report/mixed/ReportThrownKeg.java` | `—` | `—` | — |
| `report/mixed/ReportThrowTeamMateRoll.java` | `—` | `—` | — |
| `report/mixed/ReportTrapDoor.java` | `—` | `—` | — |
| `report/mixed/ReportTurnEnd.java` | `—` | `—` | — |
| `report/mixed/ReportWeatherMageResult.java` | `—` | `—` | — |
| `report/mixed/ReportWeatherMageRoll.java` | `—` | `—` | — |
| `report/mixed/ReportWinnings.java` | `—` | `—` | — |
| `report/NoDiceReport.java` | `—` | `—` | — |
| `report/ReportAlwaysHungryRoll.java` | `—` | `—` | — |
| `report/ReportAnimosityRoll.java` | `—` | `—` | — |
| `report/ReportApothecaryChoice.java` | `—` | `—` | — |
| `report/ReportBiteSpectator.java` | `—` | `—` | — |
| `report/ReportBlock.java` | `—` | `—` | — |
| `report/ReportBlockChoice.java` | `—` | `—` | — |
| `report/ReportBlockRoll.java` | `—` | `—` | — |
| `report/ReportBloodLustRoll.java` | `—` | `—` | — |
| `report/ReportBombExplodesAfterCatch.java` | `—` | `—` | — |
| `report/ReportBombOutOfBounds.java` | `—` | `—` | — |
| `report/ReportBribesRoll.java` | `—` | `—` | — |
| `report/ReportCardDeactivated.java` | `—` | `—` | — |
| `report/ReportCardEffectRoll.java` | `—` | `—` | — |
| `report/ReportCatchRoll.java` | `—` | `—` | — |
| `report/ReportChainsawRoll.java` | `—` | `—` | — |
| `report/ReportCoinThrow.java` | `—` | `—` | — |
| `report/ReportConfusionRoll.java` | `—` | `—` | — |
| `report/ReportDauntlessRoll.java` | `—` | `—` | — |
| `report/ReportDefectingPlayers.java` | `—` | `—` | — |
| `report/ReportDoubleHiredStarPlayer.java` | `—` | `—` | — |
| `report/ReportEscapeRoll.java` | `—` | `—` | — |
| `report/ReportFoul.java` | `—` | `—` | — |
| `report/ReportFoulAppearanceRoll.java` | `—` | `—` | — |
| `report/ReportFumbblResultUpload.java` | `—` | `—` | — |
| `report/ReportGameOptions.java` | `—` | `—` | — |
| `report/ReportGoForItRoll.java` | `—` | `—` | — |
| `report/ReportHandOver.java` | `—` | `—` | — |
| `report/ReportId.java` | `—` | `—` | — |
| `report/ReportInducement.java` | `—` | `—` | — |
| `report/ReportInjury.java` | `—` | `—` | — |
| `report/ReportInterceptionRoll.java` | `—` | `—` | — |
| `report/ReportJumpRoll.java` | `—` | `—` | — |
| `report/ReportJumpUpRoll.java` | `—` | `—` | — |
| `report/ReportKickoffResult.java` | `—` | `—` | — |
| `report/ReportKickoffScatter.java` | `—` | `—` | — |
| `report/ReportLeader.java` | `—` | `—` | — |
| `report/ReportList.java` | `—` | `—` | — |
| `report/ReportMasterChefRoll.java` | `—` | `—` | — |
| `report/ReportMostValuablePlayers.java` | `—` | `—` | — |
| `report/ReportPassBlock.java` | `—` | `—` | — |
| `report/ReportPassDeviate.java` | `—` | `—` | — |
| `report/ReportPettyCash.java` | `—` | `—` | — |
| `report/ReportPickupRoll.java` | `—` | `—` | — |
| `report/ReportPilingOn.java` | `—` | `—` | — |
| `report/ReportPlayCard.java` | `—` | `—` | — |
| `report/ReportPlayerAction.java` | `—` | `—` | — |
| `report/ReportPushback.java` | `—` | `—` | — |
| `report/ReportRaiseDead.java` | `—` | `—` | — |
| `report/ReportReceiveChoice.java` | `—` | `—` | — |
| `report/ReportRegenerationRoll.java` | `—` | `—` | — |
| `report/ReportReRoll.java` | `—` | `—` | — |
| `report/ReportRightStuffRoll.java` | `—` | `—` | — |
| `report/ReportRiotousRookies.java` | `—` | `—` | — |
| `report/ReportSafeThrowRoll.java` | `—` | `—` | — |
| `report/ReportScatterBall.java` | `—` | `—` | — |
| `report/ReportScatterPlayer.java` | `—` | `—` | — |
| `report/ReportSecretWeaponBan.java` | `—` | `—` | — |
| `report/ReportSkillRoll.java` | `—` | `—` | — |
| `report/ReportSkillUse.java` | `—` | `—` | — |
| `report/ReportSpecialEffectRoll.java` | `—` | `—` | — |
| `report/ReportStandUpRoll.java` | `—` | `—` | — |
| `report/ReportStartHalf.java` | `—` | `—` | — |
| `report/ReportThrowIn.java` | `—` | `—` | — |
| `report/ReportTimeoutEnforced.java` | `—` | `—` | — |
| `report/ReportWeather.java` | `—` | `—` | — |
| `report/ReportWeepingDaggerRoll.java` | `—` | `—` | — |
| `report/ReportWizardUse.java` | `—` | `—` | — |
| `report/UtilReport.java` | `—` | `—` | — |

### root/ (86 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `ApothecaryMode.java` | `ffb-model` | `src/model/apothecary_mode.rs` | ✓ |
| `ApothecaryStatus.java` | `ffb-model` | `src/model/apothecary_status.rs` | ✓ |
| `ApothecaryType.java` | `ffb-model` | `src/model/apothecary_type.rs` | ✓ |
| `BlockDiceCategory.java` | `ffb-model` | `src/model/block_dice_category.rs` | ✓ |
| `BlockResult.java` | `ffb-model` | `src/model/block_result.rs` | ✓ |
| `BloodSpot.java` | `ffb-model` | `src/model/blood_spot.rs` | ✓ |
| `BoxType.java` | `ffb-model` | `src/model/box_type.rs` | ✓ |
| `BreatheFireResult.java` | `ffb-model` | `src/model/breathe_fire_result.rs` | ✓ |
| `CardEffect.java` | `ffb-model` | `src/model/card_effect.rs` | ✓ |
| `CardTarget.java` | `ffb-model` | `src/model/card_target.rs` | ✓ |
| `CatchScatterThrowInMode.java` | `ffb-model` | `src/model/catch_scatter_throw_in_mode.rs` | ✓ |
| `ChatCommand.java` | `ffb-model` | `src/model/chat_command.rs` | ✓ |
| `ClientMode.java` | `ffb-model` | `src/model/client_mode.rs` | ✓ |
| `ClientStateId.java` | `ffb-model` | `src/model/client_state_id.rs` | ✓ |
| `CommonProperty.java` | `ffb-model` | `src/model/common_property.rs` | ✓ |
| `CommonPropertyValue.java` | `ffb-model` | `src/model/common_property_value.rs` | ✓ |
| `ConcedeGameStatus.java` | `ffb-model` | `src/model/concede_game_status.rs` | ✓ |
| `Constant.java` | `ffb-model` | `src/model/constant.rs` | ✓ |
| `DefenderAction.java` | `ffb-model` | `src/model/defender_action.rs` | ✓ |
| `DiceCategory.java` | `ffb-model` | `src/model/dice_category.rs` | ✓ |
| `DiceCategoryFactory.java` | `ffb-model` | `src/model/dice_category_factory.rs` | ✓ |
| `DiceDecoration.java` | `ffb-model` | `src/model/dice_decoration.rs` | ✓ |
| `Direction.java` | `ffb-model` | `src/model/direction.rs` | ✓ |
| `DirectionDiceCategory.java` | `ffb-model` | `src/model/direction_dice_category.rs` | ✓ |
| `FactoryManager.java` | `ffb-model` | `src/model/factory_manager.rs` | ✓ |
| `FactoryType.java` | `ffb-model` | `src/model/factory_type.rs` | ✓ |
| `FantasyFootballConstants.java` | `ffb-model` | `src/model/fantasy_football_constants.rs` | ✓ |
| `FantasyFootballException.java` | `ffb-model` | `src/model/fantasy_football_exception.rs` | ✓ |
| `FieldCoordinate.java` | `ffb-model` | `src/model/field_coordinate.rs` | ✓ |
| `FieldCoordinateBounds.java` | `ffb-model` | `src/model/field_coordinate_bounds.rs` | ✓ |
| `FieldModelChangeEvent.java` | `ffb-model` | `src/model/field_model_change_event.rs` | ✓ |
| `GameList.java` | `ffb-model` | `src/model/game_list.rs` | ✓ |
| `GameListEntry.java` | `ffb-model` | `src/model/game_list_entry.rs` | ✓ |
| `GameStatus.java` | `ffb-model` | `src/model/game_status.rs` | ✓ |
| `HasReRollProperties.java` | `ffb-model` | `src/model/has_re_roll_properties.rs` | ✓ |
| `HeatExhaustion.java` | `ffb-model` | `src/model/heat_exhaustion.rs` | ✓ |
| `IClientProperty.java` | `ffb-model` | `src/model/i_client_property.rs` | ✓ |
| `IClientPropertyValue.java` | `ffb-model` | `src/model/i_client_property_value.rs` | ✓ |
| `IDialogParameter.java` | `ffb-model` | `src/model/i_dialog_parameter.rs` | ✓ |
| `IFieldModelChangeListener.java` | `ffb-model` | `src/model/i_field_model_change_listener.rs` | ✓ |
| `IIconProperty.java` | `ffb-model` | `src/model/i_icon_property.rs` | ✓ |
| `IKeyedItem.java` | `ffb-model` | `src/model/i_keyed_item.rs` | ✓ |
| `IKickOffResult.java` | `ffb-model` | `src/model/i_kick_off_result.rs` | ✓ |
| `INamedObject.java` | `ffb-model` | `src/model/i_named_object.rs` | ✓ |
| `InjuryAttribute.java` | `ffb-model` | `src/model/injury_attribute.rs` | ✓ |
| `KeyedItemRegistry.java` | `ffb-model` | `src/model/keyed_item_registry.rs` | ✓ |
| `KeywordChoiceMode.java` | `ffb-model` | `src/model/keyword_choice_mode.rs` | ✓ |
| `KnockoutRecovery.java` | `ffb-model` | `src/model/knockout_recovery.rs` | ✓ |
| `LeaderState.java` | `ffb-model` | `src/model/leader_state.rs` | ✓ |
| `MoveSquare.java` | `ffb-model` | `src/model/move_square.rs` | ✓ |
| `Pair.java` | `ffb-model` | `src/model/pair.rs` | ✓ |
| `PassingDistance.java` | `ffb-model` | `src/model/passing_distance.rs` | ✓ |
| `PasswordChallenge.java` | `ffb-model` | `src/model/password_challenge.rs` | ✓ |
| `PlayerAction.java` | `ffb-model` | `src/model/player_action.rs` | ✓ |
| `PlayerChoiceMode.java` | `ffb-model` | `src/model/player_choice_mode.rs` | ✓ |
| `PlayerGender.java` | `ffb-model` | `src/model/player_gender.rs` | ✓ |
| `PlayerState.java` | `ffb-model` | `src/model/player_state.rs` | ✓ |
| `PlayerType.java` | `ffb-model` | `src/model/player_type.rs` | ✓ |
| `PositionChoiceMode.java` | `ffb-model` | `src/model/position_choice_mode.rs` | ✓ |
| `Pushback.java` | `ffb-model` | `src/model/pushback.rs` | ✓ |
| `PushbackMode.java` | `ffb-model` | `src/model/pushback_mode.rs` | ✓ |
| `PushbackSquare.java` | `ffb-model` | `src/model/pushback_square.rs` | ✓ |
| `RangeRuler.java` | `ffb-model` | `src/model/range_ruler.rs` | ✓ |
| `ReRolledAction.java` | `ffb-model` | `src/model/re_rolled_action.rs` | ✓ |
| `ReRolledActions.java` | `ffb-model` | `src/model/re_rolled_actions.rs` | ✓ |
| `ReRollOptions.java` | `ffb-model` | `src/model/re_roll_options.rs` | ✓ |
| `ReRollProperty.java` | `ffb-model` | `src/model/re_roll_property.rs` | ✓ |
| `ReRollSource.java` | `ffb-model` | `src/model/re_roll_source.rs` | ✓ |
| `ReRollSources.java` | `ffb-model` | `src/model/re_roll_sources.rs` | ✓ |
| `RulesCollection.java` | `ffb-model` | `src/model/rules_collection.rs` | ✓ |
| `RulesCollections.java` | `ffb-model` | `src/model/rules_collections.rs` | ✓ |
| `SendToBoxReason.java` | `ffb-model` | `src/model/send_to_box_reason.rs` | ✓ |
| `SeriousInjury.java` | `ffb-model` | `src/model/serious_injury.rs` | ✓ |
| `SkillCategory.java` | `ffb-model` | `src/model/skill_category.rs` | ✓ |
| `SkillChoiceMode.java` | `ffb-model` | `src/model/skill_choice_mode.rs` | ✓ |
| `SkillUse.java` | `ffb-model` | `src/model/skill_use.rs` | ✓ |
| `SoundId.java` | `ffb-model` | `src/model/sound_id.rs` | ✓ |
| `SpecialEffect.java` | `ffb-model` | `src/model/special_effect.rs` | ✓ |
| `StatusType.java` | `ffb-model` | `src/model/status_type.rs` | ✓ |
| `TeamList.java` | `ffb-model` | `src/model/team_list.rs` | ✓ |
| `TeamListEntry.java` | `ffb-model` | `src/model/team_list_entry.rs` | ✓ |
| `TeamSetup.java` | `ffb-model` | `src/model/team_setup.rs` | ✓ |
| `TeamStatus.java` | `ffb-model` | `src/model/team_status.rs` | ✓ |
| `TrackNumber.java` | `ffb-model` | `src/model/track_number.rs` | ✓ |
| `TurnMode.java` | `ffb-model` | `src/model/turn_mode.rs` | ✓ |
| `Weather.java` | `ffb-model` | `src/model/weather.rs` | ✓ |

### skill/ (297 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `skill/ArmourIncrease.java` | `ffb-model` | `src/skill/armour_increase.rs` | ✓ |
| `skill/bb2016/Accurate.java` | `ffb-model` | `src/skill/bb2016/accurate.rs` | ✓ |
| `skill/bb2016/AlwaysHungry.java` | `ffb-model` | `src/skill/bb2016/always_hungry.rs` | ✓ |
| `skill/bb2016/Animosity.java` | `ffb-model` | `src/skill/bb2016/animosity.rs` | ✓ |
| `skill/bb2016/ArmourIncrease.java` | `ffb-model` | `src/skill/bb2016/armour_increase.rs` | ✓ |
| `skill/bb2016/BallAndChain.java` | `ffb-model` | `src/skill/bb2016/ball_and_chain.rs` | ✓ |
| `skill/bb2016/BloodLust.java` | `ffb-model` | `src/skill/bb2016/blood_lust.rs` | ✓ |
| `skill/bb2016/Bombardier.java` | `ffb-model` | `src/skill/bb2016/bombardier.rs` | ✓ |
| `skill/bb2016/BoneHead.java` | `ffb-model` | `src/skill/bb2016/bone_head.rs` | ✓ |
| `skill/bb2016/BreakTackle.java` | `ffb-model` | `src/skill/bb2016/break_tackle.rs` | ✓ |
| `skill/bb2016/Chainsaw.java` | `ffb-model` | `src/skill/bb2016/chainsaw.rs` | ✓ |
| `skill/bb2016/Claw.java` | `ffb-model` | `src/skill/bb2016/claw.rs` | ✓ |
| `skill/bb2016/Decay.java` | `ffb-model` | `src/skill/bb2016/decay.rs` | ✓ |
| `skill/bb2016/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2016/dirty_player.rs` | ✓ |
| `skill/bb2016/Disposable.java` | `ffb-model` | `src/skill/bb2016/disposable.rs` | ✓ |
| `skill/bb2016/DivingTackle.java` | `ffb-model` | `src/skill/bb2016/diving_tackle.rs` | ✓ |
| `skill/bb2016/FanFavourite.java` | `ffb-model` | `src/skill/bb2016/fan_favourite.rs` | ✓ |
| `skill/bb2016/Frenzy.java` | `ffb-model` | `src/skill/bb2016/frenzy.rs` | ✓ |
| `skill/bb2016/Grab.java` | `ffb-model` | `src/skill/bb2016/grab.rs` | ✓ |
| `skill/bb2016/Guard.java` | `ffb-model` | `src/skill/bb2016/guard.rs` | ✓ |
| `skill/bb2016/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2016/hypnotic_gaze.rs` | ✓ |
| `skill/bb2016/KickOffReturn.java` | `ffb-model` | `src/skill/bb2016/kick_off_return.rs` | ✓ |
| `skill/bb2016/KickTeamMate.java` | `ffb-model` | `src/skill/bb2016/kick_team_mate.rs` | ✓ |
| `skill/bb2016/Leap.java` | `ffb-model` | `src/skill/bb2016/leap.rs` | ✓ |
| `skill/bb2016/Loner.java` | `ffb-model` | `src/skill/bb2016/loner.rs` | ✓ |
| `skill/bb2016/MightyBlow.java` | `ffb-model` | `src/skill/bb2016/mighty_blow.rs` | ✓ |
| `skill/bb2016/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2016/monstrous_mouth.rs` | ✓ |
| `skill/bb2016/MultipleBlock.java` | `ffb-model` | `src/skill/bb2016/multiple_block.rs` | ✓ |
| `skill/bb2016/NervesOfSteel.java` | `ffb-model` | `src/skill/bb2016/nerves_of_steel.rs` | ✓ |
| `skill/bb2016/NoHands.java` | `ffb-model` | `src/skill/bb2016/no_hands.rs` | ✓ |
| `skill/bb2016/NurglesRot.java` | `ffb-model` | `src/skill/bb2016/nurgles_rot.rs` | ✓ |
| `skill/bb2016/PassBlock.java` | `ffb-model` | `src/skill/bb2016/pass_block.rs` | ✓ |
| `skill/bb2016/PilingOn.java` | `ffb-model` | `src/skill/bb2016/piling_on.rs` | ✓ |
| `skill/bb2016/PrehensileTail.java` | `ffb-model` | `src/skill/bb2016/prehensile_tail.rs` | ✓ |
| `skill/bb2016/ReallyStupid.java` | `ffb-model` | `src/skill/bb2016/really_stupid.rs` | ✓ |
| `skill/bb2016/Regeneration.java` | `ffb-model` | `src/skill/bb2016/regeneration.rs` | ✓ |
| `skill/bb2016/RightStuff.java` | `ffb-model` | `src/skill/bb2016/right_stuff.rs` | ✓ |
| `skill/bb2016/SafeThrow.java` | `ffb-model` | `src/skill/bb2016/safe_throw.rs` | ✓ |
| `skill/bb2016/SecretWeapon.java` | `ffb-model` | `src/skill/bb2016/secret_weapon.rs` | ✓ |
| `skill/bb2016/Shadowing.java` | `ffb-model` | `src/skill/bb2016/shadowing.rs` | ✓ |
| `skill/bb2016/SideStep.java` | `ffb-model` | `src/skill/bb2016/side_step.rs` | ✓ |
| `skill/bb2016/SneakyGit.java` | `ffb-model` | `src/skill/bb2016/sneaky_git.rs` | ✓ |
| `skill/bb2016/Stab.java` | `ffb-model` | `src/skill/bb2016/stab.rs` | ✓ |
| `skill/bb2016/Stakes.java` | `ffb-model` | `src/skill/bb2016/stakes.rs` | ✓ |
| `skill/bb2016/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2016/strength_increase.rs` | ✓ |
| `skill/bb2016/StrongArm.java` | `ffb-model` | `src/skill/bb2016/strong_arm.rs` | ✓ |
| `skill/bb2016/Stunty.java` | `ffb-model` | `src/skill/bb2016/stunty.rs` | ✓ |
| `skill/bb2016/SureFeet.java` | `ffb-model` | `src/skill/bb2016/sure_feet.rs` | ✓ |
| `skill/bb2016/Swarming.java` | `ffb-model` | `src/skill/bb2016/swarming.rs` | ✓ |
| `skill/bb2016/Swoop.java` | `ffb-model` | `src/skill/bb2016/swoop.rs` | ✓ |
| `skill/bb2016/TakeRoot.java` | `ffb-model` | `src/skill/bb2016/take_root.rs` | ✓ |
| `skill/bb2016/ThrowTeamMate.java` | `ffb-model` | `src/skill/bb2016/throw_team_mate.rs` | ✓ |
| `skill/bb2016/Timmmber.java` | `ffb-model` | `src/skill/bb2016/timmmber.rs` | ✓ |
| `skill/bb2016/Titchy.java` | `ffb-model` | `src/skill/bb2016/titchy.rs` | ✓ |
| `skill/bb2016/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2016/very_long_legs.rs` | ✓ |
| `skill/bb2016/WeepingDagger.java` | `ffb-model` | `src/skill/bb2016/weeping_dagger.rs` | ✓ |
| `skill/bb2016/WildAnimal.java` | `ffb-model` | `src/skill/bb2016/wild_animal.rs` | ✓ |
| `skill/bb2020/Animosity.java` | `ffb-model` | `src/skill/bb2020/animosity.rs` | ✓ |
| `skill/bb2020/BallAndChain.java` | `ffb-model` | `src/skill/bb2020/ball_and_chain.rs` | ✓ |
| `skill/bb2020/Bombardier.java` | `ffb-model` | `src/skill/bb2020/bombardier.rs` | ✓ |
| `skill/bb2020/BoneHead.java` | `ffb-model` | `src/skill/bb2020/bone_head.rs` | ✓ |
| `skill/bb2020/Brawler.java` | `ffb-model` | `src/skill/bb2020/brawler.rs` | ✓ |
| `skill/bb2020/BreakTackle.java` | `ffb-model` | `src/skill/bb2020/break_tackle.rs` | ✓ |
| `skill/bb2020/BreatheFire.java` | `ffb-model` | `src/skill/bb2020/breathe_fire.rs` | ✓ |
| `skill/bb2020/Chainsaw.java` | `ffb-model` | `src/skill/bb2020/chainsaw.rs` | ✓ |
| `skill/bb2020/CloudBurster.java` | `ffb-model` | `src/skill/bb2020/cloud_burster.rs` | ✓ |
| `skill/bb2020/Defensive.java` | `ffb-model` | `src/skill/bb2020/defensive.rs` | ✓ |
| `skill/bb2020/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2020/dirty_player.rs` | ✓ |
| `skill/bb2020/Fumblerooskie.java` | `ffb-model` | `src/skill/bb2020/fumblerooskie.rs` | ✓ |
| `skill/bb2020/HitAndRun.java` | `ffb-model` | `src/skill/bb2020/hit_and_run.rs` | ✓ |
| `skill/bb2020/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2020/hypnotic_gaze.rs` | ✓ |
| `skill/bb2020/Leap.java` | `ffb-model` | `src/skill/bb2020/leap.rs` | ✓ |
| `skill/bb2020/MightyBlow.java` | `ffb-model` | `src/skill/bb2020/mighty_blow.rs` | ✓ |
| `skill/bb2020/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2020/monstrous_mouth.rs` | ✓ |
| `skill/bb2020/NoHands.java` | `ffb-model` | `src/skill/bb2020/no_hands.rs` | ✓ |
| `skill/bb2020/PassingIncrease.java` | `ffb-model` | `src/skill/bb2020/passing_increase.rs` | ✓ |
| `skill/bb2020/PileDriver.java` | `ffb-model` | `src/skill/bb2020/pile_driver.rs` | ✓ |
| `skill/bb2020/PilingOn.java` | `ffb-model` | `src/skill/bb2020/piling_on.rs` | ✓ |
| `skill/bb2020/PogoStick.java` | `ffb-model` | `src/skill/bb2020/pogo_stick.rs` | ✓ |
| `skill/bb2020/ProjectileVomit.java` | `ffb-model` | `src/skill/bb2020/projectile_vomit.rs` | ✓ |
| `skill/bb2020/ReallyStupid.java` | `ffb-model` | `src/skill/bb2020/really_stupid.rs` | ✓ |
| `skill/bb2020/Regeneration.java` | `ffb-model` | `src/skill/bb2020/regeneration.rs` | ✓ |
| `skill/bb2020/RightStuff.java` | `ffb-model` | `src/skill/bb2020/right_stuff.rs` | ✓ |
| `skill/bb2020/RunningPass.java` | `ffb-model` | `src/skill/bb2020/running_pass.rs` | ✓ |
| `skill/bb2020/Shadowing.java` | `ffb-model` | `src/skill/bb2020/shadowing.rs` | ✓ |
| `skill/bb2020/SideStep.java` | `ffb-model` | `src/skill/bb2020/side_step.rs` | ✓ |
| `skill/bb2020/SneakyGit.java` | `ffb-model` | `src/skill/bb2020/sneaky_git.rs` | ✓ |
| `skill/bb2020/special/ASneakyPair.java` | `ffb-model` | `src/skill/bb2020/special/a_sneaky_pair.rs` | ✓ |
| `skill/bb2020/special/BlastIt.java` | `ffb-model` | `src/skill/bb2020/special/blast_it.rs` | ✓ |
| `skill/bb2020/special/BrutalBlock.java` | `ffb-model` | `src/skill/bb2020/special/brutal_block.rs` | ✓ |
| `skill/bb2020/special/BurstOfSpeed.java` | `ffb-model` | `src/skill/bb2020/special/burst_of_speed.rs` | ✓ |
| `skill/bb2020/special/ConsummateProfessional.java` | `ffb-model` | `src/skill/bb2020/special/consummate_professional.rs` | ✓ |
| `skill/bb2020/special/DwarfenScourge.java` | `ffb-model` | `src/skill/bb2020/special/dwarfen_scourge.rs` | ✓ |
| `skill/bb2020/special/ExcuseMeAreYouAZoat.java` | `ffb-model` | `src/skill/bb2020/special/excuse_me_are_you_a_zoat.rs` | ✓ |
| `skill/bb2020/special/FrenziedRush.java` | `ffb-model` | `src/skill/bb2020/special/frenzied_rush.rs` | ✓ |
| `skill/bb2020/special/GhostlyFlames.java` | `ffb-model` | `src/skill/bb2020/special/ghostly_flames.rs` | ✓ |
| `skill/bb2020/special/Incorporeal.java` | `ffb-model` | `src/skill/bb2020/special/incorporeal.rs` | ✓ |
| `skill/bb2020/special/LordOfChaos.java` | `ffb-model` | `src/skill/bb2020/special/lord_of_chaos.rs` | ✓ |
| `skill/bb2020/special/MasterAssassin.java` | `ffb-model` | `src/skill/bb2020/special/master_assassin.rs` | ✓ |
| `skill/bb2020/special/MesmerizingDance.java` | `ffb-model` | `src/skill/bb2020/special/mesmerizing_dance.rs` | ✓ |
| `skill/bb2020/special/PumpUpTheCrowd.java` | `ffb-model` | `src/skill/bb2020/special/pump_up_the_crowd.rs` | ✓ |
| `skill/bb2020/special/PutridRegurgitation.java` | `ffb-model` | `src/skill/bb2020/special/putrid_regurgitation.rs` | ✓ |
| `skill/bb2020/special/TheBallista.java` | `ffb-model` | `src/skill/bb2020/special/the_ballista.rs` | ✓ |
| `skill/bb2020/special/ThenIStartedBlastin.java` | `ffb-model` | `src/skill/bb2020/special/then_i_started_blastin.rs` | ✓ |
| `skill/bb2020/special/TwoForOne.java` | `ffb-model` | `src/skill/bb2020/special/two_for_one.rs` | ✓ |
| `skill/bb2020/special/WhirlingDervish.java` | `ffb-model` | `src/skill/bb2020/special/whirling_dervish.rs` | ✓ |
| `skill/bb2020/special/WisdomOfTheWhiteDwarf.java` | `ffb-model` | `src/skill/bb2020/special/wisdom_of_the_white_dwarf.rs` | ✓ |
| `skill/bb2020/Stab.java` | `ffb-model` | `src/skill/bb2020/stab.rs` | ✓ |
| `skill/bb2020/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2020/strength_increase.rs` | ✓ |
| `skill/bb2020/SureFeet.java` | `ffb-model` | `src/skill/bb2020/sure_feet.rs` | ✓ |
| `skill/bb2020/Swarming.java` | `ffb-model` | `src/skill/bb2020/swarming.rs` | ✓ |
| `skill/bb2020/Swoop.java` | `ffb-model` | `src/skill/bb2020/swoop.rs` | ✓ |
| `skill/bb2020/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2020/very_long_legs.rs` | ✓ |
| `skill/bb2025/AgilityIncrease.java` | `ffb-model` | `src/skill/bb2025/agility_increase.rs` | ✓ |
| `skill/bb2025/Animosity.java` | `ffb-model` | `src/skill/bb2025/animosity.rs` | ✓ |
| `skill/bb2025/BallAndChain.java` | `ffb-model` | `src/skill/bb2025/ball_and_chain.rs` | ✓ |
| `skill/bb2025/BigHand.java` | `ffb-model` | `src/skill/bb2025/big_hand.rs` | ✓ |
| `skill/bb2025/Bombardier.java` | `ffb-model` | `src/skill/bb2025/bombardier.rs` | ✓ |
| `skill/bb2025/BoneHead.java` | `ffb-model` | `src/skill/bb2025/bone_head.rs` | ✓ |
| `skill/bb2025/Brawler.java` | `ffb-model` | `src/skill/bb2025/brawler.rs` | ✓ |
| `skill/bb2025/BreakTackle.java` | `ffb-model` | `src/skill/bb2025/break_tackle.rs` | ✓ |
| `skill/bb2025/BreatheFire.java` | `ffb-model` | `src/skill/bb2025/breathe_fire.rs` | ✓ |
| `skill/bb2025/Bullseye.java` | `ffb-model` | `src/skill/bb2025/bullseye.rs` | ✓ |
| `skill/bb2025/Chainsaw.java` | `ffb-model` | `src/skill/bb2025/chainsaw.rs` | ✓ |
| `skill/bb2025/CloudBurster.java` | `ffb-model` | `src/skill/bb2025/cloud_burster.rs` | ✓ |
| `skill/bb2025/Defensive.java` | `ffb-model` | `src/skill/bb2025/defensive.rs` | ✓ |
| `skill/bb2025/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2025/dirty_player.rs` | ✓ |
| `skill/bb2025/Dodge.java` | `ffb-model` | `src/skill/bb2025/dodge.rs` | ✓ |
| `skill/bb2025/EyeGouge.java` | `ffb-model` | `src/skill/bb2025/eye_gouge.rs` | ✓ |
| `skill/bb2025/Fumblerooski.java` | `ffb-model` | `src/skill/bb2025/fumblerooski.rs` | ✓ |
| `skill/bb2025/GiveAndGo.java` | `ffb-model` | `src/skill/bb2025/give_and_go.rs` | ✓ |
| `skill/bb2025/Hatred.java` | `ffb-model` | `src/skill/bb2025/hatred.rs` | ✓ |
| `skill/bb2025/HitAndRun.java` | `ffb-model` | `src/skill/bb2025/hit_and_run.rs` | ✓ |
| `skill/bb2025/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2025/hypnotic_gaze.rs` | ✓ |
| `skill/bb2025/Insignificant.java` | `ffb-model` | `src/skill/bb2025/insignificant.rs` | ✓ |
| `skill/bb2025/Juggernaut.java` | `ffb-model` | `src/skill/bb2025/juggernaut.rs` | ✓ |
| `skill/bb2025/Kick.java` | `ffb-model` | `src/skill/bb2025/kick.rs` | ✓ |
| `skill/bb2025/Leader.java` | `ffb-model` | `src/skill/bb2025/leader.rs` | ✓ |
| `skill/bb2025/Leap.java` | `ffb-model` | `src/skill/bb2025/leap.rs` | ✓ |
| `skill/bb2025/LethalFlight.java` | `ffb-model` | `src/skill/bb2025/lethal_flight.rs` | ✓ |
| `skill/bb2025/LoneFouler.java` | `ffb-model` | `src/skill/bb2025/lone_fouler.rs` | ✓ |
| `skill/bb2025/MightyBlow.java` | `ffb-model` | `src/skill/bb2025/mighty_blow.rs` | ✓ |
| `skill/bb2025/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2025/monstrous_mouth.rs` | ✓ |
| `skill/bb2025/NoBall.java` | `ffb-model` | `src/skill/bb2025/no_ball.rs` | ✓ |
| `skill/bb2025/PassingIncrease.java` | `ffb-model` | `src/skill/bb2025/passing_increase.rs` | ✓ |
| `skill/bb2025/PileDriver.java` | `ffb-model` | `src/skill/bb2025/pile_driver.rs` | ✓ |
| `skill/bb2025/Pogo.java` | `ffb-model` | `src/skill/bb2025/pogo.rs` | ✓ |
| `skill/bb2025/Pro.java` | `ffb-model` | `src/skill/bb2025/pro.rs` | ✓ |
| `skill/bb2025/ProjectileVomit.java` | `ffb-model` | `src/skill/bb2025/projectile_vomit.rs` | ✓ |
| `skill/bb2025/Punt.java` | `ffb-model` | `src/skill/bb2025/punt.rs` | ✓ |
| `skill/bb2025/PutTheBootIn.java` | `ffb-model` | `src/skill/bb2025/put_the_boot_in.rs` | ✓ |
| `skill/bb2025/QuickFoul.java` | `ffb-model` | `src/skill/bb2025/quick_foul.rs` | ✓ |
| `skill/bb2025/ReallyStupid.java` | `ffb-model` | `src/skill/bb2025/really_stupid.rs` | ✓ |
| `skill/bb2025/Regeneration.java` | `ffb-model` | `src/skill/bb2025/regeneration.rs` | ✓ |
| `skill/bb2025/RightStuff.java` | `ffb-model` | `src/skill/bb2025/right_stuff.rs` | ✓ |
| `skill/bb2025/Saboteur.java` | `ffb-model` | `src/skill/bb2025/saboteur.rs` | ✓ |
| `skill/bb2025/Shadowing.java` | `ffb-model` | `src/skill/bb2025/shadowing.rs` | ✓ |
| `skill/bb2025/Sidestep.java` | `ffb-model` | `src/skill/bb2025/sidestep.rs` | ✓ |
| `skill/bb2025/SneakyGit.java` | `ffb-model` | `src/skill/bb2025/sneaky_git.rs` | ✓ |
| `skill/bb2025/special/ASneakyPair.java` | `ffb-model` | `src/skill/bb2025/special/a_sneaky_pair.rs` | ✓ |
| `skill/bb2025/special/BlastinSolvesEverything.java` | `ffb-model` | `src/skill/bb2025/special/blastin_solves_everything.rs` | ✓ |
| `skill/bb2025/special/BlastIt.java` | `ffb-model` | `src/skill/bb2025/special/blast_it.rs` | ✓ |
| `skill/bb2025/special/DwarvenScourge.java` | `ffb-model` | `src/skill/bb2025/special/dwarven_scourge.rs` | ✓ |
| `skill/bb2025/special/ExcuseMeAreYouAZoat.java` | `ffb-model` | `src/skill/bb2025/special/excuse_me_are_you_a_zoat.rs` | ✓ |
| `skill/bb2025/special/FrenziedRush.java` | `ffb-model` | `src/skill/bb2025/special/frenzied_rush.rs` | ✓ |
| `skill/bb2025/special/Incorporeal.java` | `ffb-model` | `src/skill/bb2025/special/incorporeal.rs` | ✓ |
| `skill/bb2025/special/KrumpAndSmash.java` | `ffb-model` | `src/skill/bb2025/special/krump_and_smash.rs` | ✓ |
| `skill/bb2025/special/LordOfChaos.java` | `ffb-model` | `src/skill/bb2025/special/lord_of_chaos.rs` | ✓ |
| `skill/bb2025/special/MasterAssassin.java` | `ffb-model` | `src/skill/bb2025/special/master_assassin.rs` | ✓ |
| `skill/bb2025/special/MesmerisingDance.java` | `ffb-model` | `src/skill/bb2025/special/mesmerising_dance.rs` | ✓ |
| `skill/bb2025/special/PumpUpTheCrowd.java` | `ffb-model` | `src/skill/bb2025/special/pump_up_the_crowd.rs` | ✓ |
| `skill/bb2025/special/PutridRegurgitation.java` | `ffb-model` | `src/skill/bb2025/special/putrid_regurgitation.rs` | ✓ |
| `skill/bb2025/special/SlashingNails.java` | `ffb-model` | `src/skill/bb2025/special/slashing_nails.rs` | ✓ |
| `skill/bb2025/special/TeamCaptain.java` | `ffb-model` | `src/skill/bb2025/special/team_captain.rs` | ✓ |
| `skill/bb2025/special/TheBallista.java` | `ffb-model` | `src/skill/bb2025/special/the_ballista.rs` | ✓ |
| `skill/bb2025/special/WhirlingDervish.java` | `ffb-model` | `src/skill/bb2025/special/whirling_dervish.rs` | ✓ |
| `skill/bb2025/special/WisdomOfTheWhiteDwarf.java` | `ffb-model` | `src/skill/bb2025/special/wisdom_of_the_white_dwarf.rs` | ✓ |
| `skill/bb2025/special/WoodlandFury.java` | `ffb-model` | `src/skill/bb2025/special/woodland_fury.rs` | ✓ |
| `skill/bb2025/special/WorkingInTandem.java` | `ffb-model` | `src/skill/bb2025/special/working_in_tandem.rs` | ✓ |
| `skill/bb2025/Stab.java` | `ffb-model` | `src/skill/bb2025/stab.rs` | ✓ |
| `skill/bb2025/SteadyFooting.java` | `ffb-model` | `src/skill/bb2025/steady_footing.rs` | ✓ |
| `skill/bb2025/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2025/strength_increase.rs` | ✓ |
| `skill/bb2025/SureFeet.java` | `ffb-model` | `src/skill/bb2025/sure_feet.rs` | ✓ |
| `skill/bb2025/Swoop.java` | `ffb-model` | `src/skill/bb2025/swoop.rs` | ✓ |
| `skill/bb2025/Taunt.java` | `ffb-model` | `src/skill/bb2025/taunt.rs` | ✓ |
| `skill/bb2025/Unsteady.java` | `ffb-model` | `src/skill/bb2025/unsteady.rs` | ✓ |
| `skill/bb2025/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2025/very_long_legs.rs` | ✓ |
| `skill/bb2025/ViolentInnovator.java` | `ffb-model` | `src/skill/bb2025/violent_innovator.rs` | ✓ |
| `skill/common/Block.java` | `ffb-model` | `src/skill/common/block.rs` | ✓ |
| `skill/common/Catch.java` | `ffb-model` | `src/skill/common/catch.rs` | ✓ |
| `skill/common/Dauntless.java` | `ffb-model` | `src/skill/common/dauntless.rs` | ✓ |
| `skill/common/DisturbingPresence.java` | `ffb-model` | `src/skill/common/disturbing_presence.rs` | ✓ |
| `skill/common/DivingCatch.java` | `ffb-model` | `src/skill/common/diving_catch.rs` | ✓ |
| `skill/common/DumpOff.java` | `ffb-model` | `src/skill/common/dump_off.rs` | ✓ |
| `skill/common/ExtraArms.java` | `ffb-model` | `src/skill/common/extra_arms.rs` | ✓ |
| `skill/common/Fend.java` | `ffb-model` | `src/skill/common/fend.rs` | ✓ |
| `skill/common/FoulAppearance.java` | `ffb-model` | `src/skill/common/foul_appearance.rs` | ✓ |
| `skill/common/HailMaryPass.java` | `ffb-model` | `src/skill/common/hail_mary_pass.rs` | ✓ |
| `skill/common/Horns.java` | `ffb-model` | `src/skill/common/horns.rs` | ✓ |
| `skill/common/JumpUp.java` | `ffb-model` | `src/skill/common/jump_up.rs` | ✓ |
| `skill/common/MovementIncrease.java` | `ffb-model` | `src/skill/common/movement_increase.rs` | ✓ |
| `skill/common/Pass.java` | `ffb-model` | `src/skill/common/pass.rs` | ✓ |
| `skill/common/Sprint.java` | `ffb-model` | `src/skill/common/sprint.rs` | ✓ |
| `skill/common/StandFirm.java` | `ffb-model` | `src/skill/common/stand_firm.rs` | ✓ |
| `skill/common/StripBall.java` | `ffb-model` | `src/skill/common/strip_ball.rs` | ✓ |
| `skill/common/SureHands.java` | `ffb-model` | `src/skill/common/sure_hands.rs` | ✓ |
| `skill/common/Tackle.java` | `ffb-model` | `src/skill/common/tackle.rs` | ✓ |
| `skill/common/Tentacles.java` | `ffb-model` | `src/skill/common/tentacles.rs` | ✓ |
| `skill/common/ThickSkull.java` | `ffb-model` | `src/skill/common/thick_skull.rs` | ✓ |
| `skill/common/TwoHeads.java` | `ffb-model` | `src/skill/common/two_heads.rs` | ✓ |
| `skill/common/Wrestle.java` | `ffb-model` | `src/skill/common/wrestle.rs` | ✓ |
| `skill/mixed/Accurate.java` | `ffb-model` | `src/skill/mixed/accurate.rs` | ✓ |
| `skill/mixed/AgilityIncrease.java` | `ffb-model` | `src/skill/mixed/agility_increase.rs` | ✓ |
| `skill/mixed/AlwaysHungry.java` | `ffb-model` | `src/skill/mixed/always_hungry.rs` | ✓ |
| `skill/mixed/AnimalSavagery.java` | `ffb-model` | `src/skill/mixed/animal_savagery.rs` | ✓ |
| `skill/mixed/ArmBar.java` | `ffb-model` | `src/skill/mixed/arm_bar.rs` | ✓ |
| `skill/mixed/ArmourIncrease.java` | `ffb-model` | `src/skill/mixed/armour_increase.rs` | ✓ |
| `skill/mixed/BigHand.java` | `ffb-model` | `src/skill/mixed/big_hand.rs` | ✓ |
| `skill/mixed/Bloodlust.java` | `ffb-model` | `src/skill/mixed/bloodlust.rs` | ✓ |
| `skill/mixed/Cannoneer.java` | `ffb-model` | `src/skill/mixed/cannoneer.rs` | ✓ |
| `skill/mixed/Claws.java` | `ffb-model` | `src/skill/mixed/claws.rs` | ✓ |
| `skill/mixed/Decay.java` | `ffb-model` | `src/skill/mixed/decay.rs` | ✓ |
| `skill/mixed/DivingTackle.java` | `ffb-model` | `src/skill/mixed/diving_tackle.rs` | ✓ |
| `skill/mixed/Dodge.java` | `ffb-model` | `src/skill/mixed/dodge.rs` | ✓ |
| `skill/mixed/Drunkard.java` | `ffb-model` | `src/skill/mixed/drunkard.rs` | ✓ |
| `skill/mixed/Frenzy.java` | `ffb-model` | `src/skill/mixed/frenzy.rs` | ✓ |
| `skill/mixed/Grab.java` | `ffb-model` | `src/skill/mixed/grab.rs` | ✓ |
| `skill/mixed/Guard.java` | `ffb-model` | `src/skill/mixed/guard.rs` | ✓ |
| `skill/mixed/IronHardSkin.java` | `ffb-model` | `src/skill/mixed/iron_hard_skin.rs` | ✓ |
| `skill/mixed/Juggernaut.java` | `ffb-model` | `src/skill/mixed/juggernaut.rs` | ✓ |
| `skill/mixed/Kick.java` | `ffb-model` | `src/skill/mixed/kick.rs` | ✓ |
| `skill/mixed/KickTeamMate.java` | `ffb-model` | `src/skill/mixed/kick_team_mate.rs` | ✓ |
| `skill/mixed/Leader.java` | `ffb-model` | `src/skill/mixed/leader.rs` | ✓ |
| `skill/mixed/Loner.java` | `ffb-model` | `src/skill/mixed/loner.rs` | ✓ |
| `skill/mixed/MultipleBlock.java` | `ffb-model` | `src/skill/mixed/multiple_block.rs` | ✓ |
| `skill/mixed/MyBall.java` | `ffb-model` | `src/skill/mixed/my_ball.rs` | ✓ |
| `skill/mixed/NervesOfSteel.java` | `ffb-model` | `src/skill/mixed/nerves_of_steel.rs` | ✓ |
| `skill/mixed/OnTheBall.java` | `ffb-model` | `src/skill/mixed/on_the_ball.rs` | ✓ |
| `skill/mixed/PickMeUp.java` | `ffb-model` | `src/skill/mixed/pick_me_up.rs` | ✓ |
| `skill/mixed/PlagueRidden.java` | `ffb-model` | `src/skill/mixed/plague_ridden.rs` | ✓ |
| `skill/mixed/PrehensileTail.java` | `ffb-model` | `src/skill/mixed/prehensile_tail.rs` | ✓ |
| `skill/mixed/Pro.java` | `ffb-model` | `src/skill/mixed/pro.rs` | ✓ |
| `skill/mixed/SafePairOfHands.java` | `ffb-model` | `src/skill/mixed/safe_pair_of_hands.rs` | ✓ |
| `skill/mixed/SafePass.java` | `ffb-model` | `src/skill/mixed/safe_pass.rs` | ✓ |
| `skill/mixed/SecretWeapon.java` | `ffb-model` | `src/skill/mixed/secret_weapon.rs` | ✓ |
| `skill/mixed/special/AllYouCanEat.java` | `ffb-model` | `src/skill/mixed/special/all_you_can_eat.rs` | ✓ |
| `skill/mixed/special/BalefulHex.java` | `ffb-model` | `src/skill/mixed/special/baleful_hex.rs` | ✓ |
| `skill/mixed/special/BeerBarrelBash.java` | `ffb-model` | `src/skill/mixed/special/beer_barrel_bash.rs` | ✓ |
| `skill/mixed/special/BlackInk.java` | `ffb-model` | `src/skill/mixed/special/black_ink.rs` | ✓ |
| `skill/mixed/special/BlindRage.java` | `ffb-model` | `src/skill/mixed/special/blind_rage.rs` | ✓ |
| `skill/mixed/special/BoundingLeap.java` | `ffb-model` | `src/skill/mixed/special/bounding_leap.rs` | ✓ |
| `skill/mixed/special/BugmansXXXXXX.java` | `ffb-model` | `src/skill/mixed/special/bugmans_xxxxxx.rs` | ✓ |
| `skill/mixed/special/CatchOfTheDay.java` | `ffb-model` | `src/skill/mixed/special/catch_of_the_day.rs` | ✓ |
| `skill/mixed/special/CrushingBlow.java` | `ffb-model` | `src/skill/mixed/special/crushing_blow.rs` | ✓ |
| `skill/mixed/special/FuriousOutburst.java` | `ffb-model` | `src/skill/mixed/special/furious_outburst.rs` | ✓ |
| `skill/mixed/special/FuryOfTheBloodGod.java` | `ffb-model` | `src/skill/mixed/special/fury_of_the_blood_god.rs` | ✓ |
| `skill/mixed/special/GoredByTheBull.java` | `ffb-model` | `src/skill/mixed/special/gored_by_the_bull.rs` | ✓ |
| `skill/mixed/special/HalflingLuck.java` | `ffb-model` | `src/skill/mixed/special/halfling_luck.rs` | ✓ |
| `skill/mixed/special/IllBeBack.java` | `ffb-model` | `src/skill/mixed/special/ill_be_back.rs` | ✓ |
| `skill/mixed/special/Indomitable.java` | `ffb-model` | `src/skill/mixed/special/indomitable.rs` | ✓ |
| `skill/mixed/special/Kaboom.java` | `ffb-model` | `src/skill/mixed/special/kaboom.rs` | ✓ |
| `skill/mixed/special/KeenPlayer.java` | `ffb-model` | `src/skill/mixed/special/keen_player.rs` | ✓ |
| `skill/mixed/special/KickEmWhileTheyReDown.java` | `ffb-model` | `src/skill/mixed/special/kick_em_while_they_re_down.rs` | ✓ |
| `skill/mixed/special/LookIntoMyEyes.java` | `ffb-model` | `src/skill/mixed/special/look_into_my_eyes.rs` | ✓ |
| `skill/mixed/special/MaximumCarnage.java` | `ffb-model` | `src/skill/mixed/special/maximum_carnage.rs` | ✓ |
| `skill/mixed/special/OldPro.java` | `ffb-model` | `src/skill/mixed/special/old_pro.rs` | ✓ |
| `skill/mixed/special/PrimalSavagery.java` | `ffb-model` | `src/skill/mixed/special/primal_savagery.rs` | ✓ |
| `skill/mixed/special/QuickBite.java` | `ffb-model` | `src/skill/mixed/special/quick_bite.rs` | ✓ |
| `skill/mixed/special/RaidingParty.java` | `ffb-model` | `src/skill/mixed/special/raiding_party.rs` | ✓ |
| `skill/mixed/special/Ram.java` | `ffb-model` | `src/skill/mixed/special/ram.rs` | ✓ |
| `skill/mixed/special/Reliable.java` | `ffb-model` | `src/skill/mixed/special/reliable.rs` | ✓ |
| `skill/mixed/special/SavageBlow.java` | `ffb-model` | `src/skill/mixed/special/savage_blow.rs` | ✓ |
| `skill/mixed/special/SavageMauling.java` | `ffb-model` | `src/skill/mixed/special/savage_mauling.rs` | ✓ |
| `skill/mixed/special/ShotToNothing.java` | `ffb-model` | `src/skill/mixed/special/shot_to_nothing.rs` | ✓ |
| `skill/mixed/special/Slayer.java` | `ffb-model` | `src/skill/mixed/special/slayer.rs` | ✓ |
| `skill/mixed/special/SneakiestOfTheLot.java` | `ffb-model` | `src/skill/mixed/special/sneakiest_of_the_lot.rs` | ✓ |
| `skill/mixed/special/StarOfTheShow.java` | `ffb-model` | `src/skill/mixed/special/star_of_the_show.rs` | ✓ |
| `skill/mixed/special/StrongPassingGame.java` | `ffb-model` | `src/skill/mixed/special/strong_passing_game.rs` | ✓ |
| `skill/mixed/special/SwiftAsTheBreeze.java` | `ffb-model` | `src/skill/mixed/special/swift_as_the_breeze.rs` | ✓ |
| `skill/mixed/special/TastyMorsel.java` | `ffb-model` | `src/skill/mixed/special/tasty_morsel.rs` | ✓ |
| `skill/mixed/special/TheFlashingBlade.java` | `ffb-model` | `src/skill/mixed/special/the_flashing_blade.rs` | ✓ |
| `skill/mixed/special/ThinkingMansTroll.java` | `ffb-model` | `src/skill/mixed/special/thinking_mans_troll.rs` | ✓ |
| `skill/mixed/special/ToxinConnoisseur.java` | `ffb-model` | `src/skill/mixed/special/toxin_connoisseur.rs` | ✓ |
| `skill/mixed/special/Treacherous.java` | `ffb-model` | `src/skill/mixed/special/treacherous.rs` | ✓ |
| `skill/mixed/special/UnstoppableMomentum.java` | `ffb-model` | `src/skill/mixed/special/unstoppable_momentum.rs` | ✓ |
| `skill/mixed/special/ViciousVines.java` | `ffb-model` | `src/skill/mixed/special/vicious_vines.rs` | ✓ |
| `skill/mixed/special/WatchOut.java` | `ffb-model` | `src/skill/mixed/special/watch_out.rs` | ✓ |
| `skill/mixed/special/Yoink.java` | `ffb-model` | `src/skill/mixed/special/yoink.rs` | ✓ |
| `skill/mixed/StrongArm.java` | `ffb-model` | `src/skill/mixed/strong_arm.rs` | ✓ |
| `skill/mixed/Stunty.java` | `ffb-model` | `src/skill/mixed/stunty.rs` | ✓ |
| `skill/mixed/TakeRoot.java` | `ffb-model` | `src/skill/mixed/take_root.rs` | ✓ |
| `skill/mixed/ThrowTeamMate.java` | `ffb-model` | `src/skill/mixed/throw_team_mate.rs` | ✓ |
| `skill/mixed/Timmmber.java` | `ffb-model` | `src/skill/mixed/timmmber.rs` | ✓ |
| `skill/mixed/Titchy.java` | `ffb-model` | `src/skill/mixed/titchy.rs` | ✓ |
| `skill/mixed/Trickster.java` | `ffb-model` | `src/skill/mixed/trickster.rs` | ✓ |
| `skill/mixed/UnchannelledFury.java` | `ffb-model` | `src/skill/mixed/unchannelled_fury.rs` | ✓ |
| `skill/StrengthIncrease.java` | `ffb-model` | `src/skill/strength_increase.rs` | ✓ |

### stats/ (6 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `stats/DicePoolStat.java` | `—` | `—` | — |
| `stats/DieBase.java` | `—` | `—` | — |
| `stats/DieStat.java` | `—` | `—` | — |
| `stats/DoubleDiceStat.java` | `—` | `—` | — |
| `stats/SingleDieStat.java` | `—` | `—` | — |
| `stats/TeamMapping.java` | `—` | `—` | — |

### util/ (27 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `util/ArrayTool.java` | `ffb-model` | `src/util/array_tool.rs` | ✓ |
| `util/DateTool.java` | `ffb-model` | `src/util/date_tool.rs` | ✓ |
| `util/FileIterator.java` | `ffb-model` | `src/util/file_iterator.rs` | ✓ |
| `util/ListTool.java` | `ffb-model` | `src/util/list_tool.rs` | ✓ |
| `util/NaturalOrderComparator.java` | `ffb-model` | `src/util/natural_order_comparator.rs` | ✓ |
| `util/pathfinding/PathFindContext.java` | `ffb-model` | `src/util/pathfinding/path_find_context.rs` | ✓ |
| `util/pathfinding/PathFindData.java` | `ffb-model` | `src/util/pathfinding/path_find_data.rs` | ✓ |
| `util/pathfinding/PathFinderExtension.java` | `ffb-model` | `src/util/pathfinding/path_finder_extension.rs` | ✓ |
| `util/pathfinding/PathFinderWithMultiJump.java` | `ffb-model` | `src/util/pathfinding/path_finder_with_multi_jump.rs` | ✓ |
| `util/pathfinding/PathFinderWithPassBlockSupport.java` | `ffb-model` | `src/util/pathfinding/path_finder_with_pass_block_support.rs` | ✓ |
| `util/pathfinding/PathFindNode.java` | `ffb-model` | `src/util/pathfinding/path_find_node.rs` | ✓ |
| `util/pathfinding/PathFindState.java` | `ffb-model` | `src/util/pathfinding/path_find_state.rs` | ✓ |
| `util/RaiseType.java` | `ffb-model` | `src/util/raise_type.rs` | ✓ |
| `util/RawScanner.java` | `ffb-model` | `src/util/raw_scanner.rs` | ✓ |
| `util/rng/EntropySource.java` | `ffb-model` | `src/util/rng/entropy_source.rs` | ✓ |
| `util/Scanner.java` | `ffb-model` | `src/util/scanner.rs` | ✓ |
| `util/ScannerSingleton.java` | `ffb-model` | `src/util/scanner_singleton.rs` | ✓ |
| `util/StringTool.java` | `ffb-model` | `src/util/string_tool.rs` | ✓ |
| `util/UtilActingPlayer.java` | `ffb-model` | `src/util/util_acting_player.rs` | ✓ |
| `util/UtilBox.java` | `ffb-model` | `src/util/util_box.rs` | ✓ |
| `util/UtilCards.java` | `ffb-model` | `src/util/util_cards.rs` | ✓ |
| `util/UtilDisturbingPresence.java` | `ffb-model` | `src/util/util_disturbing_presence.rs` | ✓ |
| `util/UtilPassing.java` | `ffb-model` | `src/util/util_passing.rs` | ✓ |
| `util/UtilPlayer.java` | `ffb-model` | `src/util/util_player.rs` | ✓ |
| `util/UtilRangeRuler.java` | `ffb-model` | `src/util/util_range_ruler.rs` | ✓ |
| `util/UtilTeamValue.java` | `ffb-model` | `src/util/util_team_value.rs` | ✓ |
| `util/UtilUrl.java` | `ffb-model` | `src/util/util_url.rs` | ✓ |

### xml/ (5 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `xml/IXmlReadable.java` | `—` | `—` | — |
| `xml/IXmlSerializable.java` | `—` | `—` | — |
| `xml/IXmlWriteable.java` | `—` | `—` | — |
| `xml/UtilXml.java` | `—` | `—` | — |
| `xml/XmlHandler.java` | `—` | `—` | — |

## Module: ffb-server

### server/admin/ (9 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/admin/AdminConnector.java` | `—` | `—` | — |
| `server/admin/AdminList.java` | `—` | `—` | — |
| `server/admin/AdminListEntry.java` | `—` | `—` | — |
| `server/admin/AdminServlet.java` | `—` | `—` | — |
| `server/admin/BackupServlet.java` | `—` | `—` | — |
| `server/admin/GameStateConnector.java` | `—` | `—` | — |
| `server/admin/GameStateService.java` | `—` | `—` | — |
| `server/admin/GameStateServlet.java` | `—` | `—` | — |
| `server/admin/UtilBackup.java` | `—` | `—` | — |

### server/commandline/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/commandline/InifileParamFilter.java` | `—` | `—` | — |
| `server/commandline/InifileParamFilterResult.java` | `—` | `—` | — |

### server/db/ (55 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/db/DbConnectionManager.java` | `—` | `—` | — |
| `server/db/DbInitializer.java` | `—` | `—` | — |
| `server/db/DbQueryFactory.java` | `—` | `—` | — |
| `server/db/DbStatement.java` | `—` | `—` | — |
| `server/db/DbStatementId.java` | `—` | `—` | — |
| `server/db/DbTransaction.java` | `—` | `—` | — |
| `server/db/DbUpdateFactory.java` | `—` | `—` | — |
| `server/db/DbUpdateStatement.java` | `—` | `—` | — |
| `server/db/DefaultDbUpdateParameter.java` | `—` | `—` | — |
| `server/db/delete/DbGamesInfoDelete.java` | `—` | `—` | — |
| `server/db/delete/DbGamesInfoDeleteParameter.java` | `—` | `—` | — |
| `server/db/delete/DbGamesSerializedDelete.java` | `—` | `—` | — |
| `server/db/delete/DbGamesSerializedDeleteParameter.java` | `—` | `—` | — |
| `server/db/delete/DbPlayerMarkersDelete.java` | `—` | `—` | — |
| `server/db/delete/DbPlayerMarkersDeleteParameter.java` | `—` | `—` | — |
| `server/db/delete/DbTeamSetupsDelete.java` | `—` | `—` | — |
| `server/db/delete/DbTeamSetupsDeleteParameter.java` | `—` | `—` | — |
| `server/db/delete/DbUserSettingsDelete.java` | `—` | `—` | — |
| `server/db/delete/DbUserSettingsDeleteParameter.java` | `—` | `—` | — |
| `server/db/delete/DefaultDbUpdateParameter.java` | `—` | `—` | — |
| `server/db/IDbStatementFactory.java` | `—` | `—` | — |
| `server/db/IDbTableCoaches.java` | `—` | `—` | — |
| `server/db/IDbTableGamesInfo.java` | `—` | `—` | — |
| `server/db/IDbTableGamesSerialized.java` | `—` | `—` | — |
| `server/db/IDbTablePlayerMarkers.java` | `—` | `—` | — |
| `server/db/IDbTableTeamSetups.java` | `—` | `—` | — |
| `server/db/IDbTableUserSettings.java` | `—` | `—` | — |
| `server/db/IDbUpdateParameter.java` | `—` | `—` | — |
| `server/db/IDbUpdateParameterList.java` | `—` | `—` | — |
| `server/db/IDbUpdateWithGameState.java` | `—` | `—` | — |
| `server/db/insert/DbGamesSerializedInsert.java` | `—` | `—` | — |
| `server/db/insert/DbGamesSerializedInsertParameter.java` | `—` | `—` | — |
| `server/db/insert/DbPlayerMarkersInsert.java` | `—` | `—` | — |
| `server/db/insert/DbPlayerMarkersInsertParameter.java` | `—` | `—` | — |
| `server/db/insert/DbPlayerMarkersInsertParameterList.java` | `—` | `—` | — |
| `server/db/insert/DbTeamSetupsInsert.java` | `—` | `—` | — |
| `server/db/insert/DbTeamSetupsInsertParameter.java` | `—` | `—` | — |
| `server/db/insert/DbUserSettingsInsert.java` | `—` | `—` | — |
| `server/db/insert/DbUserSettingsInsertParameter.java` | `—` | `—` | — |
| `server/db/insert/DbUserSettingsInsertParameterList.java` | `—` | `—` | — |
| `server/db/query/DbAdminListByIdQuery.java` | `—` | `—` | — |
| `server/db/query/DbAdminListByStatusQuery.java` | `—` | `—` | — |
| `server/db/query/DbGameListQueryOpenGamesByCoach.java` | `—` | `—` | — |
| `server/db/query/DbGamesInfoInsertQuery.java` | `—` | `—` | — |
| `server/db/query/DbGamesSerializedQuery.java` | `—` | `—` | — |
| `server/db/query/DbPasswordForCoachQuery.java` | `—` | `—` | — |
| `server/db/query/DbPlayerMarkersQuery.java` | `—` | `—` | — |
| `server/db/query/DbTeamSetupsForTeamQuery.java` | `—` | `—` | — |
| `server/db/query/DbTeamSetupsQuery.java` | `—` | `—` | — |
| `server/db/query/DbTestGameListQuery.java` | `—` | `—` | — |
| `server/db/query/DbUserSettingsQuery.java` | `—` | `—` | — |
| `server/db/update/DbGamesInfoUpdate.java` | `—` | `—` | — |
| `server/db/update/DbGamesInfoUpdateParameter.java` | `—` | `—` | — |
| `server/db/update/DbGamesSerializedUpdate.java` | `—` | `—` | — |
| `server/db/update/DbGamesSerializedUpdateParameter.java` | `—` | `—` | — |

### server/factory/ (9 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/factory/bb2025/DeferredCommandFactory.java` | `ffb-engine` | `src/factory/bb2025/deferred_command_factory.rs` | ~ |
| `server/factory/bb2025/DeferredCommandIdFactory.java` | `ffb-engine` | `src/factory/bb2025/deferred_command_id_factory.rs` | ~ |
| `server/factory/CardHandlerFactory.java` | `ffb-engine` | `src/factory/card_handler_factory.rs` | ~ |
| `server/factory/InjuryTypeServerFactory.java` | `ffb-engine` | `src/factory/injury_type_server_factory.rs` | ~ |
| `server/factory/mixed/PrayerHandlerFactory.java` | `ffb-engine` | `src/factory/mixed/prayer_handler_factory.rs` | ~ |
| `server/factory/ObserverFactory.java` | `ffb-engine` | `src/factory/observer_factory.rs` | ~ |
| `server/factory/SequenceGeneratorFactory.java` | `ffb-engine` | `src/factory/sequence_generator_factory.rs` | ~ |
| `server/factory/StepActionFactory.java` | `ffb-engine` | `src/factory/step_action_factory.rs` | ~ |
| `server/factory/StepIdFactory.java` | `ffb-engine` | `src/factory/step_id_factory.rs` | ~ |

### server/handler/ (108 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/handler/AbstractServerCommandHandlerSketch.java` | `—` | `—` | — |
| `server/handler/IReceivedCommandHandler.java` | `—` | `—` | — |
| `server/handler/RedeployHandler.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandler.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerAddLoadedTeam.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerAddSketch.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerApplyAutomatedPlayerMarkings.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerCalculateAutomaticPlayerMarkings.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerClearSketches.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerCloseGame.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerCloseSession.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerDeleteGame.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerFactory.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerFumbblGameChecked.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerFumbblTeamLoaded.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerJoin.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerJoinApproved.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerJoinReplay.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerLoadAutomaticPlayerMarkings.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerPasswordChallenge.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerPing.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerRemoveSketches.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerReplay.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerReplayLoaded.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerReplayStatus.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerRequestVersion.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerScheduleGame.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSetMarker.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSetPreventSketching.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSketchAddCoordinate.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSketchSetColor.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSketchSetLabel.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerSocketClosed.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerTalk.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerTransferControl.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerUpdatePlayerMarkings.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerUploadGame.java` | `—` | `—` | — |
| `server/handler/ServerCommandHandlerUserSettings.java` | `—` | `—` | — |
| `server/handler/talk/CommandAdapter.java` | `—` | `—` | — |
| `server/handler/talk/DecoratingCommandAdapter.java` | `—` | `—` | — |
| `server/handler/talk/IdentityCommandAdapter.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandler.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerActivated.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerActivatedLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerActivatedTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerBox.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerBoxLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerBoxTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerCard.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerEmote.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerGameId.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerGames.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerInjury.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerInjuryLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerInjuryTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMessage.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMoveBall.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMoveBallLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMoveBallTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMovePlayer.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMovePlayerLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerMovePlayerTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerOption.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerOptions.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerPlayingLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerPrayer.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerProne.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerProneLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerProneTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerRedeploy.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerReRoll.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerReRollLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerReRollTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerResetStateLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerRoll.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetBall.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetBallLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetBallTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetPlayer.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetPlayerLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSetPlayerTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSkill.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSkillLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSkillTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSound.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSounds.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerSpecs.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStandup.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStandupLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStandupTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStat.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStatLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStatTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStun.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStunLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerStunTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerTurnLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerTurnMode.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerTurnModeLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerTurnModelTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerTurnTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerUsedActions.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerUsedActionsLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerUsedActionsTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerWeather.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerWeatherLive.java` | `—` | `—` | — |
| `server/handler/talk/TalkHandlerWeatherTest.java` | `—` | `—` | — |
| `server/handler/talk/TalkRequirements.java` | `—` | `—` | — |

### server/inducements/ (75 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/inducements/bb2016/cards/ChopBlockHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/chop_block_handler.rs` | ~ |
| `server/inducements/bb2016/cards/CustardPieHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/custard_pie_handler.rs` | ~ |
| `server/inducements/bb2016/cards/DistractHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/distract_handler.rs` | ~ |
| `server/inducements/bb2016/cards/ForceShieldHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/force_shield_handler.rs` | ~ |
| `server/inducements/bb2016/cards/IllegalSubstitutionHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/illegal_substitution_handler.rs` | ~ |
| `server/inducements/bb2016/cards/PitTrapHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/pit_trap_handler.rs` | ~ |
| `server/inducements/bb2016/cards/RabbitsFootHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/rabbits_foot_handler.rs` | ~ |
| `server/inducements/bb2016/cards/WitchBrewHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/witch_brew_handler.rs` | ~ |
| `server/inducements/bb2020/cards/ChopBlockHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/chop_block_handler.rs` | ~ |
| `server/inducements/bb2020/cards/CustardPieHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/custard_pie_handler.rs` | ~ |
| `server/inducements/bb2020/cards/DistractHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/distract_handler.rs` | ~ |
| `server/inducements/bb2020/cards/ForceShieldHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/force_shield_handler.rs` | ~ |
| `server/inducements/bb2020/cards/IllegalSubstitutionHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/illegal_substitution_handler.rs` | ~ |
| `server/inducements/bb2020/cards/PitTrapHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/pit_trap_handler.rs` | ~ |
| `server/inducements/bb2020/cards/RabbitsFootHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/rabbits_foot_handler.rs` | ~ |
| `server/inducements/bb2020/cards/WitchBrewHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/witch_brew_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/bad_habits_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/blessed_statue_of_nuffle_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/fan_interaction_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/fouling_frenzy_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/friends_with_the_ref_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/greasy_cleats_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/intensive_training_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/iron_man_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/knuckle_dusters_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/moles_under_the_pitch_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/NecessaryViolenceHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/necessary_violence_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/OpponentPlayerSelector.java` | `ffb-engine` | `src/inducements/bb2020/prayers/opponent_player_selector.rs` | ~ |
| `server/inducements/bb2020/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/perfect_passing_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/bb2020/prayers/player_selector.rs` | ~ |
| `server/inducements/bb2020/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/stiletto_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/throw_a_rock_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/treacherous_trapdoor_handler.rs` | ~ |
| `server/inducements/bb2020/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/under_scrutiny_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/bad_habits_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/blessed_statue_of_nuffle_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/DazzlingCatchingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/dazzling_catching_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/fan_interaction_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/fouling_frenzy_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/friends_with_the_ref_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/greasy_cleats_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/intensive_training_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/iron_man_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/knuckle_dusters_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/moles_under_the_pitch_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/OpponentPlayerSelector.java` | `ffb-engine` | `src/inducements/bb2025/prayers/opponent_player_selector.rs` | ~ |
| `server/inducements/bb2025/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/perfect_passing_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/bb2025/prayers/player_selector.rs` | ~ |
| `server/inducements/bb2025/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/stiletto_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/throw_a_rock_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/treacherous_trapdoor_handler.rs` | ~ |
| `server/inducements/bb2025/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/under_scrutiny_handler.rs` | ~ |
| `server/inducements/CardHandler.java` | `ffb-engine` | `src/inducements/card_handler.rs` | ~ |
| `server/inducements/mixed/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/bad_habits_handler.rs` | ~ |
| `server/inducements/mixed/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/blessed_statue_of_nuffle_handler.rs` | ~ |
| `server/inducements/mixed/prayers/DialogPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/dialog_prayer_handler.rs` | ~ |
| `server/inducements/mixed/prayers/EnhancementRemover.java` | `ffb-engine` | `src/inducements/mixed/prayers/enhancement_remover.rs` | ~ |
| `server/inducements/mixed/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/fan_interaction_handler.rs` | ~ |
| `server/inducements/mixed/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/fouling_frenzy_handler.rs` | ~ |
| `server/inducements/mixed/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/friends_with_the_ref_handler.rs` | ~ |
| `server/inducements/mixed/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/greasy_cleats_handler.rs` | ~ |
| `server/inducements/mixed/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/intensive_training_handler.rs` | ~ |
| `server/inducements/mixed/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/iron_man_handler.rs` | ~ |
| `server/inducements/mixed/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/knuckle_dusters_handler.rs` | ~ |
| `server/inducements/mixed/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/moles_under_the_pitch_handler.rs` | ~ |
| `server/inducements/mixed/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/perfect_passing_handler.rs` | ~ |
| `server/inducements/mixed/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/mixed/prayers/player_selector.rs` | ~ |
| `server/inducements/mixed/prayers/PrayerDialogSelection.java` | `ffb-engine` | `src/inducements/mixed/prayers/prayer_dialog_selection.rs` | ~ |
| `server/inducements/mixed/prayers/PrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/prayer_handler.rs` | ~ |
| `server/inducements/mixed/prayers/RandomSelectionPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/random_selection_prayer_handler.rs` | ~ |
| `server/inducements/mixed/prayers/SelectPlayerPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/select_player_prayer_handler.rs` | ~ |
| `server/inducements/mixed/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/stiletto_handler.rs` | ~ |
| `server/inducements/mixed/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/throw_a_rock_handler.rs` | ~ |
| `server/inducements/mixed/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/treacherous_trapdoor_handler.rs` | ~ |
| `server/inducements/mixed/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/under_scrutiny_handler.rs` | ~ |

### server/injury/ (69 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/injury/injuryType/AbstractInjuryTypeBombWithModifier.java` | `ffb-engine` | `src/injury/injuryType/abstract_injury_type_bomb_with_modifier.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBallAndChain.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ball_and_chain.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBitten.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bitten.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlock.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockProne.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_prone.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockProneForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_prone_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockStunned.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_stunned.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockStunnedForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_stunned_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBomb.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBombWithModifier.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb_with_modifier.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBombWithModifierForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb_with_modifier_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBreatheFire.java` | `ffb-engine` | `src/injury/injuryType/injury_type_breathe_fire.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBreatheFireForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_breathe_fire_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeChainsaw.java` | `ffb-engine` | `src/injury/injuryType/injury_type_chainsaw.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeChainsawForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_chainsaw_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowd.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowdPush.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd_push.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowdPushForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd_push_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropDodge.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_dodge.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropDodgeForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_dodge_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropGFI.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_gfi.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropJump.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_jump.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeEatPlayer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_eat_player.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFireball.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fireball.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFoul.java` | `ffb-engine` | `src/injury/injuryType/injury_type_foul.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFoulForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_foul_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFumbledKtm.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fumbled_ktm.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFumbledKtmApoKo.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fumbled_ktm_apo_ko.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKegHit.java` | `ffb-engine` | `src/injury/injuryType/injury_type_keg_hit.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKTMCrowd.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ktm_crowd.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKTMInjury.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ktm_injury.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeLightning.java` | `ffb-engine` | `src/injury/injuryType/injury_type_lightning.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnArmour.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_armour.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnInjury.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_injury.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnKnockedOut.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_knocked_out.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeProjectileVomit.java` | `ffb-engine` | `src/injury/injuryType/injury_type_projectile_vomit.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeQuickBite.java` | `ffb-engine` | `src/injury/injuryType/injury_type_quick_bite.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeSabotaged.java` | `ffb-engine` | `src/injury/injuryType/injury_type_sabotaged.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeSaboteur.java` | `ffb-engine` | `src/injury/injuryType/injury_type_saboteur.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeServer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_server.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeStab.java` | `ffb-engine` | `src/injury/injuryType/injury_type_stab.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeStabForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_stab_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThenIStartedBlastin.java` | `ffb-engine` | `src/injury/injuryType/injury_type_then_i_started_blastin.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThrowARock.java` | `ffb-engine` | `src/injury/injuryType/injury_type_throw_a_rock.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThrowARockStalling.java` | `ffb-engine` | `src/injury/injuryType/injury_type_throw_a_rock_stalling.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTrapDoorFall.java` | `ffb-engine` | `src/injury/injuryType/injury_type_trap_door_fall.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTrapDoorFallForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_trap_door_fall_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMHitPlayer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_hit_player.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMHitPlayerForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_hit_player_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMLanding.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_landing.rs` | ✓ |
| `server/injury/injuryType/ModificationAwareInjuryTypeServer.java` | `ffb-engine` | `src/injury/injuryType/modification_aware_injury_type_server.rs` | ✓ |
| `server/injury/modification/AvOrInjModification.java` | `ffb-engine` | `src/injury/modification/av_or_inj_modification.rs` | ~ |
| `server/injury/modification/bb2020/SlayerModification.java` | `ffb-engine` | `src/injury/modification/bb2020/slayer_modification.rs` | ~ |
| `server/injury/modification/bb2020/ToxinConnoisseurModification.java` | `ffb-engine` | `src/injury/modification/bb2020/toxin_connoisseur_modification.rs` | ~ |
| `server/injury/modification/bb2025/KrumpAndSmashModification.java` | `ffb-engine` | `src/injury/modification/bb2025/krump_and_smash_modification.rs` | ~ |
| `server/injury/modification/bb2025/LoneFoulerModification.java` | `ffb-engine` | `src/injury/modification/bb2025/lone_fouler_modification.rs` | ~ |
| `server/injury/modification/bb2025/MasterAssassinModification.java` | `ffb-engine` | `src/injury/modification/bb2025/master_assassin_modification.rs` | ~ |
| `server/injury/modification/bb2025/RerollArmourModification.java` | `ffb-engine` | `src/injury/modification/bb2025/reroll_armour_modification.rs` | ~ |
| `server/injury/modification/bb2025/SlayerModification.java` | `ffb-engine` | `src/injury/modification/bb2025/slayer_modification.rs` | ~ |
| `server/injury/modification/bb2025/ToxinConnoisseurModification.java` | `ffb-engine` | `src/injury/modification/bb2025/toxin_connoisseur_modification.rs` | ~ |
| `server/injury/modification/BrutalBlockModification.java` | `ffb-engine` | `src/injury/modification/brutal_block_modification.rs` | ~ |
| `server/injury/modification/CrushingBlowModification.java` | `ffb-engine` | `src/injury/modification/crushing_blow_modification.rs` | ~ |
| `server/injury/modification/GhostlyFlamesModification.java` | `ffb-engine` | `src/injury/modification/ghostly_flames_modification.rs` | ~ |
| `server/injury/modification/InjuryContextModification.java` | `ffb-engine` | `src/injury/modification/injury_context_modification.rs` | ~ |
| `server/injury/modification/MasterAssassinModification.java` | `ffb-engine` | `src/injury/modification/master_assassin_modification.rs` | ~ |
| `server/injury/modification/ModificationParams.java` | `ffb-engine` | `src/injury/modification/modification_params.rs` | ~ |
| `server/injury/modification/OldProModification.java` | `ffb-engine` | `src/injury/modification/old_pro_modification.rs` | ~ |
| `server/injury/modification/OldProModificationParams.java` | `ffb-engine` | `src/injury/modification/old_pro_modification_params.rs` | ~ |
| `server/injury/modification/SavageMaulingModification.java` | `ffb-engine` | `src/injury/modification/savage_mauling_modification.rs` | ~ |

### server/marking/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/marking/ApplyTo.java` | `ffb-engine` | `src/marking/apply_to.rs` | ~ |
| `server/marking/AutoMarkingConfig.java` | `ffb-engine` | `src/marking/auto_marking_config.rs` | ~ |
| `server/marking/AutoMarkingRecord.java` | `ffb-engine` | `src/marking/auto_marking_record.rs` | ~ |
| `server/marking/MarkerGenerator.java` | `ffb-engine` | `src/marking/marker_generator.rs` | ~ |

### server/mechanic/ (16 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/mechanic/ArmorModifierValues.java` | `ffb-engine` | `src/mechanic/armor_modifier_values.rs` | ~ |
| `server/mechanic/bb2016/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2016/roll_mechanic.rs` | ~ |
| `server/mechanic/bb2020/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2020/roll_mechanic.rs` | ~ |
| `server/mechanic/bb2025/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/roll_mechanic.rs` | ~ |
| `server/mechanic/bb2025/SetupMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/setup_mechanic.rs` | ~ |
| `server/mechanic/bb2025/StateMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/state_mechanic.rs` | ~ |
| `server/mechanic/CasualtyCalc.java` | `ffb-engine` | `src/mechanic/casualty_calc.rs` | ~ |
| `server/mechanic/InjuryCalc.java` | `ffb-engine` | `src/mechanic/injury_calc.rs` | ~ |
| `server/mechanic/InjuryModifierValues.java` | `ffb-engine` | `src/mechanic/injury_modifier_values.rs` | ~ |
| `server/mechanic/mixed/SetupMechanic.java` | `ffb-engine` | `src/mechanic/mixed/setup_mechanic.rs` | ~ |
| `server/mechanic/mixed/StateMechanic.java` | `ffb-engine` | `src/mechanic/mixed/state_mechanic.rs` | ~ |
| `server/mechanic/RollMechanic.java` | `ffb-engine` | `src/mechanic/roll_mechanic.rs` | ~ |
| `server/mechanic/SetupMechanic.java` | `ffb-engine` | `src/mechanic/setup_mechanic.rs` | ~ |
| `server/mechanic/SppCalc.java` | `ffb-engine` | `src/mechanic/spp_calc.rs` | ~ |
| `server/mechanic/StateMechanic.java` | `ffb-engine` | `src/mechanic/state_mechanic.rs` | ~ |
| `server/mechanic/WeatherModifierValues.java` | `ffb-engine` | `src/mechanic/weather_modifier_values.rs` | ~ |

### server/model/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/model/change/ChompRemovalObserver.java` | `ffb-engine` | `src/model/change/chomp_removal_observer.rs` | ~ |
| `server/model/change/ConditionalModelChangeObserver.java` | `ffb-engine` | `src/model/change/conditional_model_change_observer.rs` | ~ |
| `server/model/DropPlayerContext.java` | `ffb-engine` | `src/model/drop_player_context.rs` | ~ |
| `server/model/DropPlayerContextBuilder.java` | `ffb-engine` | `src/model/drop_player_context_builder.rs` | ~ |
| `server/model/SkillBehaviour.java` | `ffb-engine` | `src/model/skill_behaviour.rs` | ~ |
| `server/model/SteadyFootingContext.java` | `ffb-engine` | `src/model/steady_footing_context.rs` | ~ |
| `server/model/StepModifier.java` | `ffb-engine` | `src/model/step_modifier.rs` | ~ |

### server/net/ (26 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/net/commands/InternalServerCommand.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandAddLoadedTeam.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandApplyAutomatedPlayerMarkings.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandCalculateAutomaticPlayerMarkings.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandClearCache.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandCloseGame.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandDeleteGame.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandFumbblGameChecked.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandFumbblGameCreated.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandFumbblTeamLoaded.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandJoinApproved.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandReplayLoaded.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandScheduleGame.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandSocketClosed.java` | `—` | `—` | — |
| `server/net/commands/InternalServerCommandUploadGame.java` | `—` | `—` | — |
| `server/net/CommandServlet.java` | `—` | `—` | — |
| `server/net/CommandSocket.java` | `—` | `—` | — |
| `server/net/FileServlet.java` | `—` | `—` | — |
| `server/net/ReceivedCommand.java` | `—` | `—` | — |
| `server/net/ReplaySessionManager.java` | `—` | `—` | — |
| `server/net/ServerCommunication.java` | `—` | `—` | — |
| `server/net/ServerDbKeepAliveTask.java` | `—` | `—` | — |
| `server/net/ServerGameTimeTask.java` | `—` | `—` | — |
| `server/net/ServerNetworkEntropyTask.java` | `—` | `—` | — |
| `server/net/SessionManager.java` | `—` | `—` | — |
| `server/net/SessionTimeoutTask.java` | `—` | `—` | — |

### server/request/ (21 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/request/fumbbl/AbstractFumbblRequestLoadPlayerMarkings.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblGameState.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestCheckAuthorization.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestCheckGamestate.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestCreateGamestate.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestLoadPlayerMarkings.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestLoadPlayerMarkingsForGameVersion.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestLoadTeam.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestLoadTeamList.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestPasswordChallenge.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestRemoveGamestate.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestResumeGamestate.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestUpdateGamestate.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestUploadResults.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblRequestUploadTalk.java` | `—` | `—` | — |
| `server/request/fumbbl/FumbblResult.java` | `—` | `—` | — |
| `server/request/fumbbl/UtilFumbblRequest.java` | `—` | `—` | — |
| `server/request/ServerRequest.java` | `—` | `—` | — |
| `server/request/ServerRequestLoadReplay.java` | `—` | `—` | — |
| `server/request/ServerRequestProcessor.java` | `—` | `—` | — |
| `server/request/ServerRequestSaveReplay.java` | `—` | `—` | — |

### server/root/ (31 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/ActionStatus.java` | `ffb-engine` | `src/action_status.rs` | ~ |
| `server/ActiveEffects.java` | `ffb-engine` | `src/active_effects.rs` | ~ |
| `server/CardDeck.java` | `ffb-engine` | `src/card_deck.rs` | ~ |
| `server/DbUpdater.java` | `ffb-engine` | `src/db_updater.rs` | ~ |
| `server/DebugLog.java` | `ffb-engine` | `src/debug_log.rs` | ~ |
| `server/DiceInterpreter.java` | `ffb-engine` | `src/dice_interpreter.rs` | ~ |
| `server/DiceRoller.java` | `ffb-engine` | `src/dice_roller.rs` | ~ |
| `server/FantasyFootballServer.java` | `ffb-engine` | `src/fantasy_football_server.rs` | ~ |
| `server/GameCache.java` | `ffb-engine` | `src/game_cache.rs` | ~ |
| `server/GameLog.java` | `ffb-engine` | `src/game_log.rs` | ~ |
| `server/GameStartMode.java` | `ffb-engine` | `src/game_start_mode.rs` | ~ |
| `server/GameState.java` | `ffb-engine` | `src/game_state.rs` | ~ |
| `server/IdGenerator.java` | `ffb-engine` | `src/id_generator.rs` | ~ |
| `server/IGameIdListener.java` | `ffb-engine` | `src/i_game_id_listener.rs` | ~ |
| `server/InjuryResult.java` | `ffb-engine` | `src/injury_result.rs` | ~ |
| `server/IServerJsonOption.java` | `ffb-engine` | `src/i_server_json_option.rs` | ~ |
| `server/IServerLogLevel.java` | `ffb-engine` | `src/i_server_log_level.rs` | ~ |
| `server/IServerProperty.java` | `ffb-engine` | `src/i_server_property.rs` | ~ |
| `server/PrayerState.java` | `ffb-engine` | `src/prayer_state.rs` | ~ |
| `server/ReplayCache.java` | `ffb-engine` | `src/replay_cache.rs` | ~ |
| `server/ReplayState.java` | `ffb-engine` | `src/replay_state.rs` | ~ |
| `server/RosterCache.java` | `ffb-engine` | `src/roster_cache.rs` | ~ |
| `server/ServerMode.java` | `ffb-engine` | `src/server_mode.rs` | ~ |
| `server/ServerReplay.java` | `ffb-engine` | `src/server_replay.rs` | ~ |
| `server/ServerReplayer.java` | `ffb-engine` | `src/server_replayer.rs` | ~ |
| `server/ServerSketchManager.java` | `ffb-engine` | `src/server_sketch_manager.rs` | ~ |
| `server/ServerUrlProperty.java` | `ffb-engine` | `src/server_url_property.rs` | ~ |
| `server/SessionMode.java` | `ffb-engine` | `src/session_mode.rs` | ~ |
| `server/Talk.java` | `ffb-engine` | `src/talk.rs` | ~ |
| `server/TeamCache.java` | `ffb-engine` | `src/team_cache.rs` | ~ |
| `server/TeamSetupCache.java` | `ffb-engine` | `src/team_setup_cache.rs` | ~ |

### server/skillbehaviour/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/StepHook.java` | `ffb-engine` | `src/skill_behaviour/step_hook.rs` | ~ |

### server/skillbehaviour/bb2016/ (34 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2016/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/agility_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/animosity_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/ArmourIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/armour_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/blood_lust_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/bombardier_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/bone_head_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/catch_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/DauntlessBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dauntless_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/diving_tackle_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dodge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dump_off_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/foul_appearance_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/grab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/JumpUpBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/jump_up_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/LeapBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/leap_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/monstrous_mouth_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/MovementIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/movement_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/pass_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/PilingOnBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/piling_on_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/really_stupid_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/SafeThrowBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/safe_throw_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/shadowing_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/SideStepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/side_step_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/sneaky_git_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/stab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/stand_firm_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/strength_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/SwarmingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/swarming_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/swoop_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/take_root_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/tentacles_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/throw_team_mate_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/WildAnimalBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/wild_animal_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2016/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/wrestle_behaviour.rs` | ~ |

### server/skillbehaviour/bb2020/ (39 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2020/AbstractPassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/abstract_pass_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/agility_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/AnimalSavageryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/animal_savagery_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/animosity_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/blood_lust_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/bombardier_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/bone_head_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/BrutalBlockBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/brutal_block_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/catch_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/CloudBursterBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/cloud_burster_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/diving_tackle_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dodge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dump_off_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/DwarfenScourgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dwarfen_scourge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/foul_appearance_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/GhostlyFlamesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/ghostly_flames_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/grab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/MasterAssassinBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/master_assassin_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/monstrous_mouth_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/pass_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/PassingIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/passing_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/PilingOnBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/piling_on_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/really_stupid_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/shadowing_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/SideStepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/side_step_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/SlayerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/slayer_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/sneaky_git_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/stab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/stand_firm_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/strength_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/SwarmingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/swarming_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/swoop_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/take_root_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/tentacles_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/TheBallistaBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/the_ballista_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/throw_team_mate_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/ToxinConnoisseurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/toxin_connoisseur_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/UnchannelledFuryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/unchannelled_fury_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2020/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/wrestle_behaviour.rs` | ~ |

### server/skillbehaviour/bb2025/ (40 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2025/AbstractPassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/abstract_pass_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/agility_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/AnimalSavageryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/animal_savagery_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/animosity_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/blood_lust_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bombardier_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bone_head_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/BullseyeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bullseye_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/catch_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/diving_tackle_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dodge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dump_off_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/DwarvenScourgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dwarven_scourge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/EyeGougeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/eye_gouge_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/foul_appearance_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/grab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/JuggernautBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/juggernaut_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/KrumpAndSmashBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/krump_and_smash_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/LoneFoulerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/lone_fouler_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/MasterAssassinBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/master_assassin_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/monstrous_mouth_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/pass_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/PassingIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/passing_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/really_stupid_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/SaboteurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/saboteur_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/shadowing_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/SidestepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/sidestep_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/SlayerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/slayer_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/sneaky_git_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/stab_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/stand_firm_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/strength_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/swoop_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/take_root_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/tentacles_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/TheBallistaBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/the_ballista_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/throw_team_mate_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/ToxinConnoisseurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/toxin_connoisseur_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/UnchannelledFuryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/unchannelled_fury_behaviour.rs` | ~ |
| `server/skillbehaviour/bb2025/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/wrestle_behaviour.rs` | ~ |

### server/skillbehaviour/common/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/common/HornsBehaviour.java` | `ffb-engine` | `src/skill_behaviour/common/horns_behaviour.rs` | ~ |

### server/skillbehaviour/mixed/ (14 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/mixed/AbstractDodgingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/abstract_dodging_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/AbstractStepModifierMultipleBlock.java` | `ffb-engine` | `src/skill_behaviour/mixed/abstract_step_modifier_multiple_block.rs` | ~ |
| `server/skillbehaviour/mixed/ArmourIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/armour_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/BlindRageBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/blind_rage_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/CrushingBlowBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/crushing_blow_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/DauntlessBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/dauntless_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/IndomitableBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/indomitable_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/JuggernautBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/juggernaut_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/JumpUpBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/jump_up_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/MovementIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/movement_increase_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/OldProBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/old_pro_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/RamBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/ram_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/SavageMaulingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/savage_mauling_behaviour.rs` | ~ |
| `server/skillbehaviour/mixed/WatchOutBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/watch_out_behaviour.rs` | ~ |

### server/step/ (23 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/AbstractStep.java` | `ffb-engine` | `src/step/abstract_step.rs` | ~ |
| `server/step/AbstractStepWithReRoll.java` | `ffb-engine` | `src/step/abstract_step_with_re_roll.rs` | ~ |
| `server/step/DeferredCommand.java` | `ffb-engine` | `src/step/deferred_command.rs` | ~ |
| `server/step/DeferredCommandId.java` | `ffb-engine` | `src/step/deferred_command_id.rs` | ~ |
| `server/step/HasIdForSingleUseReRoll.java` | `ffb-engine` | `src/step/has_id_for_single_use_re_roll.rs` | ~ |
| `server/step/IStackModifier.java` | `ffb-engine` | `src/step/i_stack_modifier.rs` | ~ |
| `server/step/IStep.java` | `ffb-engine` | `src/step/i_step.rs` | ~ |
| `server/step/IStepLabel.java` | `ffb-engine` | `src/step/i_step_label.rs` | ~ |
| `server/step/StepAction.java` | `ffb-engine` | `src/step/step_action.rs` | ~ |
| `server/step/StepCommandStatus.java` | `ffb-engine` | `src/step/step_command_status.rs` | ~ |
| `server/step/StepException.java` | `ffb-engine` | `src/step/step_exception.rs` | ~ |
| `server/step/StepFactory.java` | `ffb-engine` | `src/step/step_factory.rs` | ~ |
| `server/step/StepGotoLabel.java` | `ffb-engine` | `src/step/step_goto_label.rs` | ~ |
| `server/step/StepId.java` | `ffb-engine` | `src/step/step_id.rs` | ~ |
| `server/step/StepNextStep.java` | `ffb-engine` | `src/step/step_next_step.rs` | ~ |
| `server/step/StepNextStepAndRepeat.java` | `ffb-engine` | `src/step/step_next_step_and_repeat.rs` | ~ |
| `server/step/StepParameter.java` | `ffb-engine` | `src/step/step_parameter.rs` | ~ |
| `server/step/StepParameterKey.java` | `ffb-engine` | `src/step/step_parameter_key.rs` | ~ |
| `server/step/StepParameterSet.java` | `ffb-engine` | `src/step/step_parameter_set.rs` | ~ |
| `server/step/StepResetToMove.java` | `ffb-engine` | `src/step/step_reset_to_move.rs` | ~ |
| `server/step/StepResult.java` | `ffb-engine` | `src/step/step_result.rs` | ~ |
| `server/step/StepStack.java` | `ffb-engine` | `src/step/step_stack.rs` | ~ |
| `server/step/UtilServerSteps.java` | `ffb-engine` | `src/step/util_server_steps.rs` | ~ |

### server/step/action/ (24 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/action/block/StepBlockStatistics.java` | `ffb-engine` | `src/step/action/block/step_block_statistics.rs` | ✓ |
| `server/step/action/block/StepDauntless.java` | `ffb-engine` | `src/step/action/block/step_dauntless.rs` | ✓ |
| `server/step/action/block/StepDropFallingPlayers.java` | `ffb-engine` | `src/step/action/block/step_drop_falling_players.rs` | ✓ |
| `server/step/action/block/StepDumpOff.java` | `ffb-engine` | `src/step/action/block/step_dump_off.rs` | ✓ |
| `server/step/action/block/StepHorns.java` | `ffb-engine` | `src/step/action/block/step_horns.rs` | ✓ |
| `server/step/action/block/StepJuggernaut.java` | `ffb-engine` | `src/step/action/block/step_juggernaut.rs` | ✓ |
| `server/step/action/block/StepStab.java` | `ffb-engine` | `src/step/action/block/step_stab.rs` | ✓ |
| `server/step/action/block/StepWrestle.java` | `ffb-engine` | `src/step/action/block/step_wrestle.rs` | ✓ |
| `server/step/action/block/UtilBlockSequence.java` | `ffb-engine` | `src/step/action/block/util_block_sequence.rs` | ✓ |
| `server/step/action/common/StepBoneHead.java` | `ffb-engine` | `src/step/action/common/step_bone_head.rs` | ✓ |
| `server/step/action/common/StepReallyStupid.java` | `ffb-engine` | `src/step/action/common/step_really_stupid.rs` | ✓ |
| `server/step/action/foul/StepReferee.java` | `ffb-engine` | `src/step/action/foul/step_referee.rs` | ✓ |
| `server/step/action/ktm/StepEndKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_end_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepInitKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_init_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepKickTeamMateDoubleRolled.java` | `ffb-engine` | `src/step/action/ktm/step_kick_team_mate_double_rolled.rs` | ✓ |
| `server/step/action/move/StepDivingTackle.java` | `ffb-engine` | `src/step/action/move/step_diving_tackle.rs` | ✓ |
| `server/step/action/pass/StepAnimosity.java` | `ffb-engine` | `src/step/action/pass/step_animosity.rs` | ✓ |
| `server/step/action/pass/StepBombardier.java` | `ffb-engine` | `src/step/action/pass/step_bombardier.rs` | ✓ |
| `server/step/action/pass/StepDispatchPassing.java` | `ffb-engine` | `src/step/action/pass/step_dispatch_passing.rs` | ✓ |
| `server/step/action/pass/StepHandOver.java` | `ffb-engine` | `src/step/action/pass/step_hand_over.rs` | ✓ |
| `server/step/action/select/StepJumpUp.java` | `ffb-engine` | `src/step/action/select/step_jump_up.rs` | ✓ |
| `server/step/action/ttm/StepEatTeamMate.java` | `ffb-engine` | `src/step/action/ttm/step_eat_team_mate.rs` | ✓ |
| `server/step/action/ttm/UtilThrowTeamMateSequence.java` | `ffb-engine` | `src/step/action/ttm/util_throw_team_mate_sequence.rs` | ✓ |

### server/step/bb2016/ (78 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2016/block/StepBlockBallAndChain.java` | `ffb-engine` | `src/step/bb2016/block/step_block_ball_and_chain.rs` | ✓ |
| `server/step/bb2016/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2016/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2016/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2016/block/step_block_choice.rs` | ✓ |
| `server/step/bb2016/block/StepBlockDodge.java` | `ffb-engine` | `src/step/bb2016/block/step_block_dodge.rs` | ✓ |
| `server/step/bb2016/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2016/block/step_block_roll.rs` | ✓ |
| `server/step/bb2016/block/StepBothDown.java` | `ffb-engine` | `src/step/bb2016/block/step_both_down.rs` | ✓ |
| `server/step/bb2016/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2016/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2016/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2016/block/step_followup.rs` | ✓ |
| `server/step/bb2016/end/StepFanFactor.java` | `ffb-engine` | `src/step/bb2016/end/step_fan_factor.rs` | ✓ |
| `server/step/bb2016/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2016/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2016/end/StepMvp.java` | `ffb-engine` | `src/step/bb2016/end/step_mvp.rs` | ✓ |
| `server/step/bb2016/end/StepPenaltyShootout.java` | `ffb-engine` | `src/step/bb2016/end/step_penalty_shootout.rs` | ✓ |
| `server/step/bb2016/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2016/end/step_player_loss.rs` | ✓ |
| `server/step/bb2016/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2016/end/step_winnings.rs` | ✓ |
| `server/step/bb2016/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2016/foul/step_bribes.rs` | ✓ |
| `server/step/bb2016/foul/StepEjectPlayer.java` | `ffb-engine` | `src/step/bb2016/foul/step_eject_player.rs` | ✓ |
| `server/step/bb2016/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2016/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2016/foul/StepFoul.java` | `ffb-engine` | `src/step/bb2016/foul/step_foul.rs` | ✓ |
| `server/step/bb2016/foul/StepFoulChainsaw.java` | `ffb-engine` | `src/step/bb2016/foul/step_foul_chainsaw.rs` | ✓ |
| `server/step/bb2016/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2016/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2016/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2016/move/step_end_moving.rs` | ✓ |
| `server/step/bb2016/move/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2016/move/step_end_selecting.rs` | ✓ |
| `server/step/bb2016/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2016/move/step_go_for_it.rs` | ✓ |
| `server/step/bb2016/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2016/move/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2016/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2016/move/step_init_moving.rs` | ✓ |
| `server/step/bb2016/move/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2016/move/step_init_selecting.rs` | ✓ |
| `server/step/bb2016/move/StepJump.java` | `ffb-engine` | `src/step/bb2016/move/step_jump.rs` | ✓ |
| `server/step/bb2016/move/StepMove.java` | `ffb-engine` | `src/step/bb2016/move/step_move.rs` | ✓ |
| `server/step/bb2016/move/StepMoveBallAndChain.java` | `ffb-engine` | `src/step/bb2016/move/step_move_ball_and_chain.rs` | ✓ |
| `server/step/bb2016/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2016/move/step_move_dodge.rs` | ✓ |
| `server/step/bb2016/move/StepTentacles.java` | `ffb-engine` | `src/step/bb2016/move/step_tentacles.rs` | ✓ |
| `server/step/bb2016/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2016/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2016/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepInitPassing.java` | `ffb-engine` | `src/step/bb2016/pass/step_init_passing.rs` | ✓ |
| `server/step/bb2016/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2016/pass/step_intercept.rs` | ✓ |
| `server/step/bb2016/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepPassBlock.java` | `ffb-engine` | `src/step/bb2016/pass/step_pass_block.rs` | ✓ |
| `server/step/bb2016/pass/StepSafeThrow.java` | `ffb-engine` | `src/step/bb2016/pass/step_safe_throw.rs` | ✓ |
| `server/step/bb2016/special/StepEndBomb.java` | `ffb-engine` | `src/step/bb2016/special/step_end_bomb.rs` | ✓ |
| `server/step/bb2016/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2016/special/step_init_bomb.rs` | ✓ |
| `server/step/bb2016/start/StepBuyCards.java` | `ffb-engine` | `src/step/bb2016/start/step_buy_cards.rs` | ✓ |
| `server/step/bb2016/start/StepBuyInducements.java` | `ffb-engine` | `src/step/bb2016/start/step_buy_inducements.rs` | ✓ |
| `server/step/bb2016/start/StepPettyCash.java` | `ffb-engine` | `src/step/bb2016/start/step_petty_cash.rs` | ✓ |
| `server/step/bb2016/start/StepSpectators.java` | `ffb-engine` | `src/step/bb2016/start/step_spectators.rs` | ✓ |
| `server/step/bb2016/StepApothecary.java` | `ffb-engine` | `src/step/bb2016/step_apothecary.rs` | ✓ |
| `server/step/bb2016/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2016/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2016/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2016/step_blitz_turn.rs` | ✓ |
| `server/step/bb2016/StepBloodLust.java` | `ffb-engine` | `src/step/bb2016/step_blood_lust.rs` | ✓ |
| `server/step/bb2016/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2016/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2016/StepDropDivingTackler.java` | `ffb-engine` | `src/step/bb2016/step_drop_diving_tackler.rs` | ✓ |
| `server/step/bb2016/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2016/step_end_feeding.rs` | ✓ |
| `server/step/bb2016/StepEndInducement.java` | `ffb-engine` | `src/step/bb2016/step_end_inducement.rs` | ✓ |
| `server/step/bb2016/StepEndTurn.java` | `ffb-engine` | `src/step/bb2016/step_end_turn.rs` | ✓ |
| `server/step/bb2016/StepFallDown.java` | `ffb-engine` | `src/step/bb2016/step_fall_down.rs` | ✓ |
| `server/step/bb2016/StepFoulAppearance.java` | `ffb-engine` | `src/step/bb2016/step_foul_appearance.rs` | ✓ |
| `server/step/bb2016/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2016/step_init_blocking.rs` | ✓ |
| `server/step/bb2016/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2016/step_init_feeding.rs` | ✓ |
| `server/step/bb2016/StepInitInducement.java` | `ffb-engine` | `src/step/bb2016/step_init_inducement.rs` | ✓ |
| `server/step/bb2016/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2016/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2016/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2016/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2016/StepPickUp.java` | `ffb-engine` | `src/step/bb2016/step_pick_up.rs` | ✓ |
| `server/step/bb2016/StepPushback.java` | `ffb-engine` | `src/step/bb2016/step_pushback.rs` | ✓ |
| `server/step/bb2016/StepSetup.java` | `ffb-engine` | `src/step/bb2016/step_setup.rs` | ✓ |
| `server/step/bb2016/StepShadowing.java` | `ffb-engine` | `src/step/bb2016/step_shadowing.rs` | ✓ |
| `server/step/bb2016/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2016/step_special_effect.rs` | ✓ |
| `server/step/bb2016/StepStandUp.java` | `ffb-engine` | `src/step/bb2016/step_stand_up.rs` | ✓ |
| `server/step/bb2016/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2016/step_take_root.rs` | ✓ |
| `server/step/bb2016/StepWildAnimal.java` | `ffb-engine` | `src/step/bb2016/step_wild_animal.rs` | ✓ |
| `server/step/bb2016/StepWizard.java` | `ffb-engine` | `src/step/bb2016/step_wizard.rs` | ✓ |
| `server/step/bb2016/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2016/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2016/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2016/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2016/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2016/ttm/StepFumbleTtmPass.java` | `ffb-engine` | `src/step/bb2016/ttm/step_fumble_ttm_pass.rs` | ✓ |
| `server/step/bb2016/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2016/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2016/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2016/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2016/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2016/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/bb2020/ (89 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2020/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2020/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2020/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2020/block/step_block_choice.rs` | ✓ |
| `server/step/bb2020/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2020/block/step_block_roll.rs` | ✓ |
| `server/step/bb2020/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2020/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2020/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2020/block/step_followup.rs` | ✓ |
| `server/step/bb2020/block/StepHitAndRun.java` | `ffb-engine` | `src/step/bb2020/block/step_hit_and_run.rs` | ✓ |
| `server/step/bb2020/block/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2020/block/step_init_blocking.rs` | ✓ |
| `server/step/bb2020/block/StepPushback.java` | `ffb-engine` | `src/step/bb2020/block/step_pushback.rs` | ✓ |
| `server/step/bb2020/block/StepTrickster.java` | `ffb-engine` | `src/step/bb2020/block/step_trickster.rs` | ✓ |
| `server/step/bb2020/end/StepAssignTouchdowns.java` | `ffb-engine` | `src/step/bb2020/end/step_assign_touchdowns.rs` | ✓ |
| `server/step/bb2020/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2020/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2020/end/StepMvp.java` | `ffb-engine` | `src/step/bb2020/end/step_mvp.rs` | ✓ |
| `server/step/bb2020/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2020/end/step_player_loss.rs` | ✓ |
| `server/step/bb2020/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2020/end/step_winnings.rs` | ✓ |
| `server/step/bb2020/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2020/foul/step_bribes.rs` | ✓ |
| `server/step/bb2020/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2020/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2020/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2020/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2020/gaze/StepSelectGazeTarget.java` | `ffb-engine` | `src/step/bb2020/gaze/step_select_gaze_target.rs` | ✓ |
| `server/step/bb2020/gaze/StepSelectGazeTargetEnd.java` | `ffb-engine` | `src/step/bb2020/gaze/step_select_gaze_target_end.rs` | ✓ |
| `server/step/bb2020/inducements/StepEndInducement.java` | `ffb-engine` | `src/step/bb2020/inducements/step_end_inducement.rs` | ✓ |
| `server/step/bb2020/inducements/StepInitInducement.java` | `ffb-engine` | `src/step/bb2020/inducements/step_init_inducement.rs` | ✓ |
| `server/step/bb2020/inducements/StepWeatherMage.java` | `ffb-engine` | `src/step/bb2020/inducements/step_weather_mage.rs` | ✓ |
| `server/step/bb2020/kickoff/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2020/kickoff/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2020/kickoff/StepSetup.java` | `ffb-engine` | `src/step/bb2020/kickoff/step_setup.rs` | ✓ |
| `server/step/bb2020/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2020/move/step_end_moving.rs` | ✓ |
| `server/step/bb2020/move/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2020/move/step_end_selecting.rs` | ✓ |
| `server/step/bb2020/move/StepFallDown.java` | `ffb-engine` | `src/step/bb2020/move/step_fall_down.rs` | ✓ |
| `server/step/bb2020/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2020/move/step_go_for_it.rs` | ✓ |
| `server/step/bb2020/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2020/move/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2020/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2020/move/step_init_moving.rs` | ✓ |
| `server/step/bb2020/move/StepJump.java` | `ffb-engine` | `src/step/bb2020/move/step_jump.rs` | ✓ |
| `server/step/bb2020/move/StepMove.java` | `ffb-engine` | `src/step/bb2020/move/step_move.rs` | ✓ |
| `server/step/bb2020/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2020/move/step_move_dodge.rs` | ✓ |
| `server/step/bb2020/move/StepPickUp.java` | `ffb-engine` | `src/step/bb2020/move/step_pick_up.rs` | ✓ |
| `server/step/bb2020/move/StepShadowing.java` | `ffb-engine` | `src/step/bb2020/move/step_shadowing.rs` | ✓ |
| `server/step/bb2020/move/StepStandUp.java` | `ffb-engine` | `src/step/bb2020/move/step_stand_up.rs` | ✓ |
| `server/step/bb2020/multiblock/StepApothecaryMultiple.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_apothecary_multiple.rs` | ✓ |
| `server/step/bb2020/multiblock/StepBlockRollMultiple.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_block_roll_multiple.rs` | ✓ |
| `server/step/bb2020/multiblock/StepMultipleBlockFork.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_multiple_block_fork.rs` | ✓ |
| `server/step/bb2020/multiblock/StepReportStabInjury.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_report_stab_injury.rs` | ✓ |
| `server/step/bb2020/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2020/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2020/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2020/pass/step_intercept.rs` | ✓ |
| `server/step/bb2020/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepResolvePass.java` | `ffb-engine` | `src/step/bb2020/pass/step_resolve_pass.rs` | ✓ |
| `server/step/bb2020/shared/StepBloodLust.java` | `ffb-engine` | `src/step/bb2020/shared/step_blood_lust.rs` | ✓ |
| `server/step/bb2020/shared/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2020/shared/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2020/shared/StepCheckStalling.java` | `ffb-engine` | `src/step/bb2020/shared/step_check_stalling.rs` | ✓ |
| `server/step/bb2020/shared/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2020/shared/step_end_feeding.rs` | ✓ |
| `server/step/bb2020/shared/StepInitActivation.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_activation.rs` | ✓ |
| `server/step/bb2020/shared/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_feeding.rs` | ✓ |
| `server/step/bb2020/shared/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_selecting.rs` | ✓ |
| `server/step/bb2020/shared/StepPlaceBall.java` | `ffb-engine` | `src/step/bb2020/shared/step_place_ball.rs` | ✓ |
| `server/step/bb2020/shared/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2020/shared/step_take_root.rs` | ✓ |
| `server/step/bb2020/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2020/special/step_init_bomb.rs` | ~ |
| `server/step/bb2020/start/StepBuyCardsAndInducements.java` | `ffb-engine` | `src/step/bb2020/start/step_buy_cards_and_inducements.rs` | ✓ |
| `server/step/bb2020/StepApothecary.java` | `ffb-engine` | `src/step/bb2020/step_apothecary.rs` | ~ |
| `server/step/bb2020/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2020/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2020/StepBalefulHex.java` | `ffb-engine` | `src/step/bb2020/step_baleful_hex.rs` | ✓ |
| `server/step/bb2020/StepBlackInk.java` | `ffb-engine` | `src/step/bb2020/step_black_ink.rs` | ✓ |
| `server/step/bb2020/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2020/step_blitz_turn.rs` | ✓ |
| `server/step/bb2020/StepBreatheFire.java` | `ffb-engine` | `src/step/bb2020/step_breathe_fire.rs` | ✓ |
| `server/step/bb2020/StepCatchOfTheDay.java` | `ffb-engine` | `src/step/bb2020/step_catch_of_the_day.rs` | ✓ |
| `server/step/bb2020/StepEndFuriousOutburst.java` | `ffb-engine` | `src/step/bb2020/step_end_furious_outburst.rs` | ✓ |
| `server/step/bb2020/StepEndTurn.java` | `ffb-engine` | `src/step/bb2020/step_end_turn.rs` | ✓ |
| `server/step/bb2020/StepHandleDropPlayerContext.java` | `ffb-engine` | `src/step/bb2020/step_handle_drop_player_context.rs` | ✓ |
| `server/step/bb2020/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2020/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2020/StepLookIntoMyEyes.java` | `ffb-engine` | `src/step/bb2020/step_look_into_my_eyes.rs` | ✓ |
| `server/step/bb2020/StepPrayer.java` | `ffb-engine` | `src/step/bb2020/step_prayer.rs` | ✓ |
| `server/step/bb2020/StepPrayers.java` | `ffb-engine` | `src/step/bb2020/step_prayers.rs` | ✓ |
| `server/step/bb2020/StepRaidingParty.java` | `ffb-engine` | `src/step/bb2020/step_raiding_party.rs` | ✓ |
| `server/step/bb2020/StepSelectBlitzTarget.java` | `ffb-engine` | `src/step/bb2020/step_select_blitz_target.rs` | ✓ |
| `server/step/bb2020/StepSetActingPlayerAndTeam.java` | `ffb-engine` | `src/step/bb2020/step_set_acting_player_and_team.rs` | ✓ |
| `server/step/bb2020/StepSetActingTeam.java` | `ffb-engine` | `src/step/bb2020/step_set_acting_team.rs` | ✓ |
| `server/step/bb2020/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2020/step_special_effect.rs` | ✓ |
| `server/step/bb2020/StepStallingPlayer.java` | `ffb-engine` | `src/step/bb2020/step_stalling_player.rs` | ✓ |
| `server/step/bb2020/StepStateMultipleRolls.java` | `ffb-engine` | `src/step/bb2020/step_state_multiple_rolls.rs` | ✓ |
| `server/step/bb2020/StepThenIStartedBlastin.java` | `ffb-engine` | `src/step/bb2020/step_then_i_started_blastin.rs` | ✓ |
| `server/step/bb2020/StepTreacherous.java` | `ffb-engine` | `src/step/bb2020/step_treacherous.rs` | ✓ |
| `server/step/bb2020/StepWisdomOfTheWhiteDwarf.java` | `ffb-engine` | `src/step/bb2020/step_wisdom_of_the_white_dwarf.rs` | ✓ |
| `server/step/bb2020/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2020/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2020/ttm/StepDispatchScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_dispatch_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2020/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2020/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2020/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2020/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/bb2025/ (109 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2025/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2025/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2025/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2025/block/step_block_choice.rs` | ✓ |
| `server/step/bb2025/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2025/block/step_block_roll.rs` | ✓ |
| `server/step/bb2025/block/StepBreatheFire.java` | `ffb-engine` | `src/step/bb2025/block/step_breathe_fire.rs` | ✓ |
| `server/step/bb2025/block/StepChomp.java` | `ffb-engine` | `src/step/bb2025/block/step_chomp.rs` | ✓ |
| `server/step/bb2025/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2025/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2025/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2025/block/step_followup.rs` | ✓ |
| `server/step/bb2025/block/StepHitAndRun.java` | `ffb-engine` | `src/step/bb2025/block/step_hit_and_run.rs` | ✓ |
| `server/step/bb2025/block/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2025/block/step_init_blocking.rs` | ✓ |
| `server/step/bb2025/block/StepPushback.java` | `ffb-engine` | `src/step/bb2025/block/step_pushback.rs` | ✓ |
| `server/step/bb2025/block/StepTrickster.java` | `ffb-engine` | `src/step/bb2025/block/step_trickster.rs` | ✓ |
| `server/step/bb2025/command/AnimalSavageryCancelActionCommand.java` | `ffb-engine` | `src/step/bb2025/command/animal_savagery_cancel_action_command.rs` | ✓ |
| `server/step/bb2025/command/AnimalSavageryControlCommand.java` | `ffb-engine` | `src/step/bb2025/command/animal_savagery_control_command.rs` | ✓ |
| `server/step/bb2025/command/DropPlayerCommand.java` | `ffb-engine` | `src/step/bb2025/command/drop_player_command.rs` | ✓ |
| `server/step/bb2025/command/DropPlayerFromBombCommand.java` | `ffb-engine` | `src/step/bb2025/command/drop_player_from_bomb_command.rs` | ✓ |
| `server/step/bb2025/command/HitPlayerTurnOverCommand.java` | `ffb-engine` | `src/step/bb2025/command/hit_player_turn_over_command.rs` | ✓ |
| `server/step/bb2025/command/RightStuffCommand.java` | `ffb-engine` | `src/step/bb2025/command/right_stuff_command.rs` | ✓ |
| `server/step/bb2025/command/StandingUpCommand.java` | `ffb-engine` | `src/step/bb2025/command/standing_up_command.rs` | ✓ |
| `server/step/bb2025/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2025/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2025/end/StepMvp.java` | `ffb-engine` | `src/step/bb2025/end/step_mvp.rs` | ✓ |
| `server/step/bb2025/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2025/end/step_player_loss.rs` | ✓ |
| `server/step/bb2025/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2025/end/step_winnings.rs` | ✓ |
| `server/step/bb2025/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2025/foul/step_bribes.rs` | ✓ |
| `server/step/bb2025/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2025/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2025/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2025/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2025/inducements/StepEndInducement.java` | `ffb-engine` | `src/step/bb2025/inducements/step_end_inducement.rs` | ✓ |
| `server/step/bb2025/inducements/StepInitInducement.java` | `ffb-engine` | `src/step/bb2025/inducements/step_init_inducement.rs` | ✓ |
| `server/step/bb2025/inducements/StepThrowARock.java` | `ffb-engine` | `src/step/bb2025/inducements/step_throw_a_rock.rs` | ✓ |
| `server/step/bb2025/inducements/StepWeatherMage.java` | `ffb-engine` | `src/step/bb2025/inducements/step_weather_mage.rs` | ✓ |
| `server/step/bb2025/kickoff/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2025/kickoff/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_blitz_turn.rs` | ✓ |
| `server/step/bb2025/kickoff/StepInitKickoff.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_init_kickoff.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoff.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffScatterRollAskAfter.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_scatter_roll_ask_after.rs` | ✓ |
| `server/step/bb2025/kickoff/StepSetup.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_setup.rs` | ✓ |
| `server/step/bb2025/kickoff/StepSwarming.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_swarming.rs` | ✓ |
| `server/step/bb2025/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2025/move_/step_end_moving.rs` | ✓ |
| `server/step/bb2025/move/StepFallDown.java` | `ffb-engine` | `src/step/bb2025/move_/step_fall_down.rs` | ✓ |
| `server/step/bb2025/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2025/move_/step_go_for_it.rs` | ✓ |
| `server/step/bb2025/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2025/move_/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2025/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2025/move_/step_init_moving.rs` | ✓ |
| `server/step/bb2025/move/StepJump.java` | `ffb-engine` | `src/step/bb2025/move_/step_jump.rs` | ✓ |
| `server/step/bb2025/move/StepMove.java` | `ffb-engine` | `src/step/bb2025/move_/step_move.rs` | ✓ |
| `server/step/bb2025/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2025/move_/step_move_dodge.rs` | ✓ |
| `server/step/bb2025/move/StepPickUp.java` | `ffb-engine` | `src/step/bb2025/move_/step_pick_up.rs` | ✓ |
| `server/step/bb2025/move/StepShadowing.java` | `ffb-engine` | `src/step/bb2025/move_/step_shadowing.rs` | ✓ |
| `server/step/bb2025/move/StepStandUp.java` | `ffb-engine` | `src/step/bb2025/move_/step_stand_up.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepApothecaryMultiple.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_apothecary_multiple.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepBlockRollMultiple.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_block_roll_multiple.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepMultipleBlockFork.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_multiple_block_fork.rs` | ✓ |
| `server/step/bb2025/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2025/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2025/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepHandOver.java` | `ffb-engine` | `src/step/bb2025/pass/step_hand_over.rs` | ✓ |
| `server/step/bb2025/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2025/pass/step_intercept.rs` | ✓ |
| `server/step/bb2025/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepResolvePass.java` | `ffb-engine` | `src/step/bb2025/pass/step_resolve_pass.rs` | ✓ |
| `server/step/bb2025/punt/StepEndPunt.java` | `ffb-engine` | `src/step/bb2025/punt/step_end_punt.rs` | ✓ |
| `server/step/bb2025/punt/StepInitPunt.java` | `ffb-engine` | `src/step/bb2025/punt/step_init_punt.rs` | ✓ |
| `server/step/bb2025/punt/StepPuntDirection.java` | `ffb-engine` | `src/step/bb2025/punt/step_punt_direction.rs` | ✓ |
| `server/step/bb2025/punt/StepPuntDistance.java` | `ffb-engine` | `src/step/bb2025/punt/step_punt_distance.rs` | ✓ |
| `server/step/bb2025/shared/StallingExtension.java` | `ffb-engine` | `src/step/bb2025/shared/stalling_extension.rs` | ✓ |
| `server/step/bb2025/shared/StepApothecary.java` | `ffb-engine` | `src/step/bb2025/shared/step_apothecary.rs` | ✓ |
| `server/step/bb2025/shared/StepBloodLust.java` | `ffb-engine` | `src/step/bb2025/shared/step_blood_lust.rs` | ✓ |
| `server/step/bb2025/shared/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2025/shared/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2025/shared/StepDropFallingPlayers.java` | `ffb-engine` | `src/step/bb2025/shared/step_drop_falling_players.rs` | ✓ |
| `server/step/bb2025/shared/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2025/shared/step_end_feeding.rs` | ✓ |
| `server/step/bb2025/shared/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2025/shared/step_end_selecting.rs` | ✓ |
| `server/step/bb2025/shared/StepForgoneStalling.java` | `ffb-engine` | `src/step/bb2025/shared/step_forgone_stalling.rs` | ✓ |
| `server/step/bb2025/shared/StepGettingEven.java` | `ffb-engine` | `src/step/bb2025/shared/step_getting_even.rs` | ✓ |
| `server/step/bb2025/shared/StepHandleDropPlayerContext.java` | `ffb-engine` | `src/step/bb2025/shared/step_handle_drop_player_context.rs` | ✓ |
| `server/step/bb2025/shared/StepInitActivation.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_activation.rs` | ✓ |
| `server/step/bb2025/shared/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_feeding.rs` | ✓ |
| `server/step/bb2025/shared/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_selecting.rs` | ✓ |
| `server/step/bb2025/shared/StepPlaceBall.java` | `ffb-engine` | `src/step/bb2025/shared/step_place_ball.rs` | ✓ |
| `server/step/bb2025/shared/StepStallingPlayer.java` | `ffb-engine` | `src/step/bb2025/shared/step_stalling_player.rs` | ✓ |
| `server/step/bb2025/shared/StepSteadyFooting.java` | `ffb-engine` | `src/step/bb2025/shared/step_steady_footing.rs` | ✓ |
| `server/step/bb2025/shared/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2025/shared/step_take_root.rs` | ✓ |
| `server/step/bb2025/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2025/special/step_init_bomb.rs` | ✓ |
| `server/step/bb2025/special/StepRecheckExplodeSkill.java` | `ffb-engine` | `src/step/bb2025/special/step_recheck_explode_skill.rs` | ✓ |
| `server/step/bb2025/special/StepResolveBomb.java` | `ffb-engine` | `src/step/bb2025/special/step_resolve_bomb.rs` | ✓ |
| `server/step/bb2025/special/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2025/special/step_special_effect.rs` | ✓ |
| `server/step/bb2025/start/StepBuyInducements.java` | `ffb-engine` | `src/step/bb2025/start/step_buy_inducements.rs` | ✓ |
| `server/step/bb2025/start/StepMasterChef.java` | `ffb-engine` | `src/step/bb2025/start/step_master_chef.rs` | ✓ |
| `server/step/bb2025/start/StepPrayers.java` | `ffb-engine` | `src/step/bb2025/start/step_prayers.rs` | ✓ |
| `server/step/bb2025/StepAutoGazeZoat.java` | `ffb-engine` | `src/step/bb2025/step_auto_gaze_zoat.rs` | ✓ |
| `server/step/bb2025/StepBalefulHex.java` | `ffb-engine` | `src/step/bb2025/step_baleful_hex.rs` | ✓ |
| `server/step/bb2025/StepBlackInk.java` | `ffb-engine` | `src/step/bb2025/step_black_ink.rs` | ✓ |
| `server/step/bb2025/StepCatchOfTheDay.java` | `ffb-engine` | `src/step/bb2025/step_catch_of_the_day.rs` | ✓ |
| `server/step/bb2025/StepEndFuriousOutburst.java` | `ffb-engine` | `src/step/bb2025/step_end_furious_outburst.rs` | ✓ |
| `server/step/bb2025/StepEndTurn.java` | `ffb-engine` | `src/step/bb2025/step_end_turn.rs` | ✓ |
| `server/step/bb2025/StepLookIntoMyEyes.java` | `ffb-engine` | `src/step/bb2025/step_look_into_my_eyes.rs` | ✓ |
| `server/step/bb2025/StepPrayer.java` | `ffb-engine` | `src/step/bb2025/step_prayer.rs` | ✓ |
| `server/step/bb2025/StepRaidingParty.java` | `ffb-engine` | `src/step/bb2025/step_raiding_party.rs` | ✓ |
| `server/step/bb2025/StepSelectBlitzTarget.java` | `ffb-engine` | `src/step/bb2025/step_select_blitz_target.rs` | ✓ |
| `server/step/bb2025/StepThenIStartedBlastin.java` | `ffb-engine` | `src/step/bb2025/step_then_i_started_blastin.rs` | ✓ |
| `server/step/bb2025/StepTreacherous.java` | `ffb-engine` | `src/step/bb2025/step_treacherous.rs` | ✓ |
| `server/step/bb2025/StepWisdomOfTheWhiteDwarf.java` | `ffb-engine` | `src/step/bb2025/step_wisdom_of_the_white_dwarf.rs` | ✓ |
| `server/step/bb2025/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2025/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2025/ttm/StepDispatchScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_dispatch_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2025/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2025/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2025/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2025/ttm/StepSwoop.java` | `ffb-engine` | `src/step/bb2025/ttm/step_swoop.rs` | ✓ |
| `server/step/bb2025/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/game/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/game/end/StepEndGame.java` | `ffb-engine` | `src/step/game/end/step_end_game.rs` | ~ |
| `server/step/game/start/StepInitStartGame.java` | `ffb-engine` | `src/step/game/start/step_init_start_game.rs` | ~ |
| `server/step/game/start/StepWeather.java` | `ffb-engine` | `src/step/game/start/step_weather.rs` | ~ |
| `server/step/game/start/UtilInducementSequence.java` | `ffb-engine` | `src/step/game/start/util_inducement_sequence.rs` | ~ |

### server/step/generator/ (114 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/generator/AutoGazeZoat.java` | `ffb-engine` | `src/step/generator/auto_gaze_zoat.rs` | ~ |
| `server/step/generator/BalefulHex.java` | `ffb-engine` | `src/step/generator/baleful_hex.rs` | ~ |
| `server/step/generator/bb2016/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2016/blitz_block.rs` | ✓ |
| `server/step/generator/bb2016/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2016/blitz_move.rs` | ✓ |
| `server/step/generator/bb2016/Block.java` | `ffb-engine` | `src/step/generator/bb2016/block.rs` | ✓ |
| `server/step/generator/bb2016/Bomb.java` | `ffb-engine` | `src/step/generator/bb2016/bomb.rs` | ✓ |
| `server/step/generator/bb2016/EndGame.java` | `ffb-engine` | `src/step/generator/bb2016/end_game.rs` | ✓ |
| `server/step/generator/bb2016/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2016/end_player_action.rs` | ✓ |
| `server/step/generator/bb2016/Foul.java` | `ffb-engine` | `src/step/generator/bb2016/foul.rs` | ✓ |
| `server/step/generator/bb2016/KickTeamMate.java` | `ffb-engine` | `src/step/generator/bb2016/kick_team_mate.rs` | ✓ |
| `server/step/generator/bb2016/Move.java` | `ffb-engine` | `src/step/generator/bb2016/move.rs` | ✓ |
| `server/step/generator/bb2016/Pass.java` | `ffb-engine` | `src/step/generator/bb2016/pass.rs` | ✓ |
| `server/step/generator/bb2016/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2016/scatter_player.rs` | ✓ |
| `server/step/generator/bb2016/Select.java` | `ffb-engine` | `src/step/generator/bb2016/select.rs` | ✓ |
| `server/step/generator/bb2016/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2016/special_effect.rs` | ✓ |
| `server/step/generator/bb2016/StartGame.java` | `ffb-engine` | `src/step/generator/bb2016/start_game.rs` | ✓ |
| `server/step/generator/bb2016/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2016/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2020/BalefulHex.java` | `ffb-engine` | `src/step/generator/bb2020/baleful_hex.rs` | ✓ |
| `server/step/generator/bb2020/BlackInk.java` | `ffb-engine` | `src/step/generator/bb2020/black_ink.rs` | ✓ |
| `server/step/generator/bb2020/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2020/blitz_block.rs` | ✓ |
| `server/step/generator/bb2020/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2020/blitz_move.rs` | ✓ |
| `server/step/generator/bb2020/Block.java` | `ffb-engine` | `src/step/generator/bb2020/block.rs` | ✓ |
| `server/step/generator/bb2020/Bomb.java` | `ffb-engine` | `src/step/generator/bb2020/bomb.rs` | ✓ |
| `server/step/generator/bb2020/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/bb2020/catch_of_the_day.rs` | ✓ |
| `server/step/generator/bb2020/EndGame.java` | `ffb-engine` | `src/step/generator/bb2020/end_game.rs` | ✓ |
| `server/step/generator/bb2020/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2020/end_player_action.rs` | ✓ |
| `server/step/generator/bb2020/Foul.java` | `ffb-engine` | `src/step/generator/bb2020/foul.rs` | ✓ |
| `server/step/generator/bb2020/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/bb2020/furious_outburst.rs` | ✓ |
| `server/step/generator/bb2020/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/bb2020/look_into_my_eyes.rs` | ✓ |
| `server/step/generator/bb2020/Move.java` | `ffb-engine` | `src/step/generator/bb2020/move.rs` | ✓ |
| `server/step/generator/bb2020/MultiBlock.java` | `ffb-engine` | `src/step/generator/bb2020/multi_block.rs` | ✓ |
| `server/step/generator/bb2020/Pass.java` | `ffb-engine` | `src/step/generator/bb2020/pass.rs` | ✓ |
| `server/step/generator/bb2020/RaidingParty.java` | `ffb-engine` | `src/step/generator/bb2020/raiding_party.rs` | ✓ |
| `server/step/generator/bb2020/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2020/scatter_player.rs` | ✓ |
| `server/step/generator/bb2020/Select.java` | `ffb-engine` | `src/step/generator/bb2020/select.rs` | ✓ |
| `server/step/generator/bb2020/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/bb2020/select_blitz_target.rs` | ✓ |
| `server/step/generator/bb2020/SelectGazeTarget.java` | `ffb-engine` | `src/step/generator/bb2020/select_gaze_target.rs` | ✓ |
| `server/step/generator/bb2020/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2020/special_effect.rs` | ✓ |
| `server/step/generator/bb2020/StartGame.java` | `ffb-engine` | `src/step/generator/bb2020/start_game.rs` | ✓ |
| `server/step/generator/bb2020/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/bb2020/then_i_started_blastin.rs` | ✓ |
| `server/step/generator/bb2020/ThrowKeg.java` | `ffb-engine` | `src/step/generator/bb2020/throw_keg.rs` | ✓ |
| `server/step/generator/bb2020/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2020/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2020/Treacherous.java` | `ffb-engine` | `src/step/generator/bb2020/treacherous.rs` | ✓ |
| `server/step/generator/bb2025/ActivationSequenceBuilder.java` | `ffb-engine` | `src/step/generator/bb2025/activation_sequence_builder.rs` | ✓ |
| `server/step/generator/bb2025/AutoGazeZoat.java` | `ffb-engine` | `src/step/generator/bb2025/auto_gaze_zoat.rs` | ✓ |
| `server/step/generator/bb2025/BalefulHex.java` | `ffb-engine` | `src/step/generator/bb2025/baleful_hex.rs` | ✓ |
| `server/step/generator/bb2025/BlackInk.java` | `ffb-engine` | `src/step/generator/bb2025/black_ink.rs` | ✓ |
| `server/step/generator/bb2025/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2025/blitz_block.rs` | ✓ |
| `server/step/generator/bb2025/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2025/blitz_move.rs` | ✓ |
| `server/step/generator/bb2025/Block.java` | `ffb-engine` | `src/step/generator/bb2025/block.rs` | ✓ |
| `server/step/generator/bb2025/Bomb.java` | `ffb-engine` | `src/step/generator/bb2025/bomb.rs` | ✓ |
| `server/step/generator/bb2025/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/bb2025/catch_of_the_day.rs` | ✓ |
| `server/step/generator/bb2025/EndGame.java` | `ffb-engine` | `src/step/generator/bb2025/end_game.rs` | ✓ |
| `server/step/generator/bb2025/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2025/end_player_action.rs` | ✓ |
| `server/step/generator/bb2025/EndTurn.java` | `ffb-engine` | `src/step/generator/bb2025/end_turn.rs` | ✓ |
| `server/step/generator/bb2025/Foul.java` | `ffb-engine` | `src/step/generator/bb2025/foul.rs` | ✓ |
| `server/step/generator/bb2025/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/bb2025/furious_outburst.rs` | ✓ |
| `server/step/generator/bb2025/Kickoff.java` | `ffb-engine` | `src/step/generator/bb2025/kickoff.rs` | ✓ |
| `server/step/generator/bb2025/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/bb2025/look_into_my_eyes.rs` | ✓ |
| `server/step/generator/bb2025/Move.java` | `ffb-engine` | `src/step/generator/bb2025/move.rs` | ✓ |
| `server/step/generator/bb2025/MultiBlock.java` | `ffb-engine` | `src/step/generator/bb2025/multi_block.rs` | ✓ |
| `server/step/generator/bb2025/Pass.java` | `ffb-engine` | `src/step/generator/bb2025/pass.rs` | ✓ |
| `server/step/generator/bb2025/Punt.java` | `ffb-engine` | `src/step/generator/bb2025/punt.rs` | ✓ |
| `server/step/generator/bb2025/RaidingParty.java` | `ffb-engine` | `src/step/generator/bb2025/raiding_party.rs` | ✓ |
| `server/step/generator/bb2025/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2025/scatter_player.rs` | ✓ |
| `server/step/generator/bb2025/Select.java` | `ffb-engine` | `src/step/generator/bb2025/select.rs` | ✓ |
| `server/step/generator/bb2025/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/bb2025/select_blitz_target.rs` | ✓ |
| `server/step/generator/bb2025/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2025/special_effect.rs` | ✓ |
| `server/step/generator/bb2025/StartGame.java` | `ffb-engine` | `src/step/generator/bb2025/start_game.rs` | ✓ |
| `server/step/generator/bb2025/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/bb2025/then_i_started_blastin.rs` | ✓ |
| `server/step/generator/bb2025/ThrowARock.java` | `ffb-engine` | `src/step/generator/bb2025/throw_a_rock.rs` | ✓ |
| `server/step/generator/bb2025/ThrowKeg.java` | `ffb-engine` | `src/step/generator/bb2025/throw_keg.rs` | ✓ |
| `server/step/generator/bb2025/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2025/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2025/Treacherous.java` | `ffb-engine` | `src/step/generator/bb2025/treacherous.rs` | ✓ |
| `server/step/generator/BlackInk.java` | `ffb-engine` | `src/step/generator/black_ink.rs` | ~ |
| `server/step/generator/BlitzBlock.java` | `ffb-engine` | `src/step/generator/blitz_block.rs` | ~ |
| `server/step/generator/BlitzMove.java` | `ffb-engine` | `src/step/generator/blitz_move.rs` | ~ |
| `server/step/generator/Block.java` | `ffb-engine` | `src/step/generator/block.rs` | ~ |
| `server/step/generator/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/catch_of_the_day.rs` | ~ |
| `server/step/generator/common/Inducement.java` | `ffb-engine` | `src/step/generator/common/inducement.rs` | ~ |
| `server/step/generator/common/RiotousRookies.java` | `ffb-engine` | `src/step/generator/common/riotous_rookies.rs` | ~ |
| `server/step/generator/common/SpikedBallApo.java` | `ffb-engine` | `src/step/generator/common/spiked_ball_apo.rs` | ~ |
| `server/step/generator/common/Wizard.java` | `ffb-engine` | `src/step/generator/common/wizard.rs` | ~ |
| `server/step/generator/EndGame.java` | `ffb-engine` | `src/step/generator/end_game.rs` | ~ |
| `server/step/generator/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/end_player_action.rs` | ~ |
| `server/step/generator/EndTurn.java` | `ffb-engine` | `src/step/generator/end_turn.rs` | ~ |
| `server/step/generator/Foul.java` | `ffb-engine` | `src/step/generator/foul.rs` | ~ |
| `server/step/generator/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/furious_outburst.rs` | ~ |
| `server/step/generator/Kickoff.java` | `ffb-engine` | `src/step/generator/kickoff.rs` | ~ |
| `server/step/generator/KickTeamMate.java` | `ffb-engine` | `src/step/generator/kick_team_mate.rs` | ~ |
| `server/step/generator/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/look_into_my_eyes.rs` | ~ |
| `server/step/generator/mixed/Card.java` | `ffb-engine` | `src/step/generator/mixed/card.rs` | ✓ |
| `server/step/generator/mixed/EndTurn.java` | `ffb-engine` | `src/step/generator/mixed/end_turn.rs` | ✓ |
| `server/step/generator/mixed/Kickoff.java` | `ffb-engine` | `src/step/generator/mixed/kickoff.rs` | ✓ |
| `server/step/generator/mixed/PileDriver.java` | `ffb-engine` | `src/step/generator/mixed/pile_driver.rs` | ✓ |
| `server/step/generator/mixed/QuickBite.java` | `ffb-engine` | `src/step/generator/mixed/quick_bite.rs` | ✓ |
| `server/step/generator/Move.java` | `ffb-engine` | `src/step/generator/move.rs` | ~ |
| `server/step/generator/Pass.java` | `ffb-engine` | `src/step/generator/pass.rs` | ~ |
| `server/step/generator/PileDriver.java` | `ffb-engine` | `src/step/generator/pile_driver.rs` | ~ |
| `server/step/generator/Punt.java` | `ffb-engine` | `src/step/generator/punt.rs` | ~ |
| `server/step/generator/QuickBite.java` | `ffb-engine` | `src/step/generator/quick_bite.rs` | ~ |
| `server/step/generator/RadingParty.java` | `ffb-engine` | `src/step/generator/rading_party.rs` | ~ |
| `server/step/generator/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/scatter_player.rs` | ~ |
| `server/step/generator/Select.java` | `ffb-engine` | `src/step/generator/select.rs` | ~ |
| `server/step/generator/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/select_blitz_target.rs` | ~ |
| `server/step/generator/SelectGazeTarget.java` | `ffb-engine` | `src/step/generator/select_gaze_target.rs` | ~ |
| `server/step/generator/Sequence.java` | `ffb-engine` | `src/step/generator/sequence.rs` | ~ |
| `server/step/generator/SequenceGenerator.java` | `ffb-engine` | `src/step/generator/sequence_generator.rs` | ~ |
| `server/step/generator/SpecialEffect.java` | `ffb-engine` | `src/step/generator/special_effect.rs` | ~ |
| `server/step/generator/StartGame.java` | `ffb-engine` | `src/step/generator/start_game.rs` | ~ |
| `server/step/generator/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/then_i_started_blastin.rs` | ~ |
| `server/step/generator/ThrowKeg.java` | `ffb-engine` | `src/step/generator/throw_keg.rs` | ~ |
| `server/step/generator/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/throw_team_mate.rs` | ~ |
| `server/step/generator/Treacherous.java` | `ffb-engine` | `src/step/generator/treacherous.rs` | ~ |

### server/step/mixed/ (53 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/mixed/blitz/StepRemoveTargetSelectionState.java` | `ffb-engine` | `src/step/mixed/blitz/step_remove_target_selection_state.rs` | ~ |
| `server/step/mixed/blitz/StepSelectBlitzTargetEnd.java` | `ffb-engine` | `src/step/mixed/blitz/step_select_blitz_target_end.rs` | ~ |
| `server/step/mixed/block/StepBlockBallAndChain.java` | `ffb-engine` | `src/step/mixed/block/step_block_ball_and_chain.rs` | ~ |
| `server/step/mixed/block/StepBothDown.java` | `ffb-engine` | `src/step/mixed/block/step_both_down.rs` | ✓ |
| `server/step/mixed/block/StepProjectileVomit.java` | `ffb-engine` | `src/step/mixed/block/step_projectile_vomit.rs` | ~ |
| `server/step/mixed/end/StepDedicatedFans.java` | `ffb-engine` | `src/step/mixed/end/step_dedicated_fans.rs` | ✓ |
| `server/step/mixed/end/StepPenaltyShootout.java` | `ffb-engine` | `src/step/mixed/end/step_penalty_shootout.rs` | ~ |
| `server/step/mixed/foul/StepEjectPlayer.java` | `ffb-engine` | `src/step/mixed/foul/step_eject_player.rs` | ✓ |
| `server/step/mixed/foul/StepFoul.java` | `ffb-engine` | `src/step/mixed/foul/step_foul.rs` | ~ |
| `server/step/mixed/foul/StepFoulChainsaw.java` | `ffb-engine` | `src/step/mixed/foul/step_foul_chainsaw.rs` | ~ |
| `server/step/mixed/foul/StepPileDriver.java` | `ffb-engine` | `src/step/mixed/foul/step_pile_driver.rs` | ✓ |
| `server/step/mixed/inducements/StepPlayCard.java` | `ffb-engine` | `src/step/mixed/inducements/step_play_card.rs` | ~ |
| `server/step/mixed/kickoff/StepInitKickoff.java` | `ffb-engine` | `src/step/mixed/kickoff/step_init_kickoff.rs` | ~ |
| `server/step/mixed/kickoff/StepKickoff.java` | `ffb-engine` | `src/step/mixed/kickoff/step_kickoff.rs` | ~ |
| `server/step/mixed/kickoff/StepSwarming.java` | `ffb-engine` | `src/step/mixed/kickoff/step_swarming.rs` | ~ |
| `server/step/mixed/move/StepDropDivingTackler.java` | `ffb-engine` | `src/step/mixed/move/step_drop_diving_tackler.rs` | ~ |
| `server/step/mixed/move/StepMoveBallAndChain.java` | `ffb-engine` | `src/step/mixed/move/step_move_ball_and_chain.rs` | ~ |
| `server/step/mixed/move/StepResetFumblerooskie.java` | `ffb-engine` | `src/step/mixed/move/step_reset_fumblerooskie.rs` | ~ |
| `server/step/mixed/move/StepTentacles.java` | `ffb-engine` | `src/step/mixed/move/step_tentacles.rs` | ~ |
| `server/step/mixed/move/StepTrapDoor.java` | `ffb-engine` | `src/step/mixed/move/step_trap_door.rs` | ~ |
| `server/step/mixed/multiblock/AbstractStepMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/abstract_step_multiple.rs` | ~ |
| `server/step/mixed/multiblock/StepDauntlessMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/step_dauntless_multiple.rs` | ~ |
| `server/step/mixed/multiblock/StepDispatchDumpOff.java` | `ffb-engine` | `src/step/mixed/multiblock/step_dispatch_dump_off.rs` | ✓ |
| `server/step/mixed/multiblock/StepDoubleStrength.java` | `ffb-engine` | `src/step/mixed/multiblock/step_double_strength.rs` | ✓ |
| `server/step/mixed/multiblock/StepFoulAppearanceMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/step_foul_appearance_multiple.rs` | ~ |
| `server/step/mixed/pass/state/PassState.java` | `ffb-engine` | `src/step/mixed/pass/state/pass_state.rs` | ~ |
| `server/step/mixed/pass/StepAllYouCanEat.java` | `ffb-engine` | `src/step/mixed/pass/step_all_you_can_eat.rs` | ~ |
| `server/step/mixed/pass/StepInitPassing.java` | `ffb-engine` | `src/step/mixed/pass/step_init_passing.rs` | ~ |
| `server/step/mixed/pass/StepPassBlock.java` | `ffb-engine` | `src/step/mixed/pass/step_pass_block.rs` | ~ |
| `server/step/mixed/shared/StepAnimalSavagery.java` | `ffb-engine` | `src/step/mixed/shared/step_animal_savagery.rs` | ~ |
| `server/step/mixed/shared/StepConsumeParameter.java` | `ffb-engine` | `src/step/mixed/shared/step_consume_parameter.rs` | ✓ |
| `server/step/mixed/shared/StepPickMeUp.java` | `ffb-engine` | `src/step/mixed/shared/step_pick_me_up.rs` | ~ |
| `server/step/mixed/shared/StepSetDefender.java` | `ffb-engine` | `src/step/mixed/shared/step_set_defender.rs` | ✓ |
| `server/step/mixed/SingleReRollUseState.java` | `ffb-engine` | `src/step/mixed/single_re_roll_use_state.rs` | ~ |
| `server/step/mixed/special/StepEndBomb.java` | `ffb-engine` | `src/step/mixed/special/step_end_bomb.rs` | ~ |
| `server/step/mixed/start/StepPettyCash.java` | `ffb-engine` | `src/step/mixed/start/step_petty_cash.rs` | ✓ |
| `server/step/mixed/start/StepSpectators.java` | `ffb-engine` | `src/step/mixed/start/step_spectators.rs` | ✓ |
| `server/step/mixed/StepBlockDodge.java` | `ffb-engine` | `src/step/mixed/step_block_dodge.rs` | ~ |
| `server/step/mixed/StepDropActingPlayer.java` | `ffb-engine` | `src/step/mixed/step_drop_acting_player.rs` | ✓ |
| `server/step/mixed/StepEndThenIStartedBlastin.java` | `ffb-engine` | `src/step/mixed/step_end_then_i_started_blastin.rs` | ~ |
| `server/step/mixed/StepEndThrowKeg.java` | `ffb-engine` | `src/step/mixed/step_end_throw_keg.rs` | ~ |
| `server/step/mixed/StepFirstMoveFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_first_move_furious_outburst.rs` | ~ |
| `server/step/mixed/StepFoulAppearance.java` | `ffb-engine` | `src/step/mixed/step_foul_appearance.rs` | ~ |
| `server/step/mixed/StepInitFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_init_furious_outburst.rs` | ~ |
| `server/step/mixed/StepInitLookIntoMyEyes.java` | `ffb-engine` | `src/step/mixed/step_init_look_into_my_eyes.rs` | ✓ |
| `server/step/mixed/StepPro.java` | `ffb-engine` | `src/step/mixed/step_pro.rs` | ~ |
| `server/step/mixed/StepQuickBite.java` | `ffb-engine` | `src/step/mixed/step_quick_bite.rs` | ~ |
| `server/step/mixed/StepSecondMoveFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_second_move_furious_outburst.rs` | ~ |
| `server/step/mixed/StepThrowKeg.java` | `ffb-engine` | `src/step/mixed/step_throw_keg.rs` | ~ |
| `server/step/mixed/StepUnchannelledFury.java` | `ffb-engine` | `src/step/mixed/step_unchannelled_fury.rs` | ~ |
| `server/step/mixed/StepWizard.java` | `ffb-engine` | `src/step/mixed/step_wizard.rs` | ~ |
| `server/step/mixed/ttm/StepSwoop.java` | `ffb-engine` | `src/step/mixed/ttm/step_swoop.rs` | ~ |
| `server/step/mixed/ttm/TtmToCrowdHandler.java` | `ffb-engine` | `src/step/mixed/ttm/ttm_to_crowd_handler.rs` | ~ |

### server/step/phase/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/phase/inducement/StepRiotousRookies.java` | `ffb-engine` | `src/step/phase/inducement/step_riotous_rookies.rs` | ~ |
| `server/step/phase/kickoff/StepCoinChoice.java` | `ffb-engine` | `src/step/phase/kickoff/step_coin_choice.rs` | ~ |
| `server/step/phase/kickoff/StepEndKickoff.java` | `ffb-engine` | `src/step/phase/kickoff/step_end_kickoff.rs` | ~ |
| `server/step/phase/kickoff/StepKickoffAnimation.java` | `ffb-engine` | `src/step/phase/kickoff/step_kickoff_animation.rs` | ~ |
| `server/step/phase/kickoff/StepKickoffReturn.java` | `ffb-engine` | `src/step/phase/kickoff/step_kickoff_return.rs` | ~ |
| `server/step/phase/kickoff/StepReceiveChoice.java` | `ffb-engine` | `src/step/phase/kickoff/step_receive_choice.rs` | ~ |
| `server/step/phase/kickoff/StepTouchback.java` | `ffb-engine` | `src/step/phase/kickoff/step_touchback.rs` | ~ |

### server/util/ (40 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/util/AgilityCalc.java` | `ffb-engine` | `src/util/agility_calc.rs` | ✓ |
| `server/util/BlockDiceCalc.java` | `ffb-engine` | `src/util/block_dice_calc.rs` | ✓ |
| `server/util/BlockResultCalc.java` | `ffb-engine` | `src/util/block_result_calc.rs` | ✓ |
| `server/util/CatchCalc.java` | `ffb-engine` | `src/util/catch_calc.rs` | ✓ |
| `server/util/FoulCalc.java` | `ffb-engine` | `src/util/foul_calc.rs` | ✓ |
| `server/util/KickoffEventCalc.java` | `ffb-engine` | `src/util/kickoff_event_calc.rs` | ✓ |
| `server/util/MarkerLoadingService.java` | `ffb-engine` | `src/util/marker_loading_service.rs` | ✓ |
| `server/util/MovementCalc.java` | `ffb-engine` | `src/util/movement_calc.rs` | ✓ |
| `server/util/PassCalc.java` | `ffb-engine` | `src/util/pass_calc.rs` | ✓ |
| `server/util/PassingDistanceCalc.java` | `ffb-engine` | `src/util/passing_distance_calc.rs` | ✓ |
| `server/util/PostMatchCalc.java` | `ffb-engine` | `src/util/post_match_calc.rs` | ✓ |
| `server/util/rng/EntropyPool.java` | `ffb-engine` | `src/util/rng/entropy_pool.rs` | ~ |
| `server/util/rng/EntropyServer.java` | `ffb-engine` | `src/util/rng/entropy_server.rs` | ~ |
| `server/util/rng/Fortuna.java` | `ffb-engine` | `src/util/rng/fortuna.rs` | ~ |
| `server/util/rng/NetworkEntropySource.java` | `ffb-engine` | `src/util/rng/network_entropy_source.rs` | ~ |
| `server/util/RollCalc.java` | `ffb-engine` | `src/util/roll_calc.rs` | ✓ |
| `server/util/ScatterCalc.java` | `ffb-engine` | `src/util/scatter_calc.rs` | ✓ |
| `server/util/ServerUtilBlock.java` | `ffb-engine` | `src/util/server_util_block.rs` | ~ |
| `server/util/ServerUtilPlayer.java` | `ffb-engine` | `src/util/server_util_player.rs` | ~ |
| `server/util/SpecialRollCalc.java` | `ffb-engine` | `src/util/special_roll_calc.rs` | ✓ |
| `server/util/StatCalc.java` | `ffb-engine` | `src/util/stat_calc.rs` | ✓ |
| `server/util/ThrowInCalc.java` | `ffb-engine` | `src/util/throw_in_calc.rs` | ✓ |
| `server/util/UtilServerCards.java` | `ffb-engine` | `src/util/util_server_cards.rs` | ~ |
| `server/util/UtilServerCatchScatterThrowIn.java` | `ffb-engine` | `src/util/util_server_catch_scatter_throw_in.rs` | ~ |
| `server/util/UtilServerDb.java` | `ffb-engine` | `src/util/util_server_db.rs` | ~ |
| `server/util/UtilServerDialog.java` | `ffb-engine` | `src/util/util_server_dialog.rs` | ✓ |
| `server/util/UtilServerGame.java` | `ffb-engine` | `src/util/util_server_game.rs` | ✓ |
| `server/util/UtilServerHttpClient.java` | `ffb-engine` | `src/util/util_server_http_client.rs` | ~ |
| `server/util/UtilServerInducementUse.java` | `ffb-engine` | `src/util/util_server_inducement_use.rs` | ✓ |
| `server/util/UtilServerInjury.java` | `ffb-engine` | `src/util/util_server_injury.rs` | ✓ |
| `server/util/UtilServerPlayerMove.java` | `ffb-engine` | `src/util/util_server_player_move.rs` | ~ |
| `server/util/UtilServerPlayerSwoop.java` | `ffb-engine` | `src/util/util_server_player_swoop.rs` | ✓ |
| `server/util/UtilServerPushback.java` | `ffb-engine` | `src/util/util_server_pushback.rs` | ~ |
| `server/util/UtilServerReplay.java` | `ffb-engine` | `src/util/util_server_replay.rs` | ~ |
| `server/util/UtilServerReRoll.java` | `ffb-engine` | `src/util/util_server_re_roll.rs` | ~ |
| `server/util/UtilServerSetup.java` | `ffb-engine` | `src/util/util_server_setup.rs` | ✓ |
| `server/util/UtilServerStartGame.java` | `ffb-engine` | `src/util/util_server_start_game.rs` | ✓ |
| `server/util/UtilServerTimer.java` | `ffb-engine` | `src/util/util_server_timer.rs` | ~ |
| `server/util/UtilSkillBehaviours.java` | `ffb-engine` | `src/util/util_skill_behaviours.rs` | ~ |
| `server/util/WeatherCalc.java` | `ffb-engine` | `src/util/weather_calc.rs` | ✓ |

## Module: ffb-client-logic

### client/animation/ (14 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/animation/AnimationFrame.java` | `ffb-client` | `src/client/animation/AnimationFrame.rs` | ~ |
| `client/animation/AnimationProjector.java` | `ffb-client` | `src/client/animation/AnimationProjector.rs` | ~ |
| `client/animation/AnimationSequenceCard.java` | `ffb-client` | `src/client/animation/AnimationSequenceCard.rs` | ~ |
| `client/animation/AnimationSequenceChained.java` | `ffb-client` | `src/client/animation/AnimationSequenceChained.rs` | ~ |
| `client/animation/AnimationSequenceFactory.java` | `ffb-client` | `src/client/animation/AnimationSequenceFactory.rs` | ~ |
| `client/animation/AnimationSequenceKickoff.java` | `ffb-client` | `src/client/animation/AnimationSequenceKickoff.rs` | ~ |
| `client/animation/AnimationSequenceMovingEffect.java` | `ffb-client` | `src/client/animation/AnimationSequenceMovingEffect.rs` | ~ |
| `client/animation/AnimationSequenceSpecialEffect.java` | `ffb-client` | `src/client/animation/AnimationSequenceSpecialEffect.rs` | ~ |
| `client/animation/AnimationSequenceThrowing.java` | `ffb-client` | `src/client/animation/AnimationSequenceThrowing.rs` | ~ |
| `client/animation/CoordinateBasedSteppingStrategy.java` | `ffb-client` | `src/client/animation/CoordinateBasedSteppingStrategy.rs` | ~ |
| `client/animation/IAnimationListener.java` | `ffb-client` | `src/client/animation/IAnimationListener.rs` | ~ |
| `client/animation/IAnimationSequence.java` | `ffb-client` | `src/client/animation/IAnimationSequence.rs` | ~ |
| `client/animation/SteppingStrategy.java` | `ffb-client` | `src/client/animation/SteppingStrategy.rs` | ~ |
| `client/animation/TimerBasedSteppingStrategy.java` | `ffb-client` | `src/client/animation/TimerBasedSteppingStrategy.rs` | ~ |

### client/dialog/ (170 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/dialog/AbstractDialogBlock.java` | `ffb-client` | `src/client/dialog/AbstractDialogBlock.rs` | ~ |
| `client/dialog/AbstractDialogForTargets.java` | `ffb-client` | `src/client/dialog/AbstractDialogForTargets.rs` | ~ |
| `client/dialog/AbstractDialogMultiBlock.java` | `ffb-client` | `src/client/dialog/AbstractDialogMultiBlock.rs` | ~ |
| `client/dialog/AbstractDialogMultiBlockProperties.java` | `ffb-client` | `src/client/dialog/AbstractDialogMultiBlockProperties.rs` | ~ |
| `client/dialog/CommonPropertyCheckList.java` | `ffb-client` | `src/client/dialog/CommonPropertyCheckList.rs` | ~ |
| `client/dialog/CommonPropertyCheckListItem.java` | `ffb-client` | `src/client/dialog/CommonPropertyCheckListItem.rs` | ~ |
| `client/dialog/CreditEntry.java` | `ffb-client` | `src/client/dialog/CreditEntry.rs` | ~ |
| `client/dialog/Dialog.java` | `ffb-client` | `src/client/dialog/Dialog.rs` | ~ |
| `client/dialog/DialogAbout.java` | `ffb-client` | `src/client/dialog/DialogAbout.rs` | ~ |
| `client/dialog/DialogAboutHandler.java` | `ffb-client` | `src/client/dialog/DialogAboutHandler.rs` | ~ |
| `client/dialog/DialogApothecaryChoice.java` | `ffb-client` | `src/client/dialog/DialogApothecaryChoice.rs` | ~ |
| `client/dialog/DialogApothecaryChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogApothecaryChoiceHandler.rs` | ~ |
| `client/dialog/DialogArgueTheCall.java` | `ffb-client` | `src/client/dialog/DialogArgueTheCall.rs` | ~ |
| `client/dialog/DialogArgueTheCallHandler.java` | `ffb-client` | `src/client/dialog/DialogArgueTheCallHandler.rs` | ~ |
| `client/dialog/DialogAutoMarking.java` | `ffb-client` | `src/client/dialog/DialogAutoMarking.rs` | ~ |
| `client/dialog/DialogBlockRoll.java` | `ffb-client` | `src/client/dialog/DialogBlockRoll.rs` | ~ |
| `client/dialog/DialogBlockRollHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollHandler.rs` | ~ |
| `client/dialog/DialogBlockRollPartialReRoll.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPartialReRoll.rs` | ~ |
| `client/dialog/DialogBlockRollPartialReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPartialReRollHandler.rs` | ~ |
| `client/dialog/DialogBlockRollProperties.java` | `ffb-client` | `src/client/dialog/DialogBlockRollProperties.rs` | ~ |
| `client/dialog/DialogBlockRollPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPropertiesHandler.rs` | ~ |
| `client/dialog/DialogBloodlustAction.java` | `ffb-client` | `src/client/dialog/DialogBloodlustAction.rs` | ~ |
| `client/dialog/DialogBloodlustActionHandler.java` | `ffb-client` | `src/client/dialog/DialogBloodlustActionHandler.rs` | ~ |
| `client/dialog/DialogBriberyAndCorruption.java` | `ffb-client` | `src/client/dialog/DialogBriberyAndCorruption.rs` | ~ |
| `client/dialog/DialogBriberyAndCorruptionHandler.java` | `ffb-client` | `src/client/dialog/DialogBriberyAndCorruptionHandler.rs` | ~ |
| `client/dialog/DialogBribes.java` | `ffb-client` | `src/client/dialog/DialogBribes.rs` | ~ |
| `client/dialog/DialogBribesHandler.java` | `ffb-client` | `src/client/dialog/DialogBribesHandler.rs` | ~ |
| `client/dialog/DialogChangeList.java` | `ffb-client` | `src/client/dialog/DialogChangeList.rs` | ~ |
| `client/dialog/DialogChatCommands.java` | `ffb-client` | `src/client/dialog/DialogChatCommands.rs` | ~ |
| `client/dialog/DialogCoinChoice.java` | `ffb-client` | `src/client/dialog/DialogCoinChoice.rs` | ~ |
| `client/dialog/DialogCoinChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogCoinChoiceHandler.rs` | ~ |
| `client/dialog/DialogConcedeGame.java` | `ffb-client` | `src/client/dialog/DialogConcedeGame.rs` | ~ |
| `client/dialog/DialogConfirmEndAction.java` | `ffb-client` | `src/client/dialog/DialogConfirmEndAction.rs` | ~ |
| `client/dialog/DialogConfirmEndActionHandler.java` | `ffb-client` | `src/client/dialog/DialogConfirmEndActionHandler.rs` | ~ |
| `client/dialog/DialogCredits.java` | `ffb-client` | `src/client/dialog/DialogCredits.rs` | ~ |
| `client/dialog/DialogDefenderActionHandler.java` | `ffb-client` | `src/client/dialog/DialogDefenderActionHandler.rs` | ~ |
| `client/dialog/DialogEndTurn.java` | `ffb-client` | `src/client/dialog/DialogEndTurn.rs` | ~ |
| `client/dialog/DialogExtensionMascot.java` | `ffb-client` | `src/client/dialog/DialogExtensionMascot.rs` | ~ |
| `client/dialog/DialogFollowupChoice.java` | `ffb-client` | `src/client/dialog/DialogFollowupChoice.rs` | ~ |
| `client/dialog/DialogFollowupChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogFollowupChoiceHandler.rs` | ~ |
| `client/dialog/DialogGameChoice.java` | `ffb-client` | `src/client/dialog/DialogGameChoice.rs` | ~ |
| `client/dialog/DialogGameConcessionHandler.java` | `ffb-client` | `src/client/dialog/DialogGameConcessionHandler.rs` | ~ |
| `client/dialog/DialogGameStatistics.java` | `ffb-client` | `src/client/dialog/DialogGameStatistics.rs` | ~ |
| `client/dialog/DialogGameStatisticsHandler.java` | `ffb-client` | `src/client/dialog/DialogGameStatisticsHandler.rs` | ~ |
| `client/dialog/DialogHandler.java` | `ffb-client` | `src/client/dialog/DialogHandler.rs` | ~ |
| `client/dialog/DialogInformation.java` | `ffb-client` | `src/client/dialog/DialogInformation.rs` | ~ |
| `client/dialog/DialogInformationOkayHandler.java` | `ffb-client` | `src/client/dialog/DialogInformationOkayHandler.rs` | ~ |
| `client/dialog/DialogInterception.java` | `ffb-client` | `src/client/dialog/DialogInterception.rs` | ~ |
| `client/dialog/DialogInterceptionHandler.java` | `ffb-client` | `src/client/dialog/DialogInterceptionHandler.rs` | ~ |
| `client/dialog/DialogInvalidSolidDefenceHandler.java` | `ffb-client` | `src/client/dialog/DialogInvalidSolidDefenceHandler.rs` | ~ |
| `client/dialog/DialogJoinHandler.java` | `ffb-client` | `src/client/dialog/DialogJoinHandler.rs` | ~ |
| `client/dialog/DialogJourneymen.java` | `ffb-client` | `src/client/dialog/DialogJourneymen.rs` | ~ |
| `client/dialog/DialogJourneymenHandler.java` | `ffb-client` | `src/client/dialog/DialogJourneymenHandler.rs` | ~ |
| `client/dialog/DialogKeyBindings.java` | `ffb-client` | `src/client/dialog/DialogKeyBindings.rs` | ~ |
| `client/dialog/DialogKickOffResult.java` | `ffb-client` | `src/client/dialog/DialogKickOffResult.rs` | ~ |
| `client/dialog/DialogKickOffResultHandler.java` | `ffb-client` | `src/client/dialog/DialogKickOffResultHandler.rs` | ~ |
| `client/dialog/DialogKickoffReturnHandler.java` | `ffb-client` | `src/client/dialog/DialogKickoffReturnHandler.rs` | ~ |
| `client/dialog/DialogKickSkillHandler.java` | `ffb-client` | `src/client/dialog/DialogKickSkillHandler.rs` | ~ |
| `client/dialog/DialogLeaveGame.java` | `ffb-client` | `src/client/dialog/DialogLeaveGame.rs` | ~ |
| `client/dialog/DialogLicense.java` | `ffb-client` | `src/client/dialog/DialogLicense.rs` | ~ |
| `client/dialog/DialogLogin.java` | `ffb-client` | `src/client/dialog/DialogLogin.rs` | ~ |
| `client/dialog/DialogManager.java` | `ffb-client` | `src/client/dialog/DialogManager.rs` | ~ |
| `client/dialog/DialogOpponentBlockSelection.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelection.rs` | ~ |
| `client/dialog/DialogOpponentBlockSelectionHandler.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionHandler.rs` | ~ |
| `client/dialog/DialogOpponentBlockSelectionProperties.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionProperties.rs` | ~ |
| `client/dialog/DialogOpponentBlockSelectionPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionPropertiesHandler.rs` | ~ |
| `client/dialog/DialogPassBlockHandler.java` | `ffb-client` | `src/client/dialog/DialogPassBlockHandler.rs` | ~ |
| `client/dialog/DialogPenaltyShootout.java` | `ffb-client` | `src/client/dialog/DialogPenaltyShootout.rs` | ~ |
| `client/dialog/DialogPenaltyShootoutHandler.java` | `ffb-client` | `src/client/dialog/DialogPenaltyShootoutHandler.rs` | ~ |
| `client/dialog/DialogPettyCash.java` | `ffb-client` | `src/client/dialog/DialogPettyCash.rs` | ~ |
| `client/dialog/DialogPettyCashHandler.java` | `ffb-client` | `src/client/dialog/DialogPettyCashHandler.rs` | ~ |
| `client/dialog/DialogPickUpChoice.java` | `ffb-client` | `src/client/dialog/DialogPickUpChoice.rs` | ~ |
| `client/dialog/DialogPickUpChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogPickUpChoiceHandler.rs` | ~ |
| `client/dialog/DialogPileDriver.java` | `ffb-client` | `src/client/dialog/DialogPileDriver.rs` | ~ |
| `client/dialog/DialogPileDriverHandler.java` | `ffb-client` | `src/client/dialog/DialogPileDriverHandler.rs` | ~ |
| `client/dialog/DialogPilingOn.java` | `ffb-client` | `src/client/dialog/DialogPilingOn.rs` | ~ |
| `client/dialog/DialogPilingOnHandler.java` | `ffb-client` | `src/client/dialog/DialogPilingOnHandler.rs` | ~ |
| `client/dialog/DialogPlayerChoice.java` | `ffb-client` | `src/client/dialog/DialogPlayerChoice.rs` | ~ |
| `client/dialog/DialogPlayerChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogPlayerChoiceHandler.rs` | ~ |
| `client/dialog/DialogProgressBar.java` | `ffb-client` | `src/client/dialog/DialogProgressBar.rs` | ~ |
| `client/dialog/DialogPuntToCrowd.java` | `ffb-client` | `src/client/dialog/DialogPuntToCrowd.rs` | ~ |
| `client/dialog/DialogPuntToCrowdHandler.java` | `ffb-client` | `src/client/dialog/DialogPuntToCrowdHandler.rs` | ~ |
| `client/dialog/DialogReceiveChoice.java` | `ffb-client` | `src/client/dialog/DialogReceiveChoice.rs` | ~ |
| `client/dialog/DialogReceiveChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogReceiveChoiceHandler.rs` | ~ |
| `client/dialog/DialogReplayModeChoice.java` | `ffb-client` | `src/client/dialog/DialogReplayModeChoice.rs` | ~ |
| `client/dialog/DialogReRoll.java` | `ffb-client` | `src/client/dialog/DialogReRoll.rs` | ~ |
| `client/dialog/DialogReRollBlockForTargets.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargets.rs` | ~ |
| `client/dialog/DialogReRollBlockForTargetsHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsHandler.rs` | ~ |
| `client/dialog/DialogReRollBlockForTargetsProperties.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsProperties.rs` | ~ |
| `client/dialog/DialogReRollBlockForTargetsPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsPropertiesHandler.rs` | ~ |
| `client/dialog/DialogReRollForTargets.java` | `ffb-client` | `src/client/dialog/DialogReRollForTargets.rs` | ~ |
| `client/dialog/DialogReRollForTargetsHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollForTargetsHandler.rs` | ~ |
| `client/dialog/DialogReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollHandler.rs` | ~ |
| `client/dialog/DialogReRollProperties.java` | `ffb-client` | `src/client/dialog/DialogReRollProperties.rs` | ~ |
| `client/dialog/DialogReRollPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollPropertiesHandler.rs` | ~ |
| `client/dialog/DialogReRollRegenerationMultiple.java` | `ffb-client` | `src/client/dialog/DialogReRollRegenerationMultiple.rs` | ~ |
| `client/dialog/DialogReRollRegenerationMultipleHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollRegenerationMultipleHandler.rs` | ~ |
| `client/dialog/DialogScalingFactor.java` | `ffb-client` | `src/client/dialog/DialogScalingFactor.rs` | ~ |
| `client/dialog/DialogSelectBlitzTargetHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectBlitzTargetHandler.rs` | ~ |
| `client/dialog/DialogSelectGazeTargetHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectGazeTargetHandler.rs` | ~ |
| `client/dialog/DialogSelectKeyword.java` | `ffb-client` | `src/client/dialog/DialogSelectKeyword.rs` | ~ |
| `client/dialog/DialogSelectKeywordHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectKeywordHandler.rs` | ~ |
| `client/dialog/DialogSelectLocalStoredProperties.java` | `ffb-client` | `src/client/dialog/DialogSelectLocalStoredProperties.rs` | ~ |
| `client/dialog/DialogSelectPosition.java` | `ffb-client` | `src/client/dialog/DialogSelectPosition.rs` | ~ |
| `client/dialog/DialogSelectPositionHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectPositionHandler.rs` | ~ |
| `client/dialog/DialogSelectSkill.java` | `ffb-client` | `src/client/dialog/DialogSelectSkill.rs` | ~ |
| `client/dialog/DialogSelectSkillHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectSkillHandler.rs` | ~ |
| `client/dialog/DialogSelectTarget.java` | `ffb-client` | `src/client/dialog/DialogSelectTarget.rs` | ~ |
| `client/dialog/DialogSelectWeather.java` | `ffb-client` | `src/client/dialog/DialogSelectWeather.rs` | ~ |
| `client/dialog/DialogSelectWeatherHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectWeatherHandler.rs` | ~ |
| `client/dialog/DialogSetupError.java` | `ffb-client` | `src/client/dialog/DialogSetupError.rs` | ~ |
| `client/dialog/DialogSetupErrorHandler.java` | `ffb-client` | `src/client/dialog/DialogSetupErrorHandler.rs` | ~ |
| `client/dialog/DialogSkillUse.java` | `ffb-client` | `src/client/dialog/DialogSkillUse.rs` | ~ |
| `client/dialog/DialogSkillUseHandler.java` | `ffb-client` | `src/client/dialog/DialogSkillUseHandler.rs` | ~ |
| `client/dialog/DialogSoundVolume.java` | `ffb-client` | `src/client/dialog/DialogSoundVolume.rs` | ~ |
| `client/dialog/DialogStartGame.java` | `ffb-client` | `src/client/dialog/DialogStartGame.rs` | ~ |
| `client/dialog/DialogStartGameHandler.java` | `ffb-client` | `src/client/dialog/DialogStartGameHandler.rs` | ~ |
| `client/dialog/DialogSwarmingErrorParameterHandler.java` | `ffb-client` | `src/client/dialog/DialogSwarmingErrorParameterHandler.rs` | ~ |
| `client/dialog/DialogSwarmingPlayersHandler.java` | `ffb-client` | `src/client/dialog/DialogSwarmingPlayersHandler.rs` | ~ |
| `client/dialog/DialogTeamChoice.java` | `ffb-client` | `src/client/dialog/DialogTeamChoice.rs` | ~ |
| `client/dialog/DialogTeamSetup.java` | `ffb-client` | `src/client/dialog/DialogTeamSetup.rs` | ~ |
| `client/dialog/DialogTeamSetupHandler.java` | `ffb-client` | `src/client/dialog/DialogTeamSetupHandler.rs` | ~ |
| `client/dialog/DialogThreeWayChoice.java` | `ffb-client` | `src/client/dialog/DialogThreeWayChoice.rs` | ~ |
| `client/dialog/DialogTouchbackHandler.java` | `ffb-client` | `src/client/dialog/DialogTouchbackHandler.rs` | ~ |
| `client/dialog/DialogUseApothecaries.java` | `ffb-client` | `src/client/dialog/DialogUseApothecaries.rs` | ~ |
| `client/dialog/DialogUseApothecariesHandler.java` | `ffb-client` | `src/client/dialog/DialogUseApothecariesHandler.rs` | ~ |
| `client/dialog/DialogUseApothecary.java` | `ffb-client` | `src/client/dialog/DialogUseApothecary.rs` | ~ |
| `client/dialog/DialogUseApothecaryHandler.java` | `ffb-client` | `src/client/dialog/DialogUseApothecaryHandler.rs` | ~ |
| `client/dialog/DialogUseChainsaw.java` | `ffb-client` | `src/client/dialog/DialogUseChainsaw.rs` | ~ |
| `client/dialog/DialogUseChainsawHandler.java` | `ffb-client` | `src/client/dialog/DialogUseChainsawHandler.rs` | ~ |
| `client/dialog/DialogUseIgor.java` | `ffb-client` | `src/client/dialog/DialogUseIgor.rs` | ~ |
| `client/dialog/DialogUseIgorHandler.java` | `ffb-client` | `src/client/dialog/DialogUseIgorHandler.rs` | ~ |
| `client/dialog/DialogUseIgorsHandler.java` | `ffb-client` | `src/client/dialog/DialogUseIgorsHandler.rs` | ~ |
| `client/dialog/DialogUseMortuaryAssistant.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistant.rs` | ~ |
| `client/dialog/DialogUseMortuaryAssistantHandler.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistantHandler.rs` | ~ |
| `client/dialog/DialogUseMortuaryAssistantsHandler.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistantsHandler.rs` | ~ |
| `client/dialog/DialogWinningsReRoll.java` | `ffb-client` | `src/client/dialog/DialogWinningsReRoll.rs` | ~ |
| `client/dialog/DialogWinningsReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogWinningsReRollHandler.rs` | ~ |
| `client/dialog/DialogWizardSpell.java` | `ffb-client` | `src/client/dialog/DialogWizardSpell.rs` | ~ |
| `client/dialog/DialogWizardSpellHandler.java` | `ffb-client` | `src/client/dialog/DialogWizardSpellHandler.rs` | ~ |
| `client/dialog/IDialog.java` | `ffb-client` | `src/client/dialog/IDialog.rs` | ~ |
| `client/dialog/IDialogCloseListener.java` | `ffb-client` | `src/client/dialog/IDialogCloseListener.rs` | ~ |
| `client/dialog/inducements/AbstractBuyInducementsDialog.java` | `ffb-client` | `src/client/dialog/inducements/AbstractBuyInducementsDialog.rs` | ~ |
| `client/dialog/inducements/DialogBuyCards.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCards.rs` | ~ |
| `client/dialog/inducements/DialogBuyCardsAndInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsAndInducements.rs` | ~ |
| `client/dialog/inducements/DialogBuyCardsAndInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsAndInducementsHandler.rs` | ~ |
| `client/dialog/inducements/DialogBuyCardsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsHandler.rs` | ~ |
| `client/dialog/inducements/DialogBuyInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyInducements.rs` | ~ |
| `client/dialog/inducements/DialogBuyInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyInducementsHandler.rs` | ~ |
| `client/dialog/inducements/DialogBuyPrayersAndInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyPrayersAndInducements.rs` | ~ |
| `client/dialog/inducements/DialogBuyPrayersAndInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyPrayersAndInducementsHandler.rs` | ~ |
| `client/dialog/inducements/DialogUseInducement.java` | `ffb-client` | `src/client/dialog/inducements/DialogUseInducement.rs` | ~ |
| `client/dialog/inducements/DialogUseInducementHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogUseInducementHandler.rs` | ~ |
| `client/dialog/inducements/DropDownPanel.java` | `ffb-client` | `src/client/dialog/inducements/DropDownPanel.rs` | ~ |
| `client/dialog/inducements/InfamousStaffTable.java` | `ffb-client` | `src/client/dialog/inducements/InfamousStaffTable.rs` | ~ |
| `client/dialog/inducements/InfamousStaffTableModel.java` | `ffb-client` | `src/client/dialog/inducements/InfamousStaffTableModel.rs` | ~ |
| `client/dialog/inducements/MercenaryTable.java` | `ffb-client` | `src/client/dialog/inducements/MercenaryTable.rs` | ~ |
| `client/dialog/inducements/MercenaryTableModel.java` | `ffb-client` | `src/client/dialog/inducements/MercenaryTableModel.rs` | ~ |
| `client/dialog/inducements/StarPlayerTable.java` | `ffb-client` | `src/client/dialog/inducements/StarPlayerTable.rs` | ~ |
| `client/dialog/inducements/StarPlayerTableModel.java` | `ffb-client` | `src/client/dialog/inducements/StarPlayerTableModel.rs` | ~ |
| `client/dialog/KeywordCheckList.java` | `ffb-client` | `src/client/dialog/KeywordCheckList.rs` | ~ |
| `client/dialog/KeywordCheckListItem.java` | `ffb-client` | `src/client/dialog/KeywordCheckListItem.rs` | ~ |
| `client/dialog/MultiReRollMnemonics.java` | `ffb-client` | `src/client/dialog/MultiReRollMnemonics.rs` | ~ |
| `client/dialog/PlayerCheckList.java` | `ffb-client` | `src/client/dialog/PlayerCheckList.rs` | ~ |
| `client/dialog/PlayerCheckListItem.java` | `ffb-client` | `src/client/dialog/PlayerCheckListItem.rs` | ~ |
| `client/dialog/PositionCheckList.java` | `ffb-client` | `src/client/dialog/PositionCheckList.rs` | ~ |
| `client/dialog/PositionCheckListItem.java` | `ffb-client` | `src/client/dialog/PositionCheckListItem.rs` | ~ |
| `client/dialog/PressedKeyListener.java` | `ffb-client` | `src/client/dialog/PressedKeyListener.rs` | ~ |
| `client/dialog/SkillCheckList.java` | `ffb-client` | `src/client/dialog/SkillCheckList.rs` | ~ |
| `client/dialog/SkillCheckListItem.java` | `ffb-client` | `src/client/dialog/SkillCheckListItem.rs` | ~ |

### client/factory/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/factory/LogicPluginFactory.java` | `ffb-client` | `src/client/factory/LogicPluginFactory.rs` | ~ |

### client/handler/ (27 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/handler/AbstractClientCommandHandlerSketch.java` | `ffb-client` | `src/client/handler/AbstractClientCommandHandlerSketch.rs` | ~ |
| `client/handler/ClientCommandHandler.java` | `ffb-client` | `src/client/handler/ClientCommandHandler.rs` | ~ |
| `client/handler/ClientCommandHandlerAddPlayer.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerAddPlayer.rs` | ~ |
| `client/handler/ClientCommandHandlerAddSketches.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerAddSketches.rs` | ~ |
| `client/handler/ClientCommandHandlerAdminMessage.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerAdminMessage.rs` | ~ |
| `client/handler/ClientCommandHandlerClearSketches.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerClearSketches.rs` | ~ |
| `client/handler/ClientCommandHandlerFactory.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerFactory.rs` | ~ |
| `client/handler/ClientCommandHandlerGameState.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerGameState.rs` | ~ |
| `client/handler/ClientCommandHandlerGameTime.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerGameTime.rs` | ~ |
| `client/handler/ClientCommandHandlerJoin.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerJoin.rs` | ~ |
| `client/handler/ClientCommandHandlerLeave.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerLeave.rs` | ~ |
| `client/handler/ClientCommandHandlerMode.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerMode.rs` | ~ |
| `client/handler/ClientCommandHandlerModelSync.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerModelSync.rs` | ~ |
| `client/handler/ClientCommandHandlerRemovePlayer.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerRemovePlayer.rs` | ~ |
| `client/handler/ClientCommandHandlerRemoveSketches.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerRemoveSketches.rs` | ~ |
| `client/handler/ClientCommandHandlerSetPreventSketching.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSetPreventSketching.rs` | ~ |
| `client/handler/ClientCommandHandlerSketchAddCoordinate.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSketchAddCoordinate.rs` | ~ |
| `client/handler/ClientCommandHandlerSketchSetColor.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSketchSetColor.rs` | ~ |
| `client/handler/ClientCommandHandlerSketchSetLabel.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSketchSetLabel.rs` | ~ |
| `client/handler/ClientCommandHandlerSocketClosed.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSocketClosed.rs` | ~ |
| `client/handler/ClientCommandHandlerSound.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerSound.rs` | ~ |
| `client/handler/ClientCommandHandlerTalk.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerTalk.rs` | ~ |
| `client/handler/ClientCommandHandlerUnzapPlayer.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerUnzapPlayer.rs` | ~ |
| `client/handler/ClientCommandHandlerUpdateLocalPlayerMarkers.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerUpdateLocalPlayerMarkers.rs` | ~ |
| `client/handler/ClientCommandHandlerUserSettings.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerUserSettings.rs` | ~ |
| `client/handler/ClientCommandHandlerZapPlayer.java` | `ffb-client` | `src/client/handler/ClientCommandHandlerZapPlayer.rs` | ~ |
| `client/handler/SubHandlerGameStateMarking.java` | `ffb-client` | `src/client/handler/SubHandlerGameStateMarking.rs` | ~ |

### client/layer/ (13 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/layer/FieldLayer.java` | `ffb-client` | `src/client/layer/FieldLayer.rs` | ~ |
| `client/layer/FieldLayerBloodspots.java` | `ffb-client` | `src/client/layer/FieldLayerBloodspots.rs` | ~ |
| `client/layer/FieldLayerEnhancements.java` | `ffb-client` | `src/client/layer/FieldLayerEnhancements.rs` | ~ |
| `client/layer/FieldLayerMarker.java` | `ffb-client` | `src/client/layer/FieldLayerMarker.rs` | ~ |
| `client/layer/FieldLayerOverPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerOverPlayers.rs` | ~ |
| `client/layer/FieldLayerPitch.java` | `ffb-client` | `src/client/layer/FieldLayerPitch.rs` | ~ |
| `client/layer/FieldLayerPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerPlayers.rs` | ~ |
| `client/layer/FieldLayerRangeGrid.java` | `ffb-client` | `src/client/layer/FieldLayerRangeGrid.rs` | ~ |
| `client/layer/FieldLayerRangeRuler.java` | `ffb-client` | `src/client/layer/FieldLayerRangeRuler.rs` | ~ |
| `client/layer/FieldLayerSketches.java` | `ffb-client` | `src/client/layer/FieldLayerSketches.rs` | ~ |
| `client/layer/FieldLayerTackleZones.java` | `ffb-client` | `src/client/layer/FieldLayerTackleZones.rs` | ~ |
| `client/layer/FieldLayerTeamLogo.java` | `ffb-client` | `src/client/layer/FieldLayerTeamLogo.rs` | ~ |
| `client/layer/FieldLayerUnderPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerUnderPlayers.rs` | ~ |

### client/model/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/model/ChangeList.java` | `ffb-client` | `src/client/model/ChangeList.rs` | ~ |
| `client/model/ControlAware.java` | `ffb-client` | `src/client/model/ControlAware.rs` | ~ |
| `client/model/OnlineAware.java` | `ffb-client` | `src/client/model/OnlineAware.rs` | ~ |
| `client/model/VersionChangeList.java` | `ffb-client` | `src/client/model/VersionChangeList.rs` | ~ |

### client/net/ (3 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/net/ClientCommunication.java` | `ffb-client` | `src/client/net/ClientCommunication.rs` | ~ |
| `client/net/ClientPingTask.java` | `ffb-client` | `src/client/net/ClientPingTask.rs` | ~ |
| `client/net/CommandEndpoint.java` | `ffb-client` | `src/client/net/CommandEndpoint.rs` | ~ |

### client/overlay/ (3 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/overlay/Overlay.java` | `ffb-client` | `src/client/overlay/Overlay.rs` | ~ |
| `client/overlay/sketch/ClientSketchManager.java` | `ffb-client` | `src/client/overlay/sketch/ClientSketchManager.rs` | ~ |
| `client/overlay/sketch/TriangleCoords.java` | `ffb-client` | `src/client/overlay/sketch/TriangleCoords.rs` | ~ |

### client/report/ (211 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/report/AlwaysHungryMessage.java` | `ffb-client` | `src/client/report/AlwaysHungryMessage.rs` | ~ |
| `client/report/AnimosityRollMessage.java` | `ffb-client` | `src/client/report/AnimosityRollMessage.rs` | ~ |
| `client/report/ApothecaryChoiceMessage.java` | `ffb-client` | `src/client/report/ApothecaryChoiceMessage.rs` | ~ |
| `client/report/bb2016/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2016/ApothecaryRollMessage.rs` | ~ |
| `client/report/bb2016/ArgueTheCallMessage.java` | `ffb-client` | `src/client/report/bb2016/ArgueTheCallMessage.rs` | ~ |
| `client/report/bb2016/BlockChoiceMessage.java` | `ffb-client` | `src/client/report/bb2016/BlockChoiceMessage.rs` | ~ |
| `client/report/bb2016/BloodLustRollMessage.java` | `ffb-client` | `src/client/report/bb2016/BloodLustRollMessage.rs` | ~ |
| `client/report/bb2016/CardsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2016/CardsBoughtMessage.rs` | ~ |
| `client/report/bb2016/FanFactorRollPostMatchMessage.java` | `ffb-client` | `src/client/report/bb2016/FanFactorRollPostMatchMessage.rs` | ~ |
| `client/report/bb2016/GoForItRollMessage.java` | `ffb-client` | `src/client/report/bb2016/GoForItRollMessage.rs` | ~ |
| `client/report/bb2016/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2016/HypnoticGazeRollMessage.rs` | ~ |
| `client/report/bb2016/InducementMessage.java` | `ffb-client` | `src/client/report/bb2016/InducementMessage.rs` | ~ |
| `client/report/bb2016/InducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2016/InducementsBoughtMessage.rs` | ~ |
| `client/report/bb2016/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2016/InjuryMessage.rs` | ~ |
| `client/report/bb2016/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2016/KickoffExtraReRollMessage.rs` | ~ |
| `client/report/bb2016/KickoffPitchInvasionMessage.java` | `ffb-client` | `src/client/report/bb2016/KickoffPitchInvasionMessage.rs` | ~ |
| `client/report/bb2016/KickoffRiotMessage.java` | `ffb-client` | `src/client/report/bb2016/KickoffRiotMessage.rs` | ~ |
| `client/report/bb2016/KickoffThrowARockMessage.java` | `ffb-client` | `src/client/report/bb2016/KickoffThrowARockMessage.rs` | ~ |
| `client/report/bb2016/KickTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2016/KickTeamMateRollMessage.rs` | ~ |
| `client/report/bb2016/MostValuablePlayersMessage.java` | `ffb-client` | `src/client/report/bb2016/MostValuablePlayersMessage.rs` | ~ |
| `client/report/bb2016/NervesOfSteelMessage.java` | `ffb-client` | `src/client/report/bb2016/NervesOfSteelMessage.rs` | ~ |
| `client/report/bb2016/NoPlayersToFieldMessage.java` | `ffb-client` | `src/client/report/bb2016/NoPlayersToFieldMessage.rs` | ~ |
| `client/report/bb2016/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2016/PassRollMessage.rs` | ~ |
| `client/report/bb2016/PenaltyShootoutMessage.java` | `ffb-client` | `src/client/report/bb2016/PenaltyShootoutMessage.rs` | ~ |
| `client/report/bb2016/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2016/RaiseDeadMessage.rs` | ~ |
| `client/report/bb2016/RefereeMessage.java` | `ffb-client` | `src/client/report/bb2016/RefereeMessage.rs` | ~ |
| `client/report/bb2016/ScatterBallMessage.java` | `ffb-client` | `src/client/report/bb2016/ScatterBallMessage.rs` | ~ |
| `client/report/bb2016/ScatterPlayerMessage.java` | `ffb-client` | `src/client/report/bb2016/ScatterPlayerMessage.rs` | ~ |
| `client/report/bb2016/SpectatorsMessage.java` | `ffb-client` | `src/client/report/bb2016/SpectatorsMessage.rs` | ~ |
| `client/report/bb2016/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2016/SwarmingPlayersRollMessage.rs` | ~ |
| `client/report/bb2016/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2016/SwoopPlayerMessage.rs` | ~ |
| `client/report/bb2016/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2016/TentaclesShadowingMessage.rs` | ~ |
| `client/report/bb2016/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2016/ThrowTeamMateRollMessage.rs` | ~ |
| `client/report/bb2016/TurnEndMessage.java` | `ffb-client` | `src/client/report/bb2016/TurnEndMessage.rs` | ~ |
| `client/report/bb2016/WinningsRollMessage.java` | `ffb-client` | `src/client/report/bb2016/WinningsRollMessage.rs` | ~ |
| `client/report/bb2020/AnimalSavageryMessage.java` | `ffb-client` | `src/client/report/bb2020/AnimalSavageryMessage.rs` | ~ |
| `client/report/bb2020/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2020/ApothecaryRollMessage.rs` | ~ |
| `client/report/bb2020/BlitzRollMessage.java` | `ffb-client` | `src/client/report/bb2020/BlitzRollMessage.rs` | ~ |
| `client/report/bb2020/CardsAndInducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2020/CardsAndInducementsBoughtMessage.rs` | ~ |
| `client/report/bb2020/CheeringFansMessage.java` | `ffb-client` | `src/client/report/bb2020/CheeringFansMessage.rs` | ~ |
| `client/report/bb2020/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2020/HypnoticGazeRollMessage.rs` | ~ |
| `client/report/bb2020/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2020/InjuryMessage.rs` | ~ |
| `client/report/bb2020/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2020/KickoffExtraReRollMessage.rs` | ~ |
| `client/report/bb2020/KickoffOfficiousRefMessage.java` | `ffb-client` | `src/client/report/bb2020/KickoffOfficiousRefMessage.rs` | ~ |
| `client/report/bb2020/KickTeamMateFumbleMessage.java` | `ffb-client` | `src/client/report/bb2020/KickTeamMateFumbleMessage.rs` | ~ |
| `client/report/bb2020/OfficiousRefRollMessage.java` | `ffb-client` | `src/client/report/bb2020/OfficiousRefRollMessage.rs` | ~ |
| `client/report/bb2020/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2020/PassRollMessage.rs` | ~ |
| `client/report/bb2020/PrayerAmountMessage.java` | `ffb-client` | `src/client/report/bb2020/PrayerAmountMessage.rs` | ~ |
| `client/report/bb2020/PrayerRollMessage.java` | `ffb-client` | `src/client/report/bb2020/PrayerRollMessage.rs` | ~ |
| `client/report/bb2020/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2020/RaiseDeadMessage.rs` | ~ |
| `client/report/bb2020/SolidDefenceRollMessage.java` | `ffb-client` | `src/client/report/bb2020/SolidDefenceRollMessage.rs` | ~ |
| `client/report/bb2020/StallerDetectedMessage.java` | `ffb-client` | `src/client/report/bb2020/StallerDetectedMessage.rs` | ~ |
| `client/report/bb2020/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2020/SwarmingPlayersRollMessage.rs` | ~ |
| `client/report/bb2020/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2020/SwoopPlayerMessage.rs` | ~ |
| `client/report/bb2020/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2020/TentaclesShadowingMessage.rs` | ~ |
| `client/report/bb2020/ThenIStartedBlastinMessage.java` | `ffb-client` | `src/client/report/bb2020/ThenIStartedBlastinMessage.rs` | ~ |
| `client/report/bb2020/ThrowAtStallingPlayerMessage.java` | `ffb-client` | `src/client/report/bb2020/ThrowAtStallingPlayerMessage.rs` | ~ |
| `client/report/bb2020/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2020/ThrowTeamMateRollMessage.rs` | ~ |
| `client/report/bb2020/TwoForOneMessage.java` | `ffb-client` | `src/client/report/bb2020/TwoForOneMessage.rs` | ~ |
| `client/report/bb2020/UseFumblerooskieMessage.java` | `ffb-client` | `src/client/report/bb2020/UseFumblerooskieMessage.rs` | ~ |
| `client/report/bb2020/WeatherMageResultMessage.java` | `ffb-client` | `src/client/report/bb2020/WeatherMageResultMessage.rs` | ~ |
| `client/report/bb2025/AnimalSavageryMessage.java` | `ffb-client` | `src/client/report/bb2025/AnimalSavageryMessage.rs` | ~ |
| `client/report/bb2025/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2025/ApothecaryRollMessage.rs` | ~ |
| `client/report/bb2025/BlitzRollMessage.java` | `ffb-client` | `src/client/report/bb2025/BlitzRollMessage.rs` | ~ |
| `client/report/bb2025/CheeringFansMessage.java` | `ffb-client` | `src/client/report/bb2025/CheeringFansMessage.rs` | ~ |
| `client/report/bb2025/ChompRemovedMessage.java` | `ffb-client` | `src/client/report/bb2025/ChompRemovedMessage.rs` | ~ |
| `client/report/bb2025/ChompRollMessage.java` | `ffb-client` | `src/client/report/bb2025/ChompRollMessage.rs` | ~ |
| `client/report/bb2025/DodgySnackRollMessage.java` | `ffb-client` | `src/client/report/bb2025/DodgySnackRollMessage.rs` | ~ |
| `client/report/bb2025/GettingEvenRollMessage.java` | `ffb-client` | `src/client/report/bb2025/GettingEvenRollMessage.rs` | ~ |
| `client/report/bb2025/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2025/HypnoticGazeRollMessage.rs` | ~ |
| `client/report/bb2025/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2025/InjuryMessage.rs` | ~ |
| `client/report/bb2025/KickoffDodgySnackMessage.java` | `ffb-client` | `src/client/report/bb2025/KickoffDodgySnackMessage.rs` | ~ |
| `client/report/bb2025/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2025/KickoffExtraReRollMessage.rs` | ~ |
| `client/report/bb2025/KickTeamMateFumbleMessage.java` | `ffb-client` | `src/client/report/bb2025/KickTeamMateFumbleMessage.rs` | ~ |
| `client/report/bb2025/MascotUsedMessage.java` | `ffb-client` | `src/client/report/bb2025/MascotUsedMessage.rs` | ~ |
| `client/report/bb2025/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2025/PassRollMessage.rs` | ~ |
| `client/report/bb2025/PickUpRollMessage.java` | `ffb-client` | `src/client/report/bb2025/PickUpRollMessage.rs` | ~ |
| `client/report/bb2025/PrayerAmountMessage.java` | `ffb-client` | `src/client/report/bb2025/PrayerAmountMessage.rs` | ~ |
| `client/report/bb2025/PrayerRollMessage.java` | `ffb-client` | `src/client/report/bb2025/PrayerRollMessage.rs` | ~ |
| `client/report/bb2025/PrayersAndInducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2025/PrayersAndInducementsBoughtMessage.rs` | ~ |
| `client/report/bb2025/PuntDirectionMessage.java` | `ffb-client` | `src/client/report/bb2025/PuntDirectionMessage.rs` | ~ |
| `client/report/bb2025/PuntDistanceMessage.java` | `ffb-client` | `src/client/report/bb2025/PuntDistanceMessage.rs` | ~ |
| `client/report/bb2025/PushbackMessage.java` | `ffb-client` | `src/client/report/bb2025/PushbackMessage.rs` | ~ |
| `client/report/bb2025/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2025/RaiseDeadMessage.rs` | ~ |
| `client/report/bb2025/SaboteurRollMessage.java` | `ffb-client` | `src/client/report/bb2025/SaboteurRollMessage.rs` | ~ |
| `client/report/bb2025/SolidDefenceRollMessage.java` | `ffb-client` | `src/client/report/bb2025/SolidDefenceRollMessage.rs` | ~ |
| `client/report/bb2025/StallerDetectedMessage.java` | `ffb-client` | `src/client/report/bb2025/StallerDetectedMessage.rs` | ~ |
| `client/report/bb2025/SteadyFootingRollMessage.java` | `ffb-client` | `src/client/report/bb2025/SteadyFootingRollMessage.rs` | ~ |
| `client/report/bb2025/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2025/SwarmingPlayersRollMessage.rs` | ~ |
| `client/report/bb2025/SwoopDirectionMessage.java` | `ffb-client` | `src/client/report/bb2025/SwoopDirectionMessage.rs` | ~ |
| `client/report/bb2025/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/SwoopPlayerMessage.rs` | ~ |
| `client/report/bb2025/TeamCaptainRollMessage.java` | `ffb-client` | `src/client/report/bb2025/TeamCaptainRollMessage.rs` | ~ |
| `client/report/bb2025/TeamEventMessage.java` | `ffb-client` | `src/client/report/bb2025/TeamEventMessage.rs` | ~ |
| `client/report/bb2025/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2025/TentaclesShadowingMessage.rs` | ~ |
| `client/report/bb2025/ThenIStartedBlastinMessage.java` | `ffb-client` | `src/client/report/bb2025/ThenIStartedBlastinMessage.rs` | ~ |
| `client/report/bb2025/ThrowAtPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/ThrowAtPlayerMessage.rs` | ~ |
| `client/report/bb2025/ThrowAtStallingPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/ThrowAtStallingPlayerMessage.rs` | ~ |
| `client/report/bb2025/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2025/ThrowTeamMateRollMessage.rs` | ~ |
| `client/report/bb2025/UseFumblerooskieMessage.java` | `ffb-client` | `src/client/report/bb2025/UseFumblerooskieMessage.rs` | ~ |
| `client/report/bb2025/WeatherMageResultMessage.java` | `ffb-client` | `src/client/report/bb2025/WeatherMageResultMessage.rs` | ~ |
| `client/report/BiteSpectatorMessage.java` | `ffb-client` | `src/client/report/BiteSpectatorMessage.rs` | ~ |
| `client/report/BlockMessage.java` | `ffb-client` | `src/client/report/BlockMessage.rs` | ~ |
| `client/report/BlockRollMessage.java` | `ffb-client` | `src/client/report/BlockRollMessage.rs` | ~ |
| `client/report/BombExplodesAfterCatchMessage.java` | `ffb-client` | `src/client/report/BombExplodesAfterCatchMessage.rs` | ~ |
| `client/report/BombOutOfBoundsMessage.java` | `ffb-client` | `src/client/report/BombOutOfBoundsMessage.rs` | ~ |
| `client/report/BribesRollMessage.java` | `ffb-client` | `src/client/report/BribesRollMessage.rs` | ~ |
| `client/report/CardDeactivatedMessage.java` | `ffb-client` | `src/client/report/CardDeactivatedMessage.rs` | ~ |
| `client/report/CardEffectRollMessage.java` | `ffb-client` | `src/client/report/CardEffectRollMessage.rs` | ~ |
| `client/report/CatchRollMessage.java` | `ffb-client` | `src/client/report/CatchRollMessage.rs` | ~ |
| `client/report/ChainsawRollMessage.java` | `ffb-client` | `src/client/report/ChainsawRollMessage.rs` | ~ |
| `client/report/CoinThrowMessage.java` | `ffb-client` | `src/client/report/CoinThrowMessage.rs` | ~ |
| `client/report/ConfusionRollMessage.java` | `ffb-client` | `src/client/report/ConfusionRollMessage.rs` | ~ |
| `client/report/DauntlessRollMessage.java` | `ffb-client` | `src/client/report/DauntlessRollMessage.rs` | ~ |
| `client/report/DefectingPlayersMessage.java` | `ffb-client` | `src/client/report/DefectingPlayersMessage.rs` | ~ |
| `client/report/DodgeRollMessage.java` | `ffb-client` | `src/client/report/DodgeRollMessage.rs` | ~ |
| `client/report/DoubleHiredStarPlayerMessage.java` | `ffb-client` | `src/client/report/DoubleHiredStarPlayerMessage.rs` | ~ |
| `client/report/EscapeRollMessage.java` | `ffb-client` | `src/client/report/EscapeRollMessage.rs` | ~ |
| `client/report/FoulAppearanceRollMessage.java` | `ffb-client` | `src/client/report/FoulAppearanceRollMessage.rs` | ~ |
| `client/report/FoulMessage.java` | `ffb-client` | `src/client/report/FoulMessage.rs` | ~ |
| `client/report/FumbblResultUploadMessage.java` | `ffb-client` | `src/client/report/FumbblResultUploadMessage.rs` | ~ |
| `client/report/GameOptionsMessage.java` | `ffb-client` | `src/client/report/GameOptionsMessage.rs` | ~ |
| `client/report/HandOverMessage.java` | `ffb-client` | `src/client/report/HandOverMessage.rs` | ~ |
| `client/report/InterceptionRollMessage.java` | `ffb-client` | `src/client/report/InterceptionRollMessage.rs` | ~ |
| `client/report/JumpRollMessage.java` | `ffb-client` | `src/client/report/JumpRollMessage.rs` | ~ |
| `client/report/JumpUpRollMessage.java` | `ffb-client` | `src/client/report/JumpUpRollMessage.rs` | ~ |
| `client/report/KickoffResultMessage.java` | `ffb-client` | `src/client/report/KickoffResultMessage.rs` | ~ |
| `client/report/KickoffScatterMessage.java` | `ffb-client` | `src/client/report/KickoffScatterMessage.rs` | ~ |
| `client/report/LeaderMessage.java` | `ffb-client` | `src/client/report/LeaderMessage.rs` | ~ |
| `client/report/MasterChefRollMessage.java` | `ffb-client` | `src/client/report/MasterChefRollMessage.rs` | ~ |
| `client/report/mixed/AllYouCanEatMessage.java` | `ffb-client` | `src/client/report/mixed/AllYouCanEatMessage.rs` | ~ |
| `client/report/mixed/ArgueTheCallMessage.java` | `ffb-client` | `src/client/report/mixed/ArgueTheCallMessage.rs` | ~ |
| `client/report/mixed/BalefulHexRollMessage.java` | `ffb-client` | `src/client/report/mixed/BalefulHexRollMessage.rs` | ~ |
| `client/report/mixed/BiasedRefMessage.java` | `ffb-client` | `src/client/report/mixed/BiasedRefMessage.rs` | ~ |
| `client/report/mixed/BlockChoiceMessage.java` | `ffb-client` | `src/client/report/mixed/BlockChoiceMessage.rs` | ~ |
| `client/report/mixed/BlockReRollMessage.java` | `ffb-client` | `src/client/report/mixed/BlockReRollMessage.rs` | ~ |
| `client/report/mixed/BloodLustRollMessage.java` | `ffb-client` | `src/client/report/mixed/BloodLustRollMessage.rs` | ~ |
| `client/report/mixed/BreatheFireMessage.java` | `ffb-client` | `src/client/report/mixed/BreatheFireMessage.rs` | ~ |
| `client/report/mixed/BriberyAndCorruptionReRollMessage.java` | `ffb-client` | `src/client/report/mixed/BriberyAndCorruptionReRollMessage.rs` | ~ |
| `client/report/mixed/BrilliantCoachingReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/BrilliantCoachingReRollsLostMessage.rs` | ~ |
| `client/report/mixed/CatchOfTheDayMessage.java` | `ffb-client` | `src/client/report/mixed/CatchOfTheDayMessage.rs` | ~ |
| `client/report/mixed/CloudBursterMessage.java` | `ffb-client` | `src/client/report/mixed/CloudBursterMessage.rs` | ~ |
| `client/report/mixed/DedicatedFansMessage.java` | `ffb-client` | `src/client/report/mixed/DedicatedFansMessage.rs` | ~ |
| `client/report/mixed/DoubleHiredStaffMessage.java` | `ffb-client` | `src/client/report/mixed/DoubleHiredStaffMessage.rs` | ~ |
| `client/report/mixed/EventMessage.java` | `ffb-client` | `src/client/report/mixed/EventMessage.rs` | ~ |
| `client/report/mixed/FanFactorMessage.java` | `ffb-client` | `src/client/report/mixed/FanFactorMessage.rs` | ~ |
| `client/report/mixed/FreePettyCashMessage.java` | `ffb-client` | `src/client/report/mixed/FreePettyCashMessage.rs` | ~ |
| `client/report/mixed/GoForItRollMessage.java` | `ffb-client` | `src/client/report/mixed/GoForItRollMessage.rs` | ~ |
| `client/report/mixed/HitAndRunMessage.java` | `ffb-client` | `src/client/report/mixed/HitAndRunMessage.rs` | ~ |
| `client/report/mixed/IndomitableMessage.java` | `ffb-client` | `src/client/report/mixed/IndomitableMessage.rs` | ~ |
| `client/report/mixed/InducementMessage.java` | `ffb-client` | `src/client/report/mixed/InducementMessage.rs` | ~ |
| `client/report/mixed/KickoffPitchInvasionMessage.java` | `ffb-client` | `src/client/report/mixed/KickoffPitchInvasionMessage.rs` | ~ |
| `client/report/mixed/KickoffSequenceActivationsCountMessage.java` | `ffb-client` | `src/client/report/mixed/KickoffSequenceActivationsCountMessage.rs` | ~ |
| `client/report/mixed/KickoffSequenceActivationsExhaustedMessage.java` | `ffb-client` | `src/client/report/mixed/KickoffSequenceActivationsExhaustedMessage.rs` | ~ |
| `client/report/mixed/KickoffTimeoutMessage.java` | `ffb-client` | `src/client/report/mixed/KickoffTimeoutMessage.rs` | ~ |
| `client/report/mixed/LookIntoMyEyesRollMessage.java` | `ffb-client` | `src/client/report/mixed/LookIntoMyEyesRollMessage.rs` | ~ |
| `client/report/mixed/ModifiedDodgeResultSuccessfulMessage.java` | `ffb-client` | `src/client/report/mixed/ModifiedDodgeResultSuccessfulMessage.rs` | ~ |
| `client/report/mixed/ModifiedPassResultMessage.java` | `ffb-client` | `src/client/report/mixed/ModifiedPassResultMessage.rs` | ~ |
| `client/report/mixed/MostValuablePlayersMessage.java` | `ffb-client` | `src/client/report/mixed/MostValuablePlayersMessage.rs` | ~ |
| `client/report/mixed/NervesOfSteelMessage.java` | `ffb-client` | `src/client/report/mixed/NervesOfSteelMessage.rs` | ~ |
| `client/report/mixed/OldProMessage.java` | `ffb-client` | `src/client/report/mixed/OldProMessage.rs` | ~ |
| `client/report/mixed/PenaltyShootoutMessage.java` | `ffb-client` | `src/client/report/mixed/PenaltyShootoutMessage.rs` | ~ |
| `client/report/mixed/PickMeUpMessage.java` | `ffb-client` | `src/client/report/mixed/PickMeUpMessage.rs` | ~ |
| `client/report/mixed/PickUpRollMessage.java` | `ffb-client` | `src/client/report/mixed/PickUpRollMessage.rs` | ~ |
| `client/report/mixed/PlaceBallDirectionMessage.java` | `ffb-client` | `src/client/report/mixed/PlaceBallDirectionMessage.rs` | ~ |
| `client/report/mixed/PlayerEventMessage.java` | `ffb-client` | `src/client/report/mixed/PlayerEventMessage.rs` | ~ |
| `client/report/mixed/PrayerEndMessage.java` | `ffb-client` | `src/client/report/mixed/PrayerEndMessage.rs` | ~ |
| `client/report/mixed/PrayerWastedMessage.java` | `ffb-client` | `src/client/report/mixed/PrayerWastedMessage.rs` | ~ |
| `client/report/mixed/ProjectileVomitMessage.java` | `ffb-client` | `src/client/report/mixed/ProjectileVomitMessage.rs` | ~ |
| `client/report/mixed/PumpUpTheCrowdReRollMessage.java` | `ffb-client` | `src/client/report/mixed/PumpUpTheCrowdReRollMessage.rs` | ~ |
| `client/report/mixed/PumpUpTheCrowdReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/PumpUpTheCrowdReRollsLostMessage.rs` | ~ |
| `client/report/mixed/QuickSnapRollMessage.java` | `ffb-client` | `src/client/report/mixed/QuickSnapRollMessage.rs` | ~ |
| `client/report/mixed/RaidingPartyMessage.java` | `ffb-client` | `src/client/report/mixed/RaidingPartyMessage.rs` | ~ |
| `client/report/mixed/RefereeMessage.java` | `ffb-client` | `src/client/report/mixed/RefereeMessage.rs` | ~ |
| `client/report/mixed/ScatterBallMessage.java` | `ffb-client` | `src/client/report/mixed/ScatterBallMessage.rs` | ~ |
| `client/report/mixed/ScatterPlayerMessage.java` | `ffb-client` | `src/client/report/mixed/ScatterPlayerMessage.rs` | ~ |
| `client/report/mixed/SelectBlitzTargetMessage.java` | `ffb-client` | `src/client/report/mixed/SelectBlitzTargetMessage.rs` | ~ |
| `client/report/mixed/SelectGazeTargetMessage.java` | `ffb-client` | `src/client/report/mixed/SelectGazeTargetMessage.rs` | ~ |
| `client/report/mixed/ShowStarReRollMessage.java` | `ffb-client` | `src/client/report/mixed/ShowStarReRollMessage.rs` | ~ |
| `client/report/mixed/ShowStarReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/ShowStarReRollsLostMessage.rs` | ~ |
| `client/report/mixed/SkillUseOtherPlayerMessage.java` | `ffb-client` | `src/client/report/mixed/SkillUseOtherPlayerMessage.rs` | ~ |
| `client/report/mixed/SkillWastedMessage.java` | `ffb-client` | `src/client/report/mixed/SkillWastedMessage.rs` | ~ |
| `client/report/mixed/ThrownKegMessage.java` | `ffb-client` | `src/client/report/mixed/ThrownKegMessage.rs` | ~ |
| `client/report/mixed/TrapDoorMessage.java` | `ffb-client` | `src/client/report/mixed/TrapDoorMessage.rs` | ~ |
| `client/report/mixed/TurnEndMessage.java` | `ffb-client` | `src/client/report/mixed/TurnEndMessage.rs` | ~ |
| `client/report/mixed/WeatherMageRollMessage.java` | `ffb-client` | `src/client/report/mixed/WeatherMageRollMessage.rs` | ~ |
| `client/report/mixed/WinningsMessage.java` | `ffb-client` | `src/client/report/mixed/WinningsMessage.rs` | ~ |
| `client/report/PassBlockMessage.java` | `ffb-client` | `src/client/report/PassBlockMessage.rs` | ~ |
| `client/report/PassDeviateMessage.java` | `ffb-client` | `src/client/report/PassDeviateMessage.rs` | ~ |
| `client/report/PettyCashMessage.java` | `ffb-client` | `src/client/report/PettyCashMessage.rs` | ~ |
| `client/report/PilingOnMessage.java` | `ffb-client` | `src/client/report/PilingOnMessage.rs` | ~ |
| `client/report/PlayCardMessage.java` | `ffb-client` | `src/client/report/PlayCardMessage.rs` | ~ |
| `client/report/PlayerActionMessage.java` | `ffb-client` | `src/client/report/PlayerActionMessage.rs` | ~ |
| `client/report/PushbackMessage.java` | `ffb-client` | `src/client/report/PushbackMessage.rs` | ~ |
| `client/report/ReceiveChoiceMessage.java` | `ffb-client` | `src/client/report/ReceiveChoiceMessage.rs` | ~ |
| `client/report/RegenerationRollMessage.java` | `ffb-client` | `src/client/report/RegenerationRollMessage.rs` | ~ |
| `client/report/ReportMessageBase.java` | `ffb-client` | `src/client/report/ReportMessageBase.rs` | ~ |
| `client/report/ReportMessageType.java` | `ffb-client` | `src/client/report/ReportMessageType.rs` | ~ |
| `client/report/ReRollMessage.java` | `ffb-client` | `src/client/report/ReRollMessage.rs` | ~ |
| `client/report/RightStuffRollMessage.java` | `ffb-client` | `src/client/report/RightStuffRollMessage.rs` | ~ |
| `client/report/RiotousRookiesMessage.java` | `ffb-client` | `src/client/report/RiotousRookiesMessage.rs` | ~ |
| `client/report/SafeThrowRollMessage.java` | `ffb-client` | `src/client/report/SafeThrowRollMessage.rs` | ~ |
| `client/report/SecretWeaponBanMessage.java` | `ffb-client` | `src/client/report/SecretWeaponBanMessage.rs` | ~ |
| `client/report/SkillUseMessage.java` | `ffb-client` | `src/client/report/SkillUseMessage.rs` | ~ |
| `client/report/SpellEffectRollMessage.java` | `ffb-client` | `src/client/report/SpellEffectRollMessage.rs` | ~ |
| `client/report/StandUpRollMessage.java` | `ffb-client` | `src/client/report/StandUpRollMessage.rs` | ~ |
| `client/report/StartHalfMessage.java` | `ffb-client` | `src/client/report/StartHalfMessage.rs` | ~ |
| `client/report/ThrowInMessage.java` | `ffb-client` | `src/client/report/ThrowInMessage.rs` | ~ |
| `client/report/TimeoutEnforcedMessage.java` | `ffb-client` | `src/client/report/TimeoutEnforcedMessage.rs` | ~ |
| `client/report/WeatherMessage.java` | `ffb-client` | `src/client/report/WeatherMessage.rs` | ~ |
| `client/report/WeepingDaggerRollMessage.java` | `ffb-client` | `src/client/report/WeepingDaggerRollMessage.rs` | ~ |
| `client/report/WizardUseMessage.java` | `ffb-client` | `src/client/report/WizardUseMessage.rs` | ~ |

### client/root/ (31 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/ActionKey.java` | `ffb-client` | `src/client/ActionKey.rs` | ~ |
| `client/ActionKeyAction.java` | `ffb-client` | `src/client/ActionKeyAction.rs` | ~ |
| `client/ActionKeyBindings.java` | `ffb-client` | `src/client/ActionKeyBindings.rs` | ~ |
| `client/ActionKeyGroup.java` | `ffb-client` | `src/client/ActionKeyGroup.rs` | ~ |
| `client/ActionKeyMultiAction.java` | `ffb-client` | `src/client/ActionKeyMultiAction.rs` | ~ |
| `client/ClientData.java` | `ffb-client` | `src/client/ClientData.rs` | ~ |
| `client/ClientLayout.java` | `ffb-client` | `src/client/ClientLayout.rs` | ~ |
| `client/ClientParameters.java` | `ffb-client` | `src/client/ClientParameters.rs` | ~ |
| `client/ClientReplayer.java` | `ffb-client` | `src/client/ClientReplayer.rs` | ~ |
| `client/Component.java` | `ffb-client` | `src/client/Component.rs` | ~ |
| `client/CoordinateConverter.java` | `ffb-client` | `src/client/CoordinateConverter.rs` | ~ |
| `client/DimensionProvider.java` | `ffb-client` | `src/client/DimensionProvider.rs` | ~ |
| `client/DugoutDimensionProvider.java` | `ffb-client` | `src/client/DugoutDimensionProvider.rs` | ~ |
| `client/FantasyFootballClient.java` | `ffb-client` | `src/client/FantasyFootballClient.rs` | ~ |
| `client/FieldComponent.java` | `ffb-client` | `src/client/FieldComponent.rs` | ~ |
| `client/FontCache.java` | `ffb-client` | `src/client/FontCache.rs` | ~ |
| `client/GameTitle.java` | `ffb-client` | `src/client/GameTitle.rs` | ~ |
| `client/IconCache.java` | `ffb-client` | `src/client/IconCache.rs` | ~ |
| `client/IProgressListener.java` | `ffb-client` | `src/client/IProgressListener.rs` | ~ |
| `client/LayoutSettings.java` | `ffb-client` | `src/client/LayoutSettings.rs` | ~ |
| `client/ParagraphStyle.java` | `ffb-client` | `src/client/ParagraphStyle.rs` | ~ |
| `client/PitchDimensionProvider.java` | `ffb-client` | `src/client/PitchDimensionProvider.rs` | ~ |
| `client/PlayerIconFactory.java` | `ffb-client` | `src/client/PlayerIconFactory.rs` | ~ |
| `client/RenderContext.java` | `ffb-client` | `src/client/RenderContext.rs` | ~ |
| `client/ReplayControl.java` | `ffb-client` | `src/client/ReplayControl.rs` | ~ |
| `client/StatusReport.java` | `ffb-client` | `src/client/StatusReport.rs` | ~ |
| `client/StyleProvider.java` | `ffb-client` | `src/client/StyleProvider.rs` | ~ |
| `client/TextStyle.java` | `ffb-client` | `src/client/TextStyle.rs` | ~ |
| `client/UiDimensionProvider.java` | `ffb-client` | `src/client/UiDimensionProvider.rs` | ~ |
| `client/UserInterface.java` | `ffb-client` | `src/client/UserInterface.rs` | ~ |
| `client/UtilStyle.java` | `ffb-client` | `src/client/UtilStyle.rs` | ~ |

### client/sound/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/sound/ISoundProperty.java` | `ffb-client` | `src/client/sound/ISoundProperty.rs` | ~ |
| `client/sound/SoundEngine.java` | `ffb-client` | `src/client/sound/SoundEngine.rs` | ~ |

### client/state/ (85 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/state/ClientState.java` | `ffb-client` | `src/client/state/ClientState.rs` | ~ |
| `client/state/ClientStateFactory.java` | `ffb-client` | `src/client/state/ClientStateFactory.rs` | ~ |
| `client/state/IPlayerPopupMenuKeys.java` | `ffb-client` | `src/client/state/IPlayerPopupMenuKeys.rs` | ~ |
| `client/state/logic/AbstractBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/AbstractBlockLogicModule.rs` | ~ |
| `client/state/logic/bb2016/KtmLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2016/KtmLogicModule.rs` | ~ |
| `client/state/logic/bb2020/GazeMoveLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/GazeMoveLogicModule.rs` | ~ |
| `client/state/logic/bb2020/KickTeamMateLikeThrowLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/KickTeamMateLikeThrowLogicModule.rs` | ~ |
| `client/state/logic/bb2020/SelectBlitzTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/SelectBlitzTargetLogicModule.rs` | ~ |
| `client/state/logic/bb2020/SelectGazeTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/SelectGazeTargetLogicModule.rs` | ~ |
| `client/state/logic/bb2020/StabLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/StabLogicModule.rs` | ~ |
| `client/state/logic/bb2020/SynchronousMultiBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/SynchronousMultiBlockLogicModule.rs` | ~ |
| `client/state/logic/bb2020/ThrowKegLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/ThrowKegLogicModule.rs` | ~ |
| `client/state/logic/bb2020/TricksterLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/TricksterLogicModule.rs` | ~ |
| `client/state/logic/bb2025/BlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/BlockLogicModule.rs` | ~ |
| `client/state/logic/bb2025/BombLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/BombLogicModule.rs` | ~ |
| `client/state/logic/bb2025/FoulLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/FoulLogicModule.rs` | ~ |
| `client/state/logic/bb2025/GazeLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/GazeLogicModule.rs` | ~ |
| `client/state/logic/bb2025/GazeMoveLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/GazeMoveLogicModule.rs` | ~ |
| `client/state/logic/bb2025/HandOverLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/HandOverLogicModule.rs` | ~ |
| `client/state/logic/bb2025/PassLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/PassLogicModule.rs` | ~ |
| `client/state/logic/bb2025/PuntLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/PuntLogicModule.rs` | ~ |
| `client/state/logic/bb2025/SelectBlitzTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/SelectBlitzTargetLogicModule.rs` | ~ |
| `client/state/logic/bb2025/SelectLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/SelectLogicModule.rs` | ~ |
| `client/state/logic/bb2025/SwarmingLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/SwarmingLogicModule.rs` | ~ |
| `client/state/logic/bb2025/SynchronousMultiBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/SynchronousMultiBlockLogicModule.rs` | ~ |
| `client/state/logic/bb2025/ThrowKegLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/ThrowKegLogicModule.rs` | ~ |
| `client/state/logic/BlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/BlitzLogicModule.rs` | ~ |
| `client/state/logic/BlockLogicExtension.java` | `ffb-client` | `src/client/state/logic/BlockLogicExtension.rs` | ~ |
| `client/state/logic/ClientAction.java` | `ffb-client` | `src/client/state/logic/ClientAction.rs` | ~ |
| `client/state/logic/DumpOffLogicModule.java` | `ffb-client` | `src/client/state/logic/DumpOffLogicModule.rs` | ~ |
| `client/state/logic/HighKickLogicModule.java` | `ffb-client` | `src/client/state/logic/HighKickLogicModule.rs` | ~ |
| `client/state/logic/IllegalSubstitutionLogicModule.java` | `ffb-client` | `src/client/state/logic/IllegalSubstitutionLogicModule.rs` | ~ |
| `client/state/logic/Influences.java` | `ffb-client` | `src/client/state/logic/Influences.rs` | ~ |
| `client/state/logic/interaction/ActionContext.java` | `ffb-client` | `src/client/state/logic/interaction/ActionContext.rs` | ~ |
| `client/state/logic/interaction/InteractionResult.java` | `ffb-client` | `src/client/state/logic/interaction/InteractionResult.rs` | ~ |
| `client/state/logic/InterceptionLogicModule.java` | `ffb-client` | `src/client/state/logic/InterceptionLogicModule.rs` | ~ |
| `client/state/logic/KickoffLogicModule.java` | `ffb-client` | `src/client/state/logic/KickoffLogicModule.rs` | ~ |
| `client/state/logic/KickoffReturnLogicModule.java` | `ffb-client` | `src/client/state/logic/KickoffReturnLogicModule.rs` | ~ |
| `client/state/logic/LogicModule.java` | `ffb-client` | `src/client/state/logic/LogicModule.rs` | ~ |
| `client/state/logic/LoginLogicModule.java` | `ffb-client` | `src/client/state/logic/LoginLogicModule.rs` | ~ |
| `client/state/logic/mixed/BlockKindLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/BlockKindLogicModule.rs` | ~ |
| `client/state/logic/mixed/BlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/BlockLogicModule.rs` | ~ |
| `client/state/logic/mixed/BombLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/BombLogicModule.rs` | ~ |
| `client/state/logic/mixed/FoulLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/FoulLogicModule.rs` | ~ |
| `client/state/logic/mixed/FuriousOutburstLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/FuriousOutburstLogicModule.rs` | ~ |
| `client/state/logic/mixed/GazeLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/GazeLogicModule.rs` | ~ |
| `client/state/logic/mixed/HandOverLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/HandOverLogicModule.rs` | ~ |
| `client/state/logic/mixed/HitAndRunLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/HitAndRunLogicModule.rs` | ~ |
| `client/state/logic/mixed/KickEmBlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/KickEmBlitzLogicModule.rs` | ~ |
| `client/state/logic/mixed/KickEmBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/KickEmBlockLogicModule.rs` | ~ |
| `client/state/logic/mixed/MaximumCarnageLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/MaximumCarnageLogicModule.rs` | ~ |
| `client/state/logic/mixed/PassLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/PassLogicModule.rs` | ~ |
| `client/state/logic/mixed/PutridRegurgitationBlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/PutridRegurgitationBlitzLogicModule.rs` | ~ |
| `client/state/logic/mixed/PutridRegurgitationBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/PutridRegurgitationBlockLogicModule.rs` | ~ |
| `client/state/logic/mixed/RaidingPartyLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/RaidingPartyLogicModule.rs` | ~ |
| `client/state/logic/mixed/SelectLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/SelectLogicModule.rs` | ~ |
| `client/state/logic/mixed/SwarmingLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/SwarmingLogicModule.rs` | ~ |
| `client/state/logic/mixed/ThenIStartedBlastinLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/ThenIStartedBlastinLogicModule.rs` | ~ |
| `client/state/logic/MoveLogicModule.java` | `ffb-client` | `src/client/state/logic/MoveLogicModule.rs` | ~ |
| `client/state/logic/PassBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/PassBlockLogicModule.rs` | ~ |
| `client/state/logic/PlaceBallLogicModule.java` | `ffb-client` | `src/client/state/logic/PlaceBallLogicModule.rs` | ~ |
| `client/state/logic/plugin/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/BaseLogicPlugin.rs` | ~ |
| `client/state/logic/plugin/bb2025/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/BaseLogicPlugin.rs` | ~ |
| `client/state/logic/plugin/bb2025/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/BlockLogicExtensionPlugin.rs` | ~ |
| `client/state/logic/plugin/bb2025/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/MoveLogicPlugin.rs` | ~ |
| `client/state/logic/plugin/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/BlockLogicExtensionPlugin.rs` | ~ |
| `client/state/logic/plugin/LogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/LogicPlugin.rs` | ~ |
| `client/state/logic/plugin/mixed/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/BaseLogicPlugin.rs` | ~ |
| `client/state/logic/plugin/mixed/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/BlockLogicExtensionPlugin.rs` | ~ |
| `client/state/logic/plugin/mixed/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/MoveLogicPlugin.rs` | ~ |
| `client/state/logic/plugin/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/MoveLogicPlugin.rs` | ~ |
| `client/state/logic/PushbackLogicModule.java` | `ffb-client` | `src/client/state/logic/PushbackLogicModule.rs` | ~ |
| `client/state/logic/QuickSnapLogicModule.java` | `ffb-client` | `src/client/state/logic/QuickSnapLogicModule.rs` | ~ |
| `client/state/logic/RangeGridState.java` | `ffb-client` | `src/client/state/logic/RangeGridState.rs` | ~ |
| `client/state/logic/ReplayLogicModule.java` | `ffb-client` | `src/client/state/logic/ReplayLogicModule.rs` | ~ |
| `client/state/logic/SetupLogicModule.java` | `ffb-client` | `src/client/state/logic/SetupLogicModule.rs` | ~ |
| `client/state/logic/SolidDefenceLogicModule.java` | `ffb-client` | `src/client/state/logic/SolidDefenceLogicModule.rs` | ~ |
| `client/state/logic/SpectateLogicModule.java` | `ffb-client` | `src/client/state/logic/SpectateLogicModule.rs` | ~ |
| `client/state/logic/StartGameLogicModule.java` | `ffb-client` | `src/client/state/logic/StartGameLogicModule.rs` | ~ |
| `client/state/logic/SwoopLogicModule.java` | `ffb-client` | `src/client/state/logic/SwoopLogicModule.rs` | ~ |
| `client/state/logic/ThrowTeamMateLogicModule.java` | `ffb-client` | `src/client/state/logic/ThrowTeamMateLogicModule.rs` | ~ |
| `client/state/logic/TouchbackLogicModule.java` | `ffb-client` | `src/client/state/logic/TouchbackLogicModule.rs` | ~ |
| `client/state/logic/WaitForOpponentLogicModule.java` | `ffb-client` | `src/client/state/logic/WaitForOpponentLogicModule.rs` | ~ |
| `client/state/logic/WaitForSetupLogicModule.java` | `ffb-client` | `src/client/state/logic/WaitForSetupLogicModule.rs` | ~ |
| `client/state/logic/WizardLogicModule.java` | `ffb-client` | `src/client/state/logic/WizardLogicModule.rs` | ~ |

### client/ui/ (69 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/ui/BoxButtonComponent.java` | `ffb-client` | `src/client/ui/BoxButtonComponent.rs` | ~ |
| `client/ui/BoxComponent.java` | `ffb-client` | `src/client/ui/BoxComponent.rs` | ~ |
| `client/ui/BoxSlot.java` | `ffb-client` | `src/client/ui/BoxSlot.rs` | ~ |
| `client/ui/chat/Autocomplete.java` | `ffb-client` | `src/client/ui/chat/Autocomplete.rs` | ~ |
| `client/ui/chat/AutocompleteGenerator.java` | `ffb-client` | `src/client/ui/chat/AutocompleteGenerator.rs` | ~ |
| `client/ui/chat/ChatSegment.java` | `ffb-client` | `src/client/ui/chat/ChatSegment.rs` | ~ |
| `client/ui/chat/EmojiLookup.java` | `ffb-client` | `src/client/ui/chat/EmojiLookup.rs` | ~ |
| `client/ui/chat/EmojiPicker.java` | `ffb-client` | `src/client/ui/chat/EmojiPicker.rs` | ~ |
| `client/ui/chat/MessageParser.java` | `ffb-client` | `src/client/ui/chat/MessageParser.rs` | ~ |
| `client/ui/ChatButtonComponent.java` | `ffb-client` | `src/client/ui/ChatButtonComponent.rs` | ~ |
| `client/ui/ChatComponent.java` | `ffb-client` | `src/client/ui/ChatComponent.rs` | ~ |
| `client/ui/ChatLogDocument.java` | `ffb-client` | `src/client/ui/ChatLogDocument.rs` | ~ |
| `client/ui/ChatLogScrollPane.java` | `ffb-client` | `src/client/ui/ChatLogScrollPane.rs` | ~ |
| `client/ui/ChatLogTextPane.java` | `ffb-client` | `src/client/ui/ChatLogTextPane.rs` | ~ |
| `client/ui/ColorIcon.java` | `ffb-client` | `src/client/ui/ColorIcon.rs` | ~ |
| `client/ui/CommandHighlightArea.java` | `ffb-client` | `src/client/ui/CommandHighlightArea.rs` | ~ |
| `client/ui/CommandHighlighter.java` | `ffb-client` | `src/client/ui/CommandHighlighter.rs` | ~ |
| `client/ui/GameTitleUpdateTask.java` | `ffb-client` | `src/client/ui/GameTitleUpdateTask.rs` | ~ |
| `client/ui/GraphicsEnhancer.java` | `ffb-client` | `src/client/ui/GraphicsEnhancer.rs` | ~ |
| `client/ui/IntegerField.java` | `ffb-client` | `src/client/ui/IntegerField.rs` | ~ |
| `client/ui/IReplayMouseListener.java` | `ffb-client` | `src/client/ui/IReplayMouseListener.rs` | ~ |
| `client/ui/LogComponent.java` | `ffb-client` | `src/client/ui/LogComponent.rs` | ~ |
| `client/ui/menu/CardsMenu.java` | `ffb-client` | `src/client/ui/menu/CardsMenu.rs` | ~ |
| `client/ui/menu/FfbMenu.java` | `ffb-client` | `src/client/ui/menu/FfbMenu.rs` | ~ |
| `client/ui/menu/game/GameModeMenu.java` | `ffb-client` | `src/client/ui/menu/game/GameModeMenu.rs` | ~ |
| `client/ui/menu/game/ReplayMenu.java` | `ffb-client` | `src/client/ui/menu/game/ReplayMenu.rs` | ~ |
| `client/ui/menu/game/StandardGameMenu.java` | `ffb-client` | `src/client/ui/menu/game/StandardGameMenu.rs` | ~ |
| `client/ui/menu/GameMenuBar.java` | `ffb-client` | `src/client/ui/menu/GameMenuBar.rs` | ~ |
| `client/ui/menu/HelpMenu.java` | `ffb-client` | `src/client/ui/menu/HelpMenu.rs` | ~ |
| `client/ui/menu/InducementsMenu.java` | `ffb-client` | `src/client/ui/menu/InducementsMenu.rs` | ~ |
| `client/ui/menu/MissingPlayersMenu.java` | `ffb-client` | `src/client/ui/menu/MissingPlayersMenu.rs` | ~ |
| `client/ui/menu/OptionsMenu.java` | `ffb-client` | `src/client/ui/menu/OptionsMenu.rs` | ~ |
| `client/ui/menu/PrayersMenu.java` | `ffb-client` | `src/client/ui/menu/PrayersMenu.rs` | ~ |
| `client/ui/menu/settings/ClientGraphicsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/ClientGraphicsMenu.rs` | ~ |
| `client/ui/menu/settings/ClientSettingsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/ClientSettingsMenu.rs` | ~ |
| `client/ui/menu/settings/GamePlayMenu.java` | `ffb-client` | `src/client/ui/menu/settings/GamePlayMenu.rs` | ~ |
| `client/ui/menu/settings/UserSettingsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/UserSettingsMenu.rs` | ~ |
| `client/ui/menu/SetupMenu.java` | `ffb-client` | `src/client/ui/menu/SetupMenu.rs` | ~ |
| `client/ui/OffsetIcon.java` | `ffb-client` | `src/client/ui/OffsetIcon.rs` | ~ |
| `client/ui/PlayerDetailComponent.java` | `ffb-client` | `src/client/ui/PlayerDetailComponent.rs` | ~ |
| `client/ui/ResourceComponent.java` | `ffb-client` | `src/client/ui/ResourceComponent.rs` | ~ |
| `client/ui/ResourceSlot.java` | `ffb-client` | `src/client/ui/ResourceSlot.rs` | ~ |
| `client/ui/ResourceValue.java` | `ffb-client` | `src/client/ui/ResourceValue.rs` | ~ |
| `client/ui/ScoreBarComponent.java` | `ffb-client` | `src/client/ui/ScoreBarComponent.rs` | ~ |
| `client/ui/SideBarComponent.java` | `ffb-client` | `src/client/ui/SideBarComponent.rs` | ~ |
| `client/ui/strategies/click/ClickStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/ClickStrategy.rs` | ~ |
| `client/ui/strategies/click/ClickStrategyRegistry.java` | `ffb-client` | `src/client/ui/strategies/click/ClickStrategyRegistry.rs` | ~ |
| `client/ui/strategies/click/DoubleClickStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/DoubleClickStrategy.rs` | ~ |
| `client/ui/strategies/click/LeftClickAltStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickAltStrategy.rs` | ~ |
| `client/ui/strategies/click/LeftClickCtrlStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickCtrlStrategy.rs` | ~ |
| `client/ui/strategies/click/LeftClickNoModifierStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickNoModifierStrategy.rs` | ~ |
| `client/ui/strategies/click/LeftClickShiftStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickShiftStrategy.rs` | ~ |
| `client/ui/swing/JButton.java` | `ffb-client` | `src/client/ui/swing/JButton.rs` | ~ |
| `client/ui/swing/JCheckBox.java` | `ffb-client` | `src/client/ui/swing/JCheckBox.rs` | ~ |
| `client/ui/swing/JComboBox.java` | `ffb-client` | `src/client/ui/swing/JComboBox.rs` | ~ |
| `client/ui/swing/JLabel.java` | `ffb-client` | `src/client/ui/swing/JLabel.rs` | ~ |
| `client/ui/swing/JList.java` | `ffb-client` | `src/client/ui/swing/JList.rs` | ~ |
| `client/ui/swing/JMenu.java` | `ffb-client` | `src/client/ui/swing/JMenu.rs` | ~ |
| `client/ui/swing/JMenuItem.java` | `ffb-client` | `src/client/ui/swing/JMenuItem.rs` | ~ |
| `client/ui/swing/JPasswordField.java` | `ffb-client` | `src/client/ui/swing/JPasswordField.rs` | ~ |
| `client/ui/swing/JProgressBar.java` | `ffb-client` | `src/client/ui/swing/JProgressBar.rs` | ~ |
| `client/ui/swing/JRadioButton.java` | `ffb-client` | `src/client/ui/swing/JRadioButton.rs` | ~ |
| `client/ui/swing/JRadioButtonMenuItem.java` | `ffb-client` | `src/client/ui/swing/JRadioButtonMenuItem.rs` | ~ |
| `client/ui/swing/JTabbedPane.java` | `ffb-client` | `src/client/ui/swing/JTabbedPane.rs` | ~ |
| `client/ui/swing/JTable.java` | `ffb-client` | `src/client/ui/swing/JTable.rs` | ~ |
| `client/ui/swing/JTextField.java` | `ffb-client` | `src/client/ui/swing/JTextField.rs` | ~ |
| `client/ui/swing/ScaledBorderFactory.java` | `ffb-client` | `src/client/ui/swing/ScaledBorderFactory.rs` | ~ |
| `client/ui/swing/WrappingEditorKit.java` | `ffb-client` | `src/client/ui/swing/WrappingEditorKit.rs` | ~ |
| `client/ui/TurnDiceStatusComponent.java` | `ffb-client` | `src/client/ui/TurnDiceStatusComponent.rs` | ~ |

### client/util/ (11 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/util/MarkerService.java` | `ffb-client` | `src/client/util/MarkerService.rs` | ~ |
| `client/util/rng/MouseEntropySource.java` | `ffb-client` | `src/client/util/rng/MouseEntropySource.rs` | ~ |
| `client/util/UtilClientActionKeys.java` | `ffb-client` | `src/client/util/UtilClientActionKeys.rs` | ~ |
| `client/util/UtilClientChat.java` | `ffb-client` | `src/client/util/UtilClientChat.rs` | ~ |
| `client/util/UtilClientCursor.java` | `ffb-client` | `src/client/util/UtilClientCursor.rs` | ~ |
| `client/util/UtilClientGraphics.java` | `ffb-client` | `src/client/util/UtilClientGraphics.rs` | ~ |
| `client/util/UtilClientJTable.java` | `ffb-client` | `src/client/util/UtilClientJTable.rs` | ~ |
| `client/util/UtilClientPlayerDrag.java` | `ffb-client` | `src/client/util/UtilClientPlayerDrag.rs` | ~ |
| `client/util/UtilClientReflection.java` | `ffb-client` | `src/client/util/UtilClientReflection.rs` | ~ |
| `client/util/UtilClientThrowTeamMate.java` | `ffb-client` | `src/client/util/UtilClientThrowTeamMate.rs` | ~ |
| `client/util/UtilClientTimeout.java` | `ffb-client` | `src/client/util/UtilClientTimeout.rs` | ~ |

