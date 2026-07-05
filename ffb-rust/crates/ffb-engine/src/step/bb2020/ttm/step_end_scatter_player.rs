/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepEndScatterPlayer`.
///
/// Step to end the TTM scatter sequence. If player, state, and coordinate are all
/// present it pushes a new ScatterPlayer sequence (looping). Optionally publishes
/// IS_KICKED_PLAYER when the flag is set.
///
/// BB2020 difference vs BB2016: adds `crashLanding` field.
///
/// Consumes: THROWN_PLAYER_ID, THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_STATE.
/// Does NOT consume THROWN_PLAYER_COORDINATE.
///
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2020::scatter_player::{ScatterPlayer, ScatterPlayerParams};

/// Java: `StepEndScatterPlayer` (bb2020/ttm).
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
    /// Java: `crashLanding` — BB2020 addition.
    crash_landing: bool,
}

impl StepEndScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_has_ball: false,
            thrown_player_state: None,
            thrown_player_coordinate: None,
            is_kicked_player: false,
            crash_landing: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        let all_present = self.thrown_player_id.is_some()
            && self.thrown_player_state.is_some()
            && self.thrown_player_coordinate.is_some();
        let mut out = StepOutcome::next();
        if all_present {
            let seq = ScatterPlayer::build_sequence(&ScatterPlayerParams {
                thrown_player_id: self.thrown_player_id.clone(),
                thrown_player_state: self.thrown_player_state,
                thrown_player_has_ball: self.thrown_player_has_ball,
                thrown_player_coordinate: self.thrown_player_coordinate,
                throw_scatter: false,
                has_swoop: false,
                deviates: false,
                crash_landing: self.crash_landing,
            });
            out = out.push_seq(seq);
        }
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
            StepParameter::ThrownPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerCoordinate(v)  => { self.thrown_player_coordinate = *v; true }
            StepParameter::KickedPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerCoordinate(v)  => { self.thrown_player_coordinate = Some(*v); true }
            StepParameter::IsKickedPlayer(v)          => { self.is_kicked_player = *v; true }
            StepParameter::CrashLanding(v)            => { self.crash_landing = *v; true }
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
    fn id_is_end_scatter_player() {
        assert_eq!(StepEndScatterPlayer::new().id(), StepId::EndScatterPlayer);
    }

    #[test]
    fn missing_state_returns_next_without_push() {
        let mut game = make_game();
        let mut step = StepEndScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        // state missing → no push, still NEXT_STEP
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
    fn set_parameter_crash_landing() {
        let mut step = StepEndScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::CrashLanding(true)));
        assert!(step.crash_landing);
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepEndScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }
}
