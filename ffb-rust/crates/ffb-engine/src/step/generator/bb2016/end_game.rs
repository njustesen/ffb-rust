/// BB2016 end-of-game step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.EndGame`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 EndGame sequence.
#[derive(Debug, Clone, Default)]
pub struct EndGameParams {
    pub admin_mode: bool,
}

pub struct EndGame;

impl EndGame {
    pub fn new() -> Self { Self }

    /// Build the BB2016 end-game step sequence (Java `pushSequence`).
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
        // 5 FAN_FACTOR
        seq.add(StepId::FanFactor, vec![]);
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
    fn end_game_starts_with_init_end_game() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert_eq!(steps[0].step_id, StepId::InitEndGame);
    }

    #[test]
    fn end_game_ends_with_end_game_labelled() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndGame);
        assert_eq!(last.label.as_deref(), Some(labels::END_GAME));
    }

    #[test]
    fn end_game_has_mvp_and_winnings() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::Mvp));
        assert!(steps.iter().any(|s| s.step_id == StepId::Winnings));
    }

    #[test]
    fn end_game_has_penalty_shootout_and_fan_factor() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::PenaltyShootout));
        assert!(steps.iter().any(|s| s.step_id == StepId::FanFactor));
    }

    #[test]
    fn admin_mode_param_passed_to_init() {
        let params = EndGameParams { admin_mode: true };
        let steps = EndGame::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::AdminMode(true)));
        assert!(has);
    }

    #[test]
    fn end_game_has_seven_steps() {
        let steps = EndGame::build_sequence(&EndGameParams::default());
        assert_eq!(steps.len(), 7);
    }
}
