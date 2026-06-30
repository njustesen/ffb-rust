/// BB2025 move action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Move`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters — mirrors Java `Move.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct MoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
    pub ball_and_chain_rr_setting: Option<String>,
    pub bloodlust_action: Option<ffb_model::enums::PlayerAction>,
}

pub struct Move;

impl Move {
    pub fn new() -> Self { Self }

    /// Build the move step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &MoveParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_MOVING;

        // 1 INIT_MOVING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GazeVictimId(params.gaze_victim_id.clone()),
            StepParameter::BallAndChainRrSetting(params.ball_and_chain_rr_setting.clone()),
        ];
        if !params.move_stack.is_empty() {
            init_params.push(StepParameter::MoveStack(params.move_stack.clone()));
        }
        seq.add(StepId::InitMoving, init_params);

        // 2-15 [ACTIVATION(END_MOVING; preventNullDefender; SET_DEFENDER if gaze victim)]
        ActivationSequenceBuilder::new()
            .with_failure_label(fl)
            .with_eventual_defender(params.gaze_victim_id.clone())
            .prevent_null_defender()
            .add_to(&mut seq);

        // 16 FOUL_APPEARANCE
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // 17 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 18 HYPNOTIC_GAZE [HYPNOTIC_GAZE]
        seq.add_labelled(StepId::HypnoticGaze, labels::HYPNOTIC_GAZE, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 19 MOVE_BALL_AND_CHAIN
        seq.add(StepId::MoveBallAndChain, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnFallDown(labels::FALL_DOWN.into()),
        ]);
        // 20 MOVE
        seq.add(StepId::Move, vec![]);
        // 21 GO_FOR_IT (plain)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 22 GO_FOR_IT (ball-and-chain variant)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 23 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 24 TENTACLES
        seq.add(StepId::Tentacles, vec![
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 25 JUMP (with MOVE_START if set)
        let mut jump_params = vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ];
        if let Some(coord) = params.move_start {
            jump_params.push(StepParameter::MoveStart(coord));
        }
        seq.add(StepId::Jump, jump_params);
        // 26 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 27 MOVE_DODGE
        seq.add(StepId::MoveDodge, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 28 DIVING_TACKLE
        seq.add(StepId::DivingTackle, vec![
            StepParameter::GotoLabelOnSuccess(labels::RETRY_DODGE.into()),
        ]);
        // 29 GOTO_LABEL → SHADOWING
        seq.jump(labels::SHADOWING);
        // 30 MOVE_DODGE [RETRY_DODGE]
        seq.add_labelled(StepId::MoveDodge, labels::RETRY_DODGE, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 31 STEADY_FOOTING [STEADY_FOOTING]
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 32 DROP_DIVING_TACKLER
        seq.add(StepId::DropDivingTackler, vec![]);
        // 33 SHADOWING [SHADOWING]
        seq.add_labelled(StepId::Shadowing, labels::SHADOWING, vec![]);
        // 34 PICK_UP
        seq.add(StepId::PickUp, vec![
            StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
        ]);
        // 35 GOTO_LABEL → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 36 DROP_DIVING_TACKLER [FALL_DOWN]
        seq.add_labelled(StepId::DropDivingTackler, labels::FALL_DOWN, vec![]);
        // 37 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 38 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 39 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 40 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 41 APOTHECARY (ATTACKER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 42 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 43 APOTHECARY (TRAP_DOOR)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor),
        ]);
        // 44 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 45 END_MOVING [END_MOVING]
        let mut end_params = vec![];
        if let Some(action) = params.bloodlust_action {
            end_params.push(StepParameter::BloodLustAction(Some(action)));
        }
        seq.add_labelled(StepId::EndMoving, fl, end_params);

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
    fn move_sequence_starts_with_init_moving() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert_eq!(steps[0].step_id, StepId::InitMoving);
    }

    #[test]
    fn move_sequence_ends_with_end_moving_labelled() {
        let steps = Move::build_sequence(&MoveParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndMoving);
        assert_eq!(last.label.as_deref(), Some(labels::END_MOVING));
    }

    #[test]
    fn move_has_three_steady_footing_steps_labelled_steady_footing() {
        let steps = Move::build_sequence(&MoveParams::default());
        let sf: Vec<_> = steps.iter()
            .filter(|s| s.step_id == StepId::SteadyFooting && s.label.as_deref() == Some(labels::STEADY_FOOTING))
            .collect();
        assert_eq!(sf.len(), 3);
    }

    #[test]
    fn move_hypnotic_gaze_is_labelled_hypnotic_gaze() {
        let steps = Move::build_sequence(&MoveParams::default());
        let hg = steps.iter().find(|s| s.step_id == StepId::HypnoticGaze).unwrap();
        assert_eq!(hg.label.as_deref(), Some(labels::HYPNOTIC_GAZE));
    }

    #[test]
    fn move_retry_dodge_is_labelled() {
        let steps = Move::build_sequence(&MoveParams::default());
        let rd = steps.iter().find(|s| s.label.as_deref() == Some(labels::RETRY_DODGE)).unwrap();
        assert_eq!(rd.step_id, StepId::MoveDodge);
    }

    #[test]
    fn move_drop_diving_tackler_fall_down_is_labelled_fall_down() {
        let steps = Move::build_sequence(&MoveParams::default());
        let fds: Vec<_> = steps.iter()
            .filter(|s| s.step_id == StepId::DropDivingTackler && s.label.as_deref() == Some(labels::FALL_DOWN))
            .collect();
        assert_eq!(fds.len(), 1);
    }

    #[test]
    fn move_shadowing_is_labelled_shadowing() {
        let steps = Move::build_sequence(&MoveParams::default());
        let sh = steps.iter().find(|s| s.step_id == StepId::Shadowing && s.label.as_deref() == Some(labels::SHADOWING)).unwrap();
        assert_eq!(sh.step_id, StepId::Shadowing);
    }

    #[test]
    fn move_trap_door_is_labelled_scatter_ball() {
        let steps = Move::build_sequence(&MoveParams::default());
        let td = steps.iter().find(|s| s.step_id == StepId::TrapDoor).unwrap();
        assert_eq!(td.label.as_deref(), Some(labels::SCATTER_BALL));
    }

    #[test]
    fn move_has_activation_block() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn move_has_two_gfi_steps() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert_eq!(steps.iter().filter(|s| s.step_id == StepId::GoForIt).count(), 2);
    }
}
