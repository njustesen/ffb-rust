/// BB2020 blitz-block step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.BlitzBlock`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 BlitzBlock sequence.
#[derive(Debug, Clone, Default)]
pub struct BlitzBlockParams {
    pub block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
    /// Whether this is a frenzy follow-up block (affects whether FoulAppearance is included; Horns is always present).
    pub frenzy_block: bool,
}

pub struct BlitzBlock;

impl BlitzBlock {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlitzBlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;

        // 1 INIT_BLOCKING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingStab(params.using_stab),
            StepParameter::UsingChainsaw(params.using_chainsaw),
            StepParameter::UsingVomit(params.using_vomit),
            StepParameter::UsingBreatheFire(params.using_breathe_fire),
            StepParameter::AskForBlockKind(params.ask_for_block_kind),
            StepParameter::PublishDefender(params.publish_defender),
        ];
        if let Some(ref id) = params.block_defender_id {
            init_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        seq.add(StepId::InitBlocking, init_params);

        // 2 GO_FOR_IT (blitz GFI)
        seq.add(StepId::GoForIt, vec![StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into())]);

        // Frenzy-only FoulAppearance
        if params.frenzy_block {
            seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        }
        // HORNS always present
        seq.add(StepId::Horns, vec![]);

        // BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![]);
        // DAUNTLESS
        seq.add(StepId::Dauntless, vec![]);
        // TRICKSTER
        seq.add(StepId::Trickster, vec![]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // STAB
        seq.add(StepId::Stab, vec![StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into())]);
        // BLOCK_CHAINSAW
        seq.add(StepId::BlockChainsaw, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // PROJECTILE_VOMIT
        seq.add(StepId::ProjectileVomit, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // BREATHE_FIRE
        seq.add(StepId::BreatheFire, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // BLOCK_ROLL
        seq.add(StepId::BlockRoll, vec![]);
        // BLOCK_CHOICE
        seq.add(StepId::BlockChoice, vec![
            StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
            StepParameter::GotoLabelOnJuggernaut(labels::JUGGERNAUT.into()),
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // JUGGERNAUT [JUGGERNAUT]
        seq.add_labelled(StepId::Juggernaut, labels::JUGGERNAUT, vec![
            StepParameter::GotoLabelOnSuccess(labels::PUSHBACK.into()),
        ]);
        // BOTH_DOWN
        seq.add(StepId::BothDown, vec![]);
        // WRESTLE
        seq.add(StepId::Wrestle, vec![]);
        // GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // BLOCK_DODGE [DODGE_BLOCK]
        seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);
        // PUSHBACK [PUSHBACK]
        seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);
        // REMOVE_TARGET_SELECTION_STATE (retain)
        seq.add(StepId::RemoveTargetSelectionState, vec![StepParameter::Retain(true)]);
        // APOTHECARY (crowd push)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::CrowdPush)]);
        // FOLLOWUP
        seq.add(StepId::Followup, vec![]);
        // TENTACLES
        seq.add(StepId::Tentacles, vec![StepParameter::GotoLabelOnSuccess(labels::DROP_FALLING_PLAYERS.into())]);
        // SHADOWING
        seq.add(StepId::Shadowing, vec![]);
        // PICK_UP
        seq.add(StepId::PickUp, vec![StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into())]);
        // GOTO → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // FALL_DOWN [FALL_DOWN]
        seq.add_labelled(StepId::FallDown, labels::FALL_DOWN, vec![]);
        // GOTO → ATTACKER_DROPPED
        seq.jump(labels::ATTACKER_DROPPED);
        // DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS]
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // REMOVE_TARGET_SELECTION_STATE (retain)
        seq.add(StepId::RemoveTargetSelectionState, vec![StepParameter::Retain(true)]);
        // RESET_FUMBLEROOSKIE (reset for failed block)
        seq.add(StepId::ResetFumblerooskie, vec![StepParameter::ResetForFailedBlock(true)]);
        // PLACE_BALL [DEFENDER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::DEFENDER_DROPPED, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // GO_FOR_IT (ball-and-chain GFI) → DROP_FALLING_PLAYERS
        seq.add(StepId::GoForIt, vec![
            StepParameter::BallAndChainGfi(true),
            StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
        ]);
        // GOTO → ATTACKER_DROPPED
        seq.jump(labels::ATTACKER_DROPPED);
        // DROP_FALLING_PLAYERS (no label — second one)
        seq.add(StepId::DropFallingPlayers, vec![]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // FALL_DOWN
        seq.add(StepId::FallDown, vec![]);
        // PLACE_BALL [ATTACKER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::ATTACKER_DROPPED, vec![]);
        // APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Attacker)]);
        // TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor)]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_BLOCKING [END_BLOCKING]
        seq.add_labelled(StepId::EndBlocking, fl, vec![]);

        seq.build()
    }
}

impl Default for BlitzBlock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitz_block_starts_with_init_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn blitz_block_ends_with_end_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndBlocking);
    }

    #[test]
    fn blitz_block_has_no_activation_block() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn blitz_block_frenzy_includes_foul_appearance_and_horns() {
        let params = BlitzBlockParams { frenzy_block: true, ..Default::default() };
        let steps = BlitzBlock::build_sequence(&params);
        assert!(steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
        assert!(steps.iter().any(|s| s.step_id == StepId::Horns));
    }

    #[test]
    fn blitz_block_no_frenzy_excludes_foul_appearance_but_has_horns() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
        // Horns is always present, regardless of frenzy
        assert!(steps.iter().any(|s| s.step_id == StepId::Horns));
    }
}
