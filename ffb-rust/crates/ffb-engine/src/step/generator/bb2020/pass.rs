/// BB2020 pass action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Pass`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 Pass sequence.
#[derive(Debug, Clone, Default)]
pub struct PassParams {
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct Pass;

impl Pass {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &PassParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_PASSING;

        // 1 INIT_PASSING
        let mut init_params = vec![StepParameter::GotoLabelOnEnd(fl.into())];
        if let Some(coord) = params.target_coordinate {
            init_params.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitPassing, init_params);

        // 2-13 ACTIVATION BLOCK (with GotoLabel, TargetCoordinate for AnimalSavagery)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        let mut as_params = vec![StepParameter::GotoLabelOnFailure(fl_s.clone())];
        if let Some(coord) = params.target_coordinate {
            as_params.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::AnimalSavagery, as_params);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // 14 BOMBARDIER
        seq.add(StepId::Bombardier, vec![]);
        // 15 ANIMOSITY
        seq.add(StepId::Animosity, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 16 PASS_BLOCK
        seq.add(StepId::PassBlock, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // 17 DISPATCH_PASSING
        seq.add(StepId::DispatchPassing, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnHandOver(labels::HAND_OVER.into()),
            StepParameter::GotoLabelOnHailMaryPass(labels::HAIL_MARY_PASS.into()),
        ]);
        // 18 PASS [PASS]
        seq.add_labelled(StepId::Pass, labels::PASS, vec![
            StepParameter::GotoLabelOnEnd(labels::INTERCEPT.into()),
            StepParameter::GotoLabelOnMissedPass(labels::MISSED_PASS.into()),
            StepParameter::GotoLabelOnSavedFumble(fl.into()),
        ]);
        // 19 GOTO → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 20 HAIL_MARY_PASS [HAIL_MARY_PASS]
        seq.add_labelled(StepId::HailMaryPass, labels::HAIL_MARY_PASS, vec![
            StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
        ]);
        // 21 MISSED_PASS [MISSED_PASS]
        seq.add_labelled(StepId::MissedPass, labels::MISSED_PASS, vec![]);
        // 22 INTERCEPT [INTERCEPT]
        seq.add_labelled(StepId::Intercept, labels::INTERCEPT, vec![
            StepParameter::GotoLabelOnFailure(labels::RESOLVE_PASS.into()),
        ]);
        // 23 RESOLVE_PASS [RESOLVE_PASS]
        seq.add_labelled(StepId::ResolvePass, labels::RESOLVE_PASS, vec![]);
        // 24 GOTO → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 25 HAND_OVER [HAND_OVER]
        seq.add_labelled(StepId::HandOver, labels::HAND_OVER, vec![]);
        // 26 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 27 RESET_TO_MOVE [END_PASSING]
        seq.add_labelled(StepId::ResetToMove, fl, vec![]);
        // 28 END_PASSING
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
    fn pass_has_activation_block() {
        let steps = Pass::build_sequence(&PassParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn pass_intercept_is_labelled() {
        let steps = Pass::build_sequence(&PassParams::default());
        let i = steps.iter().find(|s| s.step_id == StepId::Intercept).unwrap();
        assert_eq!(i.label.as_deref(), Some(labels::INTERCEPT));
    }

    #[test]
    fn pass_catch_scatter_labelled_scatter_ball() {
        let steps = Pass::build_sequence(&PassParams::default());
        let cst = steps.iter().find(|s| {
            s.step_id == StepId::CatchScatterThrowIn && s.label.as_deref() == Some(labels::SCATTER_BALL)
        });
        assert!(cst.is_some());
    }
}
