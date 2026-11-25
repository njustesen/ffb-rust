use ffb_mechanics::mechanics::block_result_for_roll;
use ffb_mechanics::mechanics::block_dice_count;
use ffb_model::enums::BlockResult;
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_block::ReportBlock;
use ffb_model::report::report_block_roll::ReportBlockRoll;
use ffb_model::report::mixed::report_block_re_roll::ReportBlockReRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::use_reroll_with_events;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepBlockRoll.
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
    /// bb2020 Java: reRolledDiceIndexes -- the dice a skill re-roll already replaced.
    pub re_rolled_dice_indexes: Vec<i32>,
    /// bb2020 Java: brawlerIndex -- the die Brawler replaced (-1 = Brawler not used).
    pub brawler_index: i32,
    /// Whether this dialog's skill half (`AgentPrompt::BlockSkillReRoll`) was already asked,
    /// so its decline falls through to the die choice instead of asking again.
    pub skill_offer_asked: bool,
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
            re_rolled_dice_indexes: Vec::new(),
            brawler_index: -1,
            skill_offer_asked: false,
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
            // Java: CLIENT_USE_RE_ROLL, handled by `AbstractStepWithReRoll.handleCommand` —
            // sets the re-rolled action and the source, then re-executes the step, where
            // `useReRoll` spends the team re-roll (and rolls Loner for a carrier who has it).
            Action::UseReRoll { use_reroll: true } => {
                self.re_rolled_action = Some("BLOCK".into());
                self.re_roll_source = Some("TRR".into());
            }
            // Declining is NOT a command in Java: the block dialog has no decline, and answering
            // it with a null-source re-roll drives `doRoll = false` → `showBlockRollDialog(true)`
            // → the same dialog forever (the loop `ParityRunner` warned about). The client
            // declines by answering the SAME dialog with a die choice, so the decline arm returns
            // straight to the die-choice prompt: re-entering `execute_step` here would roll a
            // fresh set of block dice, because `re_rolled_action` is still unset.
            //
            // The same answer declines the dialog's SKILL half (`AgentPrompt::BlockSkillReRoll`),
            // which is asked after the TRR half and before the die choice. Only the TRR decline
            // clears the source: a skill decline may follow an already-spent re-roll (bb2020
            // offers Pro after Brawler), whose source must survive into the published result.
            Action::UseReRoll { use_reroll: false } => {
                if !self.skill_offer_asked {
                    self.re_roll_source = None;
                }
                return StepOutcome::cont().with_prompt(self.after_team_reroll_prompt(game));
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
                    // Java `anyRrSkill.getRerollSource(MULTI_BLOCK_DICE)` — the ReRollSource name
                    // ("Savage Blow"), which ReportBlockReRoll carries.
                    .map(|id| id.category_and_name_for(game.rules).1.to_string());
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

    // Java: setParameter consume()s SUCCESSFUL_DAUNTLESS and DOUBLE_TARGET_STRENGTH.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::SuccessfulDauntless(_) | StepParameter::DoubleTargetStrength(_))
    }
}

/// The block-die skill re-roll sources by class name. The agents answer with Java's
/// `ReRollSource` name ("Unstoppable Momentum", which `ReportBlockReRoll` carries); older call
/// sites and tests use the class name. Both mean the same source (Java compares by identity).
fn canon_source(source: &str) -> &str {
    match source {
        "Unstoppable Momentum" => "UnstoppableMomentum",
        "Savage Blow" => "SavageBlow",
        "Lord of Chaos" => "LordOfChaos",
        "Working in Tandem" => "WorkingInTandem",
        "Woodland Fury" => "WoodlandFury",
        other => other,
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
            // Coverage event: one BlockRoll per resolved block action, carrying the final
            // dice (post-reroll), the chosen die index (drives the Skull/BothDown/Pushback/
            // PowPushback/Pow result counters) and whether any re-roll was used.
            // re_roll_source stays Some only when a re-roll was accepted (declining clears it).
            return StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::BlockRoll {
                    attacker_id: game.acting_player.player_id.clone().unwrap_or_default(),
                    defender_id: game.defender_id.clone().unwrap_or_default(),
                    nr_of_dice: self.nr_of_dice,
                    dice: self.block_roll.clone(),
                    selected_index: self.dice_index as i32,
                    own_choice: self.nr_of_dice >= 0,
                    rerolled: self.re_roll_source.is_some(),
                })
                .publish(StepParameter::NrOfDice(self.nr_of_dice))
                .publish(StepParameter::BlockRoll(self.block_roll.clone()))
                .publish(StepParameter::DiceIndex(self.dice_index))
                .publish(StepParameter::BlockResult(block_result));
        }

        // Java: boolean doRoll = true;
        //       if (ReRolledActions.BLOCK == getReRolledAction()) { ... }
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();
        let mut do_roll = true;

        // Coverage events produced while consuming the re-roll (the Loner roll), attached to
        // whichever outcome this pass returns.
        let mut reroll_events: Vec<ffb_model::events::GameEvent> = Vec::new();
        if self.re_rolled_action.as_deref() == Some("BLOCK") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                // Java: !UtilServerReRoll.useReRoll(this, getReRollSource(), player)
                // bb2020 `StepBlockRoll.executeStep` never calls useReRoll for BRAWLER,
                // UNSTOPPABLE_MOMENTUM or SAVAGE_BLOW -- those re-roll unconditionally.
                let is_bb2020 = game.rules == ffb_model::enums::Rules::Bb2020;
                let skips_use = is_bb2020
                    && matches!(canon_source(source_name), "Brawler" | "UnstoppableMomentum" | "SavageBlow");
                let source = ReRollSource::new(source_name.as_str());
                let consumed = if skips_use {
                    true
                } else {
                    let (consumed, evs) = use_reroll_with_events(game, &source, &player_id, rng);
                    reroll_events = evs;
                    consumed
                };
                if !consumed {
                    do_roll = false;
                    // bb2020: a failed Pro still uses up its die --
                    // `reRolledDiceIndexes = add(reRolledDiceIndexes, proIndex)`.
                    if is_bb2020 && source_name == "Pro" {
                        self.re_rolled_dice_indexes.push(self.die_index);
                    }
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
                Some("Brawler") if game.rules == ffb_model::enums::Rules::Bb2020 => {
                    // Java bb2020 `handleBrawler`: the first BOTH_DOWN not already re-rolled.
                    self.handle_brawler_bb2020(game, rng);
                }
                Some("Brawler") => {
                    // Java: actingPlayer.markSkillUsed(BRAWLER, SINGLE_BOTH_DOWN) unless the option
                    // allows Brawler on both blocks (bb2025 useReRoll marks only tracked skills).
                    if !ffb_model::option::util_game_option::is_option_enabled(
                        game, ffb_model::option::game_option_id::ALLOW_BRAWLER_ON_BOTH_BLOCKS)
                    {
                        if let Some(b) = game.player(&player_id)
                            .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN))
                        {
                            crate::step::util_server_steps::mark_skill_used(game, &player_id, b);
                        }
                    }
                    self.handle_implicit_reroll_index(game, rng, BlockResult::BothDown);
                }
                Some("Hatred") => {
                    self.handle_implicit_reroll_index(game, rng, BlockResult::Skull);
                }
                _ => {
                    self.handle_initial_roll_and_reroll_with_explicit_selection(game, rng);
                }
            }
            // Java: showBlockRollDialog(false) — emit the die-selection prompt so the agent
            // can answer with Action::BlockChoice (a bare cont() waits forever).
            // Java: getResult().addReport(new ReportBlockRoll(teamId, fBlockRoll))
            {
                // Java `showBlockRollDialog(false)`: the defender's team on dice-against with no
                // re-roll to offer (see `block_roll_report_team`).
                let team_id = self.block_roll_report_team(game, false);
                // Java's single-block report is the TWO-argument constructor: no defender id
                // (only `StepBlockRollMultiple` names the target). Rust passed the defender, so
                // the report stream carried a `defenderId` Java's never has (report diff, H.50).
                game.report_list.add(ReportBlockRoll::new(
                    team_id,
                    self.block_roll.clone(),
                    None,
                ));
                // Java: UtilServerDialog.showDialog(..., new DialogBlockRollPropertiesParameter(...), (fNrOfDice < 0))
                self.show_block_roll_dialog(game, false);
            }
            // Coverage event mirroring ReportBlock: emitted once per block action
            // (initial roll only — re-roll passes have re_rolled_action set).
            //
            // Java shows ONE dialog carrying the dice AND the re-roll options, and the client
            // answers it with either CLIENT_BLOCK_CHOICE or a re-roll command. Rust asks one
            // question per prompt, so the same dialog becomes two prompts in sequence: the
            // re-roll offer first (only when Java would have listed `ReRollProperty.TRR`), then
            // the die choice. Declining answers the die choice WITHOUT re-rolling — see
            // `handle_command`. `ParityRunner` mirrors the pair on one dialog: `reRollSourceFor`
            // then `heuristicBlockChoice`, in that order.
            self.skill_offer_asked = false;
            let mut out = if self.team_reroll_option(game) {
                StepOutcome::cont().with_prompt(self.block_reroll_offer_prompt(game))
            } else {
                StepOutcome::cont().with_prompt(self.after_team_reroll_prompt(game))
            };
            if self.re_rolled_action.is_none() {
                if let Some(ref did) = game.defender_id {
                    out = out.with_event(ffb_model::events::GameEvent::Block {
                        defender_id: did.clone(),
                    });
                }
            }
            for ev in reroll_events {
                out = out.with_event(ev);
            }
            return out;
        }

        // do_roll = false → show dialog (player has not yet chosen)
        // Java: if source was null → showBlockRollDialog(noReRollUsed=true)
        //       (presented when the player hasn't interacted with the re-roll dialog yet)

        // Java: showBlockRollDialog(getReRollSource() == null) — the re-roll was offered and
        // either failed to be consumed (a failed Loner) or was never accepted, so re-present the
        // already-rolled dice for selection. This is NOT a place to ask again: Java re-shows the
        // dialog with `getReRollSource() != null`, which builds an EMPTY property list.
        //
        // The offer itself is raised on the roll pass above, not here. It used to be raised here
        // via `ask_for_reroll_if_available(game, "BLOCK", 0, false)`, which was dead code by
        // POSITION — `do_roll` starts true and is only cleared inside the `re_rolled_action ==
        // Some("BLOCK")` arm, so a first pass could never reach it (BACKLOG §H.13.1). It was also
        // the wrong predicate: that helper is the shared `RollMechanic.askForReRollIfAvailable`
        // path, which prefers a SKILL source and adds Pro, while Java's block dialog carries those
        // as `actionToSource` entries the harness never answers and lists only TRR in
        // `properties`. Offering more than the dialog lists costs sampler draws Java never spends.
        // Java: showBlockRollDialog(...) re-reports the (unchanged) dice —
        // `getResult().addReport(new ReportBlockRoll(teamId, fBlockRoll))` runs on this path too.
        {
            // Java `showBlockRollDialog(getReRollSource() == null)`.
            let team_id = self.block_roll_report_team(game, self.re_roll_source.is_none());
            game.report_list.add(ReportBlockRoll::new(team_id, self.block_roll.clone(), None));
            // Java: UtilServerDialog.showDialog(..., new DialogBlockRollPropertiesParameter(...), (fNrOfDice < 0))
            self.show_block_roll_dialog(game, self.re_roll_source.is_none());
        }
        // The re-shown dialog may still carry skill options (bb2020 after a failed Pro offers
        // Brawler); bb2025 builds none once a source is set.
        self.skill_offer_asked = false;
        let mut out = StepOutcome::cont().with_prompt(self.after_team_reroll_prompt(game));
        for ev in reroll_events {
            out = out.with_event(ev);
        }
        out
    }

    /// The prompt after the dialog's TRR half: its SKILL half (`AgentPrompt::BlockSkillReRoll`)
    /// when the dialog lists Brawler or Pro and it has not been asked yet, else the die choice.
    /// Java shows both halves on one dialog; `ParityRunner` answers them in the same order.
    fn after_team_reroll_prompt(&mut self, game: &Game) -> ffb_model::prompts::AgentPrompt {
        if !self.skill_offer_asked {
            let (brawler, pro) = self.skill_reroll_options(game);
            let (single_block_die, multi_block_dice) = self.explicit_die_options(game);
            if brawler || pro || single_block_die.is_some() || multi_block_dice {
                self.skill_offer_asked = true;
                return ffb_model::prompts::AgentPrompt::BlockSkillReRoll {
                    attacker_id: game.acting_player.player_id.clone().unwrap_or_default(),
                    dice: self.block_roll.clone(),
                    own_choice: self.nr_of_dice >= 0,
                    brawler,
                    pro,
                    single_block_die,
                    multi_block_dice,
                };
            }
        }
        self.block_choice_prompt(game)
    }

    /// Whether the block dialog lists a Brawler and/or a Pro re-roll.
    ///
    /// bb2025 `showBlockRollDialog` (only while `getReRollSource() == null`):
    /// `evaluateBrawlerAvailability` maps SINGLE_BOTH_DOWN, and
    /// `addReRollSourceMapping(SINGLE_DIE_PER_ACTIVATION)` maps Pro's source.
    ///
    /// bb2020 `showBlockRollDialog`: `proReRollOption` / `brawlerOption`, which stay open after
    /// another skill re-roll while some die has not been re-rolled yet.
    fn skill_reroll_options(&self, game: &Game) -> (bool, bool) {
        let Some(pid) = game.acting_player.player_id.as_deref() else { return (false, false) };
        let Some(player) = game.player(pid) else { return (false, false) };
        let blitzing = game.acting_player.player_action.map(|a| a.is_blitzing()).unwrap_or(false);
        let unused_id = |id: ffb_model::enums::SkillId| !player.used_skills.contains(&id)
            && !game.acting_player.used_skills.contains(&id);
        let unused = |property: &str| player.all_skill_ids()
            .any(|id| id.properties().contains(&property) && unused_id(id));
        let results: Vec<BlockResult> = self.block_roll.iter().map(|r| block_result_for_roll(*r)).collect();

        if game.rules == ffb_model::enums::Rules::Bb2020 {
            let source = self.re_roll_source.as_deref();
            let skill_source = |property: &str| player.skill_id_with_property(property)
                .map(|id| source == Some(id.class_name()))
                .unwrap_or(false);
            let single_die_used = skill_source(NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD);
            let single_block_die_used = skill_source(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ);
            let any_block_dice_used = skill_source(NamedProperties::CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE);
            let dice_left = self.block_roll.len() > self.re_rolled_dice_indexes.len();
            let pro = (source.is_none()
                    || ((source == Some("Brawler") || single_die_used || single_block_die_used || any_block_dice_used)
                        && dice_left))
                && unused(NamedProperties::CAN_REROLL_ONCE_PER_TURN);
            let brawler = !blitzing
                && player.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN)
                && self.brawler_index < 0
                && (source.is_none()
                    || ((source == Some("Pro") || single_die_used || single_block_die_used || any_block_dice_used)
                        && dice_left))
                && results.iter().enumerate().any(|(i, r)| *r == BlockResult::BothDown
                    && !self.re_rolled_dice_indexes.contains(&(i as i32)));
            return (brawler, pro);
        }

        if self.re_roll_source.is_some() {
            return (false, false);
        }
        // Java UtilCards.getUnusedRerollSource(actingPlayer, action): the first unused skill
        // registering a re-roll source for that action.
        let source_for = |action: &str| player.all_skill_ids()
            .filter(|id| unused_id(*id))
            .find(|id| id.reroll_sources_for(game.rules).iter().any(|(a, _)| *a == action));
        let brawler = player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN)
            .map(|skill| !blitzing
                && (ffb_model::option::util_game_option::is_option_enabled(
                        game, ffb_model::option::game_option_id::ALLOW_BRAWLER_ON_BOTH_BLOCKS)
                    || unused_id(skill))
                && results.contains(&BlockResult::BothDown)
                && source_for("SINGLE_BOTH_DOWN").is_some())
            .unwrap_or(false);
        let pro = source_for("SINGLE_DIE_PER_ACTIVATION") == Some(ffb_model::enums::SkillId::Pro);
        (brawler, pro)
    }

    /// The dialog's explicit-die skill re-rolls (BACKLOG §M): the SINGLE_BLOCK_DIE source, by
    /// Java `ReRollSource` name, and whether a MULTI_BLOCK_DICE skill is listed.
    ///
    /// bb2025 `showBlockRollDialog` (only while `getReRollSource() == null`): the
    /// `singleBlock` if/else chain (would-be-knocked-down → blitz → once-per-period → working in
    /// tandem) into `actionToSource[SINGLE_BLOCK_DIE]`, and
    /// `addReRollSourceMapping(MULTI_BLOCK_DICE)` = `UtilCards.getUnusedRerollSource`.
    ///
    /// bb2020 `showBlockRollDialog`: `singleBlockDieOption` (blitzing + unused
    /// canRerollSingleBlockDieDuringBlitz) and `anyBlockDiceOption` (unused
    /// canReRollAnyNumberOfBlockDice), both gated on `someDiceCanBeReRolled`; the dialog lists
    /// the skills and the client takes `skill.getRerollSource(...)`.
    fn explicit_die_options(&self, game: &Game) -> (Option<String>, bool) {
        let Some(pid) = game.acting_player.player_id.as_deref() else { return (None, false) };
        let Some(player) = game.player(pid) else { return (None, false) };
        let blitzing = game.acting_player.player_action.map(|a| a.is_blitzing()).unwrap_or(false);
        let unused_id = |id: ffb_model::enums::SkillId| !player.used_skills.contains(&id)
            && !game.acting_player.used_skills.contains(&id);
        let unused_with = |property: &str| player.all_skill_ids()
            .find(|id| id.properties().contains(&property) && unused_id(*id));
        let source_name = |id: ffb_model::enums::SkillId| {
            ffb_model::skill_touch::touch(id); // §M telemetry (observational only)
            id.category_and_name_for(game.rules).1.to_string()
        };

        if game.rules == ffb_model::enums::Rules::Bb2020 {
            let some_dice = self.re_roll_source.as_deref() != Some("TRR")
                && self.block_roll.len() > self.re_rolled_dice_indexes.len();
            let single = if blitzing && some_dice {
                unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ)
                    .and_then(|_| player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ))
                    .map(source_name)
            } else {
                None
            };
            let multi = some_dice && unused_with(NamedProperties::CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE).is_some();
            return (single, multi);
        }

        if self.re_roll_source.is_some() {
            return (None, false);
        }
        let results: Vec<Option<BlockResult>> = {
            let factory = ffb_model::factory::block_result_factory::BlockResultFactory;
            self.block_roll.iter().map(|r| factory.for_roll(*r)).collect()
        };
        let single = if Self::block_would_knock_down_attacker(game, player, blitzing, &results, self.nr_of_dice < 0)
            && unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_WOULD_BE_KNOCKED_DOWN).is_some()
        {
            unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_WOULD_BE_KNOCKED_DOWN)
        } else if blitzing && unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ).is_some() {
            unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ)
        } else if unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_ONCE_PER_PERIOD).is_some() {
            unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_ONCE_PER_PERIOD)
        } else if Self::is_attacker_working_in_tandem(game, player) {
            unused_with(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_PARTNER_IS_MARKING)
        } else {
            None
        }
        .map(source_name);
        let multi = player.all_skill_ids()
            .filter(|id| unused_id(*id))
            .find(|id| id.reroll_sources_for(game.rules).iter().any(|(a, _)| *a == "MULTI_BLOCK_DICE"))
            .map(|id| { ffb_model::skill_touch::touch(id); })
            .is_some();
        (single, multi)
    }

    /// Java bb2020 `handleBrawler`: roll one die, report it, and replace the first BOTH_DOWN
    /// that no earlier skill re-roll has replaced.
    fn handle_brawler_bb2020(&mut self, game: &mut Game, rng: &mut GameRng) {
        let rerolled_die = rng.d6();
        game.report_list.add(ReportBlockReRoll::new(
            vec![rerolled_die],
            game.acting_player.player_id.clone(),
            self.re_roll_source.as_deref().map(ReRollSource::new),
        ));
        for i in 0..self.block_roll.len() {
            if block_result_for_roll(self.block_roll[i]) == BlockResult::BothDown
                && !self.re_rolled_dice_indexes.contains(&(i as i32))
            {
                self.brawler_index = i as i32;
                break;
            }
        }
        if self.brawler_index >= 0 {
            self.block_roll[self.brawler_index as usize] = rerolled_die;
            self.re_rolled_dice_indexes.push(self.brawler_index);
        }
    }

    /// Java `showBlockRollDialog`: `properties.add(ReRollProperty.TRR)` iff
    /// `UtilServerReRoll.isTeamReRollAvailable(getGameState(), actingPlayer.getPlayer())`, and
    /// only while no re-roll has been consumed yet (`getReRollSource() == null` guards the whole
    /// property-building block).
    ///
    /// The `nrOfDice < 0` branch below it cannot clear TRR: it strips the re-roll properties only
    /// when `noReRollUsed || (properties.stream().noneMatch(isActualReRoll) && actionToSource
    /// .isEmpty())`, and TRR *is* an actual re-roll, so a first pass with a team re-roll in the
    /// bank keeps the offer and addresses the dialog to the ACTING team even on 2-dice-against.
    /// (`noReRollUsed` is true only on the re-show path, which never asks again.) The die choice
    /// is unaffected either way: both engines derive it from `nr_of_dice >= 0`, not from the
    /// dialog's choosing team.
    fn team_reroll_option(&self, game: &Game) -> bool {
        if self.re_roll_source.is_some() {
            return false;
        }
        let Some(pid) = game.acting_player.player_id.as_deref() else { return false };
        let Some(player) = game.player(pid) else { return false };
        crate::util::util_server_re_roll::UtilServerReRoll::is_team_re_roll_available(game, player)
    }

    /// Java `showBlockRollDialog(noReRollUsed)`'s choice of reporting team:
    ///
    /// ```java
    /// String teamId = homePlaying ? home : away;
    /// if ((fNrOfDice < 0) && (noReRollUsed
    ///         || (properties.stream().noneMatch(ReRollProperty::isActualReRoll) && actionToSource.isEmpty())))
    ///     teamId = homePlaying ? away : home;     // the DEFENDER picks the die
    /// getResult().addReport(new ReportBlockRoll(teamId, fBlockRoll));
    /// ```
    ///
    /// `properties` / `actionToSource` are built only while no re-roll source is set, so on any
    /// pass after a re-roll both are empty. The report is the only consumer here: Rust's prompts
    /// already follow the Java dialog through `team_reroll_option` and `own_choice`.
    fn block_roll_report_team(&self, game: &Game, no_re_roll_used: bool) -> String {
        let defender_picks = self.nr_of_dice < 0
            && (no_re_roll_used || !self.dialog_has_re_roll_option(game));
        if game.home_playing != defender_picks { game.team_home.id.clone() } else { game.team_away.id.clone() }
    }

    /// Whether Java's block dialog lists any ACTUAL re-roll property (TRR, MASCOT) or any
    /// skill re-roll in `actionToSource`. Mirrors the property-building block of
    /// `showBlockRollDialog`, which runs only while `getReRollSource() == null`.
    fn dialog_has_re_roll_option(&self, game: &Game) -> bool {
        if self.re_roll_source.is_some() {
            return false;
        }
        let Some(pid) = game.acting_player.player_id.as_deref() else { return false };
        let Some(player) = game.player(pid) else { return false };
        let blitzing = game.acting_player.player_action.map(|a| a.is_blitzing()).unwrap_or(false);
        let unused = |property: &str| player.all_skill_ids()
            .any(|id| id.properties().contains(&property) && !player.used_skills.contains(&id));
        // Java UtilCards.getUnusedRerollSource(actingPlayer, action): an unused skill registering it.
        let source_for = |action: &str| player.all_skill_ids()
            .filter(|id| !player.used_skills.contains(id))
            .any(|id| id.reroll_sources_for(game.rules).iter().any(|(a, _)| *a == action));
        let factory = ffb_model::factory::block_result_factory::BlockResultFactory;
        let results: Vec<Option<BlockResult>> = self.block_roll.iter().map(|r| factory.for_roll(*r)).collect();

        // BB2020 runs this shared step, but Java's `bb2020/block/StepBlockRoll.showBlockRollDialog`
        // decides the reporting team from a different option list (no properties/actionToSource):
        // team, single-use, Pro, Consummate, Brawler, the blitz single-die skill and any-dice.
        // With no re-roll source set (guarded above) every "after a re-roll" clause is moot and
        // `reRolledDiceIndexes` is empty, so each option reduces to its availability.
        if game.rules == ffb_model::enums::Rules::Bb2020 {
            use crate::util::util_server_re_roll::UtilServerReRoll;
            let brawler = !blitzing
                && player.has_skill_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN)
                && results.contains(&Some(BlockResult::BothDown));
            return UtilServerReRoll::is_team_re_roll_available(game, player)
                || UtilServerReRoll::is_single_use_re_roll_available(game, player)
                || unused(NamedProperties::CAN_REROLL_ONCE_PER_TURN)
                || unused(NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD)
                || brawler
                || (blitzing && unused(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ))
                || unused(NamedProperties::CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE);
        }

        // singleBlock (SINGLE_BLOCK_DIE), in Java's if/else order.
        let single_block = (Self::block_would_knock_down_attacker(game, player, blitzing, &results, self.nr_of_dice < 0)
                && unused(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_WOULD_BE_KNOCKED_DOWN))
            || (blitzing && unused(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ))
            || unused(NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_ONCE_PER_PERIOD)
            || Self::is_attacker_working_in_tandem(game, player);
        if single_block
            || source_for("SINGLE_DIE")
            || source_for("MULTI_BLOCK_DICE")
            || source_for("SINGLE_DIE_PER_ACTIVATION")
        {
            return true;
        }
        // properties: TRR and MASCOT are the actual re-rolls (findAdditionalReRollProperty only ever
        // adds non-actual ones such as BRILLIANT_COACHING).
        if crate::util::util_server_re_roll::UtilServerReRoll::is_team_re_roll_available(game, player) {
            return true;
        }
        let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
        if mechanic.is_mascot_available(game, player) {
            return true;
        }
        // evaluateBrawlerAvailability: a both-down with an unused (or always-allowed) Brawler.
        if let Some(brawler) = player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN) {
            let allowed = !blitzing
                && (ffb_model::option::util_game_option::is_option_enabled(
                        game, ffb_model::option::game_option_id::ALLOW_BRAWLER_ON_BOTH_BLOCKS)
                    || !game.acting_player.used_skills.contains(&brawler));
            if allowed && results.contains(&Some(BlockResult::BothDown)) && source_for("SINGLE_BOTH_DOWN") {
                return true;
            }
        }
        // evaluateHatredAvailability: a skull against a defender carrying a hated keyword.
        if let Some(hatred) = player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_SKULL) {
            let hated = player.skill_value_excluding_temporary_ones(hatred).unwrap_or_default().to_lowercase();
            let hated_present = game.defender_id.as_deref().and_then(|d| game.player(d))
                .map(|def| def.keywords.iter().any(|k| !hated.is_empty() && hated.contains(&k.to_lowercase())))
                .unwrap_or(false);
            if hated_present && results.contains(&Some(BlockResult::Skull)) && source_for("SINGLE_SKULL") {
                return true;
            }
        }
        false
    }

    /// Java bb2025 `showBlockRollDialog(noReRollUsed)`: the dialog itself --
    /// `UtilServerDialog.showDialog(getGameState(), new DialogBlockRollPropertiesParameter(teamId,
    /// fNrOfDice, fBlockRoll, properties, convertActionsMap(actionToSource)), (fNrOfDice < 0))`.
    /// Rebuilds `properties` / `actionToSource` exactly as Java does (read-only; no state change).
    fn show_block_roll_dialog(&self, game: &mut Game, no_re_roll_used: bool) {
        use ffb_model::enums::ReRollProperty;
        use std::collections::HashMap;
        if game.rules != ffb_model::enums::Rules::Bb2025 {
            // DEVIATION: the bb2020/bb2016 Java steps show a different block dialog; the GUI
            // server runs bb2025 only.
            return;
        }
        let mut properties: Vec<ReRollProperty> = Vec::new();
        let mut action_to_source: Vec<(&'static str, String)> = Vec::new();
        if self.re_roll_source.is_none() {
            if let Some(player) = game.acting_player.player_id.as_deref().and_then(|pid| game.player(pid)) {
                let blitzing = game.acting_player.player_action.map(|a| a.is_blitzing()).unwrap_or(false);
                let unused_id = |id: ffb_model::enums::SkillId| !player.used_skills.contains(&id)
                    && !game.acting_player.used_skills.contains(&id);
                let skill_name = |id: ffb_model::enums::SkillId| id.category_and_name_for(game.rules).1.to_string();
                // Java UtilCards.getUnusedRerollSource(actingPlayer, action) -> ReRollSource.getName(game)
                let source_for = |action: &str| player.all_skill_ids()
                    .filter(|id| unused_id(*id))
                    .find(|id| id.reroll_sources_for(game.rules).iter().any(|(a, _)| *a == action))
                    .map(skill_name);
                // singleBlock (SINGLE_BLOCK_DIE), in Java's if/else order.
                if let Some(single) = self.explicit_die_options(game).0 {
                    action_to_source.push(("Single Block Die", single));
                }
                for (action, name) in [("SINGLE_DIE", "Single Die"), ("MULTI_BLOCK_DICE", "Multi Block Dice"),
                                       ("SINGLE_DIE_PER_ACTIVATION", "Single Die Per Activation")] {
                    if let Some(s) = source_for(action) {
                        action_to_source.push((name, s));
                    }
                }
                if crate::util::util_server_re_roll::UtilServerReRoll::is_team_re_roll_available(game, player) {
                    properties.push(ReRollProperty::Trr);
                }
                let factory = ffb_model::factory::block_result_factory::BlockResultFactory;
                let results: Vec<Option<BlockResult>> = self.block_roll.iter().map(|r| factory.for_roll(*r)).collect();
                // evaluateBrawlerAvailability
                if let Some(brawler) = player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_BOTH_DOWN) {
                    let allowed = !blitzing
                        && (ffb_model::option::util_game_option::is_option_enabled(
                                game, ffb_model::option::game_option_id::ALLOW_BRAWLER_ON_BOTH_BLOCKS)
                            || unused_id(brawler));
                    if allowed && results.contains(&Some(BlockResult::BothDown)) {
                        if let Some(s) = source_for("SINGLE_BOTH_DOWN") {
                            action_to_source.push(("Single BothDown", s));
                        }
                    }
                }
                // evaluateHatredAvailability
                if let Some(hatred) = player.skill_id_with_property(NamedProperties::CAN_REROLL_SINGLE_SKULL) {
                    let hated = player.skill_value_excluding_temporary_ones(hatred).unwrap_or_default().to_lowercase();
                    let hated_present = game.defender_id.as_deref().and_then(|d| game.player(d))
                        .map(|def| def.keywords.iter().any(|k| !hated.is_empty() && hated.contains(&k.to_lowercase())))
                        .unwrap_or(false);
                    if hated_present && results.contains(&Some(BlockResult::Skull)) {
                        if let Some(s) = source_for("SINGLE_SKULL") {
                            action_to_source.push(("Single Skull", s));
                        }
                    }
                }
                let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
                if let Some(p) = mechanic.find_additional_re_roll_property(game.turn_data()) {
                    properties.push(p);
                }
                if mechanic.is_mascot_available(game, player) {
                    properties.push(ReRollProperty::Mascot);
                }
            }
        }
        let mut team_id = if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() };
        if self.nr_of_dice < 0
            && (no_re_roll_used || (!properties.iter().any(|p| p.is_actual_reroll()) && action_to_source.is_empty()))
        {
            properties.retain(|p| !p.is_actual_reroll());
            action_to_source.clear();
            team_id = if game.home_playing { game.team_away.id.clone() } else { game.team_home.id.clone() };
        }
        let rr_action_to_source: HashMap<String, Option<String>> = action_to_source.into_iter()
            .map(|(k, v)| (k.to_string(), Some(v)))
            .collect();
        crate::util::util_server_dialog::UtilServerDialog::show_dialog(
            game,
            ffb_model::dialog::dialog_parameter_factory::DialogParameter::BlockRollProperties(
                ffb_model::dialog::dialog_block_roll_properties_parameter::DialogBlockRollPropertiesParameter {
                    choosing_team_id: Some(team_id),
                    nr_of_dice: self.nr_of_dice,
                    block_roll: self.block_roll.clone(),
                    re_roll_properties: properties.into_iter().map(Some).collect(),
                    rr_action_to_source,
                    ..Default::default()
                },
            ),
            self.nr_of_dice < 0,
        );
    }

    /// Java `UtilPlayer.blockWouldKnockDownAttacker(game, actingPlayer, blockRoll, defenderChooses)`.
    fn block_would_knock_down_attacker(game: &Game, player: &ffb_model::model::player::Player, blitzing: bool,
                                       results: &[Option<BlockResult>], defender_chooses: bool) -> bool {
        let _ = game;
        let both_down_safe = player.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN)
            || player.has_skill_property(NamedProperties::CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN)
            || (blitzing && player.has_skill_property(NamedProperties::CAN_CONVERT_BOTH_DOWN_TO_PUSH));
        let knocks = |r: &Option<BlockResult>| matches!(r, Some(BlockResult::Skull))
            || (matches!(r, Some(BlockResult::BothDown)) && !both_down_safe);
        if defender_chooses { results.iter().any(knocks) } else { results.iter().all(knocks) }
    }

    /// Java `UtilPlayer.isAttackerWorkingInTandem(game, attacker, defender)`: the attacker's
    /// Working in Tandem is the BLOCK variant and more than one opposing player with that skill is
    /// adjacent to the defender (`partnerMarksDefender`).
    fn is_attacker_working_in_tandem(game: &Game, player: &ffb_model::model::player::Player) -> bool {
        let Some(tandem) = player.skill_id_with_property(
            NamedProperties::CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_PARTNER_IS_MARKING) else { return false };
        let variant = player.skill_value_excluding_temporary_ones(tandem).unwrap_or_default();
        if !variant.eq_ignore_ascii_case("block") {
            return false;
        }
        let Some(def_id) = game.defender_id.as_deref() else { return false };
        let Some(def_coord) = game.field_model.player_coordinate(def_id) else { return false };
        let def_home = game.team_home.players.iter().any(|p| p.id == def_id);
        let opponents = if def_home { &game.team_away.players } else { &game.team_home.players };
        opponents.iter()
            .filter(|p| p.all_skill_ids().any(|id| id == tandem))
            .filter(|p| game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(def_coord)).unwrap_or(false))
            .count() > 1
    }

    /// The TRR half of Java's block dialog, as an agent prompt. The source is `"TRR"` because
    /// that is what `ParityRunner.reRollSourceFor` answers a block dialog with
    /// (`ReRollSources.TEAM_RE_ROLL`), and what `use_reroll` consumes.
    fn block_reroll_offer_prompt(&self, game: &Game) -> ffb_model::prompts::AgentPrompt {
        ffb_model::prompts::AgentPrompt::ReRollOffer {
            source: ReRollSource::new("TRR"),
            action: "BLOCK".to_owned(),
            re_roll_skill: None, team_re_roll_option: true,
            team_id: if game.home_playing {
                game.team_home.id.clone()
            } else {
                game.team_away.id.clone()
            },
        }
    }

    /// Java: `showBlockRollDialog(...)` — the DialogBlockRollParameter presented to the coach,
    /// as an agent prompt. `own_choice` follows Java: the attacking coach picks unless the
    /// dice are "against" (negative die count → defender's choice).
    fn block_choice_prompt(&self, game: &Game) -> ffb_model::prompts::AgentPrompt {
        ffb_model::prompts::AgentPrompt::BlockChoice {
            attacker_id: game.acting_player.player_id.clone().unwrap_or_default(),
            defender_id: game.defender_id.clone().unwrap_or_default(),
            dice: self.block_roll.clone(),
            own_choice: self.nr_of_dice >= 0,
            nr_of_dice: self.nr_of_dice,
        }
    }

    /// Java: handleImplicitReRollIndex(actingPlayer, BlockResult resultToReplace).
    ///
    /// Rolls one new block die, then finds the first die in block_roll that matches
    /// resultToReplace and replaces it. BRAWLER uses BothDown; HATRED uses Skull.
    fn handle_implicit_reroll_index(&mut self, game: &mut Game, rng: &mut GameRng, result_to_replace: BlockResult) {
        let rerolled_die = rng.d6();
        // Java: addReport(new ReportBlockReRoll(new int[]{rerolledDie}, player.getPlayerId(), getReRollSource()))
        game.report_list.add(ReportBlockReRoll::new(
            vec![rerolled_die],
            game.acting_player.player_id.clone(),
            self.re_roll_source.as_deref().map(ReRollSource::new),
        ));
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
            // The strength computation (Dauntless, Horns, weakenOpposingBlitzer, assists, Cheering
            // Fans) lives in `ServerUtilBlock::total_block_dice`, shared with the Gored by the Bull
            // decoration check so both count dice the same way.
            let defender_id = game.defender_id.clone();
            self.nr_of_dice = crate::util::server_util_block::ServerUtilBlock::total_block_dice(
                game, defender_id.as_deref(), self.successful_dauntless, self.double_target_strength);
            // Java (bb2025 :226-237, bb2020 :190-200): the selection state's canAddBlockDie skill
            // (Gored by the Bull, recorded by a StepInitSelecting/StepInitMoving/StepEndBlocking
            // CLIENT_USE_SKILL) goes into findNrOfBlockDice as addBlockDie -- one more die on a 1 or
            // 2 dice block -- and is marked used only when that die was really added.
            if let Some(skill) = crate::util::server_util_block::ServerUtilBlock::selection_state_add_block_die_skill(game) {
                if self.nr_of_dice == 1 || self.nr_of_dice == 2 {
                    self.nr_of_dice += 1;
                    if let Some(pid) = game.acting_player.player_id.clone() {
                        crate::step::util_server_steps::mark_skill_used(game, &pid, skill);
                    }
                }
            }
        }

        let source = self.re_roll_source.as_deref().unwrap_or("");

        // Java: if (source == PRO || (reRollSkill != null && reRollSkill.hasSkillProperty(canRerollSingleDieOncePerPeriod)))
        // Consummate Professional has canRerollSingleDieOncePerPeriod
        let is_pro_type = source == "Pro" || source == "ConsummateProfessional";

        let source_owned = source.to_string();
        let report = |game: &mut Game, dice: Vec<i32>| {
            game.report_list.add(ReportBlockReRoll::new(
                dice,
                game.acting_player.player_id.clone(),
                Some(ReRollSource::new(source_owned.as_str())),
            ));
        };
        if is_pro_type {
            // Java: if (source == PRO) actingPlayer.markSkillUsed(canRerollOncePerTurn);
            //       roll 1 die, addReport(ReportBlockReRoll), replace fBlockRoll[dieIndex]
            //       (bb2020 also: reRolledDiceIndexes = add(reRolledDiceIndexes, proIndex)).
            if source == "Pro" {
                let pid = game.acting_player.player_id.clone().unwrap_or_default();
                let pro = game.player(&pid)
                    .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_REROLL_ONCE_PER_TURN));
                if let Some(skill) = pro {
                    crate::step::util_server_steps::mark_skill_used(game, &pid, skill);
                }
            }
            let new_die = rng.d6();
            report(game, vec![new_die]);
            if self.die_index >= 0 {
                if let Some(slot) = self.block_roll.get_mut(self.die_index as usize) {
                    *slot = new_die;
                }
                if game.rules == ffb_model::enums::Rules::Bb2020 {
                    self.re_rolled_dice_indexes.push(self.die_index);
                }
            }
        } else if matches!(canon_source(source), "UnstoppableMomentum" | "LordOfChaos" | "WorkingInTandem" | "WoodlandFury")
            && (game.rules != ffb_model::enums::Rules::Bb2020 || canon_source(source) == "UnstoppableMomentum")
        {
            // Java: these also replace a single die at dieIndex
            if self.die_index >= 0 {
                let new_die = rng.d6();
                report(game, vec![new_die]);
                // bb2025 `actingPlayer.markSkillUsed(getReRollSource(), SINGLE_BLOCK_DIE)`;
                // bb2020 `markSkillUsed(canRerollSingleBlockDieDuringBlitz)` (bb2020 never
                // called useReRoll for Unstoppable Momentum, so nothing else marks it).
                let pid = game.acting_player.player_id.clone().unwrap_or_default();
                let skill = game.player(&pid).and_then(|p| p.all_skill_ids()
                    .find(|id| id.class_name() == canon_source(source)));
                if let Some(skill) = skill {
                    crate::step::util_server_steps::mark_skill_used(game, &pid, skill);
                }
                if let Some(slot) = self.block_roll.get_mut(self.die_index as usize) {
                    *slot = new_die;
                }
                if game.rules == ffb_model::enums::Rules::Bb2020 {
                    self.re_rolled_dice_indexes.push(self.die_index);
                }
            }
        } else if canon_source(source) == "SavageBlow" && !self.dice_indexes.is_empty() {
            // Java: markSkillUsed(canReRollAnyNumberOfBlockDice), roll diceIndexes.length dice,
            // replace each position (bb2020 also records each in reRolledDiceIndexes).
            let pid = game.acting_player.player_id.clone().unwrap_or_default();
            let skill = game.player(&pid)
                .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE));
            if let Some(skill) = skill {
                crate::step::util_server_steps::mark_skill_used(game, &pid, skill);
            }
            let n = self.dice_indexes.len();
            let new_dice: Vec<i32> = (0..n).map(|_| rng.d6()).collect();
            report(game, new_dice.clone());
            for (pos, &idx) in self.dice_indexes.iter().enumerate() {
                if let Some(slot) = self.block_roll.get_mut(idx) {
                    *slot = new_dice[pos];
                }
                if game.rules == ffb_model::enums::Rules::Bb2020 {
                    self.re_rolled_dice_indexes.push(idx as i32);
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
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
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
            is_big_guy: false,
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
    fn additional_assist_adds_a_block_die() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, PlayerState, PS_STANDING};
        fn add(game: &mut Game, home: bool, id: &str, c: FieldCoordinate) {
            let team = if home { &mut game.team_home } else { &mut game.team_away };
            team.players.push(Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
                starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
                used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
                current_spps: 0, career_spps: 0, race: None, is_big_guy: false, ..Default::default()
            });
            game.field_model.set_player_coordinate(id, c);
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        // Equal ST=3, no positional assists → 1 die; a bb2025 Cheering Fans additional assist for the
        // acting team adds +1 to the attacker strength → 2 dice.
        for (extra, expected) in [(0, 1), (1, 2)] {
            let mut game = make_game();
            add(&mut game, true, "att", FieldCoordinate::new(5, 5));
            add(&mut game, false, "def", FieldCoordinate::new(6, 5));
            game.home_playing = true;
            game.home_additional_assists = extra;
            game.acting_player.set_player("att".into(), PlayerAction::Block);
            game.defender_id = Some("def".into());
            let mut step = StepBlockRoll::new();
            step.start(&mut game, &mut GameRng::new(1));
            assert_eq!(step.nr_of_dice, expected, "extra additional assists = {}", extra);
        }
    }

    #[test]
    fn horns_adds_a_block_die_on_a_blitz_but_not_a_block() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, PlayerState, PS_STANDING, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        fn add(game: &mut Game, home: bool, id: &str, c: FieldCoordinate, skills: &[SkillId]) {
            let team = if home { &mut game.team_home } else { &mut game.team_away };
            team.players.push(Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
                starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
                extra_skills: vec![], temporary_skills: vec![],
                used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
                current_spps: 0, career_spps: 0, race: None, is_big_guy: false, ..Default::default()
            });
            game.field_model.set_player_coordinate(id, c);
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        // Equal ST=3, no assists. Java getAttackerBaseStrength adds +1 ST for the Horns
        // (addStrengthOnBlitz) skill on a BLITZ → 2 dice; a plain BLOCK stays 1 die
        // (chaos seed 1 i=100: Minotaur-vs-Minotaur blitz rolled 1 die instead of 2).
        for (action, expected) in [(PlayerAction::Blitz, 2), (PlayerAction::Block, 1)] {
            let mut game = make_game();
            add(&mut game, true, "att", FieldCoordinate::new(5, 5), &[SkillId::Horns]);
            add(&mut game, false, "def", FieldCoordinate::new(6, 5), &[]);
            game.home_playing = true;
            game.acting_player.set_player("att".into(), action);
            game.defender_id = Some("def".into());
            let mut step = StepBlockRoll::new();
            step.start(&mut game, &mut GameRng::new(1));
            assert_eq!(step.nr_of_dice, expected, "Horns on action {:?}", action);
        }
    }

    /// Regression: a SUCCESSFUL Dauntless roll must raise the attacker's block strength to at least
    /// the defender's for the die count (Java RollMechanic.getTotalAttackerStrength:
    /// `blockStrengthAttacker = max(base, defenderStrength)`). A weaker ST-3 attacker vs an ST-5
    /// defender rolls the 2-dice defender-choice block (-2) WITHOUT Dauntless, but a matched 1-die
    /// block WITH it. Rust set the successful_dauntless flag but never fed it into the count, so a
    /// Dauntless blitz (dwarf seed 1 i=101) rolled extra dice and desynced the game-die stream.
    #[test]
    fn successful_dauntless_matches_defender_strength_for_die_count() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, PlayerState, PS_STANDING};
        fn add(game: &mut Game, home: bool, id: &str, c: FieldCoordinate, strength: i32) {
            let team = if home { &mut game.team_home } else { &mut game.team_away };
            team.players.push(Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength, agility: 3, passing: 4, armour: 8,
                starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
                used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
                current_spps: 0, career_spps: 0, race: None, is_big_guy: false, ..Default::default()
            });
            game.field_model.set_player_coordinate(id, c);
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        for (dauntless, expected) in [(false, -2), (true, 1)] {
            let mut game = make_game();
            add(&mut game, true, "att", FieldCoordinate::new(5, 5), 3);
            add(&mut game, false, "def", FieldCoordinate::new(6, 5), 5);
            game.home_playing = true;
            game.acting_player.set_player("att".into(), PlayerAction::Blitz);
            game.defender_id = Some("def".into());
            let mut step = StepBlockRoll::new();
            step.successful_dauntless = dauntless;
            step.start(&mut game, &mut GameRng::new(1));
            assert_eq!(step.nr_of_dice, expected, "successful_dauntless={}", dauntless);
        }
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
        step.handle_implicit_reroll_index(&mut make_game(), &mut rng, BlockResult::BothDown);
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
        step.handle_implicit_reroll_index(&mut make_game(), &mut rng, BlockResult::BothDown);
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
        step.handle_implicit_reroll_index(&mut make_game(), &mut rng, BlockResult::Skull);
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
        step.handle_implicit_reroll_index(&mut make_game(), &mut rng, BlockResult::Skull);
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

    // ── Block re-roll offer (BACKLOG §H.14 phase 1) ───────────────────────────
    //
    // Java shows ONE dialog carrying the dice and `ReRollProperty.TRR`; Rust splits it into the
    // offer prompt then the die-choice prompt. Before this, no block was ever re-rolled in any
    // edition: 0 of 660,522 in the 2026-09-10 matrix (docs/EVENT_CENSUS_2026-09-10.md).

    /// Set up an acting attacker on the (home) acting team with a team re-roll in the bank.
    fn game_with_team_reroll() -> Game {
        let mut game = make_game();
        add_player_with_skill(&mut game, "atk", SkillId::Block);
        game.home_playing = true;
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 2;
        game.turn_data_home.reroll_used = false;
        game
    }

    #[test]
    fn block_offers_the_team_reroll_before_the_die_choice() {
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        let out = step.start(&mut game, &mut GameRng::new(1));
        match out.prompt {
            Some(ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, ref action, .. }) => {
                assert_eq!(source.name, "TRR");
                assert_eq!(action, "BLOCK");
            }
            other => panic!("expected a BLOCK ReRollOffer, got {other:?}"),
        }
        assert!(!step.block_roll.is_empty(), "the dice are rolled before the offer, as in Java");
    }

    #[test]
    fn block_asks_nothing_when_the_bank_is_empty() {
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        game.turn_data_home.rerolls = 0;
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::BlockChoice { .. })));
    }

    /// Java's `isTeamReRollAvailable` gates on `allowsTeamReRoll(turnMode)`: a block resolved
    /// during a kick-off event (Quick Snap, Blitz) may not spend one.
    #[test]
    fn block_offers_nothing_during_a_kickoff_turn_mode() {
        use ffb_model::enums::TurnMode;
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        game.set_turn_mode(TurnMode::Kickoff);
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::BlockChoice { .. })));
    }

    /// Java has no decline command: the client answers the same dialog with a die choice. The
    /// decline must therefore NOT re-enter the roll path — that would roll fresh dice.
    #[test]
    fn declining_keeps_the_same_dice_and_asks_for_the_die() {
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        step.start(&mut game, &mut GameRng::new(1));
        let rolled = step.block_roll.clone();
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(2));
        assert_eq!(step.block_roll, rolled, "a decline must not re-roll the block dice");
        assert_eq!(game.turn_data_home.rerolls, 2, "a decline must not spend a re-roll");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::BlockChoice { .. })));
    }

    #[test]
    fn accepting_spends_the_reroll_and_rolls_again() {
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        step.start(&mut game, &mut GameRng::new(1));
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(2));
        assert_eq!(step.re_rolled_action.as_deref(), Some("BLOCK"));
        assert_eq!(step.re_roll_source.as_deref(), Some("TRR"));
        assert_eq!(game.turn_data_home.rerolls, 1, "the team re-roll is spent");
        assert!(
            matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::BlockChoice { .. })),
            "after the re-roll the die choice is asked, and the offer is NOT repeated");
    }

    /// `use_reroll` rolls for Loner and spends the re-roll either way. `GameEvent::LonerRoll` had
    /// no reachable producer at all while the block offer did not exist.
    #[test]
    fn accepting_with_loner_rolls_for_it() {
        use ffb_model::events::GameEvent;
        let mut step = StepBlockRoll::new();
        let mut game = make_game();
        add_player_with_skill(&mut game, "atk", SkillId::Loner);
        game.home_playing = true;
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 2;
        step.start(&mut game, &mut GameRng::new(1));
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(2));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::LonerRoll { .. })),
            "a Loner must roll to use the team re-roll: {:?}", out.events);
        assert_eq!(game.turn_data_home.rerolls, 1, "spent either way");
    }

    #[test]
    fn a_rerolled_block_reports_rerolled_in_its_coverage_event() {
        use ffb_model::events::GameEvent;
        let mut step = StepBlockRoll::new();
        let mut game = game_with_team_reroll();
        step.start(&mut game, &mut GameRng::new(1));
        step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(2));
        let out = step.handle_command(
            &Action::BlockChoice { die_index: 0, target_id: None }, &mut game, &mut GameRng::new(3));
        let rerolled = out.events.iter().find_map(|e| match e {
            GameEvent::BlockRoll { rerolled, .. } => Some(*rerolled),
            _ => None,
        });
        assert_eq!(rerolled, Some(true));
    }

    // ── Block skill re-roll half (BACKLOG §K item 6) ─────────────────────────

    fn skill_game(rules: Rules, skills: &[SkillId]) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
        add_player_with_skill(&mut game, "atk", skills[0]);
        for s in &skills[1..] {
            game.team_home.players.last_mut().unwrap().starting_skills
                .push(SkillWithValue { skill_id: *s, value: None });
        }
        game.home_playing = true;
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Block);
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 0;
        game
    }

    fn preset(rules_dice: Vec<i32>) -> StepBlockRoll {
        let mut step = StepBlockRoll::new();
        step.nr_of_dice = rules_dice.len() as i32;
        step.block_roll = rules_dice;
        step
    }

    /// BACKLOG §M: Java bb2025 `showBlockRollDialog` maps MULTI_BLOCK_DICE to Savage Blow and the
    /// SINGLE_BLOCK_DIE chain to Unstoppable Momentum only on a blitz; bb2020's
    /// `singleBlockDieOption` likewise needs the blitz. Sources carry Java's ReRollSource names.
    #[test]
    fn explicit_die_options_follow_the_java_dialog() {
        use ffb_model::enums::PlayerAction;
        let game = skill_game(Rules::Bb2025, &[SkillId::SavageBlow]);
        let step = preset(vec![1, 1]);
        assert_eq!(step.explicit_die_options(&game), (None, true));

        for rules in [Rules::Bb2025, Rules::Bb2020] {
            let mut game = skill_game(rules, &[SkillId::UnstoppableMomentum]);
            let step = preset(vec![1, 2]);
            assert_eq!(step.explicit_die_options(&game), (None, false), "{rules:?}: not blitzing");
            game.acting_player.player_action = Some(PlayerAction::Blitz);
            assert_eq!(step.explicit_die_options(&game),
                (Some("Unstoppable Momentum".to_string()), false), "{rules:?}: blitzing");
        }

        // bb2025 builds nothing once a source is set; bb2020 keeps the multi-dice option while
        // a die is left that no skill re-roll replaced.
        let game = skill_game(Rules::Bb2025, &[SkillId::SavageBlow]);
        let mut step = preset(vec![1, 1]);
        step.re_roll_source = Some("TRR".into());
        assert_eq!(step.explicit_die_options(&game), (None, false));
        let game = skill_game(Rules::Bb2020, &[SkillId::SavageBlow]);
        let mut step = preset(vec![1, 1]);
        step.re_roll_source = Some("Pro".into());
        step.re_rolled_dice_indexes = vec![0];
        assert_eq!(step.explicit_die_options(&game), (None, true));
        step.re_rolled_dice_indexes = vec![0, 1];
        assert_eq!(step.explicit_die_options(&game), (None, false));
    }

    /// The heuristic's fixed skill-half rule (mirrored by `ParityRunner.answerBlockSkillReRoll`):
    /// after Brawler and Pro, the single-block-die source on the first Skull, else Savage Blow on
    /// every die; nothing on the defender's choice or with a good die showing.
    #[test]
    fn skill_half_answer_uses_explicit_die_skills() {
        use crate::agent::block_skill_reroll_answer;
        assert_eq!(block_skill_reroll_answer(&[2, 1], true, false, false, Some("Woodland Fury"), true),
            Action::UseSingleBlockDieReRoll { re_roll_source: "Woodland Fury".into(), die_index: 1 });
        assert_eq!(block_skill_reroll_answer(&[2, 1], true, false, false, None, true),
            Action::UseMultiBlockDiceReRoll { dice_indexes: vec![0, 1] });
        assert_eq!(block_skill_reroll_answer(&[2, 6], true, false, false, None, true),
            Action::UseReRoll { use_reroll: false });
        assert_eq!(block_skill_reroll_answer(&[2, 1], false, false, false, None, true),
            Action::UseReRoll { use_reroll: false });
    }

    /// bb2020 Java executes Savage Blow without useReRoll, so the step itself marks the skill used
    /// and records every replaced die; the report names Java's source ("Savage Blow").
    #[test]
    fn bb2020_savage_blow_marks_used_and_records_dice() {
        let mut game = skill_game(Rules::Bb2020, &[SkillId::SavageBlow]);
        let mut rng = GameRng::new(7);
        let mut step = preset(vec![1, 1]);
        step.handle_command(&Action::UseMultiBlockDiceReRoll { dice_indexes: vec![0, 1] }, &mut game, &mut rng);
        assert_eq!(step.re_roll_source.as_deref(), Some("Savage Blow"));
        assert_eq!(step.re_rolled_dice_indexes, vec![0, 1]);
        assert!(game.player("atk").unwrap().used_skills.contains(&SkillId::SavageBlow)
            || game.acting_player.used_skills.contains(&SkillId::SavageBlow));
    }

    /// bb2025 `showBlockRollDialog` maps SINGLE_BOTH_DOWN to Brawler only while a Both Down
    /// shows, and SINGLE_DIE_PER_ACTIVATION to Pro; the skill half is asked once, then the die.
    #[test]
    fn bb2025_skill_half_lists_brawler_on_a_both_down_and_pro_always() {
        use ffb_model::prompts::AgentPrompt;
        let game = skill_game(Rules::Bb2025, &[SkillId::Brawler, SkillId::Pro]);
        let mut step = preset(vec![2, 1]); // Both Down, Skull
        match step.after_team_reroll_prompt(&game) {
            AgentPrompt::BlockSkillReRoll { brawler, pro, own_choice, .. } => {
                assert!(brawler && pro && own_choice);
            }
            other => panic!("expected the skill half, got {other:?}"),
        }
        assert!(matches!(step.after_team_reroll_prompt(&game), AgentPrompt::BlockChoice { .. }),
            "asked once per dialog");

        let mut step = preset(vec![1, 6]); // no Both Down
        assert_eq!(step.skill_reroll_options(&game), (false, true));
        step.re_roll_source = Some("Pro".into());
        assert_eq!(step.skill_reroll_options(&game), (false, false),
            "bb2025 builds no options once a re-roll source is set");
    }

    /// bb2025 Brawler: useReRoll reports the skill, the first Both Down is re-rolled and
    /// `ReportBlockReRoll` names the source; the dialog re-shows with no options.
    #[test]
    fn bb2025_brawler_rerolls_the_both_down_and_reports_it() {
        use ffb_model::prompts::AgentPrompt;
        let mut game = skill_game(Rules::Bb2025, &[SkillId::Brawler]);
        let mut step = preset(vec![1, 2]);
        let out = step.handle_command(&Action::UseBrawler { target_id: None }, &mut game, &mut GameRng::new(5));
        assert_eq!(step.die_index, 1);
        assert!(matches!(out.prompt, Some(AgentPrompt::BlockChoice { .. })));
        let json: Vec<String> = game.report_list.get_reports().iter()
            .filter_map(|r| r.to_json()).map(|j| j.to_string()).collect();
        assert!(json.iter().any(|j| j.contains("blockReRoll") && j.contains("Brawler")), "{json:?}");
    }

    /// Java `RollMechanic.useReRoll`, PRO branch: spending Pro rolls the skill die, sets usedPro
    /// and reports the roll -- before the block die itself is re-rolled.
    #[test]
    fn pro_block_reroll_rolls_for_pro_and_marks_it_used() {
        let mut game = skill_game(Rules::Bb2025, &[SkillId::Pro]);
        game.field_model.set_player_state("atk", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
        let mut step = preset(vec![1, 1]);
        step.handle_command(&Action::UseProReRollForBlock { die_index: 0 }, &mut game, &mut GameRng::new(3));
        assert!(game.field_model.player_state("atk").unwrap().has_used_pro());
        assert!(game.acting_player.used_skills.contains(&SkillId::Pro));
        let json: Vec<String> = game.report_list.get_reports().iter()
            .filter_map(|r| r.to_json()).map(|j| j.to_string()).collect();
        assert!(json.iter().any(|j| j.contains("reRoll") && j.contains("Pro")), "{json:?}");
    }

    /// bb2020 Brawler skips useReRoll, replaces a not-yet-re-rolled Both Down, and leaves Pro
    /// open while a die remains un-re-rolled (`proReRollOption` after BRAWLER).
    #[test]
    fn bb2020_brawler_then_pro_stays_offered() {
        let mut game = skill_game(Rules::Bb2020, &[SkillId::Brawler, SkillId::Pro]);
        let mut step = preset(vec![2, 1]);
        assert_eq!(step.skill_reroll_options(&game), (true, true));
        step.handle_command(&Action::UseBrawler { target_id: None }, &mut game, &mut GameRng::new(5));
        assert_eq!(step.brawler_index, 0);
        assert_eq!(step.re_rolled_dice_indexes, vec![0]);
        let (brawler, pro) = step.skill_reroll_options(&game);
        assert!(!brawler, "Brawler is once per block");
        assert!(pro, "one die is still un-re-rolled");
        assert!(game.report_list.get_reports().iter().filter_map(|r| r.to_json())
            .all(|j| !j.to_string().contains("\"reRoll\"")), "bb2020 Brawler never calls useReRoll");
    }

    /// The agents' fixed rule: own choice, every die bad; Brawler before Pro; Pro on a Skull.
    #[test]
    fn block_skill_reroll_rule() {
        use crate::agent::block_skill_reroll_answer as answer;
        assert!(matches!(answer(&[2, 1], true, true, true, None, false), Action::UseBrawler { .. }));
        assert!(matches!(answer(&[2, 1], true, false, true, None, false), Action::UseProReRollForBlock { die_index: 1 }));
        assert!(matches!(answer(&[2, 2], true, false, true, None, false), Action::UseProReRollForBlock { die_index: 0 }));
        assert!(matches!(answer(&[2, 6], true, true, true, None, false), Action::UseReRoll { use_reroll: false }));
        assert!(matches!(answer(&[1], false, true, true, None, false), Action::UseReRoll { use_reroll: false }));
    }
}
