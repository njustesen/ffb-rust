/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepInitScatterPlayer`.
///
/// Step in TTM scatter sequence. Calculates where the thrown/kicked player lands:
/// - If in-bounds with a player there: injury the hit player, continue scatter loop.
/// - If in-bounds empty: place player, end loop.
/// - If out-of-bounds: crowd-injury, publish THROW_IN / CATCH_SCATTER_THROW_IN_MODE.
///
/// Init params (all mandatory): THROWN_PLAYER_ID, THROWN_PLAYER_STATE,
///   THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_COORDINATE, THROW_SCATTER.
/// Optional init: IS_KICKED_PLAYER.
///
/// TODO(InitScatterPlayer-scatter): UtilThrowTeamMateSequence.scatterPlayer/kickPlayer deferred.
/// TODO(InitScatterPlayer-injury): UtilServerInjury.handleInjury deferred.
/// TODO(InitScatterPlayer-animation): Animation/syncGameModel deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitScatterPlayer` (bb2016/ttm).
pub struct StepInitScatterPlayer {
    /// Java: `fThrownPlayerId` — mandatory init param.
    thrown_player_id: Option<String>,
    /// Java: `fThrownPlayerState` — mandatory init param.
    thrown_player_state: Option<PlayerState>,
    /// Java: `fThrownPlayerHasBall` — mandatory init param.
    thrown_player_has_ball: bool,
    /// Java: `fThrownPlayerCoordinate` — mandatory init param.
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrowScatter` — mandatory init param.
    throw_scatter: bool,
    /// Java: `fIsKickedPlayer` — optional.
    is_kicked_player: bool,
}

impl StepInitScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            is_kicked_player: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Guard: no player or coordinate → skip.
        if self.thrown_player_id.is_none() || self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }
        // TODO(InitScatterPlayer-scatter): call UtilThrowTeamMateSequence::scatterPlayer / kickPlayer
        //   to get the landing FieldCoordinate.
        // TODO(InitScatterPlayer-inBounds): if in-bounds + player → injury + continue loop.
        // TODO(InitScatterPlayer-empty): if in-bounds empty → place player + end loop.
        // TODO(InitScatterPlayer-outOfBounds): crowd injury + THROW_IN publish if has ball.
        // Always publish the carried parameters so downstream steps can consume them.
        StepOutcome::next()
            .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
            .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or(PlayerState(0))))
            .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
    }
}

impl Default for StepInitScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitScatterPlayer {
    fn id(&self) -> StepId { StepId::InitScatterPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)   => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrowScatter(v)             => { self.throw_scatter = *v; true }
            StepParameter::IsKickedPlayer(v)           => { self.is_kicked_player = *v; true }
            // Also handle kicked-player aliases (BB2016 uses same step for both).
            StepParameter::KickedPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerCoordinate(v)   => { self.thrown_player_coordinate = Some(*v); true }
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
    fn id_is_init_scatter_player() {
        assert_eq!(StepInitScatterPlayer::new().id(), StepId::InitScatterPlayer);
    }

    #[test]
    fn no_player_returns_next() {
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn publishes_thrown_player_id_when_set() {
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))));
    }

    #[test]
    fn set_parameter_throw_scatter() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrowScatter(true)));
        assert!(step.throw_scatter);
    }

    #[test]
    fn set_parameter_is_kicked() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.is_kicked_player);
    }
}
