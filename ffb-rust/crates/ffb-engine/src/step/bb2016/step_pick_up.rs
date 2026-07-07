/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepPickUp` (BB2016).
///
/// Rolls to pick up the ball.  Skips if ball not at player's square or ball not moving.
/// On success → NEXT_STEP (ball stops moving).
/// On failure → CATCH_SCATTER_THROW_IN_MODE(FailedPickUp) + END_TURN + GOTO failure label.
///
/// BB2016 differs from BB2025:
/// - No SecureTheBall action path.
/// - No optional pick-up (Trickster) path — `PickUpOptional` not in BB2016.
/// - No override player id (RaidingParty) path.
/// - preventHoldBall (NamedProperties.preventHoldBall) still applies.
/// - Agility mechanic for minimumRoll is via AgilityMechanic (same as BB2020).
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
/// Sets: CATCH_SCATTER_THROW_IN_MODE, FEEDING_ALLOWED, END_TURN.
///
/// Re-roll order:
///   1. Skill re-roll (Sure Hands — canRerollPickup) — auto-used via find_skill_reroll_source.
///   2. Team Re-Roll token (TRR) — offered via ReRollOffer prompt.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepPickUp`.
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::ReRollSource;
use ffb_model::report::report_pickup_roll::ReportPickupRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::pickup_modifier_factory::PickupModifierFactory;
use ffb_mechanics::modifiers::pickup_context::PickupContext;

pub struct StepPickUp {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Persisted roll for re-roll path
    roll: i32,
}

impl StepPickUp {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }
}

impl Default for StepPickUp {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepPickUp {
    fn id(&self) -> StepId { StepId::PickUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            // Agent declined — clear source so execute_step sees None → fail
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepPickUp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: isPickUp() — player is at ball coordinate and ball is in play + moving
        let player_coord = game.field_model.player_coordinate(&player_id);
        let ball_at_player = game.field_model.ball_in_play
            && game.field_model.ball_moving
            && player_coord.is_some()
            && player_coord == game.field_model.ball_coordinate;

        if !ball_at_player {
            return StepOutcome::next();
        }

        // Java: if (actingPlayer.getPlayer().hasSkillProperty(NamedProperties.preventHoldBall)) → FAILURE
        let prevent = game.player(&player_id)
            .map(|p| p.has_skill_property(NamedProperties::PREVENT_HOLD_BALL))
            .unwrap_or(false);
        if prevent {
            let label = self.goto_label_on_failure.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::FeedingAllowed(false))
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp));
        }

        // Java: if (ReRolledActions.PICK_UP == getReRolledAction()) { try useReRoll or fail }
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "PICKUP").unwrap_or(false);
        if already_rerolled {
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &player_id))
                .unwrap_or(false);
            if !consumed {
                return self.fail_pick_up();
            }
        }

        self.pick_up(game, rng, &player_id)
    }

    fn pick_up(&mut self, game: &mut Game, rng: &mut GameRng, player_id: &str) -> StepOutcome {
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let minimum_roll = {
            let factory = PickupModifierFactory::for_rules(game.rules);
            if let Some(player) = game.player(player_id) {
                let ctx = PickupContext::new(game, player);
                let mods = factory.find_applicable(&ctx);
                let agility = player.agility as i32;
                PickupModifierFactory::minimum_roll(agility, &mods)
            } else {
                2
            }
        };

        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        let already_rerolled_for_report = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "PICKUP").unwrap_or(false);
        game.report_list.add(ReportPickupRoll::new(
            Some(player_id.to_owned()),
            successful,
            self.roll,
            minimum_roll,
            already_rerolled_for_report,
            vec![],
        ));

        if successful {
            game.field_model.ball_moving = false;
            return StepOutcome::next();
        }

        // Failure — attempt re-roll on first failure
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "PICKUP").unwrap_or(false);

        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("PICKUP"));

            // Skill re-roll (Sure Hands)
            let skill_source = find_skill_reroll_source(game, "PICKUP");
            if let Some(source) = skill_source {
                use_reroll(game, &source, player_id);
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                let pid = player_id.to_owned();
                return self.pick_up(game, rng, &pid);
            }

            // TRR offer
            if let Some(prompt) = ask_for_reroll_if_available(game, "PICKUP", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_pick_up()
    }

    fn fail_pick_up(&self) -> StepOutcome {
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::FeedingAllowed(false))
            .publish(StepParameter::EndTurn(true))
            .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use ffb_model::report::report_id::ReportId;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player_at_ball(game: &mut Game, id: &str) {
        let coord = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn ball_not_moving_returns_next_step() {
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        game.field_model.ball_moving = false;
        let mut step = StepPickUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_sets_ball_not_moving() {
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 6; // guaranteed success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.field_model.ball_moving);
    }

    #[test]
    fn failure_without_reroll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1; // guaranteed fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_publishes_end_turn_and_catch_mode() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_then_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 6; // success on re-roll
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepPickUp::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn success_adds_pick_up_roll_report() {
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PICK_UP_ROLL), "success should add ReportPickupRoll");
    }

    #[test]
    fn failure_adds_pick_up_roll_report() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PICK_UP_ROLL), "failure should also add ReportPickupRoll");
    }
}
