/// BB2020 move action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Move`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 Move sequence.
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

        // 2-13 ACTIVATION BLOCK (with GotoLabel, SetDefender for gaze victim)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // SetDefender with block_defender_id=gaze_victim_id (IgnoreNullValue=true so null is ignored)
        seq.add(StepId::SetDefender, vec![
            StepParameter::BlockDefenderId(params.gaze_victim_id.clone().unwrap_or_default()),
            StepParameter::IgnoreNullValue(true),
        ]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // 14 HYPNOTIC_GAZE [HYPNOTIC_GAZE]
        seq.add_labelled(StepId::HypnoticGaze, labels::HYPNOTIC_GAZE, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 15 MOVE_BALL_AND_CHAIN
        seq.add(StepId::MoveBallAndChain, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::GotoLabelOnFallDown(labels::FALL_DOWN.into()),
        ]);
        // 16 MOVE
        seq.add(StepId::Move, vec![]);
        // 17 GO_FOR_IT
        seq.add(StepId::GoForIt, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 18 GO_FOR_IT (ball-and-chain)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 19 TENTACLES
        seq.add(StepId::Tentacles, vec![StepParameter::GotoLabelOnSuccess(fl.into())]);
        // 20 JUMP
        let mut jump_params = vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())];
        if let Some(coord) = params.move_start {
            jump_params.push(StepParameter::MoveStart(coord));
        }
        seq.add(StepId::Jump, jump_params);
        // 21 MOVE_DODGE
        seq.add(StepId::MoveDodge, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 22 DIVING_TACKLE
        seq.add(StepId::DivingTackle, vec![StepParameter::GotoLabelOnSuccess(labels::RETRY_DODGE.into())]);
        // 23 GOTO → SHADOWING
        seq.jump(labels::SHADOWING);
        // 24 MOVE_DODGE [RETRY_DODGE]
        seq.add_labelled(StepId::MoveDodge, labels::RETRY_DODGE, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 25 DROP_DIVING_TACKLER
        seq.add(StepId::DropDivingTackler, vec![]);
        // 26 SHADOWING [SHADOWING]
        seq.add_labelled(StepId::Shadowing, labels::SHADOWING, vec![]);
        // 27 PICK_UP
        seq.add(StepId::PickUp, vec![StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into())]);
        // 28 GOTO → SCATTER_BALL
        seq.jump(labels::SCATTER_BALL);
        // 29 DROP_DIVING_TACKLER [FALL_DOWN]
        seq.add_labelled(StepId::DropDivingTackler, labels::FALL_DOWN, vec![]);
        // 30 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 31 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 32 APOTHECARY (defender) — no PlaceBall before in BB2020
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 33 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Attacker)]);
        // 34 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 35 APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor)]);
        // 36 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 37 END_MOVING [END_MOVING]
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
    fn move_has_activation_block() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn move_has_no_steady_footing() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }

    #[test]
    fn move_has_two_gfi_steps() {
        let steps = Move::build_sequence(&MoveParams::default());
        assert_eq!(steps.iter().filter(|s| s.step_id == StepId::GoForIt).count(), 2);
    }
}
