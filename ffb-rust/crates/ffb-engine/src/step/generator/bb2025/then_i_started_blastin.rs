/// BB2025 Then I Started Blastin' step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThenIStartedBlastin`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct ThenIStartedBlastin;

impl ThenIStartedBlastin {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 THEN_I_STARTED_BLASTIN (goto END on end)
        seq.add(StepId::ThenIStartedBlastin, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 2 STEADY_FOOTING (goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END.into()),
        ]);
        // 3 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 4 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 5 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 6 END_THEN_I_STARTED_BLASTIN [END]
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
    fn then_i_started_blastin_has_6_steps() {
        let steps = ThenIStartedBlastin::build_sequence();
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn then_i_started_blastin_ends_with_end_labelled() {
        let steps = ThenIStartedBlastin::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThenIStartedBlastin);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }
}
