/// BB2025 Auto Gaze Zoat step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.AutoGazeZoat`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct AutoGazeZoatParams {
    pub failure_label: String,
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

pub struct AutoGazeZoat;

impl AutoGazeZoat {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &AutoGazeZoatParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let mut p = vec![StepParameter::GotoLabelOnFailure(params.failure_label.clone())];
        if let Some(state) = params.old_player_state {
            p.push(StepParameter::OldPlayerState(state));
        }
        seq.add_labelled(StepId::AutoGazeZoat, labels::END, p);
        seq.build()
    }
}

impl Default for AutoGazeZoat {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_gaze_zoat_single_step_labelled_end() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "someLabel".into(),
            old_player_state: None,
        });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::AutoGazeZoat);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }
}
