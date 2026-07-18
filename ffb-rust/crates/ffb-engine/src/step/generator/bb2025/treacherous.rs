/// BB2025 Treacherous step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Treacherous`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct TreacherousParams {
    pub failure_label: String,
}

pub struct Treacherous;

impl Treacherous {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &TreacherousParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // JUMP_UP
        seq.add(StepId::JumpUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END.into()),
        ]);
        // STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END.into()),
        ]);
        // TREACHEROUS [END]
        seq.add_labelled(StepId::Treacherous, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
        ]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        seq.build()
    }
}

impl Default for Treacherous {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treacherous_has_18_steps_with_activation() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) (13) + 5 own steps = 18.
        let steps = Treacherous::build_sequence(&TreacherousParams { failure_label: "X".into() });
        assert_eq!(steps.len(), 18);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn treacherous_is_labelled_end() {
        let steps = Treacherous::build_sequence(&TreacherousParams { failure_label: "X".into() });
        let t = steps.iter().find(|s| s.step_id == StepId::Treacherous).unwrap();
        assert_eq!(t.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn failure_label_wired_to_treacherous_step() {
        let steps = Treacherous::build_sequence(&TreacherousParams { failure_label: "MY_END".into() });
        let t = steps.iter().find(|s| s.step_id == StepId::Treacherous).unwrap();
        assert!(t.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "MY_END")));
    }

    #[test]
    fn jump_up_follows_activation_sub_sequence() {
        let steps = Treacherous::build_sequence(&TreacherousParams::default());
        assert_eq!(steps[13].step_id, StepId::JumpUp);
    }

    #[test]
    fn contains_apothecary_step_with_defender_mode() {
        // The activation sub-sequence's own Apothecary (AnimalSavagery mode) precedes this one,
        // so find the last Apothecary step (the Treacherous-specific one, Defender mode).
        let steps = Treacherous::build_sequence(&TreacherousParams::default());
        let apo = steps.iter().rev().find(|s| s.step_id == StepId::Apothecary).unwrap();
        assert!(apo.params.iter().any(|p| matches!(p, StepParameter::ApothecaryMode(ApothecaryMode::Defender))));
    }
}
