use ffb_model::enums::{TurnMode, PS_RESERVE};
use ffb_model::model::SpecialRule;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Handles the Swarming kickoff result: the Swarming team places extra lineman
/// reserves onto the Line of Scrimmage.
///
/// Java logic:
///  1. If game is already in TurnMode::Swarming (second+ entry — coach has submitted setup):
///     - Count active on-pitch swarming players actually placed.
///     - If over the rolled limit → show error dialog; stay.
///     - If SetupMechanic.checkSetup passes → leave().
///  2. Otherwise (first entry):
///     - If `handle_receiving_team = false`, reset kicking-swarmers counter.
///     - Find the swarming team (kicking or receiving based on `handle_receiving_team`).
///     - If team does not have SpecialRule::Swarming → NEXT_STEP immediately.
///     - Otherwise: find players on pitch (inactive) and lineman reserves.
///     - If no swarming lineman reserves exist → NEXT_STEP.
///     - Roll for swarming players (Java: `DiceRoller.rollSwarmingPlayers()` = d6).
///     - Set TurnMode::Swarming, push self back onto stack, show dialog.
///
/// `checkSetup`, `pinPlayersInTacklezones`, the `leave()` path, and error dialog
/// are TODO stubs. The TurnMode transition and reserve-detection are implemented.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepSwarming`.
pub struct StepSwarming {
    /// Java: state.status — not yet ported (ActionStatus enum).
    pub status: Option<String>,
    /// Java: state.endTurn — set true when CLIENT_END_TURN arrives.
    pub end_turn: bool,
    /// Java: state.handleReceivingTeam — init param.
    pub handle_receiving_team: bool,
    /// Java: state.rolledAmount — D6 swarming roll result.
    pub rolled_amount: i32,
    /// Java: state.teamId — ID of the swarming team.
    pub team_id: Option<String>,
}

impl StepSwarming {
    pub fn new() -> Self {
        Self {
            status: None,
            end_turn: false,
            handle_receiving_team: false,
            rolled_amount: 0,
            team_id: None,
        }
    }
}

impl Default for StepSwarming {
    fn default() -> Self { Self::new() }
}

impl Step for StepSwarming {
    fn id(&self) -> StepId { StepId::Swarming }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::ConfirmSetup => {
                // Java CLIENT_END_TURN during Swarming mode.
                self.end_turn = true;
            }
            Action::PlacePlayer { player_id, coord } => {
                // Java CLIENT_SETUP_PLAYER → UtilServerSetup.setupPlayer(...)
                game.field_model.set_player_coordinate(player_id, *coord);
                return StepOutcome::cont();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::HandleReceivingTeam(v) => { self.handle_receiving_team = *v; true }
            _ => false,
        }
    }
}

impl StepSwarming {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::Swarming {
            // Second entry: coach has submitted their swarming placement.
            if self.end_turn {
                self.end_turn = false;
                // Count active on-pitch swarming players (those not in box).
                let placed = if let Some(ref team_id) = self.team_id {
                    let team = if game.team_home.id == *team_id { &game.team_home } else { &game.team_away };
                    team.players.iter().filter(|p| {
                        game.field_model.player_coordinate(&p.id)
                            .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                            .unwrap_or(false)
                            && game.field_model.player_state(&p.id)
                                   .map(|s| s.is_active())
                                   .unwrap_or(false)
                    }).count() as i32
                } else {
                    0
                };

                if placed > self.rolled_amount {
                    // TODO: show DialogSwarmingErrorParameter and wait.
                    // For now: reject and continue.
                    return StepOutcome::cont();
                }

                // TODO: SetupMechanic.checkSetup, then call leave().
                // Stub: always proceed.
                self.leave(game);
            }
            return StepOutcome::cont();
        }

        // First entry.

        // Reset kicking-swarmers counter if we are handling the kicking team.
        // (Java: if !handleReceivingTeam → gameState.setKickingSwarmers(0).)
        // TODO: kicking_swarmers field on GameState not yet ported.

        // Determine the swarming team.
        let team_id = self.swarming_team_id(game);
        self.team_id = Some(team_id.clone());

        // Check for the Swarming special rule.
        let team = if game.team_home.id == team_id { &game.team_home } else { &game.team_away };
        let has_swarming_rule = team.special_rules.iter()
            .any(|r| SpecialRule::from(r) == Some(SpecialRule::SWARMING));
        if !has_swarming_rule {
            return StepOutcome::next();
        }
        let has_swarming_reserves = team.players.iter().any(|p| {
            // Java: Keyword::LINEMAN and PlayerState::RESERVE.
            game.field_model.player_state(&p.id)
                .map(|s| s.base() == PS_RESERVE)
                .unwrap_or(false)
        });

        if !has_swarming_reserves {
            return StepOutcome::next();
        }

        // Roll for the number of swarming players (Java: DiceRoller.rollSwarmingPlayers() = d6).
        self.rolled_amount = rng.d6();

        // Flip home_playing if we are handling the receiving team.
        if self.handle_receiving_team {
            game.home_playing = !game.home_playing;
        }

        game.turn_mode = TurnMode::Swarming;

        // TODO: push self back onto stack (StepStack::pushCurrentStepOnStack).
        // TODO: show DialogSwarmingPlayersParameter.

        StepOutcome::cont()
    }

    fn leave(&mut self, game: &mut Game) {
        // Java: restore PRONE → RESERVE, set TurnMode::Kickoff, refreshPlayersForTurnStart,
        //       clearTrackNumbers, handle receiving-team flip / kicking-swarmers count.
        // TODO: restore player states (PRONE → RESERVE).
        // TODO: UtilPlayer.refreshPlayersForTurnStart, fieldModel.clearTrackNumbers.

        if self.handle_receiving_team {
            game.home_playing = !game.home_playing;
        }
        // TODO: else gameState.setKickingSwarmers(placedSwarmingPlayers).

        game.turn_mode = TurnMode::Kickoff;
    }

    fn swarming_team_id(&self, game: &Game) -> String {
        // Java: if handleReceivingTeam → receiving team; else → kicking team.
        if self.handle_receiving_team {
            if game.home_playing {
                game.team_away.id.clone()
            } else {
                game.team_home.id.clone()
            }
        } else {
            if game.home_playing {
                game.team_home.id.clone()
            } else {
                game.team_away.id.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_no_swarming_reserves_returns_next_step() {
        let mut game = make_game();
        // No players on either team → no swarming reserves → skip.
        let mut step = StepSwarming::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_receiving_team_parameter_accepted() {
        let mut step = StepSwarming::new();
        assert!(step.set_parameter(&StepParameter::HandleReceivingTeam(true)));
        assert!(step.handle_receiving_team);
    }

    #[test]
    fn handle_receiving_team_false_by_default() {
        let step = StepSwarming::default();
        assert!(!step.handle_receiving_team);
    }

    #[test]
    fn swarming_team_kicking_is_home_when_home_playing() {
        let game = make_game();
        let mut step = StepSwarming::new();
        step.handle_receiving_team = false;
        let tid = step.swarming_team_id(&game);
        assert_eq!(tid, game.team_home.id);
    }

    #[test]
    fn swarming_team_receiving_is_away_when_home_playing() {
        let game = make_game();
        let mut step = StepSwarming::new();
        step.handle_receiving_team = true;
        let tid = step.swarming_team_id(&game);
        assert_eq!(tid, game.team_away.id);
    }

    #[test]
    fn unknown_parameter_returns_false() {
        let mut step = StepSwarming::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }
}
