use ffb_model::enums::{ReRollSource, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepShadowing +
/// inline of com.fumbbl.ffb.server.skillbehaviour.bb2020.ShadowingBehaviour.
///
/// BB2020 differences vs BB2016:
/// - `doShadowing` does NOT exclude `TurnMode::PassBlock`
/// - 1d6 escape roll: min_roll = max(6 - moveDiff, 2) where moveDiff = defenderMA - actorMA
/// - Re-roll offered to *defender* (not acting player)
/// - Re-roll action label: "SHADOWING"
/// - Has `shadowerWasPreviousDefender` field
/// - After shadower succeeds: publishes PLAYER_ENTERING_SQUARE
pub struct StepShadowing {
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
    /// Java: state.shadowerWasPreviousDefender
    pub shadower_was_previous_defender: bool,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            defender_position: None,
            using_diving_tackle: false,
            using_shadowing: None,
            shadower_was_previous_defender: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let do_shadowing = !self.using_diving_tackle
            && game.turn_mode != TurnMode::KickoffReturn;

        if do_shadowing {
            if let Some(coord_from) = self.coordinate_from {
                // Java: if (doShadowing && coordinateFrom != null && usingShadowing == null)
                if self.using_shadowing.is_none() {
                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                    let mut shadowers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_property(
                        game,
                        &actor_id,
                        coord_from,
                        NamedProperties::CAN_ATTEMPT_TO_TACKLE_DODGING_PLAYER,
                        true,
                    ).into_iter().cloned().collect();

                    // filterThrower
                    if let Some(ref tid) = game.thrower_id.clone() {
                        shadowers.retain(|id| id != tid);
                    }

                    // filterAttackerAndDefender during DUMP_OFF
                    if game.turn_mode == TurnMode::DumpOff {
                        if let Some(ref aid) = game.acting_player.player_id.clone() {
                            shadowers.retain(|id| id != aid);
                        }
                        if let Some(ref did) = game.defender_id.clone() {
                            shadowers.retain(|id| id != did);
                        }
                    }

                    if !shadowers.is_empty() {
                        let prompt = ffb_model::prompts::AgentPrompt::PlayerChoice {
                            eligible_players: shadowers,
                            reason: "SHADOWING".into(),
                            descriptions: vec![],
                        };
                        return StepOutcome::cont().with_prompt(prompt);
                    } else {
                        self.using_shadowing = Some(false);
                    }
                }

                // Java: if (doShadowing && coordinateFrom != null && usingShadowing != null)
                if let Some(using) = self.using_shadowing {
                    let mut do_next_step = true;

                    if using {
                        if let Some(ref defender_id) = game.defender_id.clone() {
                            if game.player(defender_id).is_some() {
                                let re_rolled = self.re_rolled_action.as_deref() == Some("SHADOWING");
                                let mut roll_shadowing = true;

                                if re_rolled {
                                    if let Some(ref source_str) = self.re_roll_source.clone() {
                                        let source = ReRollSource::new(source_str.as_str());
                                        // BB2020: re-roll offered to defender
                                        if !use_reroll(game, &source, defender_id) {
                                            roll_shadowing = false;
                                            self.using_shadowing = Some(false);
                                        }
                                    } else {
                                        roll_shadowing = false;
                                        self.using_shadowing = Some(false);
                                    }
                                }

                                if roll_shadowing {
                                    let roll = rng.d6();
                                    let defender_ma = game.player(defender_id)
                                        .map(|p| p.movement_with_modifiers())
                                        .unwrap_or(0);
                                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                                    let actor_ma = game.player(&actor_id)
                                        .map(|p| p.movement_with_modifiers())
                                        .unwrap_or(0);
                                    let move_diff = defender_ma - actor_ma;
                                    let min_roll = (6 - move_diff).max(2);
                                    let successful = roll >= min_roll;

                                    if !successful {
                                        if !re_rolled {
                                            // BB2020: re-roll offered to defender
                                            if let Some(prompt) = ask_for_reroll_if_available(game, "SHADOWING", min_roll, false) {
                                                self.re_rolled_action = Some("SHADOWING".into());
                                                self.re_roll_source = Some("TRR".into());
                                                do_next_step = false;
                                                return StepOutcome::cont().with_prompt(prompt);
                                            } else {
                                                self.using_shadowing = Some(false);
                                            }
                                        } else {
                                            self.using_shadowing = Some(false);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if do_next_step {
                        if self.using_shadowing == Some(true) {
                            if let (Some(ref defender_id), Some(coord)) = (game.defender_id.clone(), self.coordinate_from) {
                                // BB2020: if shadowerWasPreviousDefender, update defenderPosition too
                                if self.shadower_was_previous_defender {
                                    self.defender_position = Some(coord);
                                }
                                let has_ball = UtilPlayer::has_ball(game, defender_id);
                                game.field_model.set_player_coordinate(defender_id, coord);
                                if has_ball {
                                    game.field_model.ball_coordinate = Some(coord);
                                }
                                // BB2020: publish PLAYER_ENTERING_SQUARE
                                let outcome = StepOutcome::next()
                                    .publish(StepParameter::PlayerEnteringSquare(defender_id.clone()));
                                UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
                                if let Some(def_pos) = self.defender_position {
                                    let defender_at_pos = game.field_model.player_at(def_pos).cloned();
                                    game.defender_id = defender_at_pos;
                                }
                                return outcome;
                            }
                        }

                        if let Some(def_pos) = self.defender_position {
                            let defender_at_pos = game.field_model.player_at(def_pos).cloned();
                            game.defender_id = defender_at_pos;
                        }
                        return StepOutcome::next();
                    }
                }
            }
        }

        // doShadowing == false or no coordinateFrom
        if let Some(def_pos) = self.defender_position {
            let defender_at_pos = game.field_model.player_at(def_pos).cloned();
            game.defender_id = defender_at_pos;
        }
        StepOutcome::next()
    }
}

impl Default for StepShadowing {
    fn default() -> Self { Self::new() }
}

impl Step for StepShadowing {
    fn id(&self) -> StepId { StepId::Shadowing }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.SHADOWING)
            // BB2020: if defenderId == playerId → shadowerWasPreviousDefender = true; else setDefenderId
            Action::PlayerChoice { player_id, mode, .. } if mode == "SHADOWING" => {
                self.using_shadowing = Some(player_id.is_some());
                if let Some(ref pid) = player_id {
                    let is_prev = game.defender_id.as_deref().map(|d| d == pid.as_str()).unwrap_or(false);
                    if is_prev {
                        self.shadower_was_previous_defender = true;
                    } else {
                        game.defender_id = Some(pid.clone());
                    }
                }
            }
            // BB2020 also accepts SelectPlayer (compatibility with BB2016 dispatch)
            Action::SelectPlayer {player_id } => {
                self.using_shadowing = Some(!player_id.is_empty());
                game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v)           => { self.coordinate_from = Some(*v); true }
            StepParameter::DefenderPosition(v)         => { self.defender_position = Some(*v); true }
            StepParameter::UsingDivingTackle(v)        => { self.using_diving_tackle = *v; true }
            StepParameter::Jumped(_)                   => { self.using_shadowing = Some(false); true }
            StepParameter::UsingShadowing(v)           => { self.using_shadowing = *v; true }
            StepParameter::ShadowerWasPreviousDefender(v) => { self.shadower_was_previous_defender = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_shadowing() {
        assert_eq!(StepShadowing::new().id(), StepId::Shadowing);
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(3, 4);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn using_diving_tackle_disables_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_diving_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kickoff_return_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn pass_block_mode_allows_shadowing_in_bb2020() {
        // BB2020: PassBlock does NOT disable shadowing (unlike BB2016)
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        // No shadowers on pitch → usingShadowing=false → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn no_shadowers_sets_using_shadowing_false() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn using_shadowing_false_returns_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_shadowing = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumped_parameter_sets_using_shadowing_false() {
        let mut step = StepShadowing::new();
        step.using_shadowing = Some(true);
        assert!(step.set_parameter(&StepParameter::Jumped(true)));
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn shadower_was_previous_defender_parameter_accepted() {
        let mut step = StepShadowing::new();
        assert!(step.set_parameter(&StepParameter::ShadowerWasPreviousDefender(true)));
        assert!(step.shadower_was_previous_defender);
    }

    #[test]
    fn select_player_sets_defender_id() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer {player_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn select_empty_player_declines_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.handle_command(
            &Action::SelectPlayer {player_id: "".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.using_shadowing, Some(false));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.re_rolled_action = Some("SHADOWING".into());
        step.re_roll_source = Some("TRR".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepShadowing::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
