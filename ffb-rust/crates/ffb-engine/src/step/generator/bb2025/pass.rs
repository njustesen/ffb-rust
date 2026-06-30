/// BB2025 pass action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Pass`.
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters — mirrors Java `Pass.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct PassParams {
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct Pass;

impl Pass {
    pub fn new() -> Self { Self }

    /// Build the pass step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &PassParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_PASSING;

        // 1 INIT_PASSING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ];
        if let Some(coord) = params.target_coordinate {
            init_params.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitPassing, init_params);

        // 2 [ACTIVATION(END_PASSING; TARGET_COORDINATE → ANIMAL_SAVAGERY)]
        ActivationSequenceBuilder::new()
            .with_failure_label(fl)
            .with_target_coordinate(params.target_coordinate)
            .add_to(&mut seq);

        // 3 BOMBARDIER
        seq.add(StepId::Bombardier, vec![]);
        // 4 ANIMOSITY
        seq.add(StepId::Animosity, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // 5 PASS_BLOCK
        seq.add(StepId::PassBlock, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 6 DISPATCH_PASSING
        seq.add(StepId::DispatchPassing, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnHandOver(labels::HAND_OVER.into()),
            StepParameter::GotoLabelOnHailMaryPass(labels::HAIL_MARY_PASS.into()),
        ]);
        // 7 PASS [PASS]
        seq.add_labelled(StepId::Pass, labels::PASS, vec![
            StepParameter::GotoLabelOnEnd(labels::INTERCEPT.into()),
            StepParameter::GotoLabelOnMissedPass(labels::MISSED_PASS.into()),
            StepParameter::GotoLabelOnSavedFumble(fl.into()),
        ]);
        // 8 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 9 HAIL_MARY_PASS [HAIL_MARY_PASS]
        seq.add_labelled(StepId::HailMaryPass, labels::HAIL_MARY_PASS, vec![
            StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
        ]);
        // 10 MISSED_PASS [MISSED_PASS]
        seq.add_labelled(StepId::MissedPass, labels::MISSED_PASS, vec![]);
        // 11 INTERCEPT [INTERCEPT]
        seq.add_labelled(StepId::Intercept, labels::INTERCEPT, vec![
            StepParameter::GotoLabelOnFailure(labels::RESOLVE_PASS.into()),
        ]);
        // 12 insertHooks(PASS_INTERCEPT) — none for lineman; step would go here
        // 13 RESOLVE_PASS [RESOLVE_PASS]
        seq.add_labelled(StepId::ResolvePass, labels::RESOLVE_PASS, vec![]);
        // 14 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 15 HAND_OVER [HAND_OVER]
        seq.add_labelled(StepId::HandOver, labels::HAND_OVER, vec![]);
        // 16 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 17 RESET_TO_MOVE [END_PASSING]
        seq.add_labelled(StepId::ResetToMove, fl, vec![]);
        // 18 END_PASSING
        seq.add(StepId::EndPassing, vec![]);

        seq.build()
    }
}

impl Default for Pass {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_sequence_starts_with_init_passing() {
        let steps = Pass::build_sequence(&PassParams::default());
        assert_eq!(steps[0].step_id, StepId::InitPassing);
    }

    #[test]
    fn pass_sequence_ends_with_end_passing() {
        let steps = Pass::build_sequence(&PassParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndPassing);
    }

    #[test]
    fn pass_intercept_step_is_labelled() {
        let steps = Pass::build_sequence(&PassParams::default());
        let intercept = steps.iter().find(|s| s.step_id == StepId::Intercept).unwrap();
        assert_eq!(intercept.label.as_deref(), Some(labels::INTERCEPT));
    }

    #[test]
    fn pass_catch_scatter_is_labelled_scatter_ball() {
        let steps = Pass::build_sequence(&PassParams::default());
        let cst = steps.iter().find(|s| {
            s.step_id == StepId::CatchScatterThrowIn && s.label.as_deref() == Some(labels::SCATTER_BALL)
        });
        assert!(cst.is_some());
    }

    #[test]
    fn pass_reset_to_move_is_labelled_end_passing() {
        let steps = Pass::build_sequence(&PassParams::default());
        let rtm = steps.iter().find(|s| s.step_id == StepId::ResetToMove).unwrap();
        assert_eq!(rtm.label.as_deref(), Some(labels::END_PASSING));
    }
}
