/// BB2020 Then I Started Blastin' step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.ThenIStartedBlastin`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct ThenIStartedBlastin;

impl ThenIStartedBlastin {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END;

        // ACTIVATION BLOCK (BalefulHex-style)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);  // no label
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![]);

        // THEN_I_STARTED_BLASTIN
        seq.add(StepId::ThenIStartedBlastin, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_THEN_I_STARTED_BLASTIN [END]
        seq.add_labelled(StepId::EndThenIStartedBlastin, labels::END, vec![]);

        seq.build()
    }
}

impl Default for ThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn then_i_started_blastin_has_activation_block() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn then_i_started_blastin_ends_labelled_end() {
        let steps = ThenIStartedBlastin::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThenIStartedBlastin);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn then_i_started_blastin_bone_head_has_no_label() {
        let steps = ThenIStartedBlastin::build_sequence();
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn then_i_started_blastin_has_no_steady_footing() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }
}
