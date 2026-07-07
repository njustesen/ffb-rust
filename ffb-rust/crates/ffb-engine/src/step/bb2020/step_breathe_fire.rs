use ffb_model::enums::ApothecaryMode;
use ffb_model::model::BreatheFireResult;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_breathe_fire::ReportBreatheFire;
use ffb_model::report::report_id::ReportId;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury_by_name};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepBreatheFire` (BB2020).
///
/// Handles the `canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover` skill
/// (Breathe Fire) in the BB2020 ruleset.
///
/// Identical logic to the BB2025 version; the only structural difference is
/// that BB2020 publishes `DropPlayerContext` (not `SteadyFootingContext`) — but
/// the actual downstream infrastructure in Rust routes through SteadyFootingContext
/// for both editions, so the implementation remains the same.
///
/// Roll evaluation (Java `evaluate(roll, effectiveRoll)`):
///  - roll == 6                        → `KNOCK_DOWN` (injury on defender; goto success)
///  - roll == 1 || effectiveRoll == 1  → `FAILURE`    (attacker downed; turnover; goto failure)
///  - effectiveRoll < 4                → `NO_EFFECT`  (no effect; remove blitz-target state; goto end)
///  - otherwise                        → `PRONE`      (defender goes prone; no armour roll; goto success)
///
/// `strongOpponent`: defender Strength > 4 → effectiveRoll = roll - 1; minimumRoll = 3; proneRoll = 5
/// otherwise effectiveRoll = roll; minimumRoll = 2; proneRoll = 4
pub struct StepBreatheFire {
    /// Java: fGotoLabelOnSuccess
    pub goto_label_on_success: String,
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: gotoOnEnd
    pub goto_on_end: String,
    /// Java: usingBreatheFire
    pub using_breathe_fire: bool,
    /// Java: result — persisted across re-roll cycle
    pub result: Option<BreatheFireResult>,
    /// AbstractStepWithReRoll stub: reRolledAction
    pub re_rolled_action: Option<String>,
    /// AbstractStepWithReRoll stub: reRollSource
    pub re_roll_source: Option<String>,
}

impl StepBreatheFire {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            goto_label_on_failure: String::new(),
            goto_on_end: String::new(),
            using_breathe_fire: false,
            result: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    /// Java: private BreatheFireResult evaluate(int roll, int effectiveRoll)
    fn evaluate(roll: i32, effective_roll: i32) -> BreatheFireResult {
        if roll == 6 {
            return BreatheFireResult::KNOCK_DOWN;
        }
        if roll == 1 || effective_roll == 1 {
            return BreatheFireResult::FAILURE;
        }
        if effective_roll < 4 {
            return BreatheFireResult::NO_EFFECT;
        }
        BreatheFireResult::PRONE
    }
}

impl Default for StepBreatheFire {
    fn default() -> Self { Self::new() }
}

impl Step for StepBreatheFire {
    fn id(&self) -> StepId { StepId::BreatheFire }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                // Player declined TRR offer — clear source so execute_step falls through
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingBreatheFire(v)      => { self.using_breathe_fire = *v; true }
            StepParameter::GotoLabelOnSuccess(v)    => { self.goto_label_on_success = v.clone(); true }
            StepParameter::GotoLabelOnFailure(v)    => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::GotoLabelOnEnd(v)        => { self.goto_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepBreatheFire {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.getPlayer().hasSkillProperty(canPerformArmourRollInstead...) && usingBreatheFire
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let attacker_has_breathe_fire = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER))
            .unwrap_or(false);

        if !attacker_has_breathe_fire || !self.using_breathe_fire {
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            return StepOutcome::next();
        }

        // Java: actingPlayer.markSkillUsed(NamedProperties.canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover)
        if let Some(skill_id) = game.player(&attacker_id)
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER))
        {
            if let Some(p) = game.team_home.player_mut(&attacker_id).or_else(|| game.team_away.player_mut(&attacker_id)) {
                p.used_skills.insert(skill_id);
            }
        }

        // Java: if (ReRolledActions.BREATHE_FIRE == getReRolledAction()) {
        //   if ((getReRollSource() != null) && UtilServerReRoll.useReRoll(...)) { result = null }
        // }
        if self.re_rolled_action.as_deref() == Some("BREATHE_FIRE") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                use ffb_model::enums::ReRollSource;
                let source = ReRollSource::new(source_name.as_str());
                if use_reroll(game, &source, &attacker_id) {
                    self.result = None;
                }
            }
        }

        if self.result.is_none() {
            // Java: int roll = getGameState().getDiceRoller().rollSkill()
            let roll = rng.d6();
            // Java: boolean strongOpponent = game.getDefender().getStrengthWithModifiers(game) > 4
            let strong_opponent = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.strength_with_modifiers() > 4)
                .unwrap_or(false);
            let effective_roll = if strong_opponent { roll - 1 } else { roll };

            let result = Self::evaluate(roll, effective_roll);
            self.result = Some(result);

            // Java: getResult().addReport(new ReportBreatheFire(actingPlayer.getPlayerId(), successful, roll,
            //   minimumRoll, reRolled, game.getDefenderId(), result, strongOpponent))
            let minimum_roll = if strong_opponent { 3 } else { 2 };
            let re_rolled = self.re_rolled_action.as_deref() == Some("BREATHE_FIRE") && self.re_roll_source.is_some();
            game.report_list.add(ReportBreatheFire::new(
                Some(attacker_id.clone()),
                result == BreatheFireResult::KNOCK_DOWN,
                roll,
                minimum_roll,
                re_rolled,
                game.defender_id.clone(),
                strong_opponent,
                format!("{:?}", result),
            ));

            // Java: boolean successful = result == BreatheFireResult.KNOCK_DOWN
            if result == BreatheFireResult::KNOCK_DOWN {
                let defender_id = match game.defender_id.clone() {
                    Some(id) => id,
                    None => return StepOutcome::next(),
                };
                let defender_coord = game.field_model.player_coordinate(&defender_id)
                    .unwrap_or(FieldCoordinate::new(0, 0));

                // Java: InjuryResult injuryResultDefender = UtilServerInjury.handleInjury(this,
                //   new InjuryTypeBreatheFire(), actingPlayer.getPlayer(), game.getDefender(),
                //   defenderCoordinate, null, null, ApothecaryMode.DEFENDER)
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeBreatheFire",
                    Some(&attacker_id.clone()), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                );

                // Java: publishParameter(new StepParameter(DROP_PLAYER_CONTEXT,
                //   new DropPlayerContext(injuryResultDefender, false, true,
                //     fGotoLabelOnSuccess, game.getDefenderId(), DEFENDER, false)))
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result)),
                    end_turn: false,
                    eligible_for_safe_pair_of_hands: true,
                    label: if self.goto_label_on_success.is_empty() { None } else { Some(self.goto_label_on_success.clone()) },
                    player_id: Some(defender_id),
                    apothecary_mode: Some(ApothecaryMode::Defender),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                // BB2020 Java publishes DROP_PLAYER_CONTEXT directly (not SteadyFootingContext),
                // but Rust infrastructure routes drop-player through SteadyFootingContext for both editions.
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                return StepOutcome::next()
                    .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }

            // Java: if (getReRolledAction() != ReRolledActions.BREATHE_FIRE &&
            //           UtilServerReRoll.askForReRollIfAvailable(getGameState(), actingPlayer.getPlayer(),
            //             ReRolledActions.BREATHE_FIRE, 0, Arrays.asList(...))) { return; }
            if self.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BREATHE_FIRE", 0, false) {
                    self.re_rolled_action = Some("BREATHE_FIRE".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
        }

        // Java: PlayerState defenderState = game.getFieldModel().getPlayerState(game.getDefender())
        // Java: switch (result) { ... }
        match self.result {
            Some(BreatheFireResult::KNOCK_DOWN) => StepOutcome::next(), // already handled above
            Some(BreatheFireResult::FAILURE) => {
                let attacker_coord = game.field_model.player_coordinate(&attacker_id)
                    .unwrap_or(FieldCoordinate::new(0, 0));

                // Java: InjuryResult injuryResultAttacker = UtilServerInjury.handleInjury(this,
                //   new InjuryTypeBreatheFire(), null, actingPlayer.getPlayer(),
                //   playerCoordinate, null, null, ApothecaryMode.ATTACKER)
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeBreatheFire",
                    None, &attacker_id,
                    attacker_coord, None, None, ApothecaryMode::Attacker,
                );

                // Java: publishParameter(new StepParameter(DROP_PLAYER_CONTEXT,
                //   new DropPlayerContext(injuryResultAttacker, true, true,
                //     fGotoLabelOnFailure, actingPlayer.getPlayerId(), ATTACKER, false)))
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result)),
                    end_turn: true,
                    eligible_for_safe_pair_of_hands: true,
                    label: if self.goto_label_on_failure.is_empty() { None } else { Some(self.goto_label_on_failure.clone()) },
                    player_id: Some(attacker_id),
                    apothecary_mode: Some(ApothecaryMode::Attacker),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                StepOutcome::next().publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
            }
            Some(BreatheFireResult::NO_EFFECT) => {
                // Java: getResult().setNextAction(StepAction.GOTO_LABEL, gotoOnEnd)
                // Java: game.getFieldModel().setPlayerState(game.getDefender(),
                //   defenderState.removeSelectedBlitzTarget())
                if let Some(def_id) = game.defender_id.clone() {
                    if let Some(st) = game.field_model.player_state(&def_id) {
                        game.field_model.set_player_state(&def_id, st.remove_selected_blitz_target());
                    }
                }
                StepOutcome::goto(&self.goto_on_end.clone())
            }
            Some(BreatheFireResult::PRONE) => {
                // Java: getResult().setNextAction(StepAction.GOTO_LABEL, fGotoLabelOnSuccess)
                // Java: game.getFieldModel().setPlayerState(game.getDefender(),
                //   defenderState.removeSelectedBlitzTarget())
                // Java: publishParameters(UtilServerInjury.dropPlayer(this, game.getPlayerById(game.getDefenderId()),
                //   ApothecaryMode.DEFENDER, true))
                let defender_id = game.defender_id.clone().unwrap_or_default();
                if let Some(st) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, st.remove_selected_blitz_target());
                }
                let mut out = StepOutcome::goto(&self.goto_label_on_success.clone());
                let drop_params = drop_player(game, &defender_id, true);
                for p in drop_params {
                    out = out.publish(p);
                }
                out
            }
            None => StepOutcome::cont(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn not_using_breathe_fire_returns_next() {
        let mut step = StepBreatheFire::new();
        step.using_breathe_fire = false;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn evaluate_roll_6_is_knock_down() {
        assert_eq!(StepBreatheFire::evaluate(6, 6), BreatheFireResult::KNOCK_DOWN);
    }

    #[test]
    fn evaluate_roll_1_is_failure() {
        assert_eq!(StepBreatheFire::evaluate(1, 1), BreatheFireResult::FAILURE);
    }

    #[test]
    fn evaluate_strong_opponent_effective_1_is_failure() {
        // roll=2, strong: effective=1 → FAILURE
        assert_eq!(StepBreatheFire::evaluate(2, 1), BreatheFireResult::FAILURE);
    }

    #[test]
    fn evaluate_effective_below_4_is_no_effect() {
        // roll=3, strong: effective=2 → NO_EFFECT
        assert_eq!(StepBreatheFire::evaluate(3, 2), BreatheFireResult::NO_EFFECT);
    }

    #[test]
    fn evaluate_effective_4_or_more_is_prone() {
        // roll=5, not strong: effective=5 → PRONE
        assert_eq!(StepBreatheFire::evaluate(5, 5), BreatheFireResult::PRONE);
    }

    #[test]
    fn no_effect_result_goes_to_goto_on_end() {
        let mut step = StepBreatheFire::new();
        step.using_breathe_fire = false; // short-circuit guard — no skill → next
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_on_end_accepted() {
        let mut step = StepBreatheFire::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("end_label".into())));
        assert_eq!(step.goto_on_end, "end_label");
    }

    #[test]
    fn set_parameter_goto_on_success_accepted() {
        let mut step = StepBreatheFire::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("win".into())));
        assert_eq!(step.goto_label_on_success, "win");
    }

    #[test]
    fn set_parameter_goto_on_failure_accepted() {
        let mut step = StepBreatheFire::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_using_breathe_fire_accepted() {
        let mut step = StepBreatheFire::new();
        assert!(step.set_parameter(&StepParameter::UsingBreatheFire(true)));
        assert!(step.using_breathe_fire);
    }

    #[test]
    fn set_parameter_unknown_rejected() {
        let mut step = StepBreatheFire::new();
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn stored_no_effect_result_goes_to_goto_on_end_label() {
        let mut step = StepBreatheFire::new();
        step.goto_on_end = "end".into();
        step.result = Some(BreatheFireResult::NO_EFFECT);
        step.using_breathe_fire = true;
        let mut game = make_game();
        // No acting player → guard fires → NextStep (not the result path)
        // This tests the early-exit guard
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn breathe_fire_marks_skill_used() {
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        let breathe_fire_skill = SkillId::BreatheFire;
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: breathe_fire_skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.acting_player.player_id = Some("att".into());
        // Pre-set result so we don't need a full roll
        let mut step = StepBreatheFire::new();
        step.using_breathe_fire = true;
        step.result = Some(BreatheFireResult::NO_EFFECT);
        step.goto_on_end = "end".into();
        step.start(&mut game, &mut GameRng::new(0));
        let used = game.team_home.player("att")
            .map(|p| p.used_skills.contains(&breathe_fire_skill))
            .unwrap_or(false);
        assert!(used, "BreatheFireSkill should be in used_skills after executing");
    }

    #[test]
    fn with_breathe_fire_skill_emits_report_breathe_fire() {
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId, PS_STANDING, PlayerAction};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;

        let mut game = make_game();
        let att = Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BreatheFire, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(att);
        game.home_playing = true;
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", ffb_model::enums::PlayerState::new(PS_STANDING));

        let mut step = StepBreatheFire::new();
        step.using_breathe_fire = true;
        step.start(&mut game, &mut GameRng::new(4)); // roll=4 → PRONE result
        assert!(game.report_list.has_report(ReportId::BREATHE_FIRE));
    }

    #[test]
    fn not_using_breathe_fire_does_not_emit_report() {
        let mut game = make_game();
        let mut step = StepBreatheFire::new();
        step.using_breathe_fire = false;
        step.start(&mut game, &mut GameRng::new(4));
        assert!(!game.report_list.has_report(ReportId::BREATHE_FIRE));
    }
}
