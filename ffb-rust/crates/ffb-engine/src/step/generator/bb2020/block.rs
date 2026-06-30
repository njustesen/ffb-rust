/// BB2020 block action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Block`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 Block sequence.
#[derive(Debug, Clone, Default)]
pub struct BlockParams {
    pub block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
}

pub struct Block;

impl Block {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;

        // 1 INIT_BLOCKING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingStab(params.using_stab),
            StepParameter::UsingChainsaw(params.using_chainsaw),
            StepParameter::UsingVomit(params.using_vomit),
            StepParameter::AskForBlockKind(params.ask_for_block_kind),
            StepParameter::PublishDefender(params.publish_defender),
            StepParameter::UsingBreatheFire(params.using_breathe_fire),
        ];
        if let Some(ref id) = params.block_defender_id {
            init_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        seq.add(StepId::InitBlocking, init_params);

        // 2-13 ACTIVATION BLOCK (block style)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        let mut as_params = vec![StepParameter::GotoLabelOnFailure(fl_s.clone())];
        if let Some(ref id) = params.block_defender_id {
            as_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        seq.add(StepId::AnimalSavagery, as_params);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        if let Some(ref id) = params.block_defender_id {
            seq.add(StepId::SetDefender, vec![StepParameter::BlockDefenderId(id.clone())]);
        }
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // 14 FOUL_APPEARANCE
        seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 15 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 16 JUMP_UP
        seq.add(StepId::JumpUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 17 STAND_UP
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 18 BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![]);
        // 19 DAUNTLESS
        seq.add(StepId::Dauntless, vec![]);
        // 20 TRICKSTER
        seq.add(StepId::Trickster, vec![]);
        // 21 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 22 STAB
        seq.add(StepId::Stab, vec![StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into())]);
        // 23 BLOCK_CHAINSAW
        seq.add(StepId::BlockChainsaw, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 24 PROJECTILE_VOMIT
        seq.add(StepId::ProjectileVomit, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 25 BREATHE_FIRE
        seq.add(StepId::BreatheFire, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 26 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 27 GO_FOR_IT (ball-and-chain GFI)
        seq.add(StepId::GoForIt, vec![
            StepParameter::BallAndChainGfi(true),
            StepParameter::GotoLabelOnFailure(labels::NEXT.into()),
        ]);
        // 28 BLOCK_BALL_AND_CHAIN [NEXT]
        seq.add_labelled(StepId::BlockBallAndChain, labels::NEXT, vec![
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 29 BLOCK_ROLL
        seq.add(StepId::BlockRoll, vec![]);
        // 30 BLOCK_CHOICE
        seq.add(StepId::BlockChoice, vec![
            StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
            StepParameter::GotoLabelOnJuggernaut(labels::JUGGERNAUT.into()),
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 31 GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 32 JUGGERNAUT [JUGGERNAUT]
        seq.add_labelled(StepId::Juggernaut, labels::JUGGERNAUT, vec![
            StepParameter::GotoLabelOnSuccess(labels::PUSHBACK.into()),
        ]);
        // 33 BOTH_DOWN
        seq.add(StepId::BothDown, vec![]);
        // 34 WRESTLE
        seq.add(StepId::Wrestle, vec![]);
        // 35 GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 36 BLOCK_DODGE [DODGE_BLOCK]
        seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);
        // 37 PUSHBACK [PUSHBACK]
        seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);
        // 38 APOTHECARY (crowd push)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::CrowdPush)]);
        // 39 FOLLOWUP
        seq.add(StepId::Followup, vec![]);
        // 40 TENTACLES
        seq.add(StepId::Tentacles, vec![StepParameter::GotoLabelOnSuccess(labels::DROP_FALLING_PLAYERS.into())]);
        // 41 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 42 PICK_UP
        seq.add(StepId::PickUp, vec![StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into())]);
        // 43 GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 44 FALL_DOWN [FALL_DOWN]
        seq.add_labelled(StepId::FallDown, labels::FALL_DOWN, vec![]);
        // 45 GOTO → ATTACKER_DROPPED
        seq.jump(labels::ATTACKER_DROPPED);
        // 46 DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS]
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // 47 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 48 PLACE_BALL [DEFENDER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::DEFENDER_DROPPED, vec![]);
        // 49 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 50 GOTO → ATTACKER_DROPPED
        seq.jump(labels::ATTACKER_DROPPED);
        // 51 DROP_FALLING_PLAYERS (no label — second one for attacker path)
        seq.add(StepId::DropFallingPlayers, vec![]);
        // 52 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 53 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // 54 PLACE_BALL [ATTACKER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::ATTACKER_DROPPED, vec![]);
        // 55 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Attacker)]);
        // 56 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 57 APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor)]);
        // 58 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 59 END_BLOCKING [END_BLOCKING]
        seq.add_labelled(StepId::EndBlocking, fl, vec![]);

        seq.build()
    }
}

impl Default for Block {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_sequence_starts_with_init_blocking() {
        let steps = Block::build_sequence(&BlockParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn block_sequence_ends_with_end_blocking() {
        let steps = Block::build_sequence(&BlockParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndBlocking);
        assert_eq!(last.label.as_deref(), Some(labels::END_BLOCKING));
    }

    #[test]
    fn block_has_activation_block() {
        let steps = Block::build_sequence(&BlockParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn block_pushback_is_labelled() {
        let steps = Block::build_sequence(&BlockParams::default());
        let pb = steps.iter().find(|s| s.step_id == StepId::Pushback).unwrap();
        assert_eq!(pb.label.as_deref(), Some(labels::PUSHBACK));
    }

    #[test]
    fn block_drop_falling_players_is_labelled() {
        let steps = Block::build_sequence(&BlockParams::default());
        let dfp = steps.iter().find(|s| s.step_id == StepId::DropFallingPlayers).unwrap();
        assert_eq!(dfp.label.as_deref(), Some(labels::DROP_FALLING_PLAYERS));
    }
}
