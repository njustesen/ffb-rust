/// BB2025 Baleful Hex step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BalefulHex`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct BalefulHexParams {
    pub failure_label: String,
}

pub struct BalefulHex;

impl BalefulHex {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BalefulHexParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
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
    fn baleful_hex_single_step_labelled_end() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "X".into() });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::BalefulHex);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }

    #[test]
    fn failure_label_in_params() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "theLabel".into() });
        let has = steps[0].params.iter().any(|p| {
            matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "theLabel")
        });
        assert!(has);
    }

    #[test]
    fn baleful_hex_step_count_is_one() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams::default());
        assert_eq!(steps.len(), 1);
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
