use ffb_mechanics::mechanics::block_result_for_roll;
use ffb_mechanics::mechanics::block_dice_count;
use ffb_model::enums::BlockResult;
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_block::ReportBlock;
use ffb_model::report::report_block_roll::ReportBlockRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.block.StepBlockRoll.
/// Rolls block dice (1, 2, or 3), handles all re-roll variants (Brawler, Hatred, Pro,
/// Consummate Professional, Savage Blow, single-die, multi-die), then publishes
/// NrOfDice / BlockRoll / DiceIndex / BlockResult.
pub struct StepBlockRoll {
    /// Java: fNrOfDice
    pub nr_of_dice: i32,
    /// Java: fDiceIndex — index of die chosen by CLIENT_BLOCK_CHOICE
    pub dice_index: usize,
    /// Java: dieIndex — index of die chosen for single-die re-roll (-1 = none)
    pub die_index: i32,
    /// Java: fBlockRoll
    pub block_roll: Vec<i32>,
    /// Java: diceIndexes — die indices chosen for multi-dice re-roll (Savage Blow)
    pub dice_indexes: Vec<usize>,
    /// Java: fBlockResult — final result after optional re-roll
    pub block_result: Option<BlockResult>,
    /// Java: successfulDauntless — set by StepDauntless if check passed
    pub successful_dauntless: bool,
    /// Java: doubleTargetStrength — set when a skill doubles target strength for die count
    pub double_target_strength: bool,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepBlockRoll {
    pub fn new() -> Self {
        Self {
            nr_of_dice: 0,
            dice_index: 0,
            die_index: -1,
            block_roll: Vec::new(),
            dice_indexes: Vec::new(),
            block_result: None,
            successful_dauntless: false,
            double_target_strength: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBlockRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockRoll {
    fn id(&self) -> StepId { StepId::BlockRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::BlockChoice { die_index, .. } => {
                // Java: CLIENT_BLOCK_CHOICE — player selects which die result to apply.
                self.dice_index = *die_index;
                if let Some(&roll) = self.block_roll.get(*die_index) {
                    self.block_result = Some(block_result_for_roll(roll));
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Declined — clear source so execute_step shows dialog without re-roll.
                self.re_roll_source = None;
            }
            // Java: CLIENT_USE_BRAWLER
            Action::UseBrawler { .. } => {
                self.re_roll_source = Some("Brawler".into());
                self.re_rolled_action = Some("BLOCK".into());
            }
            // Java: CLIENT_USE_HATRED
            Action::UseHatred => {
                self.re_roll_source = Some("Hatred".into());
                self.re_rolled_action = Some("BLOCK".into());
            }
            // Java: CLIENT_USE_PRO_RE_ROLL_FOR_BLOCK
            Action::UseProReRollForBlock { die_index } => {
                self.re_rolled_action = Some("BLOCK".into());
                self.re_roll_source = Some("Pro".into());
                self.die_index = *die_index as i32;
            }
            // Java: CLIENT_USE_CONSUMMATE_RE_ROLL_FOR_BLOCK
            Action::UseConsummateReRollForBlock { die_index } => {
                let player_id = game.acting_player.player_id.clone().unwrap_or_default();
                let source = game.player(&player_id)
                    .and_then(|p| {
                        p.all_skill_ids()
                            .find(|id| id.properties().contains(&NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD))
                    })
                    .map(|id| id.class_name().to_string())
                    .unwrap_or_else(|| "ConsummateProfessional".to_string());
                self.re_rolled_action = Some("BLOCK".into());
                self.re_roll_source = Some(source);
                self.die_index = *die_index as i32;
            }
            // Java: CLIENT_USE_SINGLE_BLOCK_DIE_RE_ROLL
            Action::UseSingleBlockDieReRoll { re_roll_source, die_index } => {
                self.re_rolled_action = Some("BLOCK".into());
                self.re_roll_source = Some(re_roll_source.clone());
                self.die_index = *die_index as i32;
            }
            // Java: CLIENT_USE_MULTI_BLOCK_DICE_RE_ROLL
            Action::UseMultiBlockDiceReRoll { dice_indexes } => {
                let player_id = game.acting_player.player_id.clone().unwrap_or_default();
                let skill_name = game.player(&player_id)
                    .and_then(|p| {
                        p.all_skill_ids()
                            .find(|id| id.properties().contains(&NamedProperties::CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE))
                    })
                    .map(|id| id.class_name().to_string());
                if let Some(name) = skill_name {
                    self.re_rolled_action = Some("BLOCK".into());
                    self.re_roll_source = Some(name);
                    self.dice_indexes = dice_indexes.clone();
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: SUCCESSFUL_DAUNTLESS (consumed)
            StepParameter::SuccessfulDauntless(v) => { self.successful_dauntless = *v; true }
            // Java: DOUBLE_TARGET_STRENGTH (consumed)
            StepParameter::DoubleTargetStrength(v) => { self.double_target_strength = *v; true }
            _ => false,
        }
    }
}

impl StepBlockRoll {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.block_result.is_some() {
            // Java: getGameState().removeAdditionalAssist(game.getActingTeam().getId())
            if game.home_playing {
                game.home_additional_assists = 0;
            } else {
                game.away_additional_assists = 0;
            }
            let block_result = self.block_result.unwrap();
            return StepOutcome::next()
                .publish(StepParameter::NrOfDice(self.nr_of_dice))
                .publish(StepParameter::BlockRoll(self.block_roll.clone()))
                .publish(StepParameter::DiceIndex(self.dice_index))
                .publish(StepParameter::BlockResult(block_result));
        }

        // Java: boolean doRoll = true;
        //       if (ReRolledActions.BLOCK == getReRolledAction()) { ... }
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();
        let mut do_roll = true;

        if self.re_rolled_action.as_deref() == Some("BLOCK") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                // Java: !UtilServerReRoll.useReRoll(this, getReRollSource(), player)
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    do_roll = false;
                    // Java: showBlockRollDialog(false) — source was non-null
                }
            } else {
                // Java: source == null → short-circuit → doRoll = false
                // Java: showBlockRollDialog(true) — no re-roll was consumed
                do_roll = false;
            }
        }

        if do_roll {
            // Java: clearDiceDecorations(), addReport(ReportBlock), setSound(BLOCK)
            // Java: getResult().addReport(new ReportBlock(game.getDefenderId()))
            if let Some(ref did) = game.defender_id.clone() {
                game.report_list.add(ReportBlock::new(did.clone()));
            }
            let source = self.re_roll_source.clone();
            match source.as_deref() {
                Some("Brawler") => {
                    // Java: actingPlayer.markSkillUsed(BRAWLER, SINGLE_BOTH_DOWN) if no game option
                    // (use_reroll above already marked the skill used)
                    self.handle_implicit_reroll_index(rng, BlockResult::BothDown);
                }
                Some("Hatred") => {
                    self.handle_implicit_reroll_index(rng, BlockResult::Skull);
                }
                _ => {
                    self.handle_initial_roll_and_reroll_with_explicit_selection(game, rng);
                }
            }
            // Java: showBlockRollDialog(false)
            // Java: getResult().addReport(new ReportBlockRoll(teamId, fBlockRoll))
            {
                let team_id = if game.home_playing {
                    game.team_home.id.clone()
                } else {
                    game.team_away.id.clone()
                };
                game.report_list.add(ReportBlockRoll::new(
                    team_id,
                    self.block_roll.clone(),
                    game.defender_id.clone(),
                ));
            }
            return StepOutcome::cont();
        }

        // do_roll = false → show dialog (player has not yet chosen)
        // Java: if source was null → showBlockRollDialog(noReRollUsed=true)
        //       (presented when the player hasn't interacted with the re-roll dialog yet)

        // Java: if reRolledAction is not set, ask for re-roll if available
        if self.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "BLOCK", 0, false) {
                self.re_rolled_action = Some("BLOCK".into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        StepOutcome::cont()
    }

    /// Java: handleImplicitReRollIndex(actingPlayer, BlockResult resultToReplace).
    ///
    /// Rolls one new block die, then finds the first die in block_roll that matches
    /// resultToReplace and replaces it. BRAWLER uses BothDown; HATRED uses Skull.
    fn handle_implicit_reroll_index(&mut self, rng: &mut GameRng, result_to_replace: BlockResult) {
        let rerolled_die = rng.d6();
        // Java: for (int i = 0; i < fBlockRoll.length; i++) { if (factory.forRoll(fBlockRoll[i]) == resultToReplace) { dieIndex = i; break; } }
        for i in 0..self.block_roll.len() {
            if block_result_for_roll(self.block_roll[i]) == result_to_replace {
                self.die_index = i as i32;
                break;
            }
        }
        // Java: if (dieIndex >= 0) { fBlockRoll = Arrays.copyOf(...); fBlockRoll[dieIndex] = rerolledDie; }
        if self.die_index >= 0 {
            if let Some(slot) = self.block_roll.get_mut(self.die_index as usize) {
                *slot = rerolled_die;
            }
        }
    }

    /// Java: handleInitialRollAndReRollWithExplicitSelection(game, actingPlayer).
    ///
    /// Computes the number of block dice (if not yet set), then either performs a
    /// fresh roll or applies an explicit single-die / multi-die re-roll depending
    /// on the current re_roll_source.
    fn handle_initial_roll_and_reroll_with_explicit_selection(&mut self, game: &mut Game, rng: &mut GameRng) {
        // Java: fNrOfDice = ServerUtilBlock.findNrOfBlockDice(...).getLeft()
        // Only compute from player strengths when nr_of_dice is not already set.
        if self.nr_of_dice == 0 {
            let attacker_str = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.strength_with_modifiers())
                .unwrap_or(3);
            let defender_str = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.strength_with_modifiers())
                .unwrap_or(3);
            self.nr_of_dice = block_dice_count(attacker_str, defender_str);
        }

        let source = self.re_roll_source.as_deref().unwrap_or("");

        // Java: if (source == PRO || (reRollSkill != null && reRollSkill.hasSkillProperty(canRerollSingleDieOncePerPeriod)))
        // Consummate Professional has canRerollSingleDieOncePerPeriod
        let is_pro_type = source == "Pro" || source == "ConsummateProfessional";

        if is_pro_type {
            // Java: roll 1 die, replace fBlockRoll[dieIndex]
            let new_die = rng.d6();
            if self.die_index >= 0 {
                if let Some(slot) = self.block_roll.get_mut(self.die_index as usize) {
                    *slot = new_die;
                }
            }
        } else if matches!(source, "UnstoppableMomentum" | "LordOfChaos" | "WorkingInTandem" | "WoodlandFury") {
            // Java: these also replace a single die at dieIndex
            if self.die_index >= 0 {
                let new_die = rng.d6();
                if let Some(slot) = self.block_roll.get_mut(self.die_index as usize) {
                    *slot = new_die;
                }
            }
        } else if source == "SavageBlow" && !self.dice_indexes.is_empty() {
            // Java: roll diceIndexes.length dice, replace each position
            let n = self.dice_indexes.len();
            let new_dice: Vec<i32> = (0..n).map(|_| rng.d6()).collect();
            for (pos, &idx) in self.dice_indexes.iter().enumerate() {
                if let Some(slot) = self.block_roll.get_mut(idx) {
                    *slot = new_dice[pos];
                }
            }
        } else {
            // Java: fBlockRoll = rollBlockDice(fNrOfDice) — fresh roll
            let n = self.nr_of_dice.unsigned_abs() as usize;
            self.block_roll = (0..n).map(|_| rng.d6()).collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, BlockResult};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        let coord = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    // ── start() with no block_result ─────────────────────────────────────────

    #[test]
    fn start_with_no_result_stays_cont() {
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::Continue);
        assert!(!step.block_roll.is_empty());
    }

    #[test]
    fn start_rolls_correct_number_of_dice() {
        let mut step = StepBlockRoll::new();
        step.nr_of_dice = 3;
        let mut game = make_game();
        step.start(&mut game, &mut GameRng::new(7));
        assert_eq!(step.block_roll.len(), 3);
    }

    #[test]
    fn start_defaults_to_one_die_when_nr_of_dice_is_zero() {
        let mut step = StepBlockRoll::new();
        assert_eq!(step.nr_of_dice, 0);
        let mut game = make_game();
        step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(step.block_roll.len(), 1);
        assert_eq!(step.nr_of_dice, 1);
    }

    #[test]
    fn start_negative_nr_of_dice_treated_as_absolute_value() {
        let mut step = StepBlockRoll::new();
        step.nr_of_dice = -2;
        let mut game = make_game();
        step.start(&mut game, &mut GameRng::new(5));
        assert_eq!(step.block_roll.len(), 2);
    }

    #[test]
    fn rolled_dice_values_in_range_1_to_6() {
        let mut step = StepBlockRoll::new();
        step.nr_of_dice = 3;
        let mut game = make_game();
        step.start(&mut game, &mut GameRng::new(11));
        for &v in &step.block_roll {
            assert!((1..=6).contains(&v), "die value out of range: {v}");
        }
    }

    // ── block_result pre-set (already resolved) ───────────────────────────────

    #[test]
    fn block_result_set_publishes_parameters_and_next_step() {
        let mut step = StepBlockRoll::new();
        step.block_result = Some(BlockResult::Pow);
        step.block_roll = vec![6];
        step.nr_of_dice = 1;
        step.dice_index = 0;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockResult(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::NrOfDice(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockRoll(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DiceIndex(_))));
    }

    #[test]
    fn published_block_result_value_matches() {
        let mut step = StepBlockRoll::new();
        step.block_result = Some(BlockResult::Skull);
        step.block_roll = vec![1];
        step.nr_of_dice = 1;
        step.dice_index = 0;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let found = out.published.iter().find_map(|p| {
            if let StepParameter::BlockResult(r) = p { Some(*r) } else { None }
        });
        assert_eq!(found, Some(BlockResult::Skull));
    }

    #[test]
    fn published_nr_of_dice_value_matches() {
        let mut step = StepBlockRoll::new();
        step.block_result = Some(BlockResult::Pushback);
        step.block_roll = vec![3, 4];
        step.nr_of_dice = 2;
        step.dice_index = 0;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let found = out.published.iter().find_map(|p| {
            if let StepParameter::NrOfDice(n) = p { Some(*n) } else { None }
        });
        assert_eq!(found, Some(2));
    }

    // ── handle_command BlockChoice ────────────────────────────────────────────

    #[test]
    fn block_choice_command_sets_dice_index_and_block_result() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![1, 6, 3];
        step.nr_of_dice = 3;
        let mut game = make_game();
        let out = step.handle_command(
            &Action::BlockChoice { die_index: 1, target_id: None },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.dice_index, 1);
        assert_eq!(step.block_result, Some(BlockResult::Pow));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn block_choice_skull_mapped_correctly() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![1];
        step.nr_of_dice = 1;
        let mut game = make_game();
        step.handle_command(
            &Action::BlockChoice { die_index: 0, target_id: None },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.block_result, Some(BlockResult::Skull));
    }

    #[test]
    fn block_choice_both_down_mapped_correctly() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![2];
        step.nr_of_dice = 1;
        let mut game = make_game();
        step.handle_command(
            &Action::BlockChoice { die_index: 0, target_id: None },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.block_result, Some(BlockResult::BothDown));
    }

    #[test]
    fn block_choice_pow_pushback_mapped_correctly() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![5];
        step.nr_of_dice = 1;
        let mut game = make_game();
        step.handle_command(
            &Action::BlockChoice { die_index: 0, target_id: None },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.block_result, Some(BlockResult::PowPushback));
    }

    // ── set_parameter ─────────────────────────────────────────────────────────

    #[test]
    fn successful_dauntless_parameter_accepted() {
        let mut step = StepBlockRoll::new();
        assert!(!step.successful_dauntless);
        step.set_parameter(&StepParameter::SuccessfulDauntless(true));
        assert!(step.successful_dauntless);
    }

    #[test]
    fn double_target_strength_parameter_accepted() {
        let mut step = StepBlockRoll::new();
        assert!(!step.double_target_strength);
        step.set_parameter(&StepParameter::DoubleTargetStrength(true));
        assert!(step.double_target_strength);
    }

    // ── additional_assists cleared on result ──────────────────────────────────

    #[test]
    fn home_additional_assists_cleared_when_result_published() {
        let mut step = StepBlockRoll::new();
        step.block_result = Some(BlockResult::Pow);
        step.block_roll = vec![6];
        step.nr_of_dice = 1;
        let mut game = make_game();
        game.home_playing = true;
        game.home_additional_assists = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.home_additional_assists, 0);
    }

    #[test]
    fn away_additional_assists_cleared_when_result_published() {
        let mut step = StepBlockRoll::new();
        step.block_result = Some(BlockResult::Pushback);
        step.block_roll = vec![3];
        step.nr_of_dice = 1;
        let mut game = make_game();
        game.home_playing = false;
        game.away_additional_assists = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.away_additional_assists, 0);
    }

    // ── Brawler implicit re-roll ──────────────────────────────────────────────

    #[test]
    fn brawler_finds_both_down_die_and_rerolls_it() {
        let mut step = StepBlockRoll::new();
        // block_roll: [1=Skull, 2=BothDown, 6=Pow]
        step.block_roll = vec![1, 2, 6];
        step.nr_of_dice = 3;
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::Brawler);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        step.handle_command(&Action::UseBrawler { target_id: None }, &mut game, &mut GameRng::new(1));
        // die_index should be 1 (first BothDown at index 1)
        assert_eq!(step.die_index, 1);
        // block_roll[0] and [2] should be unchanged
        assert_eq!(step.block_roll[0], 1);
        assert_eq!(step.block_roll[2], 6);
        // block_roll[1] should be a valid die value (was re-rolled)
        assert!((1..=6).contains(&step.block_roll[1]));
    }

    #[test]
    fn brawler_no_both_down_leaves_die_index_unchanged() {
        let mut step = StepBlockRoll::new();
        // No BothDown in roll
        step.block_roll = vec![1, 6, 3];
        step.nr_of_dice = 3;
        step.die_index = -1;
        // Simulate: Brawler re-roll path already reached handle_implicit_reroll_index
        let mut rng = GameRng::new(5);
        step.handle_implicit_reroll_index(&mut rng, BlockResult::BothDown);
        // die_index should remain -1 since no BothDown found
        assert_eq!(step.die_index, -1);
        // block_roll should be unchanged
        assert_eq!(step.block_roll, vec![1, 6, 3]);
    }

    #[test]
    fn brawler_finds_first_both_down_when_multiple() {
        let mut step = StepBlockRoll::new();
        // Two BothDown dice: indices 0 and 2
        step.block_roll = vec![2, 6, 2];
        step.die_index = -1;
        let mut rng = GameRng::new(3);
        step.handle_implicit_reroll_index(&mut rng, BlockResult::BothDown);
        // Should find the FIRST BothDown (index 0)
        assert_eq!(step.die_index, 0);
    }

    // ── Hatred implicit re-roll ───────────────────────────────────────────────

    #[test]
    fn hatred_finds_skull_die_and_rerolls_it() {
        let mut step = StepBlockRoll::new();
        // block_roll: [1=Skull, 6=Pow, 3=Pushback]
        step.block_roll = vec![1, 6, 3];
        step.die_index = -1;
        let mut rng = GameRng::new(4);
        step.handle_implicit_reroll_index(&mut rng, BlockResult::Skull);
        assert_eq!(step.die_index, 0);
        // block_roll[0] should be updated
    }

    #[test]
    fn hatred_no_skull_leaves_roll_unchanged() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![2, 6, 3];
        step.die_index = -1;
        let original = step.block_roll.clone();
        let mut rng = GameRng::new(2);
        step.handle_implicit_reroll_index(&mut rng, BlockResult::Skull);
        assert_eq!(step.die_index, -1);
        assert_eq!(step.block_roll, original);
    }

    // ── Pro/Consummate single-die re-roll ─────────────────────────────────────

    #[test]
    fn pro_rerolls_single_die_at_die_index() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![1, 2, 3];
        step.nr_of_dice = 3;
        step.die_index = 1;
        step.re_rolled_action = Some("BLOCK".into());
        step.re_roll_source = Some("Pro".into());
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::Pro);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        step.handle_command(&Action::UseProReRollForBlock { die_index: 1 }, &mut game, &mut GameRng::new(6));
        // Only die at index 1 should change; others stay
        assert_eq!(step.block_roll[0], 1);
        assert_eq!(step.block_roll[2], 3);
        // die at index 1 was re-rolled (might be same value but was processed)
    }

    // ── Savage Blow multi-die re-roll ─────────────────────────────────────────

    #[test]
    fn savage_blow_rerolls_selected_dice_positions() {
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![1, 2, 3];
        step.nr_of_dice = 3;
        step.dice_indexes = vec![0, 2];
        step.re_roll_source = Some("SavageBlow".into());
        // Directly call handle_initial_roll_and_reroll_with_explicit_selection
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut rng = GameRng::new(9);
        step.handle_initial_roll_and_reroll_with_explicit_selection(&mut game, &mut rng);
        // Positions 0 and 2 were re-rolled; position 1 (value=2) stays unchanged
        assert_eq!(step.block_roll[1], 2);
        // Positions 0 and 2 should be new dice values (1..=6)
        assert!((1..=6).contains(&step.block_roll[0]));
        assert!((1..=6).contains(&step.block_roll[2]));
    }

    #[test]
    fn savage_blow_empty_dice_indexes_does_fresh_roll() {
        // If no dice_indexes, fall through to fresh roll
        let mut step = StepBlockRoll::new();
        step.block_roll = vec![1, 2, 3];
        step.nr_of_dice = 3;
        step.dice_indexes = vec![];
        step.re_roll_source = Some("SavageBlow".into());
        let original = step.block_roll.clone();
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut rng = GameRng::new(7);
        step.handle_initial_roll_and_reroll_with_explicit_selection(&mut game, &mut rng);
        // With empty dice_indexes the else branch fires → fresh roll of 3 dice
        // (original may or may not match; just check length is preserved)
        assert_eq!(step.block_roll.len(), original.len());
    }

    // ── fresh roll ────────────────────────────────────────────────────────────

    #[test]
    fn trr_reroll_produces_fresh_set_of_dice() {
        let mut step = StepBlockRoll::new();
        step.nr_of_dice = 2;
        step.block_roll = vec![1, 1];
        step.re_rolled_action = Some("BLOCK".into());
        step.re_roll_source = Some("TRR".into());
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_mut().rerolls = 1;
        let mut rng = GameRng::new(42);
        let out = step.execute_step(&mut game, &mut rng);
        // TRR consumed → fresh dice rolled → still cont (awaiting BlockChoice)
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_data().rerolls, 0);
    }

    // ── report wiring ─────────────────────────────────────────────────────────

    #[test]
    fn initial_roll_adds_report_block() {
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        step.start(&mut game, &mut GameRng::new(1));
        assert!(game.report_list.has_report(ReportId::BLOCK), "ReportBlock should appear after initial roll");
    }

    #[test]
    fn initial_roll_adds_report_block_roll() {
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        step.start(&mut game, &mut GameRng::new(2));
        assert!(game.report_list.has_report(ReportId::BLOCK_ROLL), "ReportBlockRoll should appear after initial roll");
    }
}
