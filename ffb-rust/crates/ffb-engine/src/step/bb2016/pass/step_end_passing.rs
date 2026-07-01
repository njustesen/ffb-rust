/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepEndPassing`.
///
/// Final step of the pass sequence. Consumes all expected step parameters.
/// - On bomb turn: push bomb sequence.
/// - On normal pass: record completions/passing stat, push EndPlayerAction.
/// - On interceptor: record interception, set ball coordinate, push EndPlayerAction.
///
/// Infrastructure TODOs:
/// - TODO(EndPassing-bomb): SequenceGenerator::Bomb not yet ported — bomb sequence push deferred.
/// - TODO(EndPassing-pass): SequenceGenerator::EndPlayerAction push deferred.
/// - TODO(EndPassing-animosity): SequenceGenerator::Pass push for failed animosity deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndPassing` (bb2016/pass).
pub struct StepEndPassing {
    /// Java: `fInterceptorId`
    interceptor_id: Option<String>,
    /// Java: `fCatcherId`
    catcher_id: Option<String>,
    /// Java: `fPassAccurate`
    pass_accurate: bool,
    /// Java: `fPassFumble`
    pass_fumble: bool,
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
    /// Java: `fBombOutOfBounds`
    bomb_out_of_bounds: bool,
    /// Java: `dontDropFumble`
    dont_drop_fumble: bool,
}

impl StepEndPassing {
    pub fn new() -> Self {
        Self {
            interceptor_id: None,
            catcher_id: None,
            pass_accurate: false,
            pass_fumble: false,
            end_turn: false,
            end_player_action: false,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        game.field_model.range_ruler = None;
        game.field_model.out_of_bounds = false;
        // DEFERRED(BombSequence): bomb turn → push Bomb sequence — TurnMode.BOMB not yet wired.
        // DEFERRED(Animosity): animosity re-selection pass sequence push not yet ported.
        // DEFERRED(completions): record completions / passing yards on thrower result — SPP tracking deferred.
        // DEFERRED(intercept): set ball coordinate to interceptor, record interceptions — state tracking deferred.
        // DEFERRED(EndPlayerAction): push EndPlayerAction sequence — sequence infrastructure deferred.
        StepOutcome::next()
    }
}

impl Default for StepEndPassing {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndPassing {
    fn id(&self) -> StepId { StepId::EndPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v)       => { self.catcher_id = v.clone(); true }
            StepParameter::InterceptorId(v)   => { self.interceptor_id = v.clone(); true }
            StepParameter::PassAccurate(v)    => { self.pass_accurate = *v; true }
            StepParameter::PassFumble(v)      => { self.pass_fumble = *v; true }
            StepParameter::EndTurn(v)         => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::BombOutOfBounds(v) => { self.bomb_out_of_bounds = *v; true }
            StepParameter::DontDropFumble(v)  => { self.dont_drop_fumble = *v; true }
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
    fn id_is_end_passing() {
        assert_eq!(StepEndPassing::new().id(), StepId::EndPassing);
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_pass_accurate() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::PassAccurate(true)));
        assert!(step.pass_accurate);
    }

    #[test]
    fn set_parameter_interceptor_id_none() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(None)));
        assert!(step.interceptor_id.is_none());
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndPassing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn clears_range_ruler_and_out_of_bounds() {
        let mut game = make_game();
        game.field_model.out_of_bounds = true;
        let mut step = StepEndPassing::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.field_model.out_of_bounds);
        assert!(game.field_model.range_ruler.is_none());
    }
}
