/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.multiblock.StepMultipleBlockFork`.
///
/// Forks the step stack for a multiple block action, pushing one sub-sequence per target.
///
/// Java `parameterToConsume` (fixed set at construction):
///   BLOCK_ROLL, BLOCK_RESULT, DICE_INDEX, NR_OF_DICE, STARTING_PUSHBACK_SQUARE,
///   DEFENDER_PUSHED, FOLLOWUP_CHOICE, USING_STAB, OLD_DEFENDER_STATE
///
/// init (from StepParameterSet):
///   - BLOCK_TARGETS: collect into `targets`.
///
/// setParameter:
///   - PLAYER_ID_TO_REMOVE: remove the matching BlockTarget from `targets`; consume; return true.
///
/// executeStep:
///   Groups targets by BlockKind.
///
///   For BLOCK group (non-empty):
///     Push sequence:
///       DAUNTLESS_MULTIPLE(BLOCK_TARGETS=blockGroup)
///       DOUBLE_STRENGTH
///       for each target: SET_DEFENDER(BLOCK_DEFENDER_ID=target.playerId)
///                        TRICKSTER
///                        CATCH_SCATTER_THROW_IN
///       BLOCK_ROLL_MULTIPLE(BLOCK_TARGETS=blockGroup, CONSUME_PARAMETER=parameterToConsume)
///
///   For STAB group (reversed, one sequence per target):
///     Publish OLD_DEFENDER_STATE, USING_STAB=true for each.
///     Push sequence per target:
///       SET_DEFENDER(BLOCK_DEFENDER_ID=target.playerId)
///       TRICKSTER
///       CATCH_SCATTER_THROW_IN
///       STAB(GOTO_LABEL_ON_SUCCESS=NEXT)
///       HANDLE_DROP_PLAYER_CONTEXT
///       REPORT_STAB_INJURY(label=NEXT, PLAYER_ID=target.playerId)
///       CONSUME_PARAMETER(parameterToConsume)
///
///   → NEXT_STEP
///
/// parameterToConsume is wired: passed to BLOCK_ROLL_MULTIPLE and each stab CONSUME_PARAMETER step.
///
use ffb_model::enums::{BlockResult, PlayerState};
use ffb_model::model::block_kind::BlockKind;
use ffb_model::model::block_target::BlockTarget;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{SequenceStep, Step, StepId, StepOutcome, StepParameter};

/// Java: parameterToConsume fixed set — passed to BLOCK_ROLL_MULTIPLE and stab CONSUME_PARAMETER steps.
fn params_to_consume() -> Vec<std::mem::Discriminant<StepParameter>> {
    vec![
        std::mem::discriminant(&StepParameter::BlockRoll(vec![])),
        std::mem::discriminant(&StepParameter::BlockResult(BlockResult::Pushback)),
        std::mem::discriminant(&StepParameter::DiceIndex(0)),
        std::mem::discriminant(&StepParameter::NrOfDice(0)),
        std::mem::discriminant(&StepParameter::StartingPushbackSquare(None)),
        std::mem::discriminant(&StepParameter::DefenderPushed(false)),
        std::mem::discriminant(&StepParameter::FollowupChoice(false)),
        std::mem::discriminant(&StepParameter::UsingStab(false)),
        std::mem::discriminant(&StepParameter::OldDefenderState(PlayerState::new(0))),
    ]
}

/// Java: `StepMultipleBlockFork` (bb2020/multiblock).
pub struct StepMultipleBlockFork {
    /// Java: `targets` (List<BlockTarget>)
    pub targets: Vec<BlockTarget>,
}

impl StepMultipleBlockFork {
    pub fn new() -> Self {
        Self { targets: Vec::new() }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Java: groupedTargets = targets.stream().collect(groupingBy(BlockTarget::getKind))
        let mut block_group: Vec<&BlockTarget> = Vec::new();
        let mut stab_group: Vec<&BlockTarget> = Vec::new();

        for target in &self.targets {
            match target.get_kind() {
                Some(BlockKind::BLOCK) => block_group.push(target),
                Some(BlockKind::STAB) => stab_group.push(target),
                _ => {} // VOMIT, CHAINSAW not handled here in Java
            }
        }

        let mut outcome = StepOutcome::next();

        // Java: if (blockGroup != null && !blockGroup.isEmpty()) { push block sequence }
        if !block_group.is_empty() {
            // Sequence:
            //   DAUNTLESS_MULTIPLE(BLOCK_TARGETS=blockGroup)
            //   DOUBLE_STRENGTH
            //   for each target: SET_DEFENDER / TRICKSTER / CATCH_SCATTER_THROW_IN
            //   BLOCK_ROLL_MULTIPLE(BLOCK_TARGETS=blockGroup, CONSUME_PARAMETER=parameterToConsume)
            let mut seq: Vec<SequenceStep> = Vec::new();

            // DAUNTLESS_MULTIPLE with block targets
            seq.push(SequenceStep::with_params(
                StepId::DauntlessMultiple,
                vec![StepParameter::BlockTargets(
                    block_group.iter()
                        .filter_map(|t| t.get_player_id().cloned())
                        .collect(),
                )],
            ));

            seq.push(SequenceStep::new(StepId::DoubleStrength));

            for target in &block_group {
                if let Some(pid) = target.get_player_id() {
                    seq.push(SequenceStep::with_params(
                        StepId::SetDefender,
                        vec![StepParameter::BlockDefenderId(pid.clone())],
                    ));
                    seq.push(SequenceStep::new(StepId::Trickster));
                    seq.push(SequenceStep::new(StepId::CatchScatterThrowIn));
                }
            }

            seq.push(SequenceStep::with_params(
                StepId::BlockRollMultiple,
                vec![
                    StepParameter::BlockTargets(
                        block_group.iter()
                            .filter_map(|t| t.get_player_id().cloned())
                            .collect(),
                    ),
                    StepParameter::ParametersToConsume(params_to_consume()),
                ],
            ));

            outcome = outcome.push_seq(seq);
        }

        // Java: if (stabGroup != null && !stabGroup.isEmpty()) { Collections.reverse(stabGroup); push per target }
        // Reversed so that first stab ends up on top of the stack (LIFO).
        let stab_group_rev: Vec<&BlockTarget> = stab_group.into_iter().rev().collect();
        for target in stab_group_rev {
            if let Some(pid) = target.get_player_id() {
                let mut seq: Vec<SequenceStep> = Vec::new();

                seq.push(SequenceStep::with_params(
                    StepId::SetDefender,
                    vec![StepParameter::BlockDefenderId(pid.clone())],
                ));
                seq.push(SequenceStep::new(StepId::Trickster));
                seq.push(SequenceStep::new(StepId::CatchScatterThrowIn));
                seq.push(SequenceStep::with_params(
                    StepId::Stab,
                    vec![StepParameter::GotoLabelOnSuccess("NEXT".into())],
                ));
                seq.push(SequenceStep::new(StepId::HandleDropPlayerContext));
                seq.push(SequenceStep::labelled(StepId::ReportStabInjury, "NEXT", vec![]));
                seq.push(SequenceStep::with_params(
                    StepId::ConsumeParameter,
                    vec![StepParameter::ParametersToConsume(params_to_consume())],
                ));

                outcome = outcome.push_seq(seq);

                // Java: publishParameter(OLD_DEFENDER_STATE, target.getOriginalPlayerState())
                if let Some(old_state) = target.get_original_player_state() {
                    outcome = outcome.publish(StepParameter::OldDefenderState(old_state));
                }
                // Java: publishParameter(USING_STAB, true)
                outcome = outcome.publish(StepParameter::UsingStab(true));
            }
        }

        outcome
    }
}

impl Default for StepMultipleBlockFork {
    fn default() -> Self { Self::new() }
}

impl Step for StepMultipleBlockFork {
    fn id(&self) -> StepId { StepId::MultipleBlockFork }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: StepParameterKey.PLAYER_ID_TO_REMOVE → remove target with matching playerId; consume.
            StepParameter::PlayerIdToRemove(id) => {
                self.targets.retain(|t| t.get_player_id().map(|pid| pid != id).unwrap_or(true));
                true
            }
            // Java: init(StepParameterSet) → BLOCK_TARGETS: targets.addAll(...)
            StepParameter::BlockTargets(ids) => {
                // BlockTargets in Rust carries player IDs; reconstruct BlockTargets with BLOCK kind.
                for pid in ids {
                    self.targets.push(BlockTarget::new(pid.clone(), BlockKind::BLOCK, None));
                }
                true
            }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_multiple_block_fork() {
        assert_eq!(StepMultipleBlockFork::new().id(), StepId::MultipleBlockFork);
    }

    #[test]
    fn no_targets_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn block_targets_push_sequence() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new();
        step.targets.push(BlockTarget::new("p1", BlockKind::BLOCK, None));
        step.targets.push(BlockTarget::new("p2", BlockKind::BLOCK, None));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // One sequence pushed for the block group
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn stab_targets_push_sequence_per_target() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new();
        step.targets.push(BlockTarget::new("s1", BlockKind::STAB, None));
        step.targets.push(BlockTarget::new("s2", BlockKind::STAB, None));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // One sequence per stab target
        assert_eq!(out.pushes.len(), 2);
    }

    #[test]
    fn player_id_to_remove_removes_target() {
        let mut step = StepMultipleBlockFork::new();
        step.targets.push(BlockTarget::new("p1", BlockKind::BLOCK, None));
        step.targets.push(BlockTarget::new("p2", BlockKind::BLOCK, None));
        assert!(step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into())));
        assert_eq!(step.targets.len(), 1);
        assert_eq!(step.targets[0].get_player_id().unwrap(), "p2");
    }

    #[test]
    fn set_parameter_block_targets_adds_targets() {
        let mut step = StepMultipleBlockFork::new();
        assert!(step.set_parameter(&StepParameter::BlockTargets(vec!["a".into(), "b".into()])));
        assert_eq!(step.targets.len(), 2);
    }

    #[test]
    fn stab_publishes_using_stab() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new();
        step.targets.push(BlockTarget::new("stab1", BlockKind::STAB, None));
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has_stab = out.published.iter().any(|p| matches!(p, StepParameter::UsingStab(true)));
        assert!(has_stab);
    }
}
