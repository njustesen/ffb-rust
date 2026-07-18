/// BB2025 Then I Started Blastin' step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThenIStartedBlastin`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

pub struct ThenIStartedBlastin;

impl ThenIStartedBlastin {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 THEN_I_STARTED_BLASTIN (goto END on end)
        seq.add(StepId::ThenIStartedBlastin, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 15 STEADY_FOOTING (goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END.into()),
        ]);
        // 16 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 17 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 18 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 19 END_THEN_I_STARTED_BLASTIN [END]
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
    fn then_i_started_blastin_has_19_steps_with_activation() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) (13) + 6 own steps = 19.
        let steps = ThenIStartedBlastin::build_sequence();
        assert_eq!(steps.len(), 19);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn then_i_started_blastin_ends_with_end_labelled() {
        let steps = ThenIStartedBlastin::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThenIStartedBlastin);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn then_i_started_blastin_step_follows_activation_sub_sequence() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert_eq!(steps[13].step_id, StepId::ThenIStartedBlastin);
    }

    #[test]
    fn then_i_started_blastin_step_has_goto_label_on_end() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert!(steps[13].params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnEnd(_))));
    }

    #[test]
    fn contains_apothecary_step() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::Apothecary));
    }
}
