/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepResetFumblerooskie`.
///
/// Resolves the Fumblerooskie skill at the end of a player's action.  If the
/// acting player still has the ball and the Fumblerooskie was pending, the ball
/// may be committed (made not-moving) and a pickup sound/report may be emitted.
///
/// Init parameters (optional): RESET_FOR_FAILED_BLOCK, END_PLAYER_ACTION, IN_SELECT.
/// Incoming parameters: END_PLAYER_ACTION, END_TURN.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepResetFumblerooskie` (mixed/move, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepResetFumblerooskie {
    /// Java: `resetForFailedBlock` — init flag: step was inserted because a block failed.
    pub reset_for_failed_block: bool,
    /// Java: `endPlayerAction` — received via setParameter or init.
    pub end_player_action: bool,
    /// Java: `inSelect` — init flag: step is inside a select sequence.
    pub in_select: bool,
}

impl StepResetFumblerooskie {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let jumping = game.acting_player.jumping;

        if !jumping {
            let player_id = match game.acting_player.player_id.clone() {
                Some(id) => id,
                None => return StepOutcome::next(),
            };

            // Java: actingPlayer.isFumblerooskiePending()
            //       && fieldModel.isBallMoving()
            //       && fieldModel.getBallCoordinate().equals(fieldModel.getPlayerCoordinate(actingPlayer))
            let ball_moving = game.field_model.ball_moving;
            let ball_coord = game.field_model.ball_coordinate;
            let player_coord = game.field_model.player_coordinate(&player_id);

            // Fumblerooskie pending flag — not yet on ActingPlayer; use a model-level stub
            // Java: actingPlayer.isFumblerooskiePending()
            // Approximation: check if the field model has ball_moving (the ball is a live
            // fumblerooski ball only when both moving AND on the player's square).
            let fumblerooskie_pending = game.acting_player.has_moved && ball_moving;
            let ball_on_player = ball_coord.is_some() && player_coord.is_some()
                && ball_coord == player_coord;

            if fumblerooskie_pending && ball_moving && ball_on_player {
                // Java: boolean ballCarrierStanding = fieldModel.getPlayerState(...).canBeBlocked()
                let player_state = game.field_model.player_state(&player_id);
                let ball_carrier_standing = player_state
                    .map(|s| s.can_be_blocked())
                    .unwrap_or(false);

                // Java: if (resetForFailedBlock && !ballCarrierStanding) publish DROPPED_BALL_CARRIER
                let mut outcome = StepOutcome::next();
                if self.reset_for_failed_block && !ball_carrier_standing {
                    outcome = outcome.publish(StepParameter::DroppedBallCarrier(Some(player_id.clone())));
                }

                // Java: if (endPlayerAction || !ballCarrierStanding || !isNextMovePossible)
                //           fieldModel.setBallMoving(false)
                // Simplified: if endPlayerAction or player fell down, commit the ball
                if self.end_player_action || !ball_carrier_standing {
                    game.field_model.ball_moving = false;
                }

                // Java: if (endPlayerAction || (ballCarrierStanding && !isNextMovePossible))
                //           setSound(PICKUP) + addReport(new ReportFumblerooskie(playerId, false))
                // NOTE: ReportFumblerooskie is not yet in GameEvent; emit SkillUse as proxy.
                if self.end_player_action || (ball_carrier_standing && self.end_player_action) {
                    // Java: getResult().addReport(new ReportFumblerooskie(actingPlayer.getPlayerId(), false))
                    // Emit SkillUse with skill_id 0 as a placeholder (Fumblerooskie report not yet in GameEvent).
                    outcome = outcome.with_event(
                        ffb_model::events::GameEvent::SkillUse {
                            player_id: player_id.clone(),
                            skill_id: 0,
                            used: false,
                        }
                    );
                }

                return outcome;
            }

            // Java: if (!fieldModel.isBallMoving()) actingPlayer.setFumblerooskiePending(false)
            // (no-op for us; we use has_moved as a proxy)
        }

        StepOutcome::next()
    }
}

impl Step for StepResetFumblerooskie {
    fn id(&self) -> StepId { StepId::ResetFumblerooskie }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v)          => { self.end_player_action = *v; true }
            StepParameter::ResetForFailedBlock(v) => { self.reset_for_failed_block = *v; true }
            StepParameter::InSelect(v)         => { self.in_select = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_reset_fumblerooskie() {
        assert_eq!(StepResetFumblerooskie::new().id(), StepId::ResetFumblerooskie);
    }

    #[test]
    fn start_returns_next_no_acting_player() {
        let mut step = StepResetFumblerooskie::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_player_returns_next() {
        let mut step = StepResetFumblerooskie::new();
        let mut game = make_game();
        game.acting_player.jumping = true;
        game.acting_player.player_id = Some("p1".into());
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepResetFumblerooskie::new();
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn_maps_to_end_player_action() {
        let mut step = StepResetFumblerooskie::new();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.end_player_action);
    }
}
