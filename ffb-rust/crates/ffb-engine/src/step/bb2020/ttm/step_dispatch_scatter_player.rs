/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepDispatchScatterPlayer`.
///
/// Dispatches a thrown/kicked player to the appropriate scatter sequence based on
/// the `PassResult`. On a FUMBLE + isKickedPlayer it adds a KickTeamMateFumble report
/// without pushing any scatter sequence; otherwise it pushes a ScatterPlayer sequence.
///
/// BB2020-only step (no BB2016 equivalent). Reads: THROWN_PLAYER_ID, THROWN_PLAYER_STATE,
/// THROWN_PLAYER_HAS_BALL, PASS_RESULT, IS_KICKED_PLAYER, OLD_DEFENDER_STATE.
///
/// TODO(DispatchScatterPlayer-generator): ScatterPlayer SequenceGenerator not yet ported.
/// TODO(DispatchScatterPlayer-fumbleReport): ReportKickTeamMateFumble deferred.
/// TODO(DispatchScatterPlayer-scattersSingleDirection): NamedProperties.ttmScattersInSingleDirection deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PassResult};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepDispatchScatterPlayer` (bb2020/ttm).
pub struct StepDispatchScatterPlayer {
    /// Java: `thrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `thrownPlayerState`
    thrown_player_state: Option<PlayerState>,
    /// Java: `oldPlayerState` (OLD_DEFENDER_STATE)
    old_player_state: Option<PlayerState>,
    /// Java: `thrownPlayerHasBall`
    thrown_player_has_ball: bool,
    /// Java: `isKickedPlayer` — optional init param.
    is_kicked_player: bool,
    /// Java: `passResult` — defaults to FUMBLE (Java default).
    pass_result: PassResult,
}

impl StepDispatchScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            old_player_state: None,
            thrown_player_has_ball: false,
            is_kicked_player: false,
            pass_result: PassResult::Fumble,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        if self.pass_result == PassResult::Fumble && self.is_kicked_player {
            // TODO(DispatchScatterPlayer-fumbleReport): add ReportKickTeamMateFumble report.
            return StepOutcome::next();
        }

        // TODO(DispatchScatterPlayer-generator): determine scatter flags from pass_result:
        //   FUMBLE            → throwScatter=false, deviate=false, scattersSingleDirection=false
        //   WILDLY_INACCURATE → throwScatter=false, deviate=true, scattersSingleDirection=false
        //   INACCURATE/ACCURATE → throwScatter=true, deviate=false
        // TODO(DispatchScatterPlayer-generator): push ScatterPlayer sequence with
        //   (thrownPlayerId, thrownPlayerState, thrownPlayerHasBall, throwerCoordinate,
        //    scattersSingleDirection, throwScatter, deviate, !oldPlayerState.hasTacklezones()).

        StepOutcome::next()
    }
}

impl Default for StepDispatchScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepDispatchScatterPlayer {
    fn id(&self) -> StepId { StepId::DispatchScatterPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v)       => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)    => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)  => { self.thrown_player_has_ball = *v; true }
            StepParameter::PassResultParam(v)       => { self.pass_result = *v; true }
            StepParameter::IsKickedPlayer(v)        => { self.is_kicked_player = *v; true }
            StepParameter::OldDefenderState(v)      => { self.old_player_state = Some(*v); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_dispatch_scatter_player() {
        assert_eq!(StepDispatchScatterPlayer::new().id(), StepId::DispatchScatterPlayer);
    }

    #[test]
    fn fumble_kicked_player_returns_next_without_push() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.is_kicked_player = true;
        step.pass_result = PassResult::Fumble;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn non_kicked_fumble_returns_next() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.pass_result = PassResult::Fumble;
        step.is_kicked_player = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepDispatchScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepDispatchScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Complete)));
        assert_eq!(step.pass_result, PassResult::Complete);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut step = StepDispatchScatterPlayer::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_player_state, Some(state));
    }

    #[test]
    fn unknown_parameter_returns_false() {
        let mut step = StepDispatchScatterPlayer::new();
        assert!(!step.set_parameter(&StepParameter::ThrowScatter(true)));
    }
}
