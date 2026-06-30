/// BB2025 start-of-game step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.StartGame`.
use crate::step::framework::StepId;
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct StartGame;

impl StartGame {
    pub fn new() -> Self { Self }

    /// Build the start-game step sequence (Java `pushSequence`).
    /// After BUY_INDUCEMENTS the Kickoff sequence is pushed separately.
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 INIT_START_GAME
        seq.add(StepId::InitStartGame, vec![]);
        // 2 SPECTATORS
        seq.add(StepId::Spectators, vec![]);
        // 3 WEATHER
        seq.add(StepId::Weather, vec![]);
        // 4 PETTY_CASH
        seq.add(StepId::PettyCash, vec![]);
        // 5 BUY_INDUCEMENTS → continues with Kickoff sequence after this
        seq.add(StepId::BuyInducements, vec![]);

        seq.build()
    }
}

impl Default for StartGame {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_game_sequence_has_five_steps() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn start_game_sequence_starts_with_init_start_game() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps[0].step_id, StepId::InitStartGame);
    }

    #[test]
    fn start_game_sequence_ends_with_buy_inducements() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps.last().unwrap().step_id, StepId::BuyInducements);
    }
}
