/// BB2025 Black Ink step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BlackInk`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct BlackInkParams {
    pub failure_label: String,
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

pub struct BlackInk;

impl BlackInk {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlackInkParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let mut p = vec![StepParameter::GotoLabelOnFailure(params.failure_label.clone())];
        if let Some(state) = params.old_player_state {
            p.push(StepParameter::OldPlayerState(state));
        }
        seq.add_labelled(StepId::BlackInk, labels::END, p);
        seq.build()
    }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_ink_single_step_labelled_end() {
        let steps = BlackInk::build_sequence(&BlackInkParams {
            failure_label: "X".into(),
            old_player_state: None,
        });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::BlackInk);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }
}
