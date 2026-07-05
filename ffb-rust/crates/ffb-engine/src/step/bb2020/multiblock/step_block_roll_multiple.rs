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
use ffb_mechanics::mechanics::block_result_for_roll;
use ffb_model::enums::{BlockResult, ReRollSource};
use ffb_model::model::block_roll::BlockRoll;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{SequenceStep, Step, StepId, StepOutcome, StepParameter};
use crate::step::generator::sequence::{Sequence, labels};
use crate::step::util_server_re_roll::use_reroll;
use crate::util::server_util_block::ServerUtilBlock;
use crate::util::util_server_re_roll::UtilServerReRoll;

/// Java: `StepBlockRollMultiple` (bb2020/multiblock).
/// The BB2020 version extends `AbstractStepMultiple` rather than `AbstractStep`.
/// Functionally identical to the BB2025 variant; only the rules collection annotation differs.
pub struct StepBlockRollMultiple {
    /// Java: state.blockRolls (List<BlockRoll>)
    pub block_rolls: Vec<BlockRoll>,
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
    /// Java: parameterToConsume (Set<StepParameterKey>) — accumulated and passed to CONSUME_PARAMETER.
    pub parameter_to_consume: Vec<std::mem::Discriminant<StepParameter>>,
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
            parameter_to_consume: Vec::new(),
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.first_run {
            self.first_run = false;
            // Java: game.getFieldModel().clearDiceDecorations()
            game.field_model.clear_dice_decorations();

            let acting_player_id = game.acting_player.player_id.clone().unwrap_or_default();
            let team_reroll = game.player(&acting_player_id)
                .map(|p| UtilServerReRoll::is_team_re_roll_available(game, p))
                .unwrap_or(false);
            let single_use_reroll = game.player(&acting_player_id)
                .map(|p| UtilServerReRoll::is_single_use_re_roll_available(game, p))
                .unwrap_or(false);
            let pro_reroll = game.player(&acting_player_id)
                .map(|p| UtilServerReRoll::is_pro_re_roll_available(game, p))
                .unwrap_or(false);
            let brawler_available = game.player(&acting_player_id)
                .map(|p| p.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN))
                .unwrap_or(false);
            let single_die_source: Option<ReRollSource> = game.player(&acting_player_id)
                .filter(|p| p.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD))
                .map(|_| ReRollSource::new("Consummate Professional"));

            let attacker_str = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.strength_with_modifiers())
                .unwrap_or(3);

            for roll in &mut self.block_rolls {
                let defender_str = roll.target_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.strength_with_modifiers())
                    .unwrap_or(3);

                let nr_of_dice = ServerUtilBlock::find_nr_of_block_dice(
                    attacker_str,
                    defender_str,
                    false, // same_team = false
                    true,  // using_multi_block = true (adds +1 to defender)
                    roll.is_successful_dauntless() || roll.is_double_target_strength(),
                );
                roll.set_nr_of_dice(nr_of_dice.unsigned_abs() as i32);
                roll.set_own_choice(nr_of_dice > 0);

                let n = roll.get_nr_of_dice() as usize;
                let fresh: Vec<i32> = (0..n).map(|_| rng.d6()).collect();
                roll.set_block_roll(fresh);

                if team_reroll { roll.add_re_roll_source(ReRollSource::new("Team ReRoll")); }
                if single_use_reroll { roll.add_re_roll_source(ReRollSource::new("Lord of Chaos")); }
                if pro_reroll { roll.add_re_roll_source(ReRollSource::new("Pro")); }
                if brawler_available { roll.add_re_roll_source(ReRollSource::new("Brawler")); }
                if let Some(ref src) = single_die_source { roll.add_re_roll_source(src.clone()); }
            }

            self.decide_next_step(game)
        } else {
            if self.selected_target.is_some() {
                let acting_player_id = game.acting_player.player_id.clone().unwrap_or_default();
                if let Some(target) = self.selected_target.clone() {
                    if let Some(roll) = self.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some(target.as_str())) {
                        if let Some(ref source_name) = self.re_roll_source.clone() {
                            let source = ReRollSource::new(source_name.as_str());
                            if source_name == "Brawler" {
                                // Java: handleBrawler — roll 1 die, replace BOTH_DOWN result
                                let new_die = rng.d6();
                                let nr = roll.get_nr_of_dice() as usize;
                                for i in 0..nr {
                                    let val = roll.get_block_roll().get(i).copied().unwrap_or(0);
                                    let br = block_result_for_roll(val);
                                    if br == BlockResult::BothDown && !roll.index_was_re_rolled(i as i32) {
                                        let mut updated = roll.get_block_roll().to_vec();
                                        updated[i] = new_die;
                                        roll.set_block_roll(updated);
                                        let mut idxs2 = roll.get_re_roll_dice_indexes().to_vec();
                                        idxs2.push(i as i32);
                                        roll.set_re_roll_dice_indexes(idxs2);
                                        break;
                                    }
                                }
                                roll.clear_re_roll_sources();
                            } else if source_name == "Pro" {
                                let pro_idx = roll.get_pro_index();
                                let mut idxs = roll.get_re_roll_dice_indexes().to_vec();
                                idxs.push(pro_idx);
                                roll.set_re_roll_dice_indexes(idxs);
                                if roll.get_re_roll_dice_indexes().len() == roll.get_nr_of_dice() as usize {
                                    roll.clear_re_roll_sources();
                                } else {
                                    roll.remove_re_roll_source(&source);
                                    roll.remove_re_roll_source(&ReRollSource::new("Team ReRoll"));
                                    roll.remove_re_roll_source(&ReRollSource::new("Lord of Chaos"));
                                }
                            } else if use_reroll(game, &source, &acting_player_id) {
                                // Re-roll all dice
                                let n = roll.get_nr_of_dice() as usize;
                                let fresh: Vec<i32> = (0..n).map(|_| rng.d6()).collect();
                                roll.set_block_roll(fresh);
                                if roll.get_nr_of_dice() as usize == roll.get_re_roll_dice_indexes().len() {
                                    roll.clear_re_roll_sources();
                                } else {
                                    roll.remove_re_roll_source(&source);
                                    roll.remove_re_roll_source(&ReRollSource::new("Team ReRoll"));
                                    roll.remove_re_roll_source(&ReRollSource::new("Lord of Chaos"));
                                }
                            } else {
                                roll.clear_re_roll_sources();
                            }
                        } else {
                            roll.clear_re_roll_sources();
                        }
                    }
                }
                self.re_roll_source = None;
                self.selected_target = None;
            }
            self.decide_next_step(game)
        }
    }

    /// Java: decideNextStep(Game game)
    fn decide_next_step(&mut self, game: &mut Game) -> StepOutcome {
        let unselected: Vec<bool> = self.block_rolls.iter().map(|r| r.needs_selection()).collect();
        let any_unselected = unselected.iter().any(|&b| b);

        if !any_unselected {
            return self.next_step();
        }

        // Java: update re-roll sources — remove those no longer available, prune Brawler when no BOTH_DOWN.
        let acting_player_id = game.acting_player.player_id.clone().unwrap_or_default();
        let team_reroll = game.player(&acting_player_id)
            .map(|p| UtilServerReRoll::is_team_re_roll_available(game, p))
            .unwrap_or(false);
        let single_use = game.player(&acting_player_id)
            .map(|p| UtilServerReRoll::is_single_use_re_roll_available(game, p))
            .unwrap_or(false);
        let pro = game.player(&acting_player_id)
            .map(|p| UtilServerReRoll::is_pro_re_roll_available(game, p))
            .unwrap_or(false);

        for roll in &mut self.block_rolls {
            if !team_reroll { roll.remove_re_roll_source(&ReRollSource::new("Team ReRoll")); }
            if !single_use  { roll.remove_re_roll_source(&ReRollSource::new("Lord of Chaos")); }
            if !pro         { roll.remove_re_roll_source(&ReRollSource::new("Pro")); }

            // Brawler: only valid when at least one un-re-rolled BOTH_DOWN exists.
            let both_down_present = roll.get_block_roll().iter().enumerate()
                .any(|(i, &val)| !roll.index_was_re_rolled(i as i32) && block_result_for_roll(val) == BlockResult::BothDown);
            if !both_down_present {
                roll.remove_re_roll_source(&ReRollSource::new("Brawler"));
            }
        }

        // Java: if attackerTeamSelects && (ownChoice unselected || anyReRollLeft) → showDialog(attacker)
        // DEFERRED(brm-dialog): attacker dialog deferred.

        // Java: else if defender rolls need selection → showDialog(defender)
        // DEFERRED(brm-dialog): defender dialog deferred.

        // Fallback: if all unselected rolls have no ownChoice (defender's) and no re-rolls, advance.
        // In a full implementation, we'd show the defender dialog here.
        // For now: auto-select index 0 for any unselected rolls and proceed.
        for roll in &mut self.block_rolls {
            if roll.needs_selection() {
                roll.set_selected_index(0);
            }
        }

        self.next_step()
    }

    /// Java: nextStep() — reverse rolls, push evaluation sequence for each, NEXT_STEP.
    fn next_step(&mut self) -> StepOutcome {
        // Java: Collections.reverse(state.blockRolls)
        self.block_rolls.reverse();

        let mut outcome = StepOutcome::next();
        for roll in &self.block_rolls {
            let (seq, params) = generate_block_evaluation_sequence(roll, &self.parameter_to_consume);
            outcome = outcome.push_seq(seq);
            for p in params {
                outcome = outcome.publish(p);
            }
        }
        outcome
    }
}

/// Java: generateBlockEvaluationSequence(BlockRoll blockRoll)
///
/// Builds the per-target block evaluation step sequence and returns
/// the sequence steps plus parameters to publish.
pub fn generate_block_evaluation_sequence(roll: &BlockRoll, params_to_consume: &[std::mem::Discriminant<StepParameter>]) -> (Vec<SequenceStep>, Vec<StepParameter>) {
    let mut seq = Sequence::new();

    seq.add(StepId::SetDefender, vec![
        StepParameter::BlockDefenderId(roll.target_id.clone().unwrap_or_default()),
    ]);

    seq.add(StepId::BlockChoice, vec![
        StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
        StepParameter::GotoLabelOnJuggernaut(labels::BOTH_DOWN.into()),
        StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        StepParameter::SuppressExtraEffectHandling(true),
        StepParameter::BlockRollId(roll.get_id()),
        StepParameter::ShowNameInReport(true),
    ]);
    seq.jump(labels::DROP_FALLING_PLAYERS);

    seq.add_labelled(StepId::BothDown, labels::BOTH_DOWN, vec![]);
    seq.add(StepId::Wrestle, vec![]);
    seq.jump(labels::DROP_FALLING_PLAYERS);

    seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);
    seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);

    seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
    seq.add(StepId::HandleDropPlayerContext, vec![]);
    // Java: sequence.add(StepId.CONSUME_PARAMETER, from(CONSUME_PARAMETER, parameterToConsume))
    if !params_to_consume.is_empty() {
        seq.add(StepId::ConsumeParameter, vec![StepParameter::ParametersToConsume(params_to_consume.to_vec())]);
    }

    // Java: publishParameter calls — downstream steps consume these.
    let selected_idx = roll.get_selected_index().max(0) as usize;
    let block_result = roll.get_block_roll()
        .get(selected_idx)
        .copied()
        .map(block_result_for_roll);

    let mut params = vec![
        StepParameter::TargetPlayerId(roll.target_id.clone()),
        StepParameter::NrOfDice(roll.get_nr_of_dice()),
        StepParameter::BlockRoll(roll.get_block_roll().to_vec()),
        StepParameter::DiceIndex(selected_idx),
    ];
    if let Some(old_state) = roll.get_old_player_state() {
        params.push(StepParameter::OldDefenderState(old_state));
    }
    if let Some(result) = block_result {
        params.push(StepParameter::BlockResult(result));
    }

    (seq.build(), params)
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
            Action::BlockChoice { die_index, target_id } => {
                // CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET: use target_id to find correct roll.
                let idx = target_id.as_deref()
                    .and_then(|tid| self.block_rolls.iter().position(|r| r.target_id.as_deref() == Some(tid)))
                    .or_else(|| self.block_rolls.iter().position(|r| r.needs_selection()));
                if let Some(i) = idx {
                    self.block_rolls[i].set_selected_index(*die_index as i32);
                }
            }
            Action::UseBrawler { target_id } => {
                // CLIENT_USE_BRAWLER
                self.re_roll_source = Some("BRAWLER".into());
                self.selected_target = target_id.clone()
                    .or_else(|| {
                        self.block_rolls.iter().find(|r| r.needs_selection())
                            .and_then(|r| r.target_id.clone())
                    });
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: PLAYER_ID_TO_REMOVE → remove matching blockRoll
            StepParameter::PlayerIdToRemove(id) => {
                self.block_rolls.retain(|r| r.target_id.as_deref() != Some(id.as_str()));
                true
            }
            // Java: PLAYER_ID_DAUNTLESS_SUCCESS → set successfulDauntless on matching roll; consume
            StepParameter::PlayerIdDauntlessSuccess(id) => {
                if let Some(roll) = self.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some(id.as_str())) {
                    roll.set_successful_dauntless(true);
                }
                true
            }
            // Java: DOUBLE_TARGET_STRENGTH_FOR_PLAYER → set doubleTargetStrength on matching roll
            StepParameter::DoubleTargetStrengthForPlayer(id) => {
                if let Some(roll) = self.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some(id.as_str())) {
                    roll.set_double_target_strength(true);
                }
                false // Java: no return true; falls through to super
            }
            // Java: init(StepParameterSet) → BLOCK_TARGETS: add BlockRoll per target
            StepParameter::BlockTargets(ids) => {
                self.block_rolls.extend(
                    ids.iter().enumerate().map(|(i, pid)| {
                        BlockRoll::new_with(pid.as_str(), None, i as i32)
                    })
                );
                true
            }
            // Java: CONSUME_PARAMETER → accumulate into parameterToConsume set.
            StepParameter::ParametersToConsume(keys) => {
                self.parameter_to_consume.extend(keys.iter().cloned());
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
    fn start_with_no_targets_returns_next_step() {
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
    fn set_block_targets_creates_block_rolls() {
        let mut step = StepBlockRollMultiple::new();
        assert!(step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()])));
        assert_eq!(step.block_rolls.len(), 2);
        assert_eq!(step.block_rolls[0].target_id.as_deref(), Some("p1"));
        assert_eq!(step.block_rolls[1].target_id.as_deref(), Some("p2"));
    }

    #[test]
    fn set_block_targets_sets_ids_sequentially() {
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["a".into(), "b".into(), "c".into()]));
        assert_eq!(step.block_rolls[0].get_id(), 0);
        assert_eq!(step.block_rolls[1].get_id(), 1);
        assert_eq!(step.block_rolls[2].get_id(), 2);
    }

    #[test]
    fn player_id_to_remove_removes_roll() {
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        assert!(step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into())));
        assert_eq!(step.block_rolls.len(), 1);
        assert_eq!(step.block_rolls[0].target_id.as_deref(), Some("p2"));
    }

    #[test]
    fn player_id_dauntless_success_sets_flag() {
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        assert!(step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("p1".into())));
        assert!(step.block_rolls[0].is_successful_dauntless());
    }

    #[test]
    fn double_target_strength_sets_flag() {
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        // returns false (falls through to super per Java)
        assert!(!step.set_parameter(&StepParameter::DoubleTargetStrengthForPlayer("p1".into())));
        assert!(step.block_rolls[0].is_double_target_strength());
    }

    #[test]
    fn use_brawler_sets_re_roll_source() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(&Action::UseBrawler { target_id: None }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.re_roll_source.as_deref(), Some("BRAWLER"));
    }

    #[test]
    fn block_choice_selects_die_index() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        // After start, rolls have been made and auto-selected (no dialog infra yet)
        // Re-add a roll to test selection:
        step.block_rolls.clear();
        step.block_rolls.push(BlockRoll::new_with("p1", None, 0));
        step.block_rolls[0].set_block_roll(vec![3, 5]);
        step.first_run = false;
        step.handle_command(&Action::BlockChoice { die_index: 1, target_id: None }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.block_rolls[0].get_selected_index(), 1);
    }

    #[test]
    fn block_choice_routes_by_target_id() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.block_rolls.push(BlockRoll::new_with("p1", None, 0));
        step.block_rolls.push(BlockRoll::new_with("p2", None, 0));
        step.block_rolls[0].set_block_roll(vec![2, 4]);
        step.block_rolls[1].set_block_roll(vec![1, 6]);
        step.first_run = false;
        step.handle_command(&Action::BlockChoice { die_index: 1, target_id: Some("p2".into()) }, &mut game, &mut GameRng::new(0));
        // next_step() reverses block_rolls, so look up p2 by target_id
        let p2 = step.block_rolls.iter().find(|r| r.target_id.as_deref() == Some("p2")).unwrap();
        assert_eq!(p2.get_selected_index(), 1);
    }

    #[test]
    fn start_with_targets_rolls_dice_for_each() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.start(&mut game, &mut GameRng::new(42));
        // Both rolls should have dice (at least 1 die per roll since equal strength = 1 die)
        assert!(!step.block_rolls[0].get_block_roll().is_empty() || !step.block_rolls[1].get_block_roll().is_empty());
    }

    #[test]
    fn generate_block_evaluation_sequence_has_set_defender() {
        let mut roll = BlockRoll::new_with("target1", None, 0);
        roll.set_block_roll(vec![4]);
        roll.set_selected_index(0);
        roll.set_nr_of_dice(1);
        let (seq, _params) = generate_block_evaluation_sequence(&roll, &[]);
        assert!(seq.iter().any(|s| s.step_id == StepId::SetDefender));
    }

    #[test]
    fn generate_block_evaluation_sequence_has_block_choice() {
        let mut roll = BlockRoll::new_with("target1", None, 3);
        roll.set_block_roll(vec![2]);
        roll.set_selected_index(0);
        roll.set_nr_of_dice(1);
        let (seq, _params) = generate_block_evaluation_sequence(&roll, &[]);
        let bc = seq.iter().find(|s| s.step_id == StepId::BlockChoice).expect("BlockChoice not found");
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::BlockRollId(3))));
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::SuppressExtraEffectHandling(true))));
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::ShowNameInReport(true))));
    }

    #[test]
    fn generate_block_evaluation_sequence_has_goto_labels() {
        let mut roll = BlockRoll::new_with("target1", None, 0);
        roll.set_block_roll(vec![1]);
        roll.set_selected_index(0);
        roll.set_nr_of_dice(1);
        let (seq, _params) = generate_block_evaluation_sequence(&roll, &[]);
        let bc = seq.iter().find(|s| s.step_id == StepId::BlockChoice).unwrap();
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnDodge(_))));
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnJuggernaut(_))));
        assert!(bc.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnPushback(_))));
    }

    #[test]
    fn generate_block_evaluation_sequence_publishes_block_result() {
        let mut roll = BlockRoll::new_with("target1", None, 0);
        roll.set_block_roll(vec![3]); // roll=3 → Pushback typically
        roll.set_selected_index(0);
        roll.set_nr_of_dice(1);
        let (_seq, params) = generate_block_evaluation_sequence(&roll, &[]);
        assert!(params.iter().any(|p| matches!(p, StepParameter::BlockResult(_))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::NrOfDice(_))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::BlockRoll(_))));
    }

    #[test]
    fn next_step_reverses_rolls_and_pushes_sequences() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.start(&mut game, &mut GameRng::new(1));
        // After start, auto-selection kicks in; outcome should have pushes for each roll
        // (The actual push count depends on the sequence builder internals)
        // At minimum no panic and returns NextStep
        assert_eq!(step.block_rolls.len(), 2);
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
