/// Builds the kickoff step sequence (BB2016/BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.mixed.Kickoff`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the mixed Kickoff sequence.
#[derive(Debug, Clone, Default)]
pub struct KickoffParams {
    /// Whether to insert CoinChoice/ReceiveChoice steps at the start.
    pub with_coin_choice: bool,
}

pub struct Kickoff;

impl Kickoff {
    pub fn new() -> Self { Self }

    /// Build the mixed kickoff step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &KickoffParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        if params.with_coin_choice {
            // 1 COIN_CHOICE
            seq.add(StepId::CoinChoice, vec![]);
            // 2 RECEIVE_CHOICE
            seq.add(StepId::ReceiveChoice, vec![]);
        }
        // INIT_KICKOFF (inducement sequence may be inserted after)
        seq.add(StepId::InitKickoff, vec![]);
        // SETUP (home) → END_KICKOFF on end
        seq.add(StepId::Setup, vec![StepParameter::GotoLabelOnEnd(labels::END_KICKOFF.into())]);
        // SETUP (away) → END_KICKOFF on end (inducement sequence may be inserted after)
        seq.add(StepId::Setup, vec![StepParameter::GotoLabelOnEnd(labels::END_KICKOFF.into())]);
        // KICKOFF
        seq.add(StepId::Kickoff, vec![]);
        // KICKOFF_SCATTER_ROLL
        seq.add(StepId::KickoffScatterRoll, vec![]);
        // SWARMING (kicking team)
        seq.add(StepId::Swarming, vec![StepParameter::HandleReceivingTeam(false)]);
        // SWARMING (receiving team)
        seq.add(StepId::Swarming, vec![StepParameter::HandleReceivingTeam(true)]);
        // KICKOFF_RETURN (select sequence may be inserted after)
        seq.add(StepId::KickoffReturn, vec![]);
        // KICKOFF_RESULT_ROLL
        seq.add(StepId::KickoffResultRoll, vec![]);
        // APPLY_KICKOFF_RESULT → END_KICKOFF on end, BLITZ_TURN on blitz result
        seq.add(StepId::ApplyKickoffResult, vec![
            StepParameter::GotoLabelOnEnd(labels::END_KICKOFF.into()),
            StepParameter::GotoLabelOnBlitz(labels::BLITZ_TURN.into()),
        ]);
        // APOTHECARY home (send-off steps may be inserted before)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Home)]);
        // APOTHECARY away
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Away)]);
        // GOTO_LABEL → KICKOFF_ANIMATION
        seq.jump(labels::KICKOFF_ANIMATION);
        // [BLITZ_TURN] BLITZ_TURN (select sequence may be inserted after)
        seq.add_labelled(StepId::BlitzTurn, labels::BLITZ_TURN, vec![]);
        // [KICKOFF_ANIMATION] KICKOFF_ANIMATION
        seq.add_labelled(StepId::KickoffAnimation, labels::KICKOFF_ANIMATION, vec![]);
        // CATCH_SCATTER_THROW_IN (ball on kickoff)
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // TOUCHBACK
        seq.add(StepId::Touchback, vec![]);
        // CATCH_SCATTER_THROW_IN (touchback catch)
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // [END_KICKOFF] END_KICKOFF (end-turn sequence continues after)
        seq.add_labelled(StepId::EndKickoff, labels::END_KICKOFF, vec![]);

        seq.build()
    }
}

impl Default for Kickoff {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kickoff_without_coin_choice_starts_with_init_kickoff() {
        let steps = Kickoff::build_sequence(&KickoffParams { with_coin_choice: false });
        assert_eq!(steps[0].step_id, StepId::InitKickoff);
    }

    #[test]
    fn kickoff_with_coin_choice_starts_with_coin_choice() {
        let steps = Kickoff::build_sequence(&KickoffParams { with_coin_choice: true });
        assert_eq!(steps[0].step_id, StepId::CoinChoice);
        assert_eq!(steps[1].step_id, StepId::ReceiveChoice);
    }

    #[test]
    fn kickoff_ends_with_end_kickoff_labelled() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndKickoff);
        assert_eq!(last.label.as_deref(), Some(labels::END_KICKOFF));
    }

    #[test]
    fn kickoff_has_blitz_turn_labelled() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let bt = steps.iter().find(|s| s.label.as_deref() == Some(labels::BLITZ_TURN)).unwrap();
        assert_eq!(bt.step_id, StepId::BlitzTurn);
    }

    #[test]
    fn kickoff_has_kickoff_animation_labelled() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let ka = steps.iter().find(|s| s.label.as_deref() == Some(labels::KICKOFF_ANIMATION)).unwrap();
        assert_eq!(ka.step_id, StepId::KickoffAnimation);
    }

    #[test]
    fn kickoff_has_two_swarming_steps() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let count = steps.iter().filter(|s| s.step_id == StepId::Swarming).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn kickoff_has_two_setup_steps() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let count = steps.iter().filter(|s| s.step_id == StepId::Setup).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn kickoff_has_two_catch_scatter_steps() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let count = steps.iter().filter(|s| s.step_id == StepId::CatchScatterThrowIn).count();
        assert_eq!(count, 2);
    }
}
