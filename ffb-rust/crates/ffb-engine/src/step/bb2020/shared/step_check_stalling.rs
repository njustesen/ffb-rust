use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepCheckStalling.
///
/// Checks if the ball-carrier is stalling (BB2020).
///
/// Java init param: IGNORE_ACTED_FLAG (default: true).
/// Java start(): if performCheck() → findStallingPlayer() → if found, add staller.
/// Then NEXT_STEP.
///
/// Full stalling check requires prayer state and pathfinding infrastructure.
/// `ignore_acted_flag` is kept as a field for future use; the Java init param
/// IGNORE_ACTED_FLAG is handled via set_parameter.
///
/// Note: StepParameter::IgnoreActedFlag does not yet exist in the Rust framework.
/// DEFERRED(stalling): wire IgnoreActedFlag once added to StepParameter enum.
pub struct StepCheckStalling {
    /// Java: fIgnoreActedFlag (default true in Java init)
    pub ignore_acted_flag: bool,
}

impl StepCheckStalling {
    pub fn new() -> Self {
        Self {
            ignore_acted_flag: true,
        }
    }
}

impl Default for StepCheckStalling {
    fn default() -> Self { Self::new() }
}

impl Step for StepCheckStalling {
    fn id(&self) -> StepId { StepId::CheckStalling }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(stalling): full stalling check requires prayer state, pathfinding infrastructure
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool {
        // DEFERRED(stalling): handle StepParameter::IgnoreActedFlag once added to the enum.
        // match param {
        //     StepParameter::IgnoreActedFlag(v) => { self.ignore_acted_flag = *v; true }
        //     _ => false,
        // }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn new_has_ignore_acted_flag_true() {
        let step = StepCheckStalling::new();
        assert!(step.ignore_acted_flag);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
