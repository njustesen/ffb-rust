/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepBlockRollMultiple`.
///
/// Rolls block dice for each target in a multiple-block action and manages the
/// per-target re-roll / die-selection dialogs.
///
/// Differences from BB2020:
///   - Uses `BlockRollProperties` internally (Java BB2025), modelled with the same `BlockRoll`
///     struct since the fields are identical; `ReRollProperty` enum replaces `ReRollSource` objects.
///   - Supports `CLIENT_USE_HATRED` (Hatred skill re-roll) in addition to Brawler.
///   - Dialog parameters are `DialogReRollBlockForTargetsPropertiesParameter` /
///     `DialogOpponentBlockSelectionPropertiesParameter` instead of the BB2020 variants.
///   - Mascot re-roll (CONDITIONAL_REROLL inducement) is checked.
///   - `addReRollData(game, mechanic, roll)` replaces individual re-roll source adds.
///
/// First-run executeStep:
///   clearDiceDecorations; for each roll: findNrOfBlockDice; roll; addReRollData;
///   setSound(BLOCK); removeAdditionalAssist; syncGameModel → decideNextStep
///
/// Subsequent executeStep (after command):
///   if selectedTarget set: apply re-roll (Brawler / Hatred / general);
///   re-add re-roll data for remaining rolls → decideNextStep
///
/// decideNextStep:
///   unselected = rolls that still need selection
///   if empty → nextStep
///   if attackerTeamSelects && (any ownChoice || anyReRollLeft):
///     showDialog(DialogReRollBlockForTargetsPropertiesParameter) → Continue
///   else if defender has unselected → showDialog(DialogOpponentBlockSelectionPropertiesParameter)
///   else → nextStep
///
/// nextStep:
///   reverse blockRolls; for each: generateBlockEvaluationSequence → push to stack; NEXT_STEP
use ffb_model::enums::{BlockResult, ReRollSource};
use ffb_model::model::block_roll::BlockRoll;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_block_re_roll::ReportBlockReRoll;
use ffb_model::report::report_block::ReportBlock;
use ffb_model::report::report_block_roll::ReportBlockRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};
use crate::step::bb2020::multiblock::step_block_roll_multiple::generate_block_evaluation_sequence;
use crate::step::util_server_re_roll::use_reroll;
use crate::util::server_util_block::ServerUtilBlock;
use crate::util::util_server_re_roll::UtilServerReRoll;
use ffb_mechanics::mechanics::block_result_for_roll;

/// Java: `StepBlockRollMultiple` (bb2025/mutliblock).
pub struct StepBlockRollMultiple {
    /// Java: state.blockRolls (List<BlockRollProperties>)
    pub block_rolls: Vec<BlockRoll>,
    /// Java: state.firstRun (init true)
    pub first_run: bool,
    /// Java: state.attackerTeamSelects (init true)
    pub attacker_team_selects: bool,
    /// Next roll to emit an evaluation sequence for (see next_step's push_self iteration).
    eval_index: usize,
    /// Java: state.reRollSource — stored as name.
    pub re_roll_source: Option<String>,
    /// Java: state.selectedTarget
    pub selected_target: Option<String>,
    /// Java: state.playerIdForSingleUseReRoll
    pub player_id_for_single_use_re_roll: Option<String>,
    /// Java: parameterToConsume — Set<StepParameterKey> of parameters to consume when evaluating.
    pub parameter_to_consume: Vec<std::mem::Discriminant<StepParameter>>,
}

impl StepBlockRollMultiple {
    pub fn new() -> Self {
        Self {
            block_rolls: Vec::new(),
            first_run: true,
            attacker_team_selects: true,
            eval_index: 0,
            re_roll_source: None,
            selected_target: None,
            player_id_for_single_use_re_roll: None,
            parameter_to_consume: Vec::new(),
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.first_run {
            self.first_run = false;
            // Java's BlockRolls carry originalPlayerState from the BlockTarget objects the
            // client selected (ClientCommandSynchronousMultiBlock); Rust's BLOCK_TARGETS param
            // is id-only, so capture each target's pre-block state HERE, before any block dice.
            // Leaving it None made BlockChoice's restore paths write the DEFAULT PlayerState(0)
            // — a pushed-but-standing target vanished into the reserve box (dark_elf bb2020
            // seed 10: both of Horkon's targets went Reserve while Java kept them on-pitch).
            for roll in &mut self.block_rolls {
                if roll.get_old_player_state().is_none() {
                    if let Some(pid) = roll.target_id.clone() {
                        if let Some(st) = game.field_model.player_state(&pid) {
                            roll.old_player_state = Some(st);
                        }
                    }
                }
            }
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
            let hatred_available = game.player(&acting_player_id)
                .map(|p| p.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_SKULL))
                .unwrap_or(false);
            let single_die_source: Option<ReRollSource> = game.player(&acting_player_id)
                .filter(|p| p.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD))
                .map(|_| ReRollSource::new("Consummate Professional"));

            for roll in &mut self.block_rolls {
                // Java :183: ServerUtilBlock.findNrOfBlockDice(gameState, attacker, defender,
                //   true, roll.isSuccessFulDauntless(), roll.isDoubleTargetStrength(), false)
                // The full Java path folds ASSISTS into both totals (getTotalAttackerStrength +
                // findBlockStrength) with the multi-block modifiers applied to base strength;
                // pass using_multi_block=false below so they are not applied twice.
                // (Java's isValidAssist multi-block-target exclusion is a no-op here: the
                // CLIENT_SYNCHRONOUS_MULTI_BLOCK flow never calls addMultiBlockTarget.)
                let nr_of_dice = {
                    let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
                    match (
                        game.player(&acting_player_id).cloned(),
                        roll.target_id.as_deref().and_then(|id| game.player(id).cloned()),
                    ) {
                        (Some(attacker), Some(defender)) => {
                            let defender_strength = defender.strength_with_modifiers()
                                + mechanic.multi_block_defender_modifier();
                            let block_strength_attacker = mechanic.get_total_attacker_strength(
                                game, &attacker, &defender, true,
                                roll.is_successful_dauntless(), roll.is_double_target_strength(),
                                defender_strength);
                            let atk_coord = game.field_model.player_coordinates.get(&acting_player_id).copied();
                            let def_coord = game.field_model.player_coordinates.get(&defender.id).copied();
                            let block_strength_defender = match (def_coord, atk_coord) {
                                (Some(dc), Some(ac)) => crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
                                    game, dc, defender_strength, ac),
                                _ => defender_strength,
                            };
                            let same_team = game.team_home.has_player(&acting_player_id)
                                == game.team_home.has_player(&defender.id);
                            ServerUtilBlock::find_nr_of_block_dice(
                                block_strength_attacker, block_strength_defender,
                                same_team, false, game.rules, false)
                        }
                        _ => 1,
                    }
                };
                roll.set_nr_of_dice(nr_of_dice.unsigned_abs() as i32);
                roll.set_own_choice(nr_of_dice > 0);

                let n = roll.get_nr_of_dice() as usize;
                let fresh: Vec<i32> = (0..n).map(|_| rng.d6()).collect();
                roll.set_block_roll(fresh);

                if team_reroll { roll.add_re_roll_source(ReRollSource::new("Team ReRoll")); }
                if single_use_reroll { roll.add_re_roll_source(ReRollSource::new("Lord of Chaos")); }
                if pro_reroll { roll.add_re_roll_source(ReRollSource::new("Pro")); }
                if brawler_available { roll.add_re_roll_source(ReRollSource::new("Brawler")); }
                if hatred_available { roll.add_re_roll_source(ReRollSource::new("Hatred")); }
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
                            // Java: getResult().addReport(new ReportBlock(defender.getId()))
                            if let Some(tid) = roll.target_id.clone() {
                                game.report_list.add(ReportBlock::new(tid));
                            }
                            let implicit_result = match source_name.as_str() {
                                "Brawler" => Some(BlockResult::BothDown),
                                "Hatred" => Some(BlockResult::Skull),
                                _ => None,
                            };
                            if let Some(result_to_replace) = implicit_result {
                                // Java: handleImplicitReRollIndex → addReport(new ReportBlockReRoll([reRolledDie], player.getId(), source))
                                let new_die = rng.d6();
                                game.report_list.add(ReportBlockReRoll::new(
                                    vec![new_die],
                                    Some(acting_player_id.clone()),
                                    Some(source.clone()),
                                ));
                                let nr = roll.get_nr_of_dice() as usize;
                                for i in 0..nr {
                                    let val = roll.get_block_roll().get(i).copied().unwrap_or(0);
                                    if block_result_for_roll(val) == result_to_replace && !roll.index_was_re_rolled(i as i32) {
                                        let mut updated = roll.get_block_roll().to_vec();
                                        updated[i] = new_die;
                                        roll.set_block_roll(updated);
                                        let mut idxs = roll.get_re_roll_dice_indexes().to_vec();
                                        idxs.push(i as i32);
                                        roll.set_re_roll_dice_indexes(idxs);
                                        break;
                                    }
                                }
                                roll.clear_re_roll_sources();
                            } else if source_name == "Pro" {
                                // Java: adjustRollForIndexedReRoll → addReport(new ReportBlockReRoll([reRolledDie], playerId, source))
                                let new_die = rng.d6();
                                game.report_list.add(ReportBlockReRoll::new(
                                    vec![new_die],
                                    Some(acting_player_id.clone()),
                                    Some(source.clone()),
                                ));
                                let pro_idx = roll.get_pro_index();
                                let old_roll = roll.get_block_roll().to_vec();
                                let mut updated = old_roll;
                                if (pro_idx as usize) < updated.len() { updated[pro_idx as usize] = new_die; }
                                roll.set_block_roll(updated);
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
                            } else if use_reroll(game, &source, &acting_player_id, rng) {
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
                            // Java: getResult().addReport(new ReportBlockRoll(defender.getTeam().getId(), roll.getBlockRoll(), roll.getTargetId()))
                            let team_id = roll.target_id.as_deref()
                                .and_then(|tid| game.player_team_id(tid))
                                .unwrap_or("")
                                .to_string();
                            game.report_list.add(ReportBlockRoll::new(
                                team_id,
                                roll.get_block_roll().to_vec(),
                                roll.target_id.clone(),
                            ));
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
        let any_unselected = self.block_rolls.iter().any(|r| r.needs_selection());

        if !any_unselected {
            return self.next_step();
        }

        // Java: update re-roll sources — remove unavailable, prune Brawler/Hatred when no trigger.
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

            // Brawler: valid only when an un-re-rolled BOTH_DOWN die exists.
            let both_down_present = roll.get_block_roll().iter().enumerate()
                .any(|(i, &val)| !roll.index_was_re_rolled(i as i32) && block_result_for_roll(val) == BlockResult::BothDown);
            if !both_down_present {
                roll.remove_re_roll_source(&ReRollSource::new("Brawler"));
            }

            // BB2025 Hatred: valid only when an un-re-rolled SKULL die exists.
            let skull_present = roll.get_block_roll().iter().enumerate()
                .any(|(i, &val)| !roll.index_was_re_rolled(i as i32) && block_result_for_roll(val) == BlockResult::Skull);
            if !skull_present {
                roll.remove_re_roll_source(&ReRollSource::new("Hatred"));
            }
        }

        // Java: if attackerTeamSelects && (ownChoice unselected || anyReRollLeft)
        //   → showDialog(DialogReRollBlockForTargetsPropertiesParameter)
        // client-only: attacker block-roll dialog — headless auto-selects index 0

        // Java: else if defender rolls need selection
        //   → showDialog(DialogOpponentBlockSelectionPropertiesParameter)
        // client-only: defender block-roll dialog — headless auto-selects index 0

        // Fallback: auto-select index 0 for unselected rolls and proceed.
        for roll in &mut self.block_rolls {
            if roll.needs_selection() {
                roll.set_selected_index(0);
            }
        }

        self.next_step()
    }

    /// Java: nextStep() — reverse rolls, push evaluation sequence for each.
    fn next_step(&mut self) -> StepOutcome {
        // Java: Collections.reverse(state.blockRolls); state.blockRolls.forEach(generate...)
        // — each generate call PUSHES its evaluation sequence and THEN publishes that roll's
        // parameters, so the publish is scoped to the just-pushed sequence (it stops at that
        // sequence's ConsumeParameter). The Rust driver applies pushes before publishes PER
        // OUTCOME, so batching both rolls starved the second sequence: both rolls' publishes
        // stopped at the FIRST sequence's ConsumeParameter and its BlockChoice ended up with
        // the SECOND roll's result while the second BlockChoice got nothing and parked
        // prompt-less (dark_elf bb2020 seed 5, Horkon's first Multiple Block). Emit ONE roll
        // per pass and push_self to resume — the reverse+LIFO in Java makes the ORIGINAL roll
        // order the evaluation order, which this iteration preserves directly.
        if let Some(roll) = self.block_rolls.get(self.eval_index).cloned() {
            self.eval_index += 1;
            let (seq, params) = generate_block_evaluation_sequence(&roll, &self.parameter_to_consume);
            let mut outcome = StepOutcome::next();
            outcome.push_self = true;
            outcome = outcome.push_seq(seq);
            for p in params {
                outcome = outcome.publish(p);
            }
            return outcome;
        }
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
        //   CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET → selectedTarget + roll indexes → execute
        //   CLIENT_USE_BRAWLER → reRollSource=BRAWLER; selectedTarget → execute
        //   CLIENT_USE_HATRED → reRollSource=HATRED; selectedTarget → execute
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
                self.re_roll_source = Some("BRAWLER".into());
                self.selected_target = target_id.clone()
                    .or_else(|| {
                        self.block_rolls.iter().find(|r| r.needs_selection())
                            .and_then(|r| r.target_id.clone())
                    });
            }
            // BB2025 addition: Hatred re-roll
            Action::UseHatred => {
                self.re_roll_source = Some("HATRED".into());
                if let Some(roll) = self.block_rolls.iter().find(|r| r.needs_selection()) {
                    self.selected_target = roll.target_id.clone();
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerIdToRemove(id) => {
                self.block_rolls.retain(|r| r.target_id.as_deref() != Some(id.as_str()));
                true
            }
            StepParameter::PlayerIdDauntlessSuccess(id) => {
                if let Some(roll) = self.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some(id.as_str())) {
                    roll.set_successful_dauntless(true);
                }
                true
            }
            StepParameter::DoubleTargetStrengthForPlayer(id) => {
                if let Some(roll) = self.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some(id.as_str())) {
                    roll.set_double_target_strength(true);
                }
                false
            }
            StepParameter::BlockTargets(ids) => {
                self.block_rolls.extend(
                    ids.iter().enumerate().map(|(i, target)| {
                        BlockRoll::new_with(
                            target.get_player_id().map(|p| p.as_str()).unwrap_or(""),
                            None,
                            i as i32,
                        )
                    })
                );
                true
            }
            StepParameter::ParametersToConsume(keys) => {
                self.parameter_to_consume.extend(keys.iter().cloned());
                true
            }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys (PLAYER_ID_TO_REMOVE and
    // DOUBLE_TARGET_STRENGTH_FOR_PLAYER set state without consuming).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::PlayerIdDauntlessSuccess(_))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, strength: i32) {
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::Player;
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn multi_block_dice_count_folds_assists() {
        // dark_elf bb2020 seed 53 i=8: the roll phase passed RAW strengths to
        // find_nr_of_block_dice, losing every assist — Java folds them via
        // getTotalAttackerStrength + findBlockStrength. ST3 attacker (-2 multi-block)
        // with one offensive assist = 2 vs ST3 defender = -2 dice (defender's choice);
        // the raw-strength path computed 1 vs 3 → -3 → one extra d6 → RNG desync.
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        add_player(&mut game, true, "att", 3);
        add_player(&mut game, true, "ast", 3);
        add_player(&mut game, false, "def", 3);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(13, 7));
        game.field_model.set_player_coordinate("ast", FieldCoordinate::new(13, 8));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(12, 8));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("ast", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.acting_player.strength = 3;
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["def".into()]));
        step.start(&mut game, &mut GameRng::new(1));
        let roll = &step.block_rolls[0];
        assert_eq!(roll.get_nr_of_dice(), 2,
            "ST3 (-2 MB, +1 assist) v ST3 must be a 2-dice block, got {}", roll.get_nr_of_dice());
        assert!(!roll.is_own_choice(), "attacker weaker → defender's choice");
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
    fn default_attacker_team_selects_true() {
        let step = StepBlockRollMultiple::default();
        assert!(step.attacker_team_selects);
    }

    #[test]
    fn set_block_targets_creates_block_rolls() {
        let mut step = StepBlockRollMultiple::new();
        assert!(step.set_parameter(&StepParameter::BlockTargets(vec!["a".into(), "b".into()])));
        assert_eq!(step.block_rolls.len(), 2);
        assert_eq!(step.block_rolls[0].target_id.as_deref(), Some("a"));
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
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("p1".into()));
        assert!(step.block_rolls[0].is_successful_dauntless());
    }

    #[test]
    fn double_target_strength_returns_false() {
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        assert!(!step.set_parameter(&StepParameter::DoubleTargetStrengthForPlayer("p1".into())));
    }

    #[test]
    fn use_brawler_sets_source() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(&Action::UseBrawler { target_id: None }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.re_roll_source.as_deref(), Some("BRAWLER"));
    }

    #[test]
    fn use_hatred_sets_source() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(&Action::UseHatred, &mut game, &mut GameRng::new(0));
        assert_eq!(step.re_roll_source.as_deref(), Some("HATRED"));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn brawler_reroll_adds_block_and_block_re_roll_reports() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        step.first_run = false;
        step.selected_target = Some("p1".into());
        step.re_roll_source = Some("Brawler".into());
        if let Some(roll) = step.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some("p1")) {
            roll.set_block_roll(vec![2]); // BothDown
            roll.set_nr_of_dice(1);
        }
        step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK));
        assert!(game.report_list.has_report(ReportId::BLOCK_RE_ROLL));
    }

    #[test]
    fn hatred_reroll_adds_block_and_block_roll_reports() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepBlockRollMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        step.start(&mut game, &mut GameRng::new(0));
        step.first_run = false;
        step.selected_target = Some("p1".into());
        step.re_roll_source = Some("Hatred".into());
        if let Some(roll) = step.block_rolls.iter_mut().find(|r| r.target_id.as_deref() == Some("p1")) {
            roll.set_block_roll(vec![1]); // Skull
            roll.set_nr_of_dice(1);
        }
        step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK));
        assert!(game.report_list.has_report(ReportId::BLOCK_RE_ROLL));
        assert!(game.report_list.has_report(ReportId::BLOCK_ROLL));
    }
}
