use ffb_model::enums::{BlockResult, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{SequenceStep, Step, StepOutcome, StepParameter};
use crate::step::framework::{StepAction, StepId};

/// Java: parameterToConsume fixed set for bb2025 (no UsingStab — stab not in multiple block).
/// Java: `parameterToConsume`. BB2020 adds USING_STAB (its fork has a STAB group); BB2025 does
/// not, because BB2025 has no stab in a multiple block. Everything else is identical.
fn params_to_consume_for(rules: ffb_model::enums::Rules) -> Vec<std::mem::Discriminant<StepParameter>> {
    let mut v = params_to_consume();
    if rules == ffb_model::enums::Rules::Bb2020 {
        v.push(std::mem::discriminant(&StepParameter::UsingStab(false)));
    }
    v
}

fn params_to_consume() -> Vec<std::mem::Discriminant<StepParameter>> {
    vec![
        std::mem::discriminant(&StepParameter::BlockRoll(vec![])),
        std::mem::discriminant(&StepParameter::BlockResult(BlockResult::Pushback)),
        std::mem::discriminant(&StepParameter::DiceIndex(0)),
        std::mem::discriminant(&StepParameter::NrOfDice(0)),
        std::mem::discriminant(&StepParameter::StartingPushbackSquare(None)),
        std::mem::discriminant(&StepParameter::DefenderPushed(false)),
        std::mem::discriminant(&StepParameter::FollowupChoice(false)),
        std::mem::discriminant(&StepParameter::OldDefenderState(PlayerState::new(0))),
    ]
}

/// Initial fork step for multiple block setup.
///
/// Java executeStep logic:
///   sequence = new Sequence(gameState)
///   sequence.add(DAUNTLESS_MULTIPLE, BLOCK_TARGETS=targets)
///   sequence.add(DOUBLE_STRENGTH)
///   for each target:
///     sequence.add(SET_DEFENDER, BLOCK_DEFENDER_ID=target.playerId)
///     sequence.add(TRICKSTER)
///     sequence.add(PICK_UP, GOTO_LABEL_ON_FAILURE=DROP_FALLING_PLAYERS)
///     sequence.add(CATCH_SCATTER_THROW_IN)
///   sequence.add(BLOCK_ROLL_MULTIPLE, BLOCK_TARGETS=targets, CONSUME_PARAMETER=parameterToConsume)
///   gameState.stepStack.push(sequence)
///   NEXT_STEP
///
/// setParameter:
///   PLAYER_ID_TO_REMOVE: remove target with matching playerId; consume parameter; return true
///
/// init parameters: BLOCK_TARGETS (List<BlockTarget>).
///
/// Java parameterToConsume (fixed set):
///   BLOCK_ROLL, BLOCK_RESULT, DICE_INDEX, NR_OF_DICE, STARTING_PUSHBACK_SQUARE,
///   DEFENDER_PUSHED, FOLLOWUP_CHOICE, OLD_DEFENDER_STATE
///
/// parameterToConsume is wired: passed to BLOCK_ROLL_MULTIPLE step.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepMultipleBlockFork`.
pub struct StepMultipleBlockFork {
    /// Java: `targets` is `List<BlockTarget>` — each carries its own `BlockKind`. BB2020's fork
    /// groups on that kind to build a STAB bucket; BB2025's (this one) has no stab in a multiple
    /// block, but the kind must survive the parameter either way.
    pub targets: Vec<ffb_model::model::block_target::BlockTarget>,
}

impl StepMultipleBlockFork {
    /// 1:1 port of `bb2020/multiblock/StepMultipleBlockFork.executeStep()`.
    ///
    /// Differences from the BB2025 body, all load-bearing:
    ///   - targets are grouped by `BlockKind`; only the BLOCK group drives the main sequence,
    ///   - the per-target loop has NO `PICK_UP` (that entry is BB2025-only),
    ///   - the STAB group gets one sequence PER TARGET, in reverse order, ending
    ///     `STAB -> HANDLE_DROP_PLAYER_CONTEXT -> REPORT_STAB_INJURY`, publishing USING_STAB.
    ///
    /// `REPORT_STAB_INJURY` hangs off that last branch and has no other route into a game
    /// (docs/DEAD_STEP_INVENTORY.md).
    fn execute_step_bb2020(&self, game: &mut Game) -> StepOutcome {
        use ffb_model::model::block_kind::BlockKind;
        use ffb_model::model::block_target::BlockTarget;

        let kind_of = |t: &BlockTarget| t.get_kind().unwrap_or(BlockKind::BLOCK);
        let block_group: Vec<BlockTarget> =
            self.targets.iter().filter(|t| kind_of(t) == BlockKind::BLOCK).cloned().collect();
        let stab_group: Vec<BlockTarget> =
            self.targets.iter().filter(|t| kind_of(t) == BlockKind::STAB).cloned().collect();

        let mut outcome = StepOutcome::next();

        if !block_group.is_empty() {
            let mut seq: Vec<SequenceStep> = Vec::new();
            seq.push(SequenceStep::with_params(
                StepId::DauntlessMultiple,
                vec![StepParameter::BlockTargets(block_group.clone())],
            ));
            seq.push(SequenceStep::new(StepId::DoubleStrength));
            for target in &block_group {
                let id = match target.get_player_id() { Some(i) => i.clone(), None => continue };
                seq.push(SequenceStep::with_params(
                    StepId::SetDefender,
                    vec![StepParameter::BlockDefenderId(id)],
                ));
                seq.push(SequenceStep::new(StepId::Trickster));
                seq.push(SequenceStep::new(StepId::CatchScatterThrowIn));
            }
            seq.push(SequenceStep::with_params(
                StepId::BlockRollMultiple,
                vec![
                    StepParameter::BlockTargets(block_group.clone()),
                    StepParameter::ParametersToConsume(params_to_consume_for(game.rules)),
                ],
            ));
            outcome = outcome.push_seq(seq);
        }

        // Java: Collections.reverse(stabGroup); then one pushed sequence per target.
        if !stab_group.is_empty() {
            for target in stab_group.iter().rev() {
                let id = match target.get_player_id() { Some(i) => i.clone(), None => continue };
                let seq = vec![
                    SequenceStep::with_params(
                        StepId::SetDefender,
                        vec![StepParameter::BlockDefenderId(id.clone())],
                    ),
                    SequenceStep::new(StepId::Trickster),
                    SequenceStep::new(StepId::CatchScatterThrowIn),
                    SequenceStep::with_params(
                        StepId::Stab,
                        vec![StepParameter::GotoLabelOnSuccess("NEXT".into())],
                    ),
                    SequenceStep::new(StepId::HandleDropPlayerContext),
                    SequenceStep {
                        step_id: StepId::ReportStabInjury,
                        label: Some("NEXT".into()),
                        params: vec![StepParameter::PlayerId(id.clone())],
                    },
                    SequenceStep::with_params(
                        StepId::ConsumeParameter,
                        vec![StepParameter::ParametersToConsume(params_to_consume_for(game.rules))],
                    ),
                ];
                outcome = outcome.push_seq(seq);
                if let Some(state) = target.get_original_player_state() {
                    outcome = outcome.publish(StepParameter::OldDefenderState(state));
                }
                outcome = outcome.publish(StepParameter::UsingStab(true));
            }
        }

        outcome
    }


    pub fn new(targets: Vec<ffb_model::model::block_target::BlockTarget>) -> Self {
        Self { targets }
    }
}

impl Default for StepMultipleBlockFork {
    fn default() -> Self { Self::new(Vec::new()) }
}

impl Step for StepMultipleBlockFork {
    fn id(&self) -> StepId { StepId::MultipleBlockFork }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java init(): BLOCK_TARGETS → targets.addAll(...)
            StepParameter::BlockTargets(v) => { self.targets.extend(v.iter().cloned()); true }
            // Java setParameter: PLAYER_ID_TO_REMOVE → remove the matching target
            // (consume() is mirrored separately via consumes_parameter below).
            StepParameter::PlayerIdToRemove(v) => {
                if let Some(pos) = self.targets.iter().position(|t| t.get_player_id().map(|p| p == v).unwrap_or(false)) {
                    self.targets.remove(pos);
                }
                true
            }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::PlayerIdToRemove(_))
    }
}

impl StepMultipleBlockFork {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // BB2020's fork groups targets by BlockKind and gives the STAB group its own per-target
        // sequences; BB2025's has no stab in a multiple block and runs every target through the
        // block path. Edition-gate INSIDE this shared step - the bb2020 twin is dead code
        // (driver.rs globs bb2025::mutliblock::*), so routing to it would change nothing.
        if game.rules == ffb_model::enums::Rules::Bb2020 {
            return self.execute_step_bb2020(game);
        }
        let mut seq: Vec<SequenceStep> = Vec::new();

        seq.push(SequenceStep::with_params(
            StepId::DauntlessMultiple,
            vec![StepParameter::BlockTargets(self.targets.clone())],
        ));
        seq.push(SequenceStep::new(StepId::DoubleStrength));

        for target in &self.targets {
            let target_id = match target.get_player_id() { Some(id) => id.clone(), None => continue };
            seq.push(SequenceStep::with_params(
                StepId::SetDefender,
                vec![StepParameter::BlockDefenderId(target_id)],
            ));
            seq.push(SequenceStep::new(StepId::Trickster));
            seq.push(SequenceStep::with_params(
                StepId::PickUp,
                vec![StepParameter::GotoLabelOnFailure("DROP_FALLING_PLAYERS".into())],
            ));
            seq.push(SequenceStep::new(StepId::CatchScatterThrowIn));
        }

        seq.push(SequenceStep::with_params(
            StepId::BlockRollMultiple,
            vec![
                StepParameter::BlockTargets(self.targets.clone()),
                StepParameter::ParametersToConsume(params_to_consume()),
            ],
        ));

        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn bb2020_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    /// BB2020's fork gives the STAB group its own sequence per target, ending in
    /// REPORT_STAB_INJURY - the only route that step has into a game. Rust routed every edition
    /// through the BB2025 body, which has no stab group at all, so BB2020 silently lost it.
    #[test]
    fn bb2020_stab_target_gets_a_report_stab_injury_sequence() {
        use ffb_model::model::block_kind::BlockKind;
        use ffb_model::model::block_target::BlockTarget;

        let mut game = bb2020_game();
        let step = StepMultipleBlockFork::new(vec![
            BlockTarget::block("blocked"),
            BlockTarget::new("stabbed", BlockKind::STAB, None),
        ]);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));

        let stab_seq = out.pushes.iter()
            .find(|s| s.iter().any(|e| e.step_id == StepId::ReportStabInjury))
            .expect("the STAB target must get a ReportStabInjury sequence");
        let ids: Vec<_> = stab_seq.iter().map(|e| e.step_id).collect();
        assert_eq!(ids, vec![
            StepId::SetDefender, StepId::Trickster, StepId::CatchScatterThrowIn,
            StepId::Stab, StepId::HandleDropPlayerContext, StepId::ReportStabInjury,
            StepId::ConsumeParameter,
        ], "Java bb2020 order");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingStab(true))));

        // The BLOCK target still drives the main sequence, and it has NO PickUp in BB2020.
        let block_seq = out.pushes.iter()
            .find(|s| s.iter().any(|e| e.step_id == StepId::BlockRollMultiple))
            .expect("the BLOCK group must still drive a block sequence");
        assert!(!block_seq.iter().any(|e| e.step_id == StepId::PickUp),
            "PICK_UP is BB2025-only; bb2020/Block fork has none");
    }

    /// BB2025 is unchanged: no grouping, no stab branch, and it KEEPS its PickUp. This pins the
    /// edition split rather than one side of it.
    #[test]
    fn bb2025_has_no_stab_branch_and_keeps_pick_up() {
        use ffb_model::model::block_kind::BlockKind;
        use ffb_model::model::block_target::BlockTarget;

        let mut game = make_game();
        let step = StepMultipleBlockFork::new(vec![
            BlockTarget::block("a"),
            BlockTarget::new("b", BlockKind::STAB, None),
        ]);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));

        assert!(!out.pushes.iter().flatten().any(|e| e.step_id == StepId::ReportStabInjury),
            "BB2025 has no stab in a multiple block");
        assert!(out.pushes.iter().flatten().any(|e| e.step_id == StepId::PickUp),
            "BB2025 keeps its PICK_UP");
    }

    #[test]
    fn start_no_targets_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::default();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_with_targets_pushes_sequence() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new(vec!["p1".into(), "p2".into()]);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1, "should push one sequence");
        // Sequence: DauntlessMultiple + DoubleStrength + 4 steps per target + BlockRollMultiple
        // = 1 + 1 + 2*4 + 1 = 11 steps
        assert_eq!(out.pushes[0].len(), 11);
    }

    #[test]
    fn new_stores_targets() {
        let step = StepMultipleBlockFork::new(vec!["a".into(), "b".into()]);
        assert_eq!(step.targets.len(), 2);
        assert_eq!(step.targets[0].get_player_id().map(|p| p.as_str()), Some("a"));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new(vec!["p1".into()]);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
