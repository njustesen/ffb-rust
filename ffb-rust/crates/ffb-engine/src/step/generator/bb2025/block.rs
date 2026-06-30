/// BB2025 block action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Block`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters for the Block sequence — mirrors Java `Block.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct BlockParams {
    pub block_defender_id: Option<String>,
    pub multi_block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub using_chomp: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
}

pub struct Block;

impl Block {
    pub fn new() -> Self { Self }

    /// Build the block step sequence (Java `pushSequence`).
    /// Returns steps in authored order; the stack reverses them on push.
    pub fn build_sequence(params: &BlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;

        // 1 INIT_BLOCKING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingStab(params.using_stab),
            StepParameter::UsingChainsaw(params.using_chainsaw),
            StepParameter::UsingVomit(params.using_vomit),
            StepParameter::UsingBreatheFire(params.using_breathe_fire),
            StepParameter::UsingChomp(params.using_chomp),
            StepParameter::AskForBlockKind(params.ask_for_block_kind),
            StepParameter::PublishDefender(params.publish_defender),
        ];
        if let Some(ref id) = params.block_defender_id {
            init_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        if let Some(ref id) = params.multi_block_defender_id {
            init_params.push(StepParameter::MultiBlockDefenderId(Some(id.clone())));
        }
        seq.add(StepId::InitBlocking, init_params);

        // 2 [ACTIVATION(END_BLOCKING; SET_DEFENDER)]
        ActivationSequenceBuilder::new()
            .with_failure_label(fl)
            .with_old_defender(params.block_defender_id.clone())
            .with_eventual_defender(params.block_defender_id.clone())
            .add_to(&mut seq);

        // 3 FOUL_APPEARANCE
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // 4 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 5 JUMP_UP
        seq.add(StepId::JumpUp, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // 6 STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // 7 BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![]);
        // 8 DAUNTLESS
        seq.add(StepId::Dauntless, vec![]);
        // 9 TRICKSTER
        seq.add(StepId::Trickster, vec![]);
        // 10 PICK_UP
        seq.add(StepId::PickUp, vec![
            StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
        ]);
        // 11 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 12 STAB
        seq.add(StepId::Stab, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
        ]);
        // 13 BLOCK_CHAINSAW
        seq.add(StepId::BlockChainsaw, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 14 STEADY_FOOTING (chainsaw attacker — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 15 PROJECTILE_VOMIT
        seq.add(StepId::ProjectileVomit, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 16 BREATHE_FIRE
        seq.add(StepId::BreatheFire, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 17 STEADY_FOOTING (attacker — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 18 STEADY_FOOTING (defender — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 19 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 20 CHOMP
        seq.add(StepId::Chomp, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 21 GO_FOR_IT (ball-and-chain GFI)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
            StepParameter::BallAndChainGfi(true),
        ]);
        // 22 STEADY_FOOTING [STEADY_FOOTING] (goto NEXT label on failure)
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::NEXT.into()),
        ]);
        // 23 BLOCK_BALL_AND_CHAIN [NEXT]
        seq.add_labelled(StepId::BlockBallAndChain, labels::NEXT, vec![
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 24 BLOCK_ROLL
        seq.add(StepId::BlockRoll, vec![]);
        // 25 BLOCK_CHOICE
        seq.add(StepId::BlockChoice, vec![
            StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
            StepParameter::GotoLabelOnJuggernaut(labels::JUGGERNAUT.into()),
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 26 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 27 JUGGERNAUT [JUGGERNAUT]
        seq.add_labelled(StepId::Juggernaut, labels::JUGGERNAUT, vec![
            StepParameter::GotoLabelOnSuccess(labels::PUSHBACK.into()),
        ]);
        // 28 BOTH_DOWN
        seq.add(StepId::BothDown, vec![]);
        // 29 WRESTLE
        seq.add(StepId::Wrestle, vec![]);
        // 30 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 31 BLOCK_DODGE [DODGE_BLOCK]
        seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);
        // 32 PUSHBACK [PUSHBACK]
        seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);
        // 33 APOTHECARY (crowd push)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::CrowdPush),
        ]);
        // 34 FOLLOWUP
        seq.add(StepId::Followup, vec![]);
        // 35 DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS]
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // 36 STEADY_FOOTING (defender)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 37 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 38 PLACE_BALL [DEFENDER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::DEFENDER_DROPPED, vec![]);
        // 39 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 40 STEADY_FOOTING [ATTACKER_DROPPED] (attacker)
        seq.add_labelled(StepId::SteadyFooting, labels::ATTACKER_DROPPED, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 41 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 42 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 43 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 44 APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor),
        ]);
        // 45 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 46 END_BLOCKING [END_BLOCKING]
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
    fn block_sequence_contains_block_roll() {
        let steps = Block::build_sequence(&BlockParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BlockRoll));
    }

    #[test]
    fn block_sequence_contains_pushback_labelled() {
        let steps = Block::build_sequence(&BlockParams::default());
        let pushback = steps.iter().find(|s| s.step_id == StepId::Pushback).unwrap();
        assert_eq!(pushback.label.as_deref(), Some(labels::PUSHBACK));
    }

    #[test]
    fn block_sequence_contains_drop_falling_players_labelled() {
        let steps = Block::build_sequence(&BlockParams::default());
        let dfp = steps.iter().find(|s| s.step_id == StepId::DropFallingPlayers).unwrap();
        assert_eq!(dfp.label.as_deref(), Some(labels::DROP_FALLING_PLAYERS));
    }

    #[test]
    fn block_sequence_defender_id_in_init_params() {
        let params = BlockParams { block_defender_id: Some("p42".into()), ..Default::default() };
        let steps = Block::build_sequence(&params);
        let init = &steps[0];
        assert!(init.params.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(id) if id == "p42")));
    }
}
