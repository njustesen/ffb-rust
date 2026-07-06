/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepKickoffReturn.
///
/// Handles the KICKOFF_RETURN skill. Finds the eligible kickoff-return player on the
/// receiving team and, if one exists and there is no touchback, flips the turn to the
/// receiving team and pushes a select sequence so they can activate that player.
///
/// Expects stepParameter TOUCHBACK, END_PLAYER_ACTION, END_TURN to be set by preceding steps.
/// May push new select sequence on the stack.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};
use crate::step::sequences::select_sequence;

pub struct StepKickoffReturn {
    /// Java: fTouchback
    touchback: bool,
    /// Java: fEndPlayerAction
    end_player_action: bool,
    /// Java: fEndTurn
    end_turn: bool,
}

impl StepKickoffReturn {
    pub fn new() -> Self {
        Self { touchback: false, end_player_action: false, end_turn: false }
    }
}

impl Default for StepKickoffReturn {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffReturn {
    fn id(&self) -> StepId { StepId::KickoffReturn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Touchback(v) => {
                self.touchback = *v;
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

impl StepKickoffReturn {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::KickoffReturn {
            // Already inside the kickoff-return mini-turn
            if self.end_player_action && !game.acting_player.has_acted && !self.end_turn {
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false)
                game.acting_player.player_id = None;
                // Java: getGameState().pushCurrentStepOnStack() + Select.pushSequence
                return StepOutcome::repeat().push_seq(select_sequence());
            } else if self.end_player_action || self.end_turn {
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false)
                game.acting_player.player_id = None;
                // Java: game.setHomePlaying(!game.isHomePlaying())
                game.home_playing = !game.home_playing;
                // Java: game.setTurnMode(TurnMode.KICKOFF)
                game.turn_mode = TurnMode::Kickoff;
                // Java: UtilPlayer.refreshPlayersForTurnStart(game)
                let mechanic = crate::mechanic::game_mechanic_for(game.rules);
                UtilPlayer::refresh_players_for_turn_start(game, &mechanic.enhancements_to_remove_at_end_of_turn(), &mechanic.enhancements_to_remove_at_end_of_turn_when_not_setting_active());
                // Java: game.getFieldModel().clearTrackNumbers()
                game.field_model.clear_track_numbers();
            }
        } else {
            // Java: determine receiving/kicking teams
            // home_playing = the kicking team is playing (kicking team sends the ball)
            let kickoff_return_team_ids: Vec<String> = if game.home_playing {
                game.team_away.players.iter().map(|p| p.id.clone()).collect()
            } else {
                game.team_home.players.iter().map(|p| p.id.clone()).collect()
            };

            let mut kickoff_return_player: Option<String> = None;
            let mut passive_players: Vec<String> = Vec::new();

            for pid in &kickoff_return_team_ids {
                let coord = match game.field_model.player_coordinate(pid) {
                    Some(c) if !c.is_box_coordinate() => c,
                    _ => continue,
                };

                // Java: player.hasSkillProperty(NamedProperties.canMoveDuringKickOffScatter)
                let has_property = game.team_home.players.iter()
                    .chain(game.team_away.players.iter())
                    .find(|p| p.id == *pid)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_DURING_KICK_OFF_SCATTER))
                    .unwrap_or(false);

                if has_property {
                    let los_bounds = if game.home_playing {
                        FieldCoordinateBounds::LOS_AWAY
                    } else {
                        FieldCoordinateBounds::LOS_HOME
                    };

                    if los_bounds.is_in_bounds(coord) {
                        passive_players.push(pid.clone());
                    } else {
                        let other_team = if game.home_playing { &game.team_home } else { &game.team_away };
                        let adj_opponents = UtilPlayer::find_adjacent_players_with_tacklezones(
                            game, other_team, coord, false,
                        );
                        if !adj_opponents.is_empty() {
                            passive_players.push(pid.clone());
                        } else {
                            kickoff_return_player = Some(pid.clone());
                        }
                    }
                } else {
                    passive_players.push(pid.clone());
                }
            }

            if kickoff_return_player.is_some() && !self.touchback {
                // Java: setPlayerState inactive for passive players
                for pid in &passive_players {
                    if let Some(ps) = game.field_model.player_state(pid) {
                        game.field_model.set_player_state(pid, ps.change_active(false));
                    }
                }
                // Java: game.setHomePlaying(!game.isHomePlaying())
                game.home_playing = !game.home_playing;
                // Java: game.setTurnMode(TurnMode.KICKOFF_RETURN)
                game.turn_mode = TurnMode::KickoffReturn;
                // Java: UtilServerDialog.showDialog(…, new DialogKickoffReturnParameter(), false)
                let eligible: Vec<String> = kickoff_return_player.into_iter().collect();
                // Java: pushCurrentStepOnStack() + Select.pushSequence
                return StepOutcome::repeat()
                    .with_prompt(AgentPrompt::KickoffReturn { eligible_players: eligible })
                    .push_seq(select_sequence());
            }
        }

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_with_no_kickoff_return_players_returns_next_step() {
        // No players on the field with CAN_MOVE_DURING_KICK_OFF_SCATTER -> no return player -> next step
        let mut game = make_game();
        let mut step = StepKickoffReturn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_touchback_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_unrecognized_returns_false() {
        let mut step = StepKickoffReturn::new();
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn kickoff_return_mode_end_turn_flips_home_playing_and_sets_kickoff_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        game.home_playing = true;
        let mut step = StepKickoffReturn::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.home_playing);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn kickoff_return_mode_end_player_action_and_end_turn_flips_and_sets_kickoff() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        game.home_playing = false;
        let mut step = StepKickoffReturn::new();
        step.end_player_action = true;
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.home_playing);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn touchback_flag_prevents_kickoff_return_select() {
        let mut game = make_game();
        let mut step = StepKickoffReturn::new();
        step.touchback = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn default_creates_fresh_instance() {
        let step = StepKickoffReturn::default();
        assert!(!step.touchback);
        assert!(!step.end_player_action);
        assert!(!step.end_turn);
    }
}
