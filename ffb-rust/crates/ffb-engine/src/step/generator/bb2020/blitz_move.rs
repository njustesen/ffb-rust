/// BB2020 blitz-move step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.BlitzMove`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 BlitzMove sequence.
#[derive(Debug, Clone, Default)]
pub struct BlitzMoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
}

pub struct BlitzMove;

impl BlitzMove {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlitzMoveParams) -> Vec<SequenceStep> {
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

        // NO activation block for BlitzMove

        // 2 MOVE_BALL_AND_CHAIN
        seq.add(StepId::MoveBallAndChain, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnFallDown(labels::FALL_DOWN.into()),
        ]);
        // 3 MOVE
        seq.add(StepId::Move, vec![]);
        // 4 GO_FOR_IT
        seq.add(StepId::GoForIt, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 5 GO_FOR_IT (ball-and-chain)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 6 TENTACLES
        seq.add(StepId::Tentacles, vec![StepParameter::GotoLabelOnSuccess(fl.into())]);
        // 7 JUMP
        let mut jump_params = vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())];
        if let Some(coord) = params.move_start {
            jump_params.push(StepParameter::MoveStart(coord));
        }
        seq.add(StepId::Jump, jump_params);
        // 8 MOVE_DODGE
        seq.add(StepId::MoveDodge, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 9 DIVING_TACKLE
        seq.add(StepId::DivingTackle, vec![StepParameter::GotoLabelOnSuccess(labels::RETRY_DODGE.into())]);
        // 10 GOTO → SHADOWING
        seq.jump(labels::SHADOWING);
        // 11 MOVE_DODGE [RETRY_DODGE]
        seq.add_labelled(StepId::MoveDodge, labels::RETRY_DODGE, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 12 DROP_DIVING_TACKLER
        seq.add(StepId::DropDivingTackler, vec![]);
        // 13 SHADOWING [SHADOWING]
        seq.add_labelled(StepId::Shadowing, labels::SHADOWING, vec![]);
        // 14 PICK_UP
        seq.add(StepId::PickUp, vec![StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into())]);
        // 15 GOTO → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 16 DROP_DIVING_TACKLER [FALL_DOWN]
        seq.add_labelled(StepId::DropDivingTackler, labels::FALL_DOWN, vec![]);
        // 17 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 18 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 19 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 20 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Attacker)]);
        // 21 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 22 APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor)]);
        // 23 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 24 END_MOVING [END_MOVING]
        seq.add_labelled(StepId::EndMoving, fl, vec![]);

        seq.build()
    }
}

impl Default for BlitzMove {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitz_move_starts_with_init_moving() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        assert_eq!(steps[0].step_id, StepId::InitMoving);
    }

    #[test]
    fn blitz_move_ends_with_end_moving_labelled() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndMoving);
        assert_eq!(last.label.as_deref(), Some(labels::END_MOVING));
    }

    #[test]
    fn blitz_move_has_no_activation_block() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::InitActivation));
        assert!(!steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn blitz_move_has_no_steady_footing() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }

    #[test]
    fn blitz_move_trap_door_is_labelled_scatter_ball() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let td = steps.iter().find(|s| s.step_id == StepId::TrapDoor).unwrap();
        assert_eq!(td.label.as_deref(), Some(labels::SCATTER_BALL));
    }
}
