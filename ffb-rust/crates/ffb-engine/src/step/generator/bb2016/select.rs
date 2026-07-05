/// BB2016 player-selection step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.Select`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 Select sequence.
#[derive(Debug, Clone, Default)]
pub struct SelectParams {
    pub update_persistence: bool,
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }

    /// Build the BB2016 select step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &SelectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_SELECTING;

        // 1 INIT_SELECTING
        seq.add(StepId::InitSelecting, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UpdatePersistence(params.update_persistence),
        ]);
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
        // 7 JUMP_UP
        seq.add(StepId::JumpUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 8 STAND_UP
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 9 END_SELECTING [END_SELECTING]
        seq.add_labelled(StepId::EndSelecting, fl, vec![]);

        seq.build()
    }
}

impl Default for Select {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_starts_with_init_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps[0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn select_ends_with_end_selecting_labelled() {
        let steps = Select::build_sequence(&SelectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndSelecting);
        assert_eq!(last.label.as_deref(), Some(labels::END_SELECTING));
    }

    #[test]
    fn select_has_bone_head_and_blood_lust() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
        assert!(steps.iter().any(|s| s.step_id == StepId::BloodLust));
    }

    #[test]
    fn select_has_jump_up_and_stand_up() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::JumpUp));
        assert!(steps.iter().any(|s| s.step_id == StepId::StandUp));
    }

    #[test]
    fn update_persistence_param_passed_to_init_selecting() {
        let params = SelectParams { update_persistence: true };
        let steps = Select::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::UpdatePersistence(true)));
        assert!(has);
    }

    #[test]
    fn select_has_nine_steps() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps.len(), 9);
    }
}
