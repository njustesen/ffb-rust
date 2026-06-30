use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId};

/// Rolls all block dice for multiple simultaneous blocks and manages the
/// re-roll / die-selection dialogs for each target.
///
/// Java first-run path (executeStep when state.firstRun):
///   state.firstRun = false
///   clearDiceDecorations
///   for each blockRoll in state.blockRolls:
///     nrOfDice = ServerUtilBlock.findNrOfBlockDice(...)
///     roll.setNrOfDice(abs(nrOfDice)); roll.setOwnChoice(nrOfDice > 0)
///     roll(roll, false, actingPlayer, singleDieReRollSource) [roll block dice]
///     addReRollData(teamReRollAvailable, mascotAvailable, mechanic, roll)
///     setSound(BLOCK); removeAdditionalAssist; syncGameModel
///   decideNextStep
///
/// Java second-run path (executeStep after command):
///   if selectedTarget is set:
///     find roll for selectedTarget
///     if reRollSource set: apply re-roll (Brawler / Hatred / general)
///     addReport(ReportBlock); setSound(BLOCK)
///     clear re-rolls; addReRollData for remaining rolls
///   decideNextStep
///
/// decideNextStep:
///   unselected = rolls that still need selection
///   if empty -> nextStep (push block evaluation sequences for each roll in reverse)
///   else if attackerTeamSelects && (any with ownChoice || anyReRollLeft):
///     showDialog(DialogReRollBlockForTargetsPropertiesParameter) [attacker]
///   else if defender has unselected rolls:
///     showDialog(DialogOpponentBlockSelectionPropertiesParameter) [defender]
///   else -> nextStep
///
/// nextStep:
///   reverse blockRolls; for each: generateBlockEvaluationSequence (push to stack)
///   NEXT_STEP
///
/// Unported utilities:
///   TODO: ServerUtilBlock.findNrOfBlockDice
///   TODO: BlockRollProperties (full type with nrOfDice, blockRoll, reRolls, selection index)
///   TODO: UtilServerReRoll.isTeamReRollAvailable / useReRoll / askForReRollIfAvailable
///   TODO: InducementSet.forUsage(CONDITIONAL_REROLL) / mascot check
///   TODO: Dialog parameters (DialogReRollBlockForTargetsPropertiesParameter / DialogOpponentBlockSelectionPropertiesParameter)
///   TODO: generateBlockEvaluationSequence (pushes per-target block resolution sequence)
///   TODO: ReRollSources (BRAWLER, HATRED, PRO, etc.)
///   TODO: handleImplicitReRollIndex (Brawler / Hatred single-die re-roll)
///   TODO: adjustRollForIndexedReRoll (Pro / Tandem / Lord of Chaos / etc.)
///   TODO: UtilCards.getUnusedRerollSource
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepBlockRollMultiple`.
pub struct StepBlockRollMultiple {
    /// Java: state.blockRolls (List<BlockRollProperties>) — stored as target player IDs
    pub block_rolls: Vec<String>,
    /// Java: state.firstRun (init true)
    pub first_run: bool,
    /// Java: state.attackerTeamSelects (init true)
    pub attacker_team_selects: bool,
    /// Java: state.reRollSource — stored as name
    pub re_roll_source: Option<String>,
    /// Java: state.selectedTarget
    pub selected_target: Option<String>,
    /// Java: state.playerIdForSingleUseReRoll
    pub player_id_for_single_use_re_roll: Option<String>,
}

impl StepBlockRollMultiple {
    pub fn new() -> Self {
        Self {
            block_rolls: Vec::new(),
            first_run: true,
            attacker_team_selects: true,
            re_roll_source: None,
            selected_target: None,
            player_id_for_single_use_re_roll: None,
        }
    }
}

impl Default for StepBlockRollMultiple {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockRollMultiple {
    fn id(&self) -> StepId { StepId::BlockRollMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java handles:
        //   CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET -> selectedTarget + roll indexes -> execute
        //   CLIENT_USE_BRAWLER -> reRollSource=BRAWLER; selectedTarget -> execute
        //   CLIENT_USE_HATRED -> reRollSource=HATRED; selectedTarget -> execute
        match action {
            Action::BlockChoice { die_index } => {
                // CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET substitute
                // TODO: map die_index to a BlockRollProperties.selectedIndex for the selectedTarget
                let _ = die_index;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }
}

impl StepBlockRollMultiple {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if self.first_run {
            self.first_run = false;
            // TODO: clearDiceDecorations
            // TODO: for each block_roll entry:
            //   nrOfDice = ServerUtilBlock.findNrOfBlockDice(gameState, actingPlayer, defender, ...)
            //   roll block dice; addReRollData; setSound(BLOCK); syncGameModel
            // TODO: decideNextStep -> show dialog or push evaluation sequences
        } else {
            // TODO: if selectedTarget is set: apply re-roll if reRollSource set
            // TODO: decideNextStep
        }
        // Stub: no dice infrastructure ported yet -> advance immediately.
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
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn first_run_is_set_false_after_start() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        assert!(step.first_run);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.first_run);
    }

    #[test]
    fn default_block_rolls_empty() {
        let step = StepBlockRollMultiple::default();
        assert!(step.block_rolls.is_empty());
    }

    #[test]
    fn default_attacker_team_selects_true() {
        let step = StepBlockRollMultiple::default();
        assert!(step.attacker_team_selects);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.start(&mut game, &mut GameRng::new(0)); // consume first_run
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
