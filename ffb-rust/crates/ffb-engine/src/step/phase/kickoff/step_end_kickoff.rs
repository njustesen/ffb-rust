/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepEndKickoff.
///
/// Pushes EndTurnSequence then Inducement(AfterKickoffToOpponent) on the stack.
use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};
use crate::step::sequences::{end_turn_sequence, inducement_sequence};

pub struct StepEndKickoff;

impl StepEndKickoff {
    pub fn new() -> Self { Self }
}

impl Default for StepEndKickoff {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndKickoff {
    fn id(&self) -> StepId { StepId::EndKickoff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepEndKickoff {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let end_turn_seq = end_turn_sequence(false);
        let inducement_seq = inducement_sequence(InducementPhase::AfterKickoffToOpponent, game.home_playing);
        StepOutcome::next().push_seq(end_turn_seq).push_seq(inducement_seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_pushes_end_turn_and_inducement_sequences() {
        let mut game = make_game();
        let mut step = StepEndKickoff::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2, "should push end_turn + inducement sequences");
    }

    #[test]
    fn id_is_end_kickoff() {
        assert_eq!(StepEndKickoff::new().id(), StepId::EndKickoff);
    }

    #[test]
    fn handle_command_also_pushes_sequences() {
        let mut game = make_game();
        let mut step = StepEndKickoff::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2);
    }

    #[test]
    fn set_parameter_always_returns_false() {
        let mut step = StepEndKickoff::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
    #[test]
    fn default_creates_equivalent_instance() {
        let _a = StepEndKickoff::new();
        let _b = StepEndKickoff::default();
    }
}
