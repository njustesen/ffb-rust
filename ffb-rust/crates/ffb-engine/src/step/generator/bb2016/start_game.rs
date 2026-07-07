/// BB2016 start-of-game step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.StartGame`.
use crate::step::framework::StepId;
use crate::step::generator::sequence::{Sequence, SequenceStep};

/// Parameters for the BB2016 StartGame sequence (none required by Java).
#[derive(Debug, Clone, Default)]
pub struct StartGameParams;

pub struct StartGame;

impl StartGame {
    pub fn new() -> Self { Self }

    /// Build the BB2016 start-game step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 INIT_START_GAME
        seq.add(StepId::InitStartGame, vec![]);
        // 2 WEATHER
        seq.add(StepId::Weather, vec![]);
        // 3 PETTY_CASH
        seq.add(StepId::PettyCash, vec![]);
        // 4 BUY_CARDS
        seq.add(StepId::BuyCards, vec![]);
        // 5 BUY_INDUCEMENTS
        seq.add(StepId::BuyInducements, vec![]);
        // 6 SPECTATORS
        seq.add(StepId::Spectators, vec![]);
        // continues with kickoffSequence at runtime

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
    fn start_game_starts_with_init_start_game() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps[0].step_id, StepId::InitStartGame);
    }

    #[test]
    fn start_game_ends_with_spectators() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps.last().unwrap().step_id, StepId::Spectators);
    }

    #[test]
    fn start_game_has_6_steps() {
        let steps = StartGame::build_sequence();
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn start_game_contains_weather_petty_cash_buy_cards_buy_inducements() {
        let steps = StartGame::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::Weather));
        assert!(steps.iter().any(|s| s.step_id == StepId::PettyCash));
        assert!(steps.iter().any(|s| s.step_id == StepId::BuyCards));
        assert!(steps.iter().any(|s| s.step_id == StepId::BuyInducements));
    }
    #[test]
    fn build_sequence_returns_vec() {
        let seq = StartGame::build_sequence();
        assert!(!seq.is_empty(), "sequence should not be empty");
    }

}
