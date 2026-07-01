/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.multiblock.StepBlockRollMultiple`.
///
/// Rolls block dice for each target in a multiple-block action and manages the
/// per-target re-roll / die-selection dialogs.
///
/// State (inner `State` / `SingleReRollUseState` in Java):
///   - blockRolls: List<BlockRoll> — one entry per target
///   - firstRun: bool (true initially)
///   - attackerTeamSelects: bool (true initially)
///   - reRollSource: Option<ReRollSource>
///   - selectedTarget: Option<String>
///   - playerIdForSingleUseReRoll: Option<String>
///
/// init parameters:
///   - BLOCK_TARGETS (List<BlockTarget>) → build one BlockRoll per target.
///   - CONSUME_PARAMETER (Set<StepParameterKey>) → parameterToConsume.
///
/// setParameter:
///   - PLAYER_ID_TO_REMOVE: remove matching block roll.
///   - PLAYER_ID_DAUNTLESS_SUCCESS: set successfulDauntless on matching roll; consume.
///   - DOUBLE_TARGET_STRENGTH_FOR_PLAYER: set doubleTargetStrength on matching roll.
///
/// First-run executeStep:
///   clearDiceDecorations; for each roll: findNrOfBlockDice; roll; add re-roll sources;
///   setSound(BLOCK); syncGameModel; → decideNextStep
///
/// Subsequent executeStep (after command):
///   if selectedTarget set: apply re-roll (Brawler / Pro / singleDie / team); → decideNextStep
///
/// decideNextStep:
///   unselected = rolls that still need selection
///   if empty → nextStep (push block evaluation sequences for each roll in reverse; NEXT_STEP)
///   if attackerTeamSelects && (any ownChoice or any re-rolls left):
///     showDialog(DialogReRollBlockForTargetsParameter) → Continue
///   else:
///     show defender opponent selection dialog → Continue (or → nextStep)
///
/// nextStep:
///   for each blockRoll (reversed): generateBlockEvaluationSequence → push to stack
///   NEXT_STEP
///
/// DEFERRED(brm-dice): ServerUtilBlock.findNrOfBlockDice, BlockRoll struct, dice rolling deferred.
/// DEFERRED(brm-reroll): UtilServerReRoll.isTeamReRollAvailable / useReRoll / Brawler handling deferred.
/// DEFERRED(brm-dialog): DialogReRollBlockForTargetsParameter, DialogOpponentBlockSelectionParameter deferred.
/// DEFERRED(brm-sequence): generateBlockEvaluationSequence (push per-target evaluation sequences) deferred.
/// DEFERRED(brm-sound): SoundId.BLOCK deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBlockRollMultiple` (bb2020/multiblock).
/// The BB2020 version extends `AbstractStepMultiple` rather than `AbstractStep`.
/// Functionally identical to the BB2025 variant; only the rules collection annotation differs.
pub struct StepBlockRollMultiple {
    /// Java: state.blockRolls (List<BlockRoll>) — stored as target player IDs.
    pub block_rolls: Vec<String>,
    /// Java: state.firstRun (init true)
    pub first_run: bool,
    /// Java: state.attackerTeamSelects (init true)
    pub attacker_team_selects: bool,
    /// Java: state.reRollSource — stored as name.
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

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if self.first_run {
            self.first_run = false;
            // DEFERRED(brm-dice): clearDiceDecorations
            // DEFERRED(brm-dice): teamReRollAvailable / singleUseReRollAvailable / proReRollAvailable / brawlerAvailable
            // DEFERRED(brm-dice): for each blockRoll entry:
            //   nrOfDice = ServerUtilBlock.findNrOfBlockDice(gameState, actingPlayer, defender, ...)
            //   roll.setNrOfDice(abs(nrOfDice)); roll.setOwnChoice(nrOfDice > 0)
            //   roll(roll, false, actingPlayer, singleDieReRollSource) [roll dice]
            //   add re-roll sources; setSound(BLOCK); syncGameModel
            // DEFERRED(brm-dialog): decideNextStep → show dialog or push evaluation sequences
        } else {
            if self.selected_target.is_some() {
                // DEFERRED(brm-reroll): find roll for selectedTarget
                // DEFERRED(brm-reroll): if reRollSource == BRAWLER: handleBrawler
                //   else if UtilServerReRoll.useReRoll(...): roll(roll, true, ...)
                //   else if PRO: adjustRollForIndexedReRoll(...)
                //   clear or update re-roll sources
                // DEFERRED(brm-dice): addReport(ReportBlock); setSound(BLOCK)
            }
            // DEFERRED(brm-dialog): decideNextStep
        }
        // Stub: advance immediately until dice/dialog infrastructure is ported.
        StepOutcome::next()
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
        //   CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET →
        //     reRollSourceSuccessfully(command.reRollSource) → EXECUTE_STEP
        //     selectedTarget = command.targetId; update roll selectedIndex / proIndex
        //   CLIENT_USE_BRAWLER → reRollSource = BRAWLER; selectedTarget = targetId → EXECUTE_STEP
        match action {
            Action::BlockChoice { die_index } => {
                // Substitute for CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET
                // DEFERRED(brm-command): map die_index to selected roll index for selectedTarget.
                let _ = die_index;
            }
            Action::UseBrawler => {
                // CLIENT_USE_BRAWLER
                self.re_roll_source = Some("BRAWLER".into());
                // DEFERRED(brm-command): extract targetId from context or game state
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: PLAYER_ID_TO_REMOVE → remove matching blockRoll
            StepParameter::PlayerIdToRemove(id) => {
                self.block_rolls.retain(|pid| pid != id);
                true
            }
            // Java: PLAYER_ID_DAUNTLESS_SUCCESS → set successfulDauntless on matching roll; consume
            StepParameter::PlayerIdDauntlessSuccess(id) => {
                // DEFERRED(brm-dauntless): set successfulDauntless flag on BlockRoll when BlockRoll is ported
                let _ = id;
                true
            }
            // Java: DOUBLE_TARGET_STRENGTH_FOR_PLAYER → set doubleTargetStrength on matching roll
            StepParameter::DoubleTargetStrengthForPlayer(id) => {
                // DEFERRED(brm-dauntless): set doubleTargetStrength on BlockRoll when BlockRoll is ported
                let _ = id;
                false // Java: no return true; falls through to super
            }
            // Java: init(StepParameterSet) → BLOCK_TARGETS: add BlockRoll per target
            StepParameter::BlockTargets(ids) => {
                for pid in ids {
                    self.block_rolls.push(pid.clone());
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
    fn id_is_block_roll_multiple() {
        assert_eq!(StepBlockRollMultiple::new().id(), StepId::BlockRollMultiple);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn first_run_cleared_after_start() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        assert!(step.first_run);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.first_run);
    }

    #[test]
    fn default_attacker_team_selects_true() {
        let step = StepBlockRollMultiple::default();
        assert!(step.attacker_team_selects);
    }

    #[test]
    fn set_block_targets_adds_rolls() {
        let mut step = StepBlockRollMultiple::new();
        assert!(step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()])));
        assert_eq!(step.block_rolls.len(), 2);
        assert_eq!(step.block_rolls[0], "p1");
    }

    #[test]
    fn player_id_to_remove_removes_roll() {
        let mut step = StepBlockRollMultiple::new();
        step.block_rolls.push("p1".into());
        step.block_rolls.push("p2".into());
        assert!(step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into())));
        assert_eq!(step.block_rolls.len(), 1);
        assert_eq!(step.block_rolls[0], "p2");
    }

    #[test]
    fn use_brawler_sets_re_roll_source() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.start(&mut game, &mut GameRng::new(0)); // consume first_run
        step.handle_command(&Action::UseBrawler, &mut game, &mut GameRng::new(0));
        assert_eq!(step.re_roll_source.as_deref(), Some("BRAWLER"));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
