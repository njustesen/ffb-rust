/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepThenIStartedBlastin (BB2025).
///
/// Resolves the "Then I Started Blastin'!" ability: throw a keg at a target, causing injury.
///
/// Commands: CLIENT_TARGET_SELECTED (target selection), CLIENT_END_TURN.
///
/// Stub: NamedProperties.canBlastRemotePlayer not translated → skill check always fails.
/// InjuryTypeThenIStartedBlastin not translated → NEXT_STEP immediately.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepThenIStartedBlastin {
    /// Java: gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: roll — the skill die result.
    pub roll: i32,
}

impl StepThenIStartedBlastin {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            roll: 0,
        }
    }
}

impl Default for StepThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl Step for StepThenIStartedBlastin {
    fn id(&self) -> StepId { StepId::ThenIStartedBlastin }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Stub: NamedProperties.canBlastRemotePlayer not translated → NEXT_STEP
        StepOutcome::next()
    }

    fn handle_command(&mut self, action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { .. } => {}
            Action::EndTurn => {
                // Java: restoreTurnModes + publish END_PLAYER_ACTION + NEXT_STEP
            }
            _ => {}
        }
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_end_turn_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepThenIStartedBlastin::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
    }
}
