use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::go_for_it_modifier_factory::GoForItModifierFactory;
use ffb_mechanics::modifiers::go_for_it_context::GoForItContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepGoForIt.
///
/// BB2020 is identical to BB2025 except that GoForItModifierFactory uses BB2020 rules.
///
/// client-only: canChooseToIgnoreRushModifierAfterRoll dialog — headless never ignores rush modifier.
/// failedRushForJumpAlwaysLandsInTargetSquare skill check → wired in fail_gfi.
pub struct StepGoForIt {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: fBallandChainGfi
    pub ball_and_chain_gfi: bool,
    /// Java: fSecondGoForIt
    pub second_go_for_it: bool,
    /// Java: moveStart (set via setParameter)
    pub move_start: Option<FieldCoordinate>,
    /// Java: usingModifierIgnoringSkill (Boolean tristate)
    pub using_modifier_ignoring_skill: Option<bool>,
    /// Java: roll
    pub roll: i32,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepGoForIt {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            ball_and_chain_gfi: false,
            second_go_for_it: false,
            move_start: None,
            using_modifier_ignoring_skill: None,
            roll: 0,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Step for StepGoForIt {
    fn id(&self) -> StepId { StepId::GoForIt }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => {
                self.execute_step(game, rng)
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::BallAndChainGfi(v) => { self.ball_and_chain_gfi = *v; true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            _ => false,
        }
    }
}

impl StepGoForIt {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = game.acting_player.player_id.clone();
        let go_for_it_after_block = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::GO_FOR_IT_AFTER_BLOCK))
            .unwrap_or(false);
        let run_gfi = go_for_it_after_block == self.ball_and_chain_gfi;

        if !run_gfi {
            return StepOutcome::next();
        }

        let going_for_it = game.acting_player.goes_for_it;
        let current_move = game.acting_player.current_move;
        let ma = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.movement as i32)
            .unwrap_or(4);

        if !going_for_it || current_move <= ma {
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);
        let using_modifier_ignoring = self.using_modifier_ignoring_skill == Some(true);

        if already_rerolled && !using_modifier_ignoring {
            let pid = player_id.as_deref().unwrap_or("");
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, pid))
                .unwrap_or(false);
            if !consumed {
                return self.fail_gfi(game);
            }
        }

        self.rush(game, rng)
    }

    fn rush(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        let factory = GoForItModifierFactory::for_rules(game.rules);
        let (minimum_roll, mod_names): (i32, Vec<String>) = if let Some(pid) = player_id.as_deref() {
            if let Some(player) = game.player(pid) {
                let ctx = GoForItContext::new(game, player);
                let mods = factory.find_applicable(&ctx);
                let min = GoForItModifierFactory::minimum_roll_going_for_it(&mods);
                let names: Vec<String> = mods.iter().map(|m| m.get_report_string().to_string()).collect();
                (min, names)
            } else {
                (2, vec![])
            }
        } else {
            (2, vec![])
        };

        let successful = self.roll >= minimum_roll;

        // Java line 234-238: if (usingModifierIgnoringSkill == null) addReport(new ReportGoForItRoll(...))
        if self.using_modifier_ignoring_skill.is_none() {
            use ffb_model::report::report_go_for_it_roll::ReportGoForItRoll;
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "GFI").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            game.report_list.add(ReportGoForItRoll::new(
                player_id.clone(),
                successful,
                self.roll,
                minimum_roll,
                re_rolled,
                mod_names,
            ));
        }

        if successful {
            // Java: succeedGfi — if jumping and !secondGfi and currentMove > ma+1 → repeat
            let jumping = game.acting_player.jumping;
            let current_move = game.acting_player.current_move;
            let ma = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.movement as i32)
                .unwrap_or(4);
            if jumping && !self.second_go_for_it && current_move > ma + 1 {
                self.second_go_for_it = true;
                self.using_modifier_ignoring_skill = None;
                self.re_roll_state.re_rolled_action = None;
                self.roll = 0;
                return StepOutcome::repeat();
            }
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);

        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("GFI"));

            let skill_source = find_skill_reroll_source(game, "GFI");
            if let Some(source) = skill_source {
                let pid = player_id.as_deref().unwrap_or("").to_owned();
                use_reroll(game, &source, &pid);
                self.re_roll_state.re_roll_source = Some(source);
                self.using_modifier_ignoring_skill = None;
                self.roll = 0;
                return self.rush(game, rng);
            }

            if let Some(prompt) = ask_for_reroll_if_available(game, "GFI", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_gfi(game)
    }

    fn fail_gfi(&mut self, game: &mut Game) -> StepOutcome {
        let jumping = game.acting_player.jumping;
        let current_move = game.acting_player.current_move;
        let pid = game.acting_player.player_id.clone();
        let ma = pid.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.movement as i32)
            .unwrap_or(4);
        // Java: if (jumping && !secondGfi && currentMove > ma+1 && !failedRushForJumpAlwaysLandsInTargetSquare)
        let always_lands = pid.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::FAILED_RUSH_FOR_JUMP_ALWAYS_LANDS_IN_TARGET_SQUARE))
            .unwrap_or(false);
        let mut outcome = StepOutcome::goto(&self.goto_label_on_failure.clone())
            .publish(StepParameter::EndTurn(true));
        if jumping && !self.second_go_for_it && current_move > ma + 1 && !always_lands {
            if let Some(start) = self.move_start {
                if let Some(id) = pid.as_deref() {
                    game.field_model.set_player_coordinate(id, start);
                }
            }
            outcome = outcome.publish(StepParameter::CoordinateFrom(FieldCoordinate::new(0, 0)));
        }
        if self.ball_and_chain_gfi {
            game.acting_player.fell_from_rush = true;
        }
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropGFI".into());
        outcome.publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{SkillId, PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn make_gfi_game() -> Game {
        let mut game = make_game();
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10;
        game
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
    }

    #[test]
    fn success_on_roll_two_or_above_returns_next_step() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_on_roll_one_goes_to_failure_label() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_publishes_end_turn() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepGoForIt::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn jumping_with_extra_move_on_success_triggers_second_gfi_repeat() {
        // ma=4, current_move=6 (> ma+1=5), jumping=true, !second_gfi → Repeat
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 6;
        game.acting_player.jumping = true;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4; // success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Repeat);
        assert!(step.second_go_for_it);
    }

    #[test]
    fn second_gfi_success_does_not_repeat_again() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 6;
        game.acting_player.jumping = true;
        let mut step = StepGoForIt::new("fail".into());
        step.second_go_for_it = true; // already did first gfi repeat
        step.roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_with_extra_move_on_failure_moves_player_to_move_start() {
        let start = FieldCoordinate::new(3, 3);
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 6;
        game.acting_player.jumping = true;
        let mut step = StepGoForIt::new("fail".into());
        step.move_start = Some(start);
        step.roll = 1; // fail
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(start));
    }

    #[test]
    fn success_emits_go_for_it_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GO_FOR_IT_ROLL));
    }

    #[test]
    fn failure_emits_go_for_it_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GO_FOR_IT_ROLL));
    }
}
