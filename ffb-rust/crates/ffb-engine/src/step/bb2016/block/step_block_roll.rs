/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockRoll`.
///
/// Step in block sequence to handle the block roll.
///
/// Sets stepParameter BLOCK_DICE_INDEX for all steps on the stack.
/// Sets stepParameter BLOCK_RESULT for all steps on the stack.
/// Sets stepParameter BLOCK_ROLL for all steps on the stack.
/// Sets stepParameter NR_OF_BLOCK_DICE for all steps on the stack.
use ffb_mechanics::mechanics::block_result_for_roll;
use ffb_model::dialog::dialog_id::DialogId;
use ffb_model::enums::{BlockResult, PlayerAction, ReRollSource};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::util_server_dialog::UtilServerDialog;
use crate::util::ServerUtilBlock;

/// Java: `StepBlockRoll` (bb2016/block).
pub struct StepBlockRoll {
    /// Java: `fNrOfDice`
    pub nr_of_dice: i32,
    /// Java: `fBlockRoll`
    pub block_roll: Vec<i32>,
    /// Java: `fDiceIndex`
    pub dice_index: usize,
    /// Java: `fBlockResult`
    pub block_result: Option<BlockResult>,
    /// Java: `successfulDauntless`
    pub successful_dauntless: bool,
    /// AbstractStepWithReRoll embedded state.
    pub re_roll: ReRollState,
}

impl StepBlockRoll {
    pub fn new() -> Self {
        Self {
            nr_of_dice: 0,
            block_roll: Vec::new(),
            dice_index: 0,
            block_result: None,
            successful_dauntless: false,
            re_roll: ReRollState::new(),
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let acting_id = game.acting_player.player_id.clone().unwrap_or_default();
        let player_action = game.acting_player.player_action;

        // Java: if (fBlockResult == null)
        if self.block_result.is_none() {
            let mut do_roll = true;

            // Java: if (ReRolledActions.BLOCK == getReRolledAction()) {
            //   if ((getReRollSource() == null) || !UtilServerReRoll.useReRoll(...))
            //     doRoll = false; showBlockRollDialog(doRoll); }
            let is_block_reroll = self.re_roll.re_rolled_action.as_ref()
                .map(|a| a.name == "BLOCK")
                .unwrap_or(false);

            if is_block_reroll {
                if let Some(ref source) = self.re_roll.re_roll_source.clone() {
                    if !use_reroll(game, source, &acting_id) {
                        do_roll = false;
                        // Java: showBlockRollDialog(false)
                    }
                } else {
                    do_roll = false;
                    // Java: showBlockRollDialog(true)
                }
            }

            if do_roll {
                // Java: game.getFieldModel().clearDiceDecorations()
                game.field_model.clear_dice_decorations();

                // Java: fNrOfDice = ServerUtilBlock.findNrOfBlockDice(gameState, actingPlayer,
                //         game.getDefender(), (playerAction == MULTIPLE_BLOCK), successfulDauntless)
                // Java: `attacker.getTeam() == defender.getTeam()` (same-team block, e.g. Ball & Chain)
                // is resolved internally by findNrOfBlockDice; the Rust helper takes it as an
                // explicit parameter instead, so it must be computed here.
                let is_multiple_block = player_action == Some(PlayerAction::MultipleBlock);
                let attacker_str = game.acting_player.strength;
                let defender_str = game.defender_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.strength_with_modifiers())
                    .unwrap_or(3);
                let attacker_on_home = game.team_home.has_player(&acting_id);
                let same_team = game.defender_id.as_deref()
                    .map(|id| game.team_home.has_player(id) == attacker_on_home)
                    .unwrap_or(false);
                self.nr_of_dice = ServerUtilBlock::find_nr_of_block_dice(
                    attacker_str, defender_str, same_team, is_multiple_block, false);

                // Java: fBlockRoll = getGameState().getDiceRoller().rollBlockDice(fNrOfDice)
                let n = self.nr_of_dice.unsigned_abs() as usize;
                self.block_roll = (0..n.max(1)).map(|_| rng.d6()).collect();

                // Java: getResult().addReport(new ReportBlock(game.getDefenderId()))
                // Java: getResult().setSound(SoundId.BLOCK)
                if let Some(ref did) = game.defender_id {
                    use ffb_model::report::report_block::ReportBlock;
                    game.report_list.add(ReportBlock::new(did.clone()));
                }
                let block_event = game.defender_id.as_ref().map(|did| {
                    GameEvent::Block { defender_id: did.clone() }
                });

                // Java: showBlockRollDialog(doRoll)
                // → show dialog (CONTINUE) waiting for block choice
                self.show_block_roll_dialog(game, true);
                let mut outcome = StepOutcome::cont();
                if let Some(ev) = block_event { outcome = outcome.with_event(ev); }
                return outcome;
            } else {
                // Java: showBlockRollDialog(doRoll) — re-roll path, show dialog
                self.show_block_roll_dialog(game, false);
                return StepOutcome::cont();
            }
        } else {
            // Java: publishParameter(NR_OF_DICE, fNrOfDice)
            // Java: publishParameter(BLOCK_ROLL, fBlockRoll)
            // Java: publishParameter(DICE_INDEX, fDiceIndex)
            // Java: publishParameter(BLOCK_RESULT, fBlockResult)
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            let block_result = self.block_result.unwrap();
            return StepOutcome::next()
                .publish(StepParameter::NrOfDice(self.nr_of_dice))
                .publish(StepParameter::BlockRoll(self.block_roll.clone()))
                .publish(StepParameter::DiceIndex(self.dice_index))
                .publish(StepParameter::BlockResult(block_result));
        }
    }

    /// Java: showBlockRollDialog(boolean pDoRoll)
    /// Determines which team gets the re-roll option and shows the dialog.
    fn show_block_roll_dialog(&self, game: &mut Game, do_roll: bool) {
        let acting_id = game.acting_player.player_id.as_deref().unwrap_or("").to_string();

        // Java: boolean teamReRollOption = (getReRollSource() == null) && !reRollUsed && (reRolls > 0)
        let td = game.turn_data();
        let mut team_reroll_option = self.re_roll.re_roll_source.is_none()
            && !td.reroll_used
            && td.rerolls > 0;

        // Java: boolean proReRollOption = (getReRollSource() == null)
        //   && UtilCards.hasUnusedSkillWithProperty(actingPlayer, canRerollOncePerTurn)
        let mut pro_reroll_option = self.re_roll.re_roll_source.is_none()
            && game.player(&acting_id)
                .map(|p| {
                    p.all_skill_ids()
                        .any(|id| id.properties().contains(&NamedProperties::CAN_REROLL_ONCE_PER_TURN)
                            && !p.used_skills.contains(&id))
                })
                .unwrap_or(false);

        // Java: String teamId = homePlaying ? teamHome.getId() : teamAway.getId()
        let mut team_id = if game.home_playing {
            game.team_home.id.clone()
        } else {
            game.team_away.id.clone()
        };

        // Java: if ((fNrOfDice < 0) && (!pDoRoll || (getReRollSource() != null)
        //         || (!teamReRollOption && !proReRollOption))) {
        //         teamId = homePlaying ? teamAway.getId() : teamHome.getId();
        //         teamReRollOption = false; proReRollOption = false; }
        if self.nr_of_dice < 0
            && (!do_roll
                || self.re_roll.re_roll_source.is_some()
                || (!team_reroll_option && !pro_reroll_option))
        {
            team_id = if game.home_playing {
                game.team_away.id.clone()
            } else {
                game.team_home.id.clone()
            };
            team_reroll_option = false;
            pro_reroll_option = false;
        }
        // team_reroll_option / pro_reroll_option are not modeled further: the simplified
        // dialog system here does not carry DialogBlockRollParameter's reroll-option flags.
        let _ = (team_reroll_option, pro_reroll_option);

        // Java: getResult().addReport(new ReportBlockRoll(teamId, fBlockRoll))
        {
            use ffb_model::report::report_block_roll::ReportBlockRoll;
            game.report_list.add(ReportBlockRoll::new(
                team_id,
                self.block_roll.clone(),
                game.defender_id.clone(),
            ));
        }

        // Java: UtilServerDialog.showDialog(gameState, new DialogBlockRollParameter(teamId, fNrOfDice, fBlockRoll,
        //     teamReRollOption, proReRollOption), (fNrOfDice < 0))
        UtilServerDialog::show_dialog(game, DialogId::BLOCK_ROLL, self.nr_of_dice < 0);
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
        // Java: StepCommandStatus commandStatus = super.handleCommand(pReceivedCommand)
        // Java: if (commandStatus == UNHANDLED_COMMAND) {
        //   case CLIENT_BLOCK_CHOICE: fDiceIndex = ...; fBlockResult = forRoll(fBlockRoll[fDiceIndex]) }
        match action {
            Action::BlockChoice { die_index, .. } => {
                // Java: fDiceIndex = blockChoiceCommand.getDiceIndex()
                self.dice_index = *die_index;
                // Java: fBlockResult = game.getRules().<BlockResultFactory>getFactory(BLOCK_RESULT).forRoll(fBlockRoll[fDiceIndex])
                if let Some(&roll) = self.block_roll.get(*die_index) {
                    self.block_result = Some(block_result_for_roll(roll));
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Java: super.handleCommand declining re-roll
                self.re_roll.re_roll_source = None;
            }
            Action::UseReRoll { use_reroll: true } => {
                // Java: TRR accepted — re_roll_source was already set
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: SUCCESSFUL_DAUNTLESS (consumed)
            StepParameter::SuccessfulDauntless(v) => { self.successful_dauntless = *v; true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_block_roll() {
        assert_eq!(StepBlockRoll::new().id(), StepId::BlockRoll);
    }

    #[test]
    fn start_with_no_result_stays_cont() {
        // Java: when fBlockResult is null, show dialog → CONTINUE
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::Continue);
        assert!(!step.block_roll.is_empty());
    }

    #[test]
    fn block_result_set_publishes_parameters_and_next_step() {
        // Java: when fBlockResult is set, publish and NEXT_STEP
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
    fn block_choice_command_sets_dice_index_and_block_result() {
        // Java: CLIENT_BLOCK_CHOICE sets fDiceIndex + fBlockResult
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
    fn successful_dauntless_parameter_accepted() {
        let mut step = StepBlockRoll::new();
        assert!(!step.successful_dauntless);
        step.set_parameter(&StepParameter::SuccessfulDauntless(true));
        assert!(step.successful_dauntless);
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

    #[test]
    fn roll_emits_report_block() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(1));
        assert!(game.report_list.has_report(ReportId::BLOCK), "ReportBlock must be emitted on roll");
    }

    #[test]
    fn roll_emits_report_block_roll() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(1));
        assert!(game.report_list.has_report(ReportId::BLOCK_ROLL), "ReportBlockRoll must be emitted via showBlockRollDialog");
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
    fn multi_block_applies_defender_strength_modifier_not_same_team_abs() {
        // Java: findNrOfBlockDice adds +1 to defender strength when usingMultiBlock, and only
        // forces nrOfDice positive (abs) when attacker.getTeam() == defender.getTeam().
        // Bug: the Rust call previously passed `is_multiple_block` into the `same_team` slot
        // and `successful_dauntless` into the `using_multi_block` slot, so a cross-team
        // multiple-block declaration was wrongly treated as same-team (forcing nrOfDice
        // positive via abs()) instead of applying the +1 defender strength modifier.
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        add_player(&mut game, true, "att", 4);
        add_player(&mut game, false, "def", 4);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.strength = 4;
        game.acting_player.player_action = Some(PlayerAction::MultipleBlock);
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(
            step.nr_of_dice, -2,
            "equal strength + multi-block should give defender the +1 modifier (nrOfDice = -2), \
             not force abs() as if same-team"
        );
    }

    #[test]
    fn negative_nr_of_dice_swaps_report_team_to_defender_when_no_reroll_available() {
        // Java: showBlockRollDialog swaps teamId (and the ReportBlockRoll it reports) to the
        // defending team when fNrOfDice < 0 and the attacking team has no team/skill re-roll
        // available. This swap was entirely missing from the Rust translation.
        use ffb_model::report::report_block_roll::ReportBlockRoll;
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        add_player(&mut game, true, "att", 3);
        add_player(&mut game, false, "def", 4);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.strength = 3;
        game.defender_id = Some("def".into());
        // test_team() has 0 rerolls and the attacker has no canRerollOncePerTurn skill,
        // so both teamReRollOption and proReRollOption are false → swap condition applies.
        step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(step.nr_of_dice, -2, "attacker weaker than defender → defender chooses");
        let report = game.report_list.get_reports().iter()
            .find(|r| r.get_id() == ReportId::BLOCK_ROLL)
            .expect("ReportBlockRoll must be emitted");
        let block_roll_report: &ReportBlockRoll = (&**report as &dyn std::any::Any)
            .downcast_ref()
            .expect("report must be a ReportBlockRoll");
        assert_eq!(
            block_roll_report.get_choosing_team_id(),
            game.team_away.id.as_str(),
            "with fNrOfDice < 0 and no reroll available, teamId must swap to the defending team"
        );
    }
}
