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
}
