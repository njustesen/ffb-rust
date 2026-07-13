use ffb_model::enums::{ReRollSource, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepShadowing +
/// inline of com.fumbbl.ffb.server.skillbehaviour.bb2016.ShadowingBehaviour.
///
/// Handles the Shadowing skill (BB2016 edition).
///
/// BB2016 differences vs BB2020:
/// - `doShadowing` also excludes `TurnMode::PassBlock`
/// - 2d6 escape roll via `DiceInterpreter::is_shadowing_escape_successful`
/// - Re-roll offered to *acting player* (not defender)
/// - Re-roll action label: "SHADOWING_ESCAPE"
/// - No `shadowerWasPreviousDefender` field
///
/// Expects: COORDINATE_FROM, DEFENDER_POSITION, USING_DIVING_TACKLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowingStatus {
    None,
    Success,
    Failure,
}

pub struct StepShadowing {
    /// Java: state.status
    pub status: ShadowingStatus,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            status: ShadowingStatus::None,
            defender_position: None,
            coordinate_from: None,
            using_diving_tackle: false,
            using_shadowing: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let do_shadowing = !self.using_diving_tackle
            && game.turn_mode != TurnMode::KickoffReturn
            && game.turn_mode != TurnMode::PassBlock;

        // Java: if (doShadowing && coordinateFrom != null && usingShadowing == null) { ... find shadowers ... }
        if do_shadowing {
            if let Some(coord_from) = self.coordinate_from {
                if self.using_shadowing.is_none() {
                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                    let mut shadowers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_property(
                        game,
                        &actor_id,
                        coord_from,
                        NamedProperties::CAN_ATTEMPT_TO_TACKLE_DODGING_PLAYER,
                        true,
                    ).into_iter().cloned().collect();

                    // filterThrower: remove thrower from candidates
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

                // Java: if (doShadowing && coordinateFrom != null && usingShadowing != null) { ... roll ... }
                if let Some(using) = self.using_shadowing {
                    let mut do_next_step = true;

                    if using {
                        if let Some(ref defender_id) = game.defender_id.clone() {
                            if game.player(defender_id).is_some() {
                                let re_rolled = self.re_rolled_action.as_deref() == Some("SHADOWING_ESCAPE");
                                let mut roll_shadowing = true;

                                if re_rolled {
                                    if let Some(ref source_str) = self.re_roll_source.clone() {
                                        let source = ReRollSource::new(source_str.as_str());
                                        let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                                        if !use_reroll(game, &source, &actor_id) {
                                            roll_shadowing = false;
                                        }
                                    } else {
                                        roll_shadowing = false;
                                    }
                                }

                                if roll_shadowing {
                                    let dice = [rng.d6(), rng.d6()];
                                    let defender_ma = game.player(defender_id)
                                        .map(|p| p.movement_with_modifiers())
                                        .unwrap_or(0);
                                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                                    let actor_ma = game.player(&actor_id)
                                        .map(|p| p.movement_with_modifiers())
                                        .unwrap_or(0);
                                    let min_roll = DiceInterpreter::minimum_roll_shadowing_escape(defender_ma, actor_ma);
                                    let successful = DiceInterpreter::is_shadowing_escape_successful(&dice, defender_ma, actor_ma);

                                    if successful {
                                        self.using_shadowing = Some(false);
                                    } else if !re_rolled {
                                        if let Some(prompt) = ask_for_reroll_if_available(game, "SHADOWING_ESCAPE", min_roll, false) {
                                            self.re_rolled_action = Some("SHADOWING_ESCAPE".into());
                                            self.re_roll_source = Some("TRR".into());
                                            do_next_step = false;
                                            return StepOutcome::cont().with_prompt(prompt);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Java: if (doNextStep && state.usingShadowing) { updatePlayerAndBallPosition ... }
                    if do_next_step {
                        if self.using_shadowing == Some(true) {
                            if let (Some(ref defender_id), Some(coord)) = (game.defender_id.clone(), self.coordinate_from) {
                                let has_ball = UtilPlayer::has_ball(game, defender_id);
                                game.field_model.set_player_coordinate(defender_id, coord);
                                if has_ball {
                                    game.field_model.ball_coordinate = Some(coord);
                                }
                                UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
                            }
                        }

                        // Java: if (defenderPosition != null) { setDefenderId from player at defenderPosition }
                        if let Some(def_pos) = self.defender_position {
                            let defender_at_pos = game.field_model
                                .player_at(def_pos)
                                .cloned();
                            game.defender_id = defender_at_pos;
                        }

                        return StepOutcome::next();
                    }
                }
            }
        }

        // doShadowing == false or no coordinateFrom: go to NEXT_STEP after restoring defender
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
            Action::SelectPlayer {player_id } => {
                self.using_shadowing = Some(!player_id.is_empty());
                game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
            }
            // Re-roll declined: clear source so execute_step detects decline
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(c)     => { self.coordinate_from = Some(*c); true }
            StepParameter::DefenderPosition(c)   => { self.defender_position = Some(*c); true }
            StepParameter::UsingDivingTackle(v)  => { self.using_diving_tackle = *v; true }
            StepParameter::UsingShadowing(v)     => { self.using_shadowing = *v; true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_shadowing() {
        assert_eq!(StepShadowing::new().id(), StepId::Shadowing);
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(5, 3);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn defender_position_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(7, 4);
        assert!(step.set_parameter(&StepParameter::DefenderPosition(coord)));
        assert_eq!(step.defender_position, Some(coord));
    }

    #[test]
    fn using_diving_tackle_disables_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_diving_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // With usingDivingTackle=true, doShadowing=false → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kickoff_return_mode_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn pass_block_mode_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_coordinate_from_returns_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_shadowers_sets_using_shadowing_false_and_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        // No opponents with Shadowing skill → usingShadowing = false → NEXT_STEP
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
    fn select_player_sets_using_shadowing_true() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer {player_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        // using_shadowing set to true, then rolls dice (no defender on pitch → next_step)
        assert!(game.defender_id.as_deref() == Some("p1"));
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
        step.re_rolled_action = Some("SHADOWING_ESCAPE".into());
        step.re_roll_source = Some("TRR".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn set_parameter_using_shadowing_accepted() {
        let mut step = StepShadowing::new();
        assert!(step.set_parameter(&StepParameter::UsingShadowing(Some(true))));
        assert_eq!(step.using_shadowing, Some(true));
    }
}
