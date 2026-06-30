/// BB2016 move action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.Move`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 Move sequence.
#[derive(Debug, Clone, Default)]
pub struct MoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
}

pub struct Move;

impl Move {
    pub fn new() -> Self { Self }

    /// Build the BB2016 move step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &MoveParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_MOVING;

        // 1 INIT_MOVING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GazeVictimId(params.gaze_victim_id.clone()),
        ];
        if !params.move_stack.is_empty() {
            init_params.push(StepParameter::MoveStack(params.move_stack.clone()));
        }
        seq.add(StepId::InitMoving, init_params);

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
        // 7 HYPNOTIC_GAZE [HYPNOTIC_GAZE]
        seq.add_labelled(StepId::HypnoticGaze, labels::HYPNOTIC_GAZE, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 8 MOVE_BALL_AND_CHAIN
        seq.add(StepId::MoveBallAndChain, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnFallDown(labels::FALL_DOWN.into()),
        ]);
        // 9 MOVE
        seq.add(StepId::Move, vec![]);
        // 10 GO_FOR_IT (plain)
        seq.add(StepId::GoForIt, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 11 GO_FOR_IT (ball-and-chain variant)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 12 TENTACLES
        seq.add(StepId::Tentacles, vec![StepParameter::GotoLabelOnSuccess(fl.into())]);
        // 13 JUMP (with move_start param)
        let mut jump_params = vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())];
        if let Some(coord) = params.move_start {
            jump_params.push(StepParameter::MoveStart(coord));
        }
        seq.add(StepId::Jump, jump_params);
        // 14 MOVE_DODGE
        seq.add(StepId::MoveDodge, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 15 DIVING_TACKLE
        seq.add(StepId::DivingTackle, vec![StepParameter::GotoLabelOnSuccess(labels::RETRY_DODGE.into())]);
        // 16 GOTO_LABEL → SHADOWING
        seq.jump(labels::SHADOWING);
        // 17 MOVE_DODGE [RETRY_DODGE]
        seq.add_labelled(StepId::MoveDodge, labels::RETRY_DODGE, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 18 DROP_DIVING_TACKLER
        seq.add(StepId::DropDivingTackler, vec![]);
        // 19 SHADOWING [SHADOWING]
        seq.add_labelled(StepId::Shadowing, labels::SHADOWING, vec![]);
        // 20 PICK_UP
        seq.add(StepId::PickUp, vec![StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into())]);
        // 21 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 22 DROP_DIVING_TACKLER [FALL_DOWN]
        seq.add_labelled(StepId::DropDivingTackler, labels::FALL_DOWN, vec![]);
        // 23 SHADOWING (falling player can be shadowed)
        seq.add(StepId::Shadowing, vec![]);
        // 24 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 25 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 26 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Attacker)]);
        // 27 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 28 END_MOVING [END_MOVING]
        seq.add_labelled(StepId::EndMoving, fl, vec![]);

        seq.build()
    }
}

impl Default for Move {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_starts_with_init_moving() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert_eq!(steps[0].step_id, StepId::InitMoving);
    }

    #[test]
    fn move_ends_with_end_moving() {
        let steps = Move::build_sequence(&MoveParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndMoving);
    }

    #[test]
    fn move_contains_bone_head_and_blood_lust() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
        assert!(steps.iter().any(|s| s.step_id == StepId::BloodLust));
    }

    #[test]
    fn move_jump_has_move_start_param_when_set() {
        use ffb_model::types::FieldCoordinate;
        let params = MoveParams { move_start: Some(FieldCoordinate { x: 5, y: 5 }), ..Default::default() };
        let steps = Move::build_sequence(&params);
        let jump = steps.iter().find(|s| s.step_id == StepId::Jump).unwrap();
        assert!(jump.params.iter().any(|p| matches!(p, StepParameter::MoveStart(_))));
    }

    #[test]
    fn move_scatter_ball_labelled() {
        let steps = Move::build_sequence(&MoveParams::default());
        let sb = steps.iter().find(|s| s.label.as_deref() == Some(labels::SCATTER_BALL)).unwrap();
        assert_eq!(sb.step_id, StepId::CatchScatterThrowIn);
    }
}
