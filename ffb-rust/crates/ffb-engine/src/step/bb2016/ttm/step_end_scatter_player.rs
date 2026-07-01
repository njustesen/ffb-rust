/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepEndScatterPlayer`.
///
/// Step to end TTM scatter sequence. If player, state, and coordinate are all
/// present it pushes a ScatterPlayer sequence; otherwise falls through.
///
/// Consumes: THROWN_PLAYER_ID, THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_STATE (all).
/// Does NOT consume THROWN_PLAYER_COORDINATE (left for driver to propagate further).
///
/// DEFERRED(generator): ScatterPlayer SequenceGenerator not yet ported for BB2016.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndScatterPlayer` (bb2016/ttm).
pub struct StepEndScatterPlayer {
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: bool,
    /// Java: `fThrownPlayerState`
    thrown_player_state: Option<PlayerState>,
    /// Java: `fThrownPlayerCoordinate`
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `fIsKickedPlayer`
    is_kicked_player: bool,
}

impl StepEndScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_has_ball: false,
            thrown_player_state: None,
            thrown_player_coordinate: None,
            is_kicked_player: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        let all_present = self.thrown_player_id.is_some()
            && self.thrown_player_state.is_some()
            && self.thrown_player_coordinate.is_some();
        if all_present {
            // DEFERRED(generator): push ScatterPlayer sequence not yet ported.
        }
        let mut out = StepOutcome::next();
        if self.is_kicked_player {
            out = out.publish(StepParameter::IsKickedPlayer(true));
        }
        out
    }
}

impl Default for StepEndScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndScatterPlayer {
    fn id(&self) -> StepId { StepId::EndScatterPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerCoordinate(v)   => { self.thrown_player_coordinate = *v; true }
            StepParameter::KickedPlayerId(v)            => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerHasBall(v)       => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerState(v)         => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerCoordinate(v)   => { self.thrown_player_coordinate = Some(*v); true }
            StepParameter::IsKickedPlayer(v)            => { self.is_kicked_player = *v; true }
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
    fn id_is_end_scatter_player() {
        assert_eq!(StepEndScatterPlayer::new().id(), StepId::EndScatterPlayer);
    }

    #[test]
    fn missing_state_returns_next_without_push() {
        let mut game = make_game();
        let mut step = StepEndScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        // coordinate set, but state missing → no push, still NEXT_STEP
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn is_kicked_player_publishes_param() {
        let mut game = make_game();
        let mut step = StepEndScatterPlayer::new();
        step.is_kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(true))));
    }

    #[test]
    fn not_kicked_player_no_extra_publish() {
        let mut game = make_game();
        let out = StepEndScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(out.published.is_empty());
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepEndScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }
}
