/// BB2025 end-of-game step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndGame`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct EndGameParams {
    pub admin_mode: bool,
}

pub struct EndGame;

impl EndGame {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &EndGameParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 INIT_END_GAME
        seq.add(StepId::InitEndGame, vec![
            StepParameter::GotoLabelOnEnd(labels::END_GAME.into()),
            StepParameter::AdminMode(params.admin_mode),
        ]);
        // 2 PENALTY_SHOOTOUT
        seq.add(StepId::PenaltyShootout, vec![]);
        // 3 MVP
        seq.add(StepId::Mvp, vec![]);
        // 4 WINNINGS
        seq.add(StepId::Winnings, vec![]);
        // 5 DEDICATED_FANS
        seq.add(StepId::DedicatedFans, vec![]);
        // 6 PLAYER_LOSS
        seq.add(StepId::PlayerLoss, vec![]);
        // 7 END_GAME [END_GAME]
        seq.add_labelled(StepId::EndGame, labels::END_GAME, vec![]);
        seq.build()
    }
}

impl Default for EndGame {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_game_has_7_steps() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert_eq!(steps.len(), 7);
    }

    #[test]
    fn end_game_ends_with_end_game_labelled_end_game() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndGame);
        assert_eq!(last.label.as_deref(), Some(labels::END_GAME));
    }

    #[test]
    fn first_step_is_init_end_game() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert_eq!(steps[0].step_id, StepId::InitEndGame);
    }

    #[test]
    fn admin_mode_param_set_in_first_step() {
        let steps = EndGame::build_sequence(&EndGameParams { admin_mode: true });
        let init = &steps[0];
        assert!(init.params.iter().any(|p| matches!(p, StepParameter::AdminMode(true))));
    }

    #[test]
    fn contains_mvp_step() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::Mvp));
    }

    #[test]
    fn contains_winnings_step() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::Winnings));
    }
}
