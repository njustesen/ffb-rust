/// BB2016 pass action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.Pass`.
/// Note: Java `sequence.insertHooks(PASS_INTERCEPT)` is skipped — StepHooks not yet ported.
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 Pass sequence.
#[derive(Debug, Clone, Default)]
pub struct PassParams {
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct Pass;

impl Pass {
    pub fn new() -> Self { Self }

    /// Build the BB2016 pass step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &PassParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_PASSING;

        // 1 INIT_PASSING
        let mut init_params = vec![StepParameter::GotoLabelOnEnd(fl.into())];
        if let Some(coord) = params.target_coordinate {
            init_params.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitPassing, init_params);

        // 2 BONE_HEAD
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 3 REALLY_STUPID
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 4 TAKE_ROOT
        seq.add(StepId::TakeRoot, vec![]);
        // 5 WILD_ANIMAL
        seq.add(StepId::WildAnimal, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 6 BLOOD_LUST
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 7 BOMBARDIER
        seq.add(StepId::Bombardier, vec![]);
        // 8 ANIMOSITY
        seq.add(StepId::Animosity, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 9 PASS_BLOCK
        seq.add(StepId::PassBlock, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // 10 DISPATCH_PASSING
        seq.add(StepId::DispatchPassing, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnHandOver(labels::HAND_OVER.into()),
            StepParameter::GotoLabelOnHailMaryPass(labels::HAIL_MARY_PASS.into()),
        ]);
        // 11 INTERCEPT
        seq.add(StepId::Intercept, vec![
            StepParameter::GotoLabelOnFailure(labels::PASS.into()),
        ]);
        // NOTE: Java insertHooks(PASS_INTERCEPT) skipped — StepHooks not yet ported
        // 12 PASS [PASS]
        seq.add_labelled(StepId::Pass, labels::PASS, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnMissedPass(labels::MISSED_PASS.into()),
        ]);
        // 13 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 14 HAIL_MARY_PASS [HAIL_MARY_PASS]
        seq.add_labelled(StepId::HailMaryPass, labels::HAIL_MARY_PASS, vec![
            StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
        ]);
        // 15 MISSED_PASS [MISSED_PASS]
        seq.add_labelled(StepId::MissedPass, labels::MISSED_PASS, vec![]);
        // 16 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 17 HAND_OVER [HAND_OVER]
        seq.add_labelled(StepId::HandOver, labels::HAND_OVER, vec![]);
        // 18 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 19 END_PASSING [END_PASSING]
        seq.add_labelled(StepId::EndPassing, fl, vec![]);

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
    fn pass_starts_with_init_passing() {
        let steps = Pass::build_sequence(&PassParams::default());
        assert_eq!(steps[0].step_id, StepId::InitPassing);
    }

    #[test]
    fn pass_ends_with_end_passing_labelled() {
        let steps = Pass::build_sequence(&PassParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndPassing);
        assert_eq!(last.label.as_deref(), Some(labels::END_PASSING));
    }

    #[test]
    fn pass_has_bone_head_and_blood_lust() {
        let steps = Pass::build_sequence(&PassParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
        assert!(steps.iter().any(|s| s.step_id == StepId::BloodLust));
    }

    #[test]
    fn pass_pass_step_is_labelled() {
        let steps = Pass::build_sequence(&PassParams::default());
        let p = steps.iter().find(|s| s.step_id == StepId::Pass && s.label.is_some()).unwrap();
        assert_eq!(p.label.as_deref(), Some(labels::PASS));
    }

    #[test]
    fn pass_catch_scatter_is_labelled_scatter_ball() {
        let steps = Pass::build_sequence(&PassParams::default());
        let cst = steps.iter().find(|s| {
            s.step_id == StepId::CatchScatterThrowIn && s.label.as_deref() == Some(labels::SCATTER_BALL)
        });
        assert!(cst.is_some());
    }
}
