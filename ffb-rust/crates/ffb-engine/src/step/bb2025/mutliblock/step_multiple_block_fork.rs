use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId};

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
/// Unported utilities:
///   TODO: Sequence builder (Sequence.add / gameState.stepStack.push)
///   TODO: BlockTarget type (currently stored as plain String player IDs)
///   TODO: IStepLabel.DROP_FALLING_PLAYERS constant
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
        // DEFERRED: build and push per-target sequence:
        //   sequence.add(DAUNTLESS_MULTIPLE, BLOCK_TARGETS=targets)
        //   sequence.add(DOUBLE_STRENGTH)
        //   for target in targets:
        //     sequence.add(SET_DEFENDER, BLOCK_DEFENDER_ID=target)
        //     sequence.add(TRICKSTER)
        //     sequence.add(PICK_UP, GOTO_LABEL_ON_FAILURE=DROP_FALLING_PLAYERS)
        //     sequence.add(CATCH_SCATTER_THROW_IN)
        //   sequence.add(BLOCK_ROLL_MULTIPLE, BLOCK_TARGETS=targets, CONSUME_PARAMETER=...)
        //   gameState.stepStack.push(sequence)
        StepOutcome::next()
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
    fn start_with_targets_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMultipleBlockFork::new(vec!["p1".into(), "p2".into()]);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
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
