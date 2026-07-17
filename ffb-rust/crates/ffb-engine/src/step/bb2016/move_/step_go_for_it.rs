use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::PlayerAction;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_go_for_it_roll::ReportGoForItRoll;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::go_for_it_modifier_factory::GoForItModifierFactory;
use ffb_mechanics::modifiers::go_for_it_context::GoForItContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepGoForIt.
///
/// Resolves a Go-For-It (rush) roll in BB2016.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), BALL_AND_CHAIN_GFI (optional).
///
/// BB2016 differences from BB2025:
/// - On BLITZ action + first call: sets blitzUsed=true, increments currentMove,
///   recomputes goesForIt via isNextMoveGoingForIt.
/// - On success: if jumping and currentMove > MA+1 and !secondGoForIt →
///   pushCurrentStepOnStack (Repeat) for a second GFI.
/// - Publishes INJURY_TYPE(InjuryTypeDropGFI) instead of STEADY_FOOTING_CONTEXT.
///
pub struct StepGoForIt {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: fBallandChainGfi
    pub ball_and_chain_gfi: bool,
    /// Java: fSecondGoForIt
    pub second_go_for_it: bool,
    /// Java: roll (internal)
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
            roll: 0,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepGoForIt {
    fn default() -> Self { Self::new(String::new()) }
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

        // Java: if (BLITZ == actingPlayer.getPlayerAction()) && (getReRolledAction() == null)
        //         → setBlitzUsed(true), increment currentMove, recompute goingForIt
        let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);
        let not_rerolled = self.re_roll_state.re_rolled_action.is_none();
        if is_blitz && not_rerolled {
            game.turn_data_mut().blitz_used = true;
            game.acting_player.current_move += 1;
            // Java: actingPlayer.setGoingForIt(UtilPlayer.isNextMoveGoingForIt(game))
            let ma = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.movement as i32)
                .unwrap_or(4);
            use crate::util::movement_calc::MovementCalc;
            game.acting_player.goes_for_it =
                MovementCalc::is_next_move_going_for_it(game.acting_player.current_move, ma);
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

        // Java: if (GO_FOR_IT == reRolledAction)
        //         if (source==null || !useReRoll) → failGfi
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);

        if already_rerolled {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                return self.fail_gfi(game);
            }
        }

        self.do_go_for_it(game, rng)
    }

    fn do_go_for_it(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        let minimum_roll = {
            let factory = GoForItModifierFactory::for_rules(game.rules);
            if let Some(pid) = player_id.as_deref() {
                if let Some(player) = game.player(pid) {
                    let ctx = GoForItContext::new(game, player);
                    let mods = factory.find_applicable(&ctx);
                    let card_mods = factory.find_card_modifiers(&ctx);
                    let all: Vec<&ffb_mechanics::modifiers::go_for_it_modifier::GoForItModifier> = mods.iter().copied().chain(card_mods.iter()).collect();
                    GoForItModifierFactory::minimum_roll_going_for_it(&all)
                } else { 2 }
            } else { 2 }
        };

        let successful = self.roll >= minimum_roll;

        // Java: getResult().addReport(new ReportGoForItRoll(actingPlayer.getPlayerId(), successful, roll,
        //         minimumRoll, reRolled, goForItModifiers.toArray(...)))
        let re_rolled = self.re_roll_state.re_rolled_action.as_ref().map(|a| a.name == "GFI").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        game.report_list.add(ReportGoForItRoll::new(
            player_id.clone(),
            successful,
            self.roll,
            minimum_roll,
            re_rolled,
            vec![],
        ));

        if successful {
            // Java: if (actingPlayer.isJumping() && currentMove > MA+1 && !fSecondGoForIt)
            //         → fSecondGoForIt=true, setReRolledAction(null), pushCurrentStepOnStack (Repeat)
            let jumping = game.acting_player.jumping;
            let current_move = game.acting_player.current_move;
            let ma = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.movement as i32)
                .unwrap_or(4);
            if jumping && current_move > ma + 1 && !self.second_go_for_it {
                self.second_go_for_it = true;
                self.re_roll_state.re_rolled_action = None;
                return StepOutcome::repeat();
            }
            return StepOutcome::next();
        }

        // First failure: try re-roll
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);

        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("GFI"));

            // Skill re-roll (e.g. Sprint)
            let skill_source = find_skill_reroll_source(game, "GFI");
            if let Some(source) = skill_source {
                let pid = player_id.as_deref().unwrap_or("").to_owned();
                use_reroll(game, &source, &pid);
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                return self.do_go_for_it(game, rng);
            }

            // TRR offer
            if let Some(prompt) = ask_for_reroll_if_available(game, "GFI", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_gfi(game)
    }

    fn fail_gfi(&self, game: &mut Game) -> StepOutcome {
        if self.ball_and_chain_gfi {
            game.acting_player.fell_from_rush = true;
        }
        // Java: publishParameter(INJURY_TYPE, new InjuryTypeDropGFI())
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndTurn(true))
            .publish(StepParameter::InjuryTypeName("InjuryTypeDropGFI".into()))
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
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn make_gfi_game() -> Game {
        let mut game = make_game();
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10;
        game
    }

    fn add_player(game: &mut Game, id: &str) {
        use ffb_model::enums::{PlayerType, PlayerGender};
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
    fn ball_and_chain_gfi_skips_gfi_check_when_flag_mismatch() {
        // goForItAfterBlock==false, ball_and_chain_gfi==true → runGfi=false → NEXT_STEP
        let mut game = make_game();
        let mut step = StepGoForIt::new("fail".into());
        step.ball_and_chain_gfi = true;
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepGoForIt::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_ball_and_chain_gfi_accepted() {
        let mut step = StepGoForIt::new("fail".into());
        assert!(step.set_parameter(&StepParameter::BallAndChainGfi(true)));
        assert!(step.ball_and_chain_gfi);
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
    fn accept_reroll_then_success_returns_next_step() {
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
        step.roll = 4;
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn not_going_for_it_returns_next_step() {
        let mut game = make_game();
        game.acting_player.goes_for_it = false;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn current_move_at_ma_returns_next_step() {
        // current_move <= ma → not actually going for it
        let mut game = make_game();
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 4; // = default MA
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepGoForIt::new("fail".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn blitz_action_sets_blitz_used_and_increments_current_move() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10; // well past MA(4)+1 → GFI
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4; // success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used, "blitz_used should be set on BLITZ action");
    }

    #[test]
    fn blitz_action_does_not_set_blitz_used_when_already_rerolled() {
        // Java: only sets blitzUsed if reRolledAction == null
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4;
        use ffb_model::model::re_rolled_action::ReRolledAction;
        step.re_roll_state.re_rolled_action = Some(ReRolledAction::new("GFI"));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.turn_data().blitz_used, "blitz_used should NOT be set when already rerolled");
    }

    #[test]
    fn second_go_for_it_repeat_on_jumping_player() {
        // Java: if jumping && currentMove > MA+1 && !secondGoForIt → Repeat
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.jumping = true;
        game.acting_player.current_move = 6; // MA=4, so 6 > 4+1=5 → second GFI
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4; // success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Repeat, "jumping player should trigger second GFI on first success");
        assert!(step.second_go_for_it, "second_go_for_it flag should be set");
    }

    #[test]
    fn second_go_for_it_not_triggered_if_already_set() {
        // Once secondGoForIt is set, the next success returns NEXT_STEP
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.jumping = true;
        game.acting_player.current_move = 6;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4;
        step.second_go_for_it = true; // already done first repeat
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_adds_go_for_it_roll_report() {
        // Java: getResult().addReport(new ReportGoForItRoll(...)) on success
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 4; // success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::GO_FOR_IT_ROLL),
            "GO_FOR_IT_ROLL report should be added on a successful roll"
        );
    }

    #[test]
    fn failure_adds_go_for_it_roll_report() {
        // Java: addReport is added regardless of success/failure before the reroll branch
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1; // failure, no rerolls → goes directly to fail_gfi
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::GO_FOR_IT_ROLL),
            "GO_FOR_IT_ROLL report should be added on a failed roll"
        );
    }
}
