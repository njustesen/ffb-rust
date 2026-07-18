/// BB2025 Baleful Hex step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BalefulHex`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct BalefulHexParams {
    pub failure_label: String,
}

pub struct BalefulHex;

impl BalefulHex {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BalefulHexParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 BALEFUL_HEX [END]
        seq.add_labelled(StepId::BalefulHex, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
        ]);
        seq.build()
    }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baleful_hex_last_step_labelled_end() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "X".into() });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::BalefulHex);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn failure_label_in_params() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "theLabel".into() });
        let last = steps.last().unwrap();
        let has = last.params.iter().any(|p| {
            matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "theLabel")
        });
        assert!(has);
    }

    #[test]
    fn baleful_hex_step_count_is_fourteen_with_activation() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) before BALEFUL_HEX.
        let steps = BalefulHex::build_sequence(&BalefulHexParams::default());
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn params_with_fields_set() {
        let p = BalefulHexParams { failure_label: "myLabel".into() };
        assert_eq!(p.failure_label, "myLabel");
    }

    #[test]
    fn params_clone() {
        let p = BalefulHexParams { failure_label: "lbl".into() };
        let q = p.clone();
        assert_eq!(q.failure_label, "lbl");
    }
}
