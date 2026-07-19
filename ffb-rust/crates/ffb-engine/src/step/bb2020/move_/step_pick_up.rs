use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::pickup_modifier_factory::PickupModifierFactory;
use ffb_mechanics::modifiers::pickup_context::PickupContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepPickUp.
///
/// Note: unlike the BB2025 sibling, BB2020 has no SECURE_THE_BALL / Trickster
/// (PICK_UP_OPTIONAL / ATTEMPT_PICK_UP / PLAYER_ON_BALL_ID) support — those
/// parameters/fields do not exist on the Java BB2020 class.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), THROWN_PLAYER_ID (optional).
/// Sets: CATCH_SCATTER_THROW_IN_MODE, FEEDING_ALLOWED, END_TURN.
pub struct StepPickUp {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: thrownPlayerId (init param — for TTM sequences)
    pub thrown_player_id: Option<String>,
    /// Java: ignore — set via FollowupChoice(false), skip pick-up
    pub ignore: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Persisted roll for re-roll (Java rolls inside pickUp() on first call)
    roll: i32,
}

impl StepPickUp {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            thrown_player_id: None,
            ignore: false,
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }
}

impl Step for StepPickUp {
    fn id(&self) -> StepId { StepId::PickUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
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
            StepParameter::FollowupChoice(v) => { self.ignore = !v; true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepPickUp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = self.thrown_player_id.clone()
            .or_else(|| game.acting_player.player_id.clone());

        let is_pick_up = {
            let coord = player_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            !self.ignore
                && game.field_model.ball_in_play
                && game.field_model.ball_moving
                && coord.is_some()
                && coord == game.field_model.ball_coordinate
        };
        if player_id.is_none() || !is_pick_up {
            return StepOutcome::next();
        }

        let has_tacklezones = player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .map(|s| s.has_tacklezones())
            .unwrap_or(true);
        if !has_tacklezones {
            // Java: a player of the own team without tackle zone was moved onto the ball
            // with e.g. Raiding Party or some other voluntary movement (no chain pushes).
            // This should be considered a pickup fail, unless the player has e.g. Ball And Chain.
            let label = self.goto_label_on_failure.clone();
            let acting_team_has_player = player_id.as_deref()
                .map(|id| game.active_team().has_player(id))
                .unwrap_or(false);
            let has_prevent = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::PREVENT_PICKUP))
                .unwrap_or(false);
            let mut out = StepOutcome::goto(&label)
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp));
            if acting_team_has_player && !has_prevent {
                out = out.publish(StepParameter::EndTurn(true));
            }
            return out;
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "PICKUP").unwrap_or(false);
        if already_rerolled {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                // Java: getReRollSource() == null || !useReRoll(...) → unconditional END_TURN,
                // regardless of preventPickup (unlike the pickUp()-FAILURE path below).
                let label = self.goto_label_on_failure.clone();
                return StepOutcome::goto(&label)
                    .publish(StepParameter::FeedingAllowed(false))
                    .publish(StepParameter::EndTurn(true))
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp));
            }
        }

        self.pick_up(game, rng, player_id)
    }

    fn pick_up(&mut self, game: &mut Game, rng: &mut GameRng, player_id: Option<String>) -> StepOutcome {
        let prevent = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| {
                p.has_skill_property(NamedProperties::PREVENT_HOLD_BALL)
                    || p.has_skill_property(NamedProperties::PREVENT_PICKUP)
            })
            .unwrap_or(false);
        if prevent {
            return self.fail_pick_up(game, &player_id);
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let factory = PickupModifierFactory::for_rules(game.rules);
        let (minimum_roll, mod_names): (i32, Vec<String>) = if let Some(pid) = player_id.as_deref() {
            if let Some(player) = game.player(pid) {
                let ctx = PickupContext::new(game, player);
                let mods = factory.find_applicable(&ctx);
                let skill_mods = factory.find_skill_modifiers(&ctx);
                let all: Vec<&ffb_mechanics::modifiers::pickup_modifier::PickupModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
                let min = PickupModifierFactory::minimum_roll(player.agility as i32, &all);
                let names: Vec<String> = all.iter().map(|m| m.get_report_string().to_string()).collect();
                (min, names)
            } else {
                (2, vec![])
            }
        } else {
            (2, vec![])
        };

        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        // Java line 191: addReport(new ReportPickupRoll(...))
        {
            use ffb_model::report::mixed::report_pickup_roll::ReportPickupRoll;
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "PICKUP").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            game.report_list.add(ReportPickupRoll::new(
                player_id.clone(),
                successful,
                self.roll,
                minimum_roll,
                re_rolled,
                mod_names,
            ));
        }

        if successful {
            game.field_model.ball_moving = false;
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "PICKUP").unwrap_or(false);

        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("PICKUP"));

            let skill_source = find_skill_reroll_source(game, "PICKUP");
            if let Some(source) = skill_source {
                let pid = player_id.as_deref().unwrap_or("").to_owned();
                use_reroll(game, &source, &pid);
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                return self.pick_up(game, rng, player_id);
            }

            if let Some(prompt) = ask_for_reroll_if_available(game, "PICKUP", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_pick_up(game, &player_id)
    }

    fn fail_pick_up(&self, game: &Game, player_id: &Option<String>) -> StepOutcome {
        let prevent_pickup = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::PREVENT_PICKUP))
            .unwrap_or(false);
        let label = self.goto_label_on_failure.clone();
        let mut out = StepOutcome::goto(&label)
            .publish(StepParameter::FeedingAllowed(false));
        if !prevent_pickup {
            out = out.publish(StepParameter::EndTurn(true));
        }
        out.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::FailedPickUp))
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
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
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
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn ignore_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPickUp::new("fail".into());
        step.ignore = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_followup_choice_false_sets_ignore() {
        let mut step = StepPickUp::new("fail".into());
        step.set_parameter(&StepParameter::FollowupChoice(false));
        assert!(step.ignore);
    }

    #[test]
    fn failure_without_reroll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
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
    fn success_sets_ball_not_moving() {
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 6;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.field_model.ball_moving);
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
    fn decline_reroll_publishes_unconditional_end_turn() {
        // Java StepPickUp (bb2020) executeStep(): when the re-roll-consumption path fails
        // (getReRollSource() == null after declining), END_TURN is published unconditionally
        // — it is not gated on preventPickup like the pickUp()-FAILURE path is. This test
        // pins that behavior (previously this file had been overwritten with BB2025's
        // Trickster/Secure-The-Ball logic, which doesn't apply to BB2020 at all).
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
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::FeedingAllowed(false))));
    }

    #[test]
    fn success_emits_pickup_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PICK_UP_ROLL));
    }

    #[test]
    fn failure_emits_pickup_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player_at_ball(&mut game, "p1");
        let mut step = StepPickUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PICK_UP_ROLL));
    }
}
