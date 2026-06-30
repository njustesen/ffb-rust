/// BB2016 block action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.Block`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 Block sequence.
#[derive(Debug, Clone, Default)]
pub struct BlockParams {
    pub block_defender_id: Option<String>,
    pub multi_block_defender_id: Option<String>,
    pub using_stab: bool,
    /// When true, the Foul Appearance check is skipped (Frenzy follow-up block).
    pub frenzy_block: bool,
}

pub struct Block;

impl Block {
    pub fn new() -> Self { Self }

    /// Build the BB2016 block step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &BlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;

        // 1 INIT_BLOCKING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingStab(params.using_stab),
        ];
        if let Some(ref id) = params.block_defender_id {
            init_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        if let Some(ref id) = params.multi_block_defender_id {
            init_params.push(StepParameter::MultiBlockDefenderId(Some(id.clone())));
        }
        seq.add(StepId::InitBlocking, init_params);

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
        // 7 GO_FOR_IT
        seq.add(StepId::GoForIt, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);
        // 8 FOUL_APPEARANCE (skipped for frenzy follow-up blocks)
        if !params.frenzy_block {
            seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        }
        // 9 HORNS
        seq.add(StepId::Horns, vec![]);
        // 10 BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![]);
        // 11 DAUNTLESS
        seq.add(StepId::Dauntless, vec![]);
        // 12 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 13 STAB
        seq.add(StepId::Stab, vec![
            StepParameter::GotoLabelOnSuccess(labels::APOTHECARY_DEFENDER.into()),
        ]);
        // 14 BLOCK_CHAINSAW
        seq.add(StepId::BlockChainsaw, vec![
            StepParameter::GotoLabelOnSuccess(labels::APOTHECARY_DEFENDER.into()),
            StepParameter::GotoLabelOnFailure(labels::APOTHECARY_ATTACKER.into()),
        ]);
        // 15 BLOCK_BALL_AND_CHAIN
        seq.add(StepId::BlockBallAndChain, vec![
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 16 BLOCK_ROLL
        seq.add(StepId::BlockRoll, vec![]);
        // 17 BLOCK_CHOICE
        seq.add(StepId::BlockChoice, vec![
            StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
            StepParameter::GotoLabelOnJuggernaut(labels::JUGGERNAUT.into()),
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 18 GOTO_LABEL → DROP_FALLING_PLAYERS (skull)
        seq.jump(labels::DROP_FALLING_PLAYERS);

        // on blockChoice = BOTH_DOWN
        // 19 JUGGERNAUT [JUGGERNAUT]
        seq.add_labelled(StepId::Juggernaut, labels::JUGGERNAUT, vec![
            StepParameter::GotoLabelOnSuccess(labels::PUSHBACK.into()),
        ]);
        // 20 BOTH_DOWN
        seq.add(StepId::BothDown, vec![]);
        // 21 WRESTLE
        seq.add(StepId::Wrestle, vec![]);
        // 22 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);

        // on blockChoice = POW_PUSHBACK
        // 23 BLOCK_DODGE [DODGE_BLOCK]
        seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);

        // on blockChoice = POW or PUSHBACK
        // 24 PUSHBACK [PUSHBACK]
        seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);
        // 25 APOTHECARY (crowd push)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::CrowdPush),
        ]);
        // 26 FOLLOWUP
        seq.add(StepId::Followup, vec![]);
        // 27 SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // 28 PICK_UP
        seq.add(StepId::PickUp, vec![
            StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
        ]);
        // 29 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);

        // 30 FALL_DOWN [FALL_DOWN]
        seq.add_labelled(StepId::FallDown, labels::FALL_DOWN, vec![]);
        // 31 GOTO_LABEL → APOTHECARY_ATTACKER
        seq.jump(labels::APOTHECARY_ATTACKER);

        // 32 DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS]
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // 33 APOTHECARY [APOTHECARY_DEFENDER] (defender)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_DEFENDER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 34 GO_FOR_IT (ball-and-chain GFI)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 35 GOTO_LABEL → APOTHECARY_ATTACKER
        seq.jump(labels::APOTHECARY_ATTACKER);

        // 36 DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS] (second)
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // 37 FALL_DOWN
        seq.add(StepId::FallDown, vec![]);

        // 38 APOTHECARY [APOTHECARY_ATTACKER] (attacker)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_ATTACKER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 39 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 40 END_BLOCKING [END_BLOCKING]
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
    fn block_sequence_contains_bone_head() {
        let steps = Block::build_sequence(&BlockParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn block_sequence_includes_foul_appearance_when_not_frenzy() {
        let steps = Block::build_sequence(&BlockParams { frenzy_block: false, ..Default::default() });
        assert!(steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn block_sequence_omits_foul_appearance_when_frenzy() {
        let steps = Block::build_sequence(&BlockParams { frenzy_block: true, ..Default::default() });
        assert!(!steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn block_sequence_has_apothecary_defender_label() {
        let steps = Block::build_sequence(&BlockParams::default());
        assert!(steps.iter().any(|s| s.label.as_deref() == Some(labels::APOTHECARY_DEFENDER)));
    }

    #[test]
    fn block_sequence_pushback_is_labelled() {
        let steps = Block::build_sequence(&BlockParams::default());
        let pb = steps.iter().find(|s| s.step_id == StepId::Pushback).unwrap();
        assert_eq!(pb.label.as_deref(), Some(labels::PUSHBACK));
    }
}
