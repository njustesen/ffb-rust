/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepRightStuff`.
///
/// Step in TTM sequence to handle skill RIGHT_STUFF (landing roll).
/// - If player state is FALLING (thrown out of bounds): publish END_TURN +
///   THROWN_PLAYER_COORDINATE(null) → NEXT_STEP.
/// - If player has ball: move ball to player coordinate.
/// - If drop_thrown_player == false: roll landing (minimumRollRightStuff + modifiers).
///   - Success + has ball → touchdown check.
///   - Success without ball on ball square → SCATTER_BALL.
///   - Failure → re-roll if available.
/// - If drop_thrown_player == true (or roll failed, re-roll exhausted): TTMLanding injury.
///
/// TODO(RightStuff-modifier): RightStuffModifierFactory deferred.
/// TODO(RightStuff-mechanic): AgilityMechanic.minimumRollRightStuff deferred.
/// TODO(RightStuff-reroll): AbstractStepWithReRoll / UtilServerReRoll deferred.
/// TODO(RightStuff-injury): UtilServerInjury.handleInjury(InjuryTypeTTMLanding) deferred.
/// TODO(RightStuff-touchdown): UtilServerSteps.checkTouchdown deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PS_FALLING;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::CatchScatterThrowInMode;
use ffb_model::model::kick_team_mate_range::KickTeamMateRange;

/// Java: `StepRightStuff` (bb2016/ttm).
pub struct StepRightStuff {
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: Option<bool>,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fDropThrownPlayer`
    drop_thrown_player: bool,
    /// Java: `ktmRange`
    ktm_range: Option<KickTeamMateRange>,
}

impl StepRightStuff {
    pub fn new() -> Self {
        Self {
            thrown_player_has_ball: None,
            thrown_player_id: None,
            drop_thrown_player: false,
            ktm_range: None,
        }
    }

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.thrown_player_id {
            Some(id) => id.clone(),
            None     => return StepOutcome::next(),
        };
        let has_ball = self.thrown_player_has_ball.unwrap_or(false);

        // If player is in FALLING state (was thrown out of bounds): skip landing roll.
        let is_falling = game.field_model.player_state(&player_id)
            .map(|s| s.base() == PS_FALLING)
            .unwrap_or(false);
        if is_falling {
            return StepOutcome::next()
                .publish(StepParameter::EndTurn(has_ball))
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // Sync ball to player coordinate when holding ball.
        if has_ball {
            if let Some(coord) = game.field_model.player_coordinate(&player_id) {
                game.field_model.ball_coordinate = Some(coord);
            }
        }

        if self.drop_thrown_player {
            // TODO(RightStuff-injury): UtilServerInjury.handleInjury(InjuryTypeTTMLanding, ApothecaryMode::THROWN_PLAYER).
            return StepOutcome::next()
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // TODO(RightStuff-mechanic): roll and evaluate landing; for now stub → success.
        let _ = rng.d6();
        // TODO(RightStuff-touchdown): check touchdown on landing with ball.
        // Stub: landing succeeds.
        let mut out = StepOutcome::next()
            .publish(StepParameter::ThrownPlayerCoordinate(None));
        if !has_ball {
            // Check if player landed on ball square.
            let player_coord = game.field_model.player_coordinate(&player_id);
            let ball_coord   = game.field_model.ball_coordinate;
            if player_coord.is_some() && player_coord == ball_coord {
                out = out.publish(StepParameter::CatchScatterThrowInMode(
                    CatchScatterThrowInMode::ScatterBall));
            }
        }
        out
    }
}

impl Default for StepRightStuff {
    fn default() -> Self { Self::new() }
}

impl Step for StepRightStuff {
    fn id(&self) -> StepId { StepId::RightStuff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::ThrownPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            StepParameter::DropThrownPlayer(v)    => { self.drop_thrown_player = *v; true }
            StepParameter::KtmModifier(v)         => { self.ktm_range = Some(*v); true }
            // Also accept kicked-player aliases.
            StepParameter::KickedPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::KickedPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_right_stuff() {
        assert_eq!(StepRightStuff::new().id(), StepId::RightStuff);
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let out = StepRightStuff::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn drop_thrown_player_publishes_coordinate_null() {
        let mut game = make_game();
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.drop_thrown_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn set_parameter_drop_thrown_player() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::DropThrownPlayer(true)));
        assert!(step.drop_thrown_player);
    }

    #[test]
    fn set_parameter_ktm_range() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::KtmModifier(KickTeamMateRange::SHORT)));
        assert_eq!(step.ktm_range, Some(KickTeamMateRange::SHORT));
    }
}
