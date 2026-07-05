use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepTentacles +
/// inline of com.fumbbl.ffb.server.skillbehaviour.bb2016.TentaclesBehaviour.
///
/// Handles the Tentacles skill check during movement (BB2016).
///
/// When the acting player moves out of a tackle zone containing a player with Tentacles,
/// the Tentacles player may attempt to hold them. The hold is contested on 2d6
/// (each adds their ST): if actor fails to escape, they are pushed back to `coordinateFrom`.
///
/// Init params: GOTO_LABEL_ON_SUCCESS (mandatory), COORDINATE_FROM.
/// The dialog (opponent picks tentacles player) is via `AgentPrompt::PlayerChoice`.
///
/// BB2016 vs BB2020:
/// - `usingTentacles` checked only if actor is dodging or jumping
/// - 2d6 roll: `is_tentacles_escape_successful(roll, defenderST, actorST)`
/// - Re-roll action: "TENTACLES_ESCAPE"
/// - On Tentacles win: publish FEEDING_ALLOWED=false, END_PLAYER_ACTION=true, COORDINATE_FROM=null
///   then GOTO_LABEL(gotoLabelOnSuccess)
pub struct StepTentacles {
    /// Java: StepState.goToLabelOnSuccess
    pub goto_label_on_success: String,
    /// Java: StepState.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: StepState.usingTentacles (Boolean tristate)
    pub using_tentacles: Option<bool>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepTentacles {
    pub fn new(goto_label_on_success: impl Into<String>) -> Self {
        Self {
            goto_label_on_success: goto_label_on_success.into(),
            coordinate_from: None,
            using_tentacles: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (state.usingTentacles == null) { ... find tentacles players ... }
        if self.using_tentacles.is_none() {
            if game.acting_player.dodging || game.acting_player.jumping {
                if let Some(coord_from) = self.coordinate_from {
                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                    let tentaclers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_property(
                        game,
                        &actor_id,
                        coord_from,
                        NamedProperties::CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES,
                        false,
                    ).into_iter().cloned().collect();

                    if !tentaclers.is_empty() {
                        let prompt = ffb_model::prompts::AgentPrompt::PlayerChoice {
                            eligible_players: tentaclers,
                            reason: "TENTACLES".into(),
                        };
                        return StepOutcome::cont().with_prompt(prompt);
                    } else {
                        self.using_tentacles = Some(false);
                    }
                } else {
                    self.using_tentacles = Some(false);
                }
            } else {
                self.using_tentacles = Some(false);
            }
        }

        // Java: if (state.usingTentacles != null) { ... roll or next step ... }
        if let Some(using) = self.using_tentacles {
            let mut do_next_step = true;

            if using {
                if let Some(ref defender_id) = game.defender_id.clone() {
                    if game.player(defender_id).is_some() {
                        let re_rolled = self.re_rolled_action.as_deref() == Some("TENTACLES_ESCAPE");
                        let mut roll_tentacles = true;

                        if re_rolled {
                            if let Some(ref source_str) = self.re_roll_source.clone() {
                                let source = ReRollSource::new(source_str.as_str());
                                let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                                if !use_reroll(game, &source, &actor_id) {
                                    roll_tentacles = false;
                                }
                            } else {
                                roll_tentacles = false;
                            }
                        }

                        if roll_tentacles {
                            let dice = [rng.d6(), rng.d6()];
                            let defender_st = game.player(defender_id)
                                .map(|p| p.strength_with_modifiers())
                                .unwrap_or(0);
                            let actor_st = game.acting_player.strength;
                            let min_roll = DiceInterpreter::minimum_roll_tentacles_escape(defender_st, actor_st);
                            let successful = DiceInterpreter::is_tentacles_escape_successful(&dice, defender_st, actor_st);

                            if successful {
                                self.using_tentacles = Some(false);
                            } else if !re_rolled {
                                let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                                if let Some(prompt) = ask_for_reroll_if_available(game, "TENTACLES_ESCAPE", min_roll, false) {
                                    self.re_rolled_action = Some("TENTACLES_ESCAPE".into());
                                    self.re_roll_source = Some("TRR".into());
                                    do_next_step = false;
                                    let _ = actor_id;
                                    return StepOutcome::cont().with_prompt(prompt);
                                }
                                // No re-roll available — tentacles wins, fall through to do_next_step
                            }
                        }
                    }
                }
            }

            if do_next_step {
                if self.using_tentacles == Some(true) {
                    // Tentacles wins: move actor back to coordinateFrom
                    if let (Some(ref actor_id), Some(coord)) = (game.acting_player.player_id.clone(), self.coordinate_from) {
                        let has_ball = UtilPlayer::has_ball(game, actor_id);
                        game.field_model.set_player_coordinate(actor_id, coord);
                        if has_ball {
                            game.field_model.ball_coordinate = Some(coord);
                        }
                    }
                    let label = self.goto_label_on_success.clone();
                    return StepOutcome::goto(&label)
                        .publish(StepParameter::FeedingAllowed(false))
                        .publish(StepParameter::EndPlayerAction(true))
                        .publish(StepParameter::CoordinateFrom(FieldCoordinate::new(0, 0)));
                }
                return StepOutcome::next();
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepTentacles {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepTentacles {
    fn id(&self) -> StepId { StepId::Tentacles }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.TENTACLES)
            Action::PlayerChoice { player_id, mode, .. } if mode == "TENTACLES" => {
                self.using_tentacles = Some(player_id.is_some());
                if let Some(ref pid) = player_id {
                    game.last_defender_id = game.defender_id.clone();
                    game.defender_id = Some(pid.clone());
                }
            }
            Action::SelectPlayer {player_id } => {
                self.using_tentacles = Some(!player_id.is_empty());
                if !player_id.is_empty() {
                    game.last_defender_id = game.defender_id.clone();
                    game.defender_id = Some(player_id.clone());
                }
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
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::CoordinateFrom(v)     => { self.coordinate_from = Some(*v); true }
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
    fn id_is_tentacles() {
        assert_eq!(StepTentacles::new("success").id(), StepId::Tentacles);
    }

    #[test]
    fn goto_label_on_success_parameter_accepted() {
        let mut step = StepTentacles::new("old");
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("new".into())));
        assert_eq!(step.goto_label_on_success, "new");
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepTentacles::new("label");
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn not_dodging_not_jumping_sets_using_tentacles_false() {
        let mut game = make_game();
        game.acting_player.dodging = false;
        game.acting_player.jumping = false;
        let mut step = StepTentacles::new("label");
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_tentacles, Some(false));
    }

    #[test]
    fn dodging_with_no_tentacles_players_sets_false() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepTentacles::new("label");
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_tentacles, Some(false));
    }

    #[test]
    fn using_tentacles_false_returns_next_step() {
        let mut game = make_game();
        let mut step = StepTentacles::new("label");
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_tentacles = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_player_sets_using_tentacles_true() {
        let mut game = make_game();
        let mut step = StepTentacles::new("label");
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer {player_id: "tentpid".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.defender_id.as_deref(), Some("tentpid"));
    }

    #[test]
    fn select_empty_player_declines_tentacles() {
        let mut game = make_game();
        let mut step = StepTentacles::new("label");
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.handle_command(
            &Action::SelectPlayer {player_id: "".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.using_tentacles, Some(false));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepTentacles::new("label");
        step.re_rolled_action = Some("TENTACLES_ESCAPE".into());
        step.re_roll_source = Some("TRR".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepTentacles::new("label");
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn no_coordinate_from_sets_tentacles_false() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepTentacles::new("label");
        // No coordinate_from → using_tentacles = false
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_tentacles, Some(false));
    }
}
