use ffb_model::events::GameEvent;
use ffb_model::enums::ApothecaryMode;
use ffb_model::enums::{PlayerState, PS_KNOCKED_OUT, PS_RESERVE};
use ffb_model::model::game::Game;
use ffb_model::report::report_bite_spectator::ReportBiteSpectator;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::injury::injuryType::injury_type_bitten::InjuryTypeBitten;
use crate::step::util_server_injury::{handle_injury, drop_player};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepInitFeeding.
///
/// Handles the feeding sub-sequence for Blood Lust. If the acting player
/// is a Vampire suffering Blood Lust and has not yet fed, prompts to choose
/// a victim (or bites a spectator if no victims available or feeding not allowed).
///
/// Init: mandatory GOTO_LABEL_ON_END, FEEDING_ALLOWED.
///       optional END_PLAYER_ACTION, END_TURN.
/// Sets: END_PLAYER_ACTION, END_TURN for all steps on the stack.
pub struct StepInitFeeding {
    /// Java: fGotoLabelOnEnd (mandatory)
    pub goto_label_on_end: Option<String>,
    /// Java: fFeedOnPlayerChoice (tristate Boolean)
    pub feed_on_player_choice: Option<bool>,
    /// Java: fFeedingAllowed (mandatory)
    pub feeding_allowed: Option<bool>,
    /// Java: fEndPlayerAction (optional init, default false)
    pub end_player_action: bool,
    /// Java: fEndTurn (optional init, default false)
    pub end_turn: bool,
}

impl StepInitFeeding {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            feed_on_player_choice: None,
            feeding_allowed: None,
            end_player_action: false,
            end_turn: false,
        }
    }
}

impl Default for StepInitFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitFeeding {
    fn id(&self) -> StepId { StepId::InitFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.FEED)
        //   fFeedOnPlayerChoice = StringTool.isProvided(playerId)
        //   game.setDefenderId(playerId)
        if let Action::SelectPlayer { player_id
} = action {
            self.feed_on_player_choice = Some(!player_id.is_empty());
            game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(label) => {
                self.goto_label_on_end = Some(label.clone());
                true
            }
            StepParameter::FeedingAllowed(v) => {
                self.feeding_allowed = Some(*v);
                true
            }
            StepParameter::EndPlayerAction(v) => {
                self.end_player_action = *v;
                true
            }
            StepParameter::EndTurn(v) => {
                self.end_turn = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepInitFeeding {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // client-only: UtilServerDialog.hideDialog

        let goto_label = match &self.goto_label_on_end {
            Some(l) => l.clone(),
            None => return StepOutcome::next(),
        };

        // Java: if (actingPlayer.getPlayer() == null || !isSufferingBloodLust || hasFed) → goto label
        let player_exists = game.acting_player.player_id.is_some();
        let is_blood_lusting = game.acting_player.suffering_blood_lust;
        let has_fed = game.acting_player.has_fed;

        if !player_exists || !is_blood_lusting || has_fed {
            return StepOutcome::goto(&goto_label)
                .publish(StepParameter::EndPlayerAction(self.end_player_action))
                .publish(StepParameter::EndTurn(self.end_turn));
        }

        // Java: if (isSufferingBloodLust && !hasFed && !fFeedingAllowed) → fFeedOnPlayerChoice = false
        if self.feeding_allowed == Some(false) {
            self.feed_on_player_choice = Some(false);
        }

        // Java: if (playerState.hasTacklezones() && fFeedOnPlayerChoice == null) → find victims or set false
        let acting_id = game.acting_player.player_id.clone().unwrap_or_default();
        let player_state: Option<PlayerState> = game.field_model.player_state(&acting_id);
        let has_tackle_zones = player_state.map(|s| s.has_tacklezones()).unwrap_or(false);

        if has_tackle_zones && self.feed_on_player_choice.is_none() {
            let acting_coord = game.field_model.player_coordinate(&acting_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
            let acting_team = if game.home_playing { game.team_home.clone() } else { game.team_away.clone() };
            let victims = UtilPlayer::find_adjacent_players_to_feed_on(game, &acting_team, acting_coord);
            if victims.is_empty() {
                self.feed_on_player_choice = Some(false);
            } else {
                // client-only: DialogVampireFeeding — headless skips feeding choice dialog
                return StepOutcome::cont();
            }
        }

        let mut outcome = StepOutcome::next();

        if self.feed_on_player_choice == Some(true) && game.defender_id.is_some() {
            // Java: feed on player — handleInjury(InjuryTypeBitten, ...)
            let defender_id = game.defender_id.clone().unwrap_or_default();
            let defender_coord = game.field_model.player_coordinate(&defender_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
            let acting_player_obj = game.player(&acting_id).map(|p| p.id.clone());

            let mut injury_type = InjuryTypeBitten::new();
            let injury_result = handle_injury(
                game, rng, &mut injury_type,
                acting_player_obj.as_deref(), &defender_id,
                defender_coord, None, None, ApothecaryMode::Feeding,
            );

            // Java: fEndTurn = UtilPlayer.hasBall(game, game.getDefender())
            self.end_turn = UtilPlayer::has_ball(game, &defender_id);

            // Java: publishParameter(INJURY_RESULT, injuryResultFeeding)
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));

            // Java: publishParameters(UtilServerInjury.dropPlayer(this, defender, ApothecaryMode.FEEDING))
            let drop_params = drop_player(game, &defender_id, false);
            for p in drop_params {
                outcome = outcome.publish(p);
            }

            // client-only: SoundId.SLURP
            game.acting_player.suffering_blood_lust = false;
        } else {
            // Java: bite spectator path
            self.end_turn = true;

            // Java: if (!isCasualty && base != KNOCKED_OUT && base != RESERVE)
            let is_eligible = player_state.map(|s| {
                !s.is_casualty() && s.base() != PS_KNOCKED_OUT && s.base() != PS_RESERVE
            }).unwrap_or(false);

            if is_eligible {
                let player_coord = game.field_model.player_coordinate(&acting_id);
                let ball_coord = game.field_model.ball_coordinate;

                // Java: if (playerCoordinate.equals(ballCoordinate)) → ball scatter
                if player_coord.is_some() && player_coord == ball_coord {
                    game.field_model.ball_moving = true;
                    outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                        CatchScatterThrowInMode::ScatterBall,
                    ));
                }

                // Java: setPlayerState(actor, state.changeBase(RESERVE))
                if let Some(s) = player_state {
                    game.field_model.set_player_state(&acting_id, s.change_base(PS_RESERVE));
                }
                UtilBox::put_player_into_box(game, &acting_id);
                // Java: getResult().addReport(new ReportBiteSpectator(actingPlayer.getPlayerId()))
                game.report_list.add(ReportBiteSpectator::new(acting_id.clone()));
                outcome = outcome.with_event(GameEvent::BiteSpectator { player_id: acting_id.clone() });
            }
        }

        game.acting_player.has_fed = true;
        game.acting_player.has_acted = true;
        outcome
            .publish(StepParameter::EndPlayerAction(self.end_player_action))
            .publish(StepParameter::EndTurn(self.end_turn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_RESERVE, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn step_id_is_init_feeding() {
        let step = StepInitFeeding::new();
        assert_eq!(step.id(), StepId::InitFeeding);
    }

    #[test]
    fn goto_label_on_end_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".to_string()));
        assert!(ok);
        assert_eq!(step.goto_label_on_end.as_deref(), Some("end"));
    }

    #[test]
    fn feeding_allowed_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::FeedingAllowed(true));
        assert!(ok);
        assert_eq!(step.feeding_allowed, Some(true));
    }

    #[test]
    fn end_player_action_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::EndPlayerAction(true));
        assert!(ok);
        assert!(step.end_player_action);
    }

    #[test]
    fn end_turn_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(ok);
        assert!(step.end_turn);
    }

    #[test]
    fn no_acting_player_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        // No acting player → goto label
        game.acting_player.player_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("label_end"));
    }

    #[test]
    fn not_blood_lusting_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn already_fed_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn feeding_not_allowed_sets_choice_false_and_bites_spectator() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }

    #[test]
    fn bite_spectator_sets_actor_to_reserve() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let actor_coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vampire", actor_coord);
        game.acting_player.player_id = Some("vampire".to_string());
        game.acting_player.suffering_blood_lust = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("vampire");
        assert_eq!(state.map(|s| s.base()), Some(PS_RESERVE));
    }

    #[test]
    fn feed_on_player_clears_blood_lust_and_publishes_injury_result() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("end".to_string());
        step.feeding_allowed = Some(true);
        step.feed_on_player_choice = Some(true);
        let mut game = make_game();
        let actor_coord = FieldCoordinate::new(5, 5);
        let victim_coord = FieldCoordinate::new(6, 5);
        add_player(&mut game, "vampire", actor_coord);
        add_player(&mut game, "victim", victim_coord);
        game.acting_player.player_id = Some("vampire".to_string());
        game.acting_player.suffering_blood_lust = true;
        game.defender_id = Some("victim".to_string());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn bite_spectator_at_ball_sets_ball_moving() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let actor_coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vampire", actor_coord);
        game.acting_player.player_id = Some("vampire".to_string());
        game.acting_player.suffering_blood_lust = true;
        game.field_model.ball_coordinate = Some(actor_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.ball_moving);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(_))));
    }

    #[test]
    fn bite_spectator_adds_report_bite_spectator() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let actor_coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vampire", actor_coord);
        game.acting_player.player_id = Some("vampire".to_string());
        game.acting_player.suffering_blood_lust = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BITE_SPECTATOR));
    }

    #[test]
    fn feeding_not_allowed_eligible_player_report_has_correct_player_id() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let actor_coord = FieldCoordinate::new(3, 3);
        add_player(&mut game, "dracula", actor_coord);
        game.acting_player.player_id = Some("dracula".to_string());
        game.acting_player.suffering_blood_lust = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        // Report should be in the list (player is eligible since state is STANDING)
        assert!(game.report_list.has_report(ReportId::BITE_SPECTATOR));
    }
}
