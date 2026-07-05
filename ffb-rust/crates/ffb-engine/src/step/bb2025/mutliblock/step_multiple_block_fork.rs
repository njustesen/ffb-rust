use ffb_model::enums::{BlockResult, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{SequenceStep, Step, StepOutcome, StepParameter};
use crate::step::framework::{StepAction, StepId};

/// Java: parameterToConsume fixed set for bb2025 (no UsingStab — stab not in multiple block).
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
    /// Java: targets (List<BlockTarget>) — stored as player IDs until BlockTarget is ported
    pub targets: Vec<String>,
}

impl StepMultipleBlockFork {
    pub fn new(targets: Vec<String>) -> Self {
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
}

impl StepMultipleBlockFork {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let mut seq: Vec<SequenceStep> = Vec::new();

        seq.push(SequenceStep::with_params(
            StepId::DauntlessMultiple,
            vec![StepParameter::BlockTargets(self.targets.clone())],
        ));
        seq.push(SequenceStep::new(StepId::DoubleStrength));

        for target in &self.targets {
            seq.push(SequenceStep::with_params(
                StepId::SetDefender,
                vec![StepParameter::BlockDefenderId(target.clone())],
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
        assert_eq!(step.targets[0], "a");
    }

    #[test]
    fn default_empty_targets() {
        let step = StepMultipleBlockFork::default();
        assert!(step.targets.is_empty());
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new(vec!["p1".into()]);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
