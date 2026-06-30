/// BB2025 blitz-move step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BlitzMove`.
/// Like Move but NO activation block and NO FoulAppearance/DumpOff/HypnoticGaze preamble;
/// EndMoving carries no bloodlust action.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters — mirrors Java `BlitzMove.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct BlitzMoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
}

pub struct BlitzMove;

impl BlitzMove {
    pub fn new() -> Self { Self }

    /// Build the blitz-move step sequence (Java `pushSequence`).
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

        // 2 MOVE_BALL_AND_CHAIN
        seq.add(StepId::MoveBallAndChain, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnFallDown(labels::FALL_DOWN.into()),
        ]);
        // 3 MOVE
        seq.add(StepId::Move, vec![]);
        // 4 GO_FOR_IT (plain)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 5 GO_FOR_IT (ball-and-chain variant)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 6 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 7 TENTACLES
        seq.add(StepId::Tentacles, vec![
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 8 JUMP (with MOVE_START if set)
        let mut jump_params = vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ];
        if let Some(coord) = params.move_start {
            jump_params.push(StepParameter::MoveStart(coord));
        }
        seq.add(StepId::Jump, jump_params);
        // 9 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 10 MOVE_DODGE
        seq.add(StepId::MoveDodge, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 11 DIVING_TACKLE
        seq.add(StepId::DivingTackle, vec![
            StepParameter::GotoLabelOnSuccess(labels::RETRY_DODGE.into()),
        ]);
        // 12 GOTO_LABEL → SHADOWING
        seq.jump(labels::SHADOWING);
        // 13 MOVE_DODGE [RETRY_DODGE]
        seq.add_labelled(StepId::MoveDodge, labels::RETRY_DODGE, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 14 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 15 DROP_DIVING_TACKLER
        seq.add(StepId::DropDivingTackler, vec![]);
        // 16 SHADOWING [SHADOWING]
        seq.add_labelled(StepId::Shadowing, labels::SHADOWING, vec![]);
        // 17 PICK_UP
        seq.add(StepId::PickUp, vec![
            StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
        ]);
        // 18 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 19 DROP_DIVING_TACKLER [FALL_DOWN]
        seq.add_labelled(StepId::DropDivingTackler, labels::FALL_DOWN, vec![]);
        // 20 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 21 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 22 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 23 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 24 APOTHECARY (ATTACKER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 25 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 26 APOTHECARY (TRAP_DOOR)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor),
        ]);
        // 27 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 28 END_MOVING [END_MOVING]
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
    fn blitz_move_has_no_foul_appearance_or_dump_off() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
        assert!(!steps.iter().any(|s| s.step_id == StepId::DumpOff));
        assert!(!steps.iter().any(|s| s.step_id == StepId::HypnoticGaze));
    }

    #[test]
    fn blitz_move_has_three_steady_footing_steps_labelled() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let sf: Vec<_> = steps.iter()
            .filter(|s| s.step_id == StepId::SteadyFooting && s.label.as_deref() == Some(labels::STEADY_FOOTING))
            .collect();
        assert_eq!(sf.len(), 3);
    }

    #[test]
    fn blitz_move_retry_dodge_is_labelled() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let rd = steps.iter().find(|s| s.label.as_deref() == Some(labels::RETRY_DODGE)).unwrap();
        assert_eq!(rd.step_id, StepId::MoveDodge);
    }

    #[test]
    fn blitz_move_drop_diving_tackler_is_labelled_fall_down() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let fds: Vec<_> = steps.iter()
            .filter(|s| s.step_id == StepId::DropDivingTackler && s.label.as_deref() == Some(labels::FALL_DOWN))
            .collect();
        assert_eq!(fds.len(), 1);
    }

    #[test]
    fn blitz_move_trap_door_is_labelled_scatter_ball() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        let td = steps.iter().find(|s| s.step_id == StepId::TrapDoor).unwrap();
        assert_eq!(td.label.as_deref(), Some(labels::SCATTER_BALL));
    }

    #[test]
    fn blitz_move_has_28_steps() {
        let steps = BlitzMove::build_sequence(&BlitzMoveParams::default());
        assert_eq!(steps.len(), 28);
    }
}
