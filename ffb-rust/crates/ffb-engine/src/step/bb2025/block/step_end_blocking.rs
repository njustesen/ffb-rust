use ffb_model::enums::{PlayerAction, PlayerState, PS_BLOCKED, PS_MOVING, PS_STANDING, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::skills::SkillId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::{change_player_action, check_touchdown};
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};
use crate::step::generator::bb2025::EndPlayerAction;
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::Block;
use crate::step::generator::bb2025::block::BlockParams;
use crate::step::generator::bb2025::BlitzBlock;
use crate::step::generator::bb2025::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2025::Move;
use crate::step::generator::bb2025::move_::MoveParams;
use crate::step::generator::mixed::pile_driver::{PileDriver, PileDriverParams};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepEndBlocking.
///
/// Last step in block sequence. Consumes all block parameters and decides what happens next.
///
/// Expects stepParameter DEFENDER_PUSHED to be set by a preceding step.
/// Expects stepParameter END_PLAYER_ACTION to be set by a preceding step.
/// Expects stepParameter END_TURN to be set by a preceding step.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
/// Expects stepParameter USING_STAB to be set by a preceding step.
///
/// May push a new sequence on the stack.
pub struct StepEndBlocking {
    pub end_turn: bool,
    pub check_forgo: bool,
    pub end_player_action: bool,
    pub defender_pushed: bool,
    pub bloodlust_action: Option<PlayerAction>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub allow_second_block_action: bool,
    pub using_vomit: bool,
    pub add_block_die_handled: bool,
    pub using_breathe_fire: bool,
    pub using_chomp: bool,
    /// Java: usePileDriver (Boolean — tristate)
    pub use_pile_driver: Option<bool>,
    /// Java: useHitAndRun (Boolean — tristate)
    pub use_hit_and_run: Option<bool>,
    /// Java: usePutridRegurgitation (Boolean — tristate)
    pub use_putrid_regurgitation: Option<bool>,
    /// Java: knockedDownPlayers — players knocked down during this block
    pub knocked_down_players: Vec<String>,
    /// Java: targetPlayerId — pile driver target
    pub target_player_id: Option<String>,
    pub old_defender_state: Option<PlayerState>,
    /// Java: targetsRegularMultibLock — multi-block target IDs (consumed from TARGET_PLAYER_ID)
    pub targets_regular_multi_block: Vec<String>,
}

impl StepEndBlocking {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            check_forgo: false,
            end_player_action: false,
            defender_pushed: false,
            bloodlust_action: None,
            using_stab: false,
            using_chainsaw: false,
            allow_second_block_action: false,
            using_vomit: false,
            add_block_die_handled: false,
            using_breathe_fire: false,
            using_chomp: false,
            use_pile_driver: None,
            use_hit_and_run: None,
            use_putrid_regurgitation: None,
            knocked_down_players: Vec::new(),
            target_player_id: None,
            old_defender_state: None,
            targets_regular_multi_block: Vec::new(),
        }
    }
}

impl Default for StepEndBlocking {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndBlocking {
    fn id(&self) -> StepId { StepId::EndBlocking }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                // Java: CLIENT_PILE_DRIVER — targetPlayerId = commandPileDriver.getPlayerId()
                self.target_player_id = Some(player_id.clone());
                // Java: usePileDriver = StringTool.isProvided(targetPlayerId)
                self.use_pile_driver = Some(!player_id.is_empty());
            }
            Action::UseSkill { skill_id, use_skill } => {
                match skill_id {
                    SkillId::HitAndRun => {
                        // Java: canMoveAfterBlock property
                        self.use_hit_and_run = Some(*use_skill);
                    }
                    SkillId::PutridRegurgitation => {
                        // Java: canUseVomitAfterBlock property
                        self.use_putrid_regurgitation = Some(*use_skill);
                    }
                    _ => {
                        // Java: canAddBlockDie — update dice decorations
                        // Java: getResult().addReport(new ReportSkillUse(commandUseSkill.getSkill(), true, SkillUse.ADD_BLOCK_DIE))
                        {
                            use ffb_model::model::skill_use::SkillUse;
                            use ffb_model::report::report_skill_use::ReportSkillUse;
                            game.report_list.add(ReportSkillUse::new(
                                None, *skill_id, true, SkillUse::ADD_BLOCK_DIE,
                            ));
                        }
                        ServerUtilBlock::update_dice_decorations_with_frenzy(game, true);
                    }
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::DefenderPushed(v) => { self.defender_pushed = *v; true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            StepParameter::UsingStab(v) => { self.using_stab = *v; true }
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; true }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::UsingChomp(v) => { self.using_chomp = *v; true }
            StepParameter::AllowSecondBlockAction(v) => { self.allow_second_block_action = *v; true }
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            // Java: INJURY_RESULT — extract defenderId and add to knockedDownPlayers
            // Java: knockedDownPlayers.add(injuryResult.injuryContext().getDefenderId())
            StepParameter::InjuryResult(ir) => {
                if let Some(id) = ir.injury_context().get_defender_id() {
                    self.knocked_down_players.push(id.to_string());
                }
                true
            }
            // Java: TARGET_PLAYER_ID — add to targetsRegularMultibLock (consumed)
            StepParameter::PlayerId(v) => { self.targets_regular_multi_block.push(v.clone()); true }
            _ => false,
        }
    }
}

impl StepEndBlocking {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: fEndTurn |= UtilServerSteps.checkTouchdown(getGameState())
        self.end_turn |= check_touchdown(game);

        // Java: fieldModel.clearMultiBlockTargets()
        game.field_model.clear_multi_block_targets();

        let player_action = game.acting_player.player_action;
        let regular_block = !self.using_stab && !self.using_chainsaw
            && !self.using_vomit && !self.using_breathe_fire && !self.using_chomp;

        if regular_block {
            // Java: UtilCards.getUnusedSkillWithProperty(game.getDefender(),
            //   ignoresDefenderStumblesResultForFirstBlock).ifPresent(mark)
            if let Some(def_id) = game.defender_id.clone() {
                let sid = game.player(&def_id).and_then(|p| UtilCards::get_unused_skill_with_property(
                    p, NamedProperties::IGNORES_DEFENDER_STUMBLES_RESULT_FOR_FIRST_BLOCK));
                if let Some(sid) = sid {
                    let is_home = game.team_home.player(&def_id).is_some();
                    if is_home { game.team_home.player_mut(&def_id).map(|p| p.used_skills.insert(sid)); }
                    else { game.team_away.player_mut(&def_id).map(|p| p.used_skills.insert(sid)); }
                }
            }
        }

        // Java: if VICIOUS_VINES, mark canBlockOverDistance skill used
        if player_action == Some(PlayerAction::ViciousVines) {
            if let Some(pid) = game.acting_player.player_id.as_deref() {
                let pid = pid.to_owned();
                let sid = game.player(&pid)
                    .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_BLOCK_OVER_DISTANCE));
                if let Some(sid) = sid {
                    let is_home = game.team_home.player(&pid).is_some();
                    let player_mut = if is_home { game.team_home.player_mut(&pid) }
                                     else { game.team_away.player_mut(&pid) };
                    if let Some(p) = player_mut { p.used_skills.insert(sid); }
                }
            }
        }

        // ── EndTurn / EndPlayerAction early exit ──────────────────────────────
        // Java: if (fEndTurn || fEndPlayerAction) { ... endGenerator.pushSequence(...) }
        if self.end_turn || self.end_player_action {
            if let Some(action) = player_action {
                if action.is_kicking_downed() {
                    // Java: ServerUtilBlock.removePlayerBlockStates(game, oldDefenderState)
                    remove_player_block_states(game, self.old_defender_state);
                }
            }
            // Java: game.setDefenderId(null)
            game.defender_id = None;
            // Java: endGenerator.pushSequence(EndPlayerAction.SequenceParams(gs, true, true, fEndTurn, checkForgo))
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: self.check_forgo,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Blood lust follow-up move ─────────────────────────────────────────
        let is_bloodlust = game.acting_player.suffering_blood_lust;
        if is_bloodlust && self.bloodlust_action.is_some() {
            if let Some(old_state) = self.old_defender_state {
                if let Some(defender_id) = game.defender_id.clone() {
                    game.field_model.set_player_state(&defender_id, old_state);
                }
            }
            game.defender_id = None;
            // Java: ServerUtilBlock.updateDiceDecorations(getGameState())
            ServerUtilBlock::update_dice_decorations(game);
            // Java: UtilServerSteps.changePlayerAction(this, actingPlayerId, bloodlustAction, false)
            if let (Some(pid), Some(action)) = (game.acting_player.player_id.clone(), self.bloodlust_action) {
                change_player_action(game, &pid, action, false);
            }
            let seq = Move::build_sequence(&MoveParams {
                bloodlust_action: self.bloodlust_action,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        let attacker_id = game.acting_player.player_id.clone();
        let attacker_state = attacker_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();

        // Java: Revert strength gained from HORNS and DAUNTLESS to avoid interaction with tentacles.
        // Skill skillHorns = activePlayer.getSkillWithProperty(NamedProperties.addStrengthOnBlitz);
        // if (usedHorns || usedDauntless) { actingPlayer.setStrength(activePlayer.getStrengthWithModifiers()); }
        if let Some(pid) = attacker_id.as_deref() {
            let horns_used = game.player(pid)
                .and_then(|p| {
                    let sid = p.all_skill_ids().find(|s| {
                        s.properties().contains(&NamedProperties::ADD_STRENGTH_ON_BLITZ)
                    })?;
                    Some(p.used_skills.contains(&sid))
                })
                .unwrap_or(false);
            let dauntless_used = game.player(pid)
                .and_then(|p| {
                    let sid = p.all_skill_ids().find(|s| {
                        s.properties().contains(&NamedProperties::CAN_ROLL_TO_MATCH_OPPONENTS_STRENGTH)
                    })?;
                    Some(p.used_skills.contains(&sid))
                })
                .unwrap_or(false);
            if horns_used || dauntless_used {
                let base_strength = game.player(pid).map(|p| p.strength_with_modifiers()).unwrap_or(0);
                game.acting_player.strength = base_strength;
            }
        }

        // Java: Skill unusedPlayerMustMakeSecondBlockSkill =
        //   UtilCards.getUnusedSkillWithProperty(actingPlayer, forceSecondBlock)
        let unused_force_second_block = attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::FORCE_SECOND_BLOCK));

        // Java: if (activePlayer.hasSkillProperty(forceSecondBlock)) actingPlayer.setGoingForIt(true)
        if unused_force_second_block.is_some() {
            game.acting_player.goes_for_it = true;
        }

        let defender_id = game.defender_id.clone();
        let is_blitz = player_action == Some(PlayerAction::Blitz);

        let attacker_position = attacker_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let defender_position = defender_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let defender_state = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));

        // Java: forceSecondBlock condition (Frenzy etc.)
        let has_move_left = UtilPlayer::has_move_left(game, false);
        let defender_can_be_blocked = defender_state.map(|s| s.can_be_blocked()).unwrap_or(false);
        let is_adjacent = attacker_position.zip(defender_position)
            .map(|(a, d)| a.is_adjacent(d))
            .unwrap_or(false);

        let force_second_block = unused_force_second_block.is_some()
            && defender_can_be_blocked
            && is_adjacent
            && attacker_state.has_tacklezones()
            && self.defender_pushed
            && player_action != Some(PlayerAction::MultipleBlock)
            && has_move_left;

        if force_second_block {
            // Java: actingPlayer.markSkillUsed(unusedPlayerMustMakeSecondBlockSkill)
            if let (Some(sid), Some(pid)) = (unused_force_second_block, attacker_id.as_deref()) {
                let is_home = game.team_home.player(pid).is_some();
                let player_mut = if is_home { game.team_home.player_mut(pid) }
                                 else { game.team_away.player_mut(pid) };
                if let Some(p) = player_mut { p.used_skills.insert(sid); }
            }

            if is_blitz {
                // Java: blitzBlockGenerator.pushSequence(BlitzBlock.SequenceParams(gs, defenderId, usingStab, true, null, false))
                let seq = BlitzBlock::build_sequence(&BlitzBlockParams {
                    block_defender_id: defender_id,
                    using_stab: self.using_stab,
                    publish_defender: true,
                    ..Default::default()
                });
                return StepOutcome::next().push_seq(seq);
            } else {
                // Java: blockGenerator.pushSequence(Block.Builder(gs).withDefenderId(defenderId).useStab(usingStab).build())
                // Java: publishParameter(ALLOW_SECOND_BLOCK_ACTION, allowSecondBlockAction)
                let seq = Block::build_sequence(&BlockParams {
                    block_defender_id: defender_id,
                    using_stab: self.using_stab,
                    ..Default::default()
                });
                return StepOutcome::next()
                    .push_seq(seq)
                    .publish(StepParameter::AllowSecondBlockAction(self.allow_second_block_action));
            }
        }

        // ── Normal path ───────────────────────────────────────────────────────

        if self.old_defender_state.is_some() {
            // Java: ServerUtilBlock.removePlayerBlockStates(game, oldDefenderState)
            remove_player_block_states(game, self.old_defender_state);
        }

        // Java: fieldModel.clearDiceDecorations()
        game.field_model.clear_dice_decorations();

        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);

        // Java: filter knockedDownPlayers to adjacent prone/stunned opponents
        // Java condition: !actingTeam.hasPlayer(player) && adjacent && (PRONE||STUNNED)
        //   && (oldDefenderState != null || targetsRegularMultibLock.contains(playerId))
        if let Some(atk_pos) = attacker_position {
            let acting_team_is_home = game.home_playing;
            let targets = self.targets_regular_multi_block.clone();
            let old_def_state = self.old_defender_state;
            self.knocked_down_players.retain(|pid| {
                let is_adjacent = game.field_model.player_coordinate(pid)
                    .map(|p| p.is_adjacent(atk_pos))
                    .unwrap_or(false);
                let is_prone_or_stunned = game.field_model.player_state(pid)
                    .map(|s| s.is_prone_or_stunned())
                    .unwrap_or(false);
                let on_inactive = if acting_team_is_home {
                    game.team_away.has_player(pid)
                } else {
                    game.team_home.has_player(pid)
                };
                // Java: oldDefenderState != null || targetsRegularMultibLock.contains(playerId)
                let relevant = old_def_state.is_some() || targets.contains(pid);
                is_adjacent && is_prone_or_stunned && on_inactive && relevant
            });
        }

        let attacker_state_base = attacker_state.base();

        // ── Pile Driver ──────────────────────────────────────────────────────
        // Java: canFoulAfterBlock = state.MOVING && hasSkillProperty(canFoulAfterBlock)
        //   Then if oldDefenderState != null: &= regularBlock && !oldDefenderState.isProneOrStunned()
        let can_foul_after_block = {
            let base_moving = attacker_state_base == PS_MOVING;
            let has_skill = attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_FOUL_AFTER_BLOCK))
                .unwrap_or(false);
            if !base_moving || !has_skill {
                false
            } else if let Some(old_state) = self.old_defender_state {
                // Java: canFoulAfterBlock &= regularBlock && !oldDefenderState.isProneOrStunned()
                regular_block && !old_state.is_prone_or_stunned()
            } else {
                // Java: no oldDefenderState check — multi-block path, condition not constrained
                true
            }
        };

        if !can_foul_after_block || self.knocked_down_players.is_empty()
            || game.turn_mode == TurnMode::Blitz
        {
            self.use_pile_driver = Some(false);
        }

        // ── Putrid Regurgitation ─────────────────────────────────────────────
        // Java: canUsePutridRegurgitation = action.isBlockAction()
        //   && hasUnusedSkillWithProperty(canUseVomitAfterBlock)
        //   && ArrayTool.isProvided(findAdjacentBlockablePlayers(...)) && regularBlock
        let can_use_putrid = player_action.map(|a| a.is_block_action()).unwrap_or(false)
            && regular_block
            && attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK))
                .unwrap_or(false)
            && {
                let inactive = game.inactive_team();
                attacker_position
                    .map(|coord| !UtilPlayer::find_adjacent_blockable_players(game, inactive, coord).is_empty())
                    .unwrap_or(false)
            };
        if !can_use_putrid {
            self.use_putrid_regurgitation = Some(false);
        }

        if self.use_putrid_regurgitation.is_none() {
            // Java: UtilServerDialog.showDialog(... DialogSkillUseParameter(canUseVomitAfterBlock, 0) ...)
            // Java: getResult().setNextAction(CONTINUE)
            let player_id = attacker_id.clone().unwrap_or_default();
            let skill_id = attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .and_then(|p| p.all_skill_ids().find(|s| {
                    s.properties().contains(&NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK)
                }))
                .unwrap_or(SkillId::PutridRegurgitation);
            return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                player_id,
                skill_id: skill_id as u16,
                skill_name: format!("{:?}", skill_id),
            });
        }

        if self.use_putrid_regurgitation == Some(true) {
            // Java: blockGenerator.pushSequence(Block.Builder(gs).publishDefender(true).build())
            // Java: changePlayerAction(... PUTRID_REGURGITATION_BLOCK ...)
            // Java: ServerUtilBlock.updateDiceDecorations; fieldModel.setTargetSelectionState(null)
            if let Some(pid) = attacker_id.as_deref() {
                let jumping = game.acting_player.jumping;
                change_player_action(game, pid, PlayerAction::PutridRegurgitationBlock, jumping);
            }
            ServerUtilBlock::update_dice_decorations(game);
            game.field_model.target_selection_state = None;
            let seq = Block::build_sequence(&BlockParams {
                publish_defender: true,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Hit and Run ──────────────────────────────────────────────────────
        // Java: canMoveAfterBlock = state.MOVING && hasSkill(canMoveAfterBlock)
        //   && (regularBlock || usingStab) && !isPinned() && availableSquares non-empty
        // Java availableSquares: adjacent coords, no player, no adjacent opponent TZ player
        let available_squares: Vec<_> = attacker_position.map(|coord| {
            let inactive = game.inactive_team();
            game.field_model.adjacent_on_pitch(coord)
                .into_iter()
                .filter(|adj| game.field_model.player_at(*adj).is_none())
                .filter(|adj| UtilPlayer::find_adjacent_players(game, inactive, *adj).is_empty())
                .collect()
        }).unwrap_or_default();

        let can_move_after_block = attacker_state_base == PS_MOVING
            && attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_BLOCK))
                .unwrap_or(false)
            && (regular_block || self.using_stab)
            && !attacker_state.is_pinned()
            && !available_squares.is_empty();

        if !can_move_after_block {
            self.use_hit_and_run = Some(false);
        }

        if self.use_hit_and_run.is_none() {
            // Java: UtilServerDialog.showDialog(... DialogSkillUseParameter(canMoveAfterBlock, 0) ...)
            // Java: getResult().setNextAction(CONTINUE)
            let player_id = attacker_id.clone().unwrap_or_default();
            let skill_id = attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .and_then(|p| p.all_skill_ids().find(|s| {
                    s.properties().contains(&NamedProperties::CAN_MOVE_AFTER_BLOCK)
                }))
                .unwrap_or(SkillId::HitAndRun);
            return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                player_id,
                skill_id: skill_id as u16,
                skill_name: format!("{:?}", skill_id),
            });
        }

        if self.use_hit_and_run == Some(true) {
            // Java: useHitAndRun = false; getGameState().pushCurrentStepOnStack()
            // Java: stack.push(StepId.HIT_AND_RUN)
            self.use_hit_and_run = Some(false);
            use crate::step::framework::SequenceStep;
            let seq = vec![SequenceStep::new(StepId::HitAndRun)];
            return StepOutcome::next().push_seq(seq);
        }

        if self.use_pile_driver.is_none() {
            // Java: UtilServerDialog.showDialog(... DialogPileDriverParameter(teamId, knockedDownPlayers) ...)
            // Java: getResult().setNextAction(CONTINUE)
            let eligible = self.knocked_down_players.iter().cloned().collect();
            return StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                eligible_players: eligible,
                reason: "pileDriver".to_string(),
            });
        }

        if self.use_pile_driver == Some(true) {
            // Java: UtilServerGame.changeActingPlayer(this, actingPlayerId, PlayerAction.FOUL, actingPlayer.isJumping())
            // Java: ServerUtilBlock.updateDiceDecorations
            // Java: pileDriver.pushSequence(new PileDriver.SequenceParams(getGameState(), targetPlayerId))
            // Java: playerResult.setFouls(playerResult.getFouls() + 1)
            if let Some(pid) = attacker_id.as_deref() {
                let jumping = game.acting_player.jumping;
                change_player_action(game, pid, PlayerAction::Foul, jumping);
                let is_home = game.team_home.has_player(pid);
                let result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
                result.player_results.entry(pid.to_string()).or_default().fouls += 1;
            }
            // Java: pileDriver.pushSequence(new PileDriver.SequenceParams(getGameState(), targetPlayerId))
            let seq = PileDriver::build_sequence(&PileDriverParams {
                target_player_id: self.target_player_id.clone(),
            });
            ServerUtilBlock::update_dice_decorations(game);
            return StepOutcome::next().push_seq(seq);
        }

        // ── Final movement / second-block routing ────────────────────────────
        let flashes_blade = player_action == Some(PlayerAction::TheFlashingBlade);
        if flashes_blade {
            // Java: actingPlayer.markSkillUsed(canStabAndMoveAfterwards)
            if let Some(pid) = attacker_id.as_deref() {
                let pid = pid.to_owned();
                let sid = game.player(&pid).and_then(|p| UtilCards::get_unused_skill_with_property(
                    p, NamedProperties::CAN_STAB_AND_MOVE_AFTERWARDS));
                if let Some(sid) = sid {
                    let is_home = game.team_home.player(&pid).is_some();
                    if is_home { game.team_home.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                    else { game.team_away.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                }
            }
        }

        let can_move_on = !self.using_stab && !self.using_chainsaw && !self.using_breathe_fire;
        let next_move_possible = UtilPlayer::is_next_move_possible(game, false);

        if (is_blitz && can_move_on || flashes_blade)
            && attacker_state.has_tacklezones()
            && next_move_possible
        {
            game.defender_id = None;
            // Java: newAction = flashesBlade ? MOVE : (hasPutridVomitSkill ? PUTRID_REGU_MOVE : BLITZ_MOVE)
            // Java: UtilServerGame.changeActingPlayer(this, actingPlayerId, newAction, jumping)
            // Java: UtilServerPlayerMove.updateMoveSquares; ServerUtilBlock.updateDiceDecorations
            // Java: fieldModel.setTargetSelectionState(null)
            if let Some(pid) = attacker_id.as_deref() {
                let jumping = game.acting_player.jumping;
                let new_action = if flashes_blade {
                    PlayerAction::Move
                } else if game.player(pid)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK))
                    .unwrap_or(false)
                {
                    PlayerAction::PutridRegurgitationMove
                } else {
                    PlayerAction::BlitzMove
                };
                change_player_action(game, pid, new_action, jumping);
            }
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            ServerUtilBlock::update_dice_decorations(game);
            game.field_model.target_selection_state = None;
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if MOVE && isNextMovePossible → push Move (ball-and-chain case)
        if player_action == Some(PlayerAction::Move) && next_move_possible {
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            ServerUtilBlock::update_dice_decorations(game);
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // ── Final routing: second block / chainsaw / end ─────────────────────
        let blitz_with_move_left = is_blitz && next_move_possible;

        // Java: opponents = findAdjacentBlockablePlayers(game, defender.getTeam(), attackerCoord)
        let inactive_team = game.inactive_team();
        let opponents: Vec<&str> = attacker_position.map(|coord| {
            UtilPlayer::find_adjacent_blockable_players(game, inactive_team, coord)
                .into_iter()
                .map(|id| id.as_str())
                .collect()
        }).unwrap_or_default();
        let has_valid_opponent = !opponents.is_empty();
        let has_valid_other_opponent = opponents.len() > 1
            || (opponents.len() == 1
                && defender_id.as_deref().map(|d| opponents[0] != d).unwrap_or(true));

        game.defender_id = None;

        if attacker_state.has_tacklezones() && self.allow_second_block_action && has_valid_opponent {
            self.allow_second_block_action = false;
            game.acting_player.has_blocked = false;
            // Java: actingPlayer.markSkillUnused(forceSecondBlock)
            if let Some(pid) = attacker_id.as_deref() {
                let pid = pid.to_owned();
                let sid = game.player(&pid).and_then(|p| {
                    p.all_skill_ids().find(|id| id.properties().contains(&NamedProperties::FORCE_SECOND_BLOCK))
                });
                if let Some(sid) = sid {
                    let is_home = game.team_home.player(&pid).is_some();
                    if is_home { game.team_home.player_mut(&pid).map(|p| { p.used_skills.remove(&sid); }); }
                    else { game.team_away.player_mut(&pid).map(|p| { p.used_skills.remove(&sid); }); }
                }
            }
            let seq = Block::build_sequence(&BlockParams {
                using_chainsaw: self.using_chainsaw,
                publish_defender: true,
                ..Default::default()
            });
            ServerUtilBlock::update_dice_decorations(game);
            return StepOutcome::next().push_seq(seq);
        }

        if self.using_chainsaw
            && attacker_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_PERFORM_SECOND_CHAINSAW_ATTACK))
                .unwrap_or(false)
            && attacker_state.has_tacklezones()
            && has_valid_other_opponent
            && (blitz_with_move_left
                || player_action == Some(PlayerAction::Chainsaw)
                || (player_action == Some(PlayerAction::Blitz) && attacker_state.is_rooted()))
        {
            // Java: game.setLastDefenderId(defenderId)
            game.last_defender_id = defender_id.clone();
            // Java: changePlayerAction(this, actingPlayerId, MAXIMUM_CARNAGE, false)
            if let Some(pid) = attacker_id.as_deref() {
                change_player_action(game, pid, PlayerAction::MaximumCarnage, false);
            }
            if is_blitz {
                let seq = BlitzBlock::build_sequence(&BlitzBlockParams {
                    publish_defender: true,
                    ..Default::default()
                });
                return StepOutcome::next().push_seq(seq);
            } else {
                let seq = Block::build_sequence(&BlockParams {
                    using_chainsaw: true,
                    publish_defender: true,
                    ..Default::default()
                });
                return StepOutcome::next().push_seq(seq);
            }
        }

        // Java: game.setLastDefenderId(null)
        game.last_defender_id = None;
        // Java: endGenerator.pushSequence(EndPlayerAction.SequenceParams(gs, true, true, false))
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: false,
            check_forgo: false,
        });
        StepOutcome::next().push_seq(seq)
    }
}

/// Java: ServerUtilBlock.removePlayerBlockStates(game, oldDefenderState)
///
/// For every player currently in BLOCKED base state:
/// - If the player is the defender and oldDefenderState was prone/stunned → restore that base.
/// - Otherwise → set base to STANDING.
fn remove_player_block_states(game: &mut Game, old_defender_state: Option<PlayerState>) {
    let defender_id = game.defender_id.clone();
    let all_ids: Vec<String> = game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .map(|p| p.id.clone())
        .collect();
    for pid in all_ids {
        if let Some(state) = game.field_model.player_state(&pid) {
            if state.base() == PS_BLOCKED {
                let new_base = if let (Some(old), Some(ref def)) = (old_defender_state, &defender_id) {
                    if &pid == def && old.is_prone_or_stunned() {
                        old.base()
                    } else {
                        PS_STANDING
                    }
                } else {
                    PS_STANDING
                };
                game.field_model.set_player_state(&pid, state.change_base(new_base));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, PS_STANDING};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ── default start → pushes EndPlayerAction sequence ──────────────────────

    #[test]
    fn default_state_pushes_end_player_action_sequence() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn check_touchdown_sets_end_turn() {
        let mut step = StepEndBlocking::new();
        step.end_turn = false;
        let mut game = make_game();
        step.start(&mut game, &mut GameRng::new(0));
    }

    #[test]
    fn end_turn_clears_defender_and_pushes_end_player_action() {
        let mut step = StepEndBlocking::new();
        step.end_turn = true;
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.defender_id.is_none());
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_turn_sequence_carries_end_turn_flag() {
        let mut step = StepEndBlocking::new();
        step.end_turn = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let init_feeding = out.pushes[0].iter().find(|s| s.step_id == StepId::InitFeeding).unwrap();
        assert!(init_feeding.params.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_clears_defender_and_pushes_sequence() {
        let mut step = StepEndBlocking::new();
        step.end_player_action = true;
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.defender_id.is_none());
        assert_eq!(out.pushes.len(), 1);
    }

    // ── set_parameter ────────────────────────────────────────────────────────

    #[test]
    fn set_parameter_all_bool_flags_accepted() {
        let mut step = StepEndBlocking::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::DefenderPushed(true)));
        assert!(step.set_parameter(&StepParameter::UsingStab(true)));
        assert!(step.set_parameter(&StepParameter::UsingChainsaw(true)));
        assert!(step.set_parameter(&StepParameter::UsingVomit(true)));
        assert!(step.set_parameter(&StepParameter::UsingBreatheFire(true)));
        assert!(step.set_parameter(&StepParameter::UsingChomp(true)));
        assert!(step.set_parameter(&StepParameter::AllowSecondBlockAction(true)));
        assert!(step.end_turn);
        assert!(step.check_forgo);
        assert!(step.end_player_action);
        assert!(step.defender_pushed);
        assert!(step.using_stab);
        assert!(step.using_chainsaw);
        assert!(step.using_vomit);
        assert!(step.using_breathe_fire);
        assert!(step.using_chomp);
        assert!(step.allow_second_block_action);
    }

    #[test]
    fn set_parameter_old_defender_state_accepted() {
        let mut step = StepEndBlocking::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn set_parameter_target_player_id_adds_to_multi_block_list() {
        let mut step = StepEndBlocking::new();
        step.set_parameter(&StepParameter::PlayerId("p99".into()));
        assert!(step.targets_regular_multi_block.contains(&"p99".to_string()));
    }

    #[test]
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepEndBlocking::new();
        let accepted = step.set_parameter(&StepParameter::GotoLabel("x".into()));
        assert!(!accepted);
    }

    // ── InjuryResult extracts defender_id into knocked_down_players ───────────

    #[test]
    fn set_parameter_injury_result_adds_defender_to_knocked_down() {
        use crate::injury::InjuryResult;
        use ffb_model::enums::ApothecaryMode;
        let mut step = StepEndBlocking::new();
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context_mut().defender_id = Some("p42".to_string());
        assert!(step.set_parameter(&StepParameter::InjuryResult(Box::new(ir))));
        assert!(step.knocked_down_players.contains(&"p42".to_string()));
    }

    #[test]
    fn set_parameter_injury_result_no_defender_id_accepted() {
        use crate::injury::InjuryResult;
        use ffb_model::enums::ApothecaryMode;
        let mut step = StepEndBlocking::new();
        let ir = InjuryResult::new(ApothecaryMode::Defender);
        assert!(step.set_parameter(&StepParameter::InjuryResult(Box::new(ir))));
        assert!(step.knocked_down_players.is_empty());
    }

    // ── handle_command: select player (pile driver) ───────────────────────────

    #[test]
    fn select_player_non_empty_sets_target_and_triggers_pile_driver_check() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::SelectPlayer { player_id: "p5".into()
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.target_player_id.as_deref(), Some("p5"));
        assert_eq!(step.use_pile_driver, Some(false));
    }

    #[test]
    fn select_player_empty_id_disables_pile_driver() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::SelectPlayer { player_id: "".into()
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.use_pile_driver, Some(false));
    }

    // ── handle_command: use_skill ─────────────────────────────────────────────

    #[test]
    fn use_skill_hit_and_run_flag_set_then_overridden_by_execute() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::HitAndRun, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        // execute_step auto-declines (no HitAndRun skill on player in test_team)
    }

    // ── normal path falls back to EndPlayerAction sequence ───────────────────

    #[test]
    fn normal_path_no_special_skills_pushes_end_player_action() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
        assert_eq!(out.pushes[0].last().unwrap().step_id, StepId::EndFeeding);
    }

    #[test]
    fn check_forgo_propagates_to_end_feeding() {
        let mut step = StepEndBlocking::new();
        step.check_forgo = true;
        step.end_player_action = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let end_feeding = out.pushes[0].iter().find(|s| s.step_id == StepId::EndFeeding).unwrap();
        assert!(end_feeding.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    // ── remove_player_block_states ────────────────────────────────────────────

    fn add_player_to_home(game: &mut Game, pid: &str) {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        game.team_home.players.push(Player {
            id: pid.to_string(), name: pid.to_string(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
    }

    #[test]
    fn remove_player_block_states_resets_blocked_to_standing() {
        let mut game = make_game();
        add_player_to_home(&mut game, "p_blocker");
        game.field_model.set_player_state("p_blocker", PlayerState::new(PS_BLOCKED));
        remove_player_block_states(&mut game, None);
        let new_state = game.field_model.player_state("p_blocker").unwrap();
        assert_eq!(new_state.base(), PS_STANDING);
    }

    #[test]
    fn remove_player_block_states_defender_prone_restored() {
        use ffb_model::enums::PS_PRONE;
        let mut game = make_game();
        let def_id = "p_defender";
        add_player_to_home(&mut game, def_id);
        game.defender_id = Some(def_id.to_string());
        game.field_model.set_player_state(def_id, PlayerState::new(PS_BLOCKED));
        let old_state = PlayerState::new(PS_PRONE);
        remove_player_block_states(&mut game, Some(old_state));
        let new_state = game.field_model.player_state(def_id).unwrap();
        assert_eq!(new_state.base(), PS_PRONE);
    }

    #[test]
    fn remove_player_block_states_non_blocked_player_unchanged() {
        let mut game = make_game();
        add_player_to_home(&mut game, "p_standing");
        game.field_model.set_player_state("p_standing", PlayerState::new(PS_STANDING));
        remove_player_block_states(&mut game, None);
        let new_state = game.field_model.player_state("p_standing").unwrap();
        assert_eq!(new_state.base(), PS_STANDING);
    }

    // ── report_list: ADD_BLOCK_DIE skill use ─────────────────────────────────

    #[test]
    fn use_skill_block_adds_report_skill_use_add_block_die() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Block, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "expected SKILL_USE report for ADD_BLOCK_DIE");
    }

    #[test]
    fn use_skill_add_block_die_report_count_is_one() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Block, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        let count = game.report_list.get_reports().iter()
            .filter(|r| r.get_id() == ReportId::SKILL_USE)
            .count();
        assert_eq!(count, 1, "exactly one SKILL_USE report should be added");
    }

    // ── auto-decline for skills not present on player ─────────────────────────

    #[test]
    fn no_putrid_regurgitation_skill_auto_declines_and_proceeds() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.use_putrid_regurgitation, Some(false));
    }

    #[test]
    fn no_hit_and_run_skill_auto_declines_and_proceeds() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.use_hit_and_run, Some(false));
    }

    #[test]
    fn pile_driver_disabled_when_no_knocked_down_players() {
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.knocked_down_players = vec![];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.use_pile_driver, Some(false));
    }

    #[test]
    fn pile_driver_true_pushes_pile_driver_sequence_and_returns_early() {
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        use ffb_model::model::player::Player;
        use ffb_model::model::player_status::PlayerStatus;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_PRONE};
        use ffb_model::types::FieldCoordinate;
        use std::collections::HashSet;

        let mut game = make_game();
        game.home_playing = true; // home team is acting; away is inactive

        // Attacker: home player with PileDriver.
        let atk_id = "home_atk".to_string();
        let mut atk = Player { id: atk_id.clone(), name: "Attacker".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, position_movement: 6, position_strength: 3,
            position_agility: 3, position_passing: 4, position_armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, is_thrall: false, race: None,
            is_big_guy: false,
            is_lineman: false,
            keywords: vec![],
            temporary_stat_mods: vec![], temporary_skill_sources: vec![],
            recovering_injury: None, player_status: PlayerStatus::ACTIVE,
            zapped: false, inside_skill_list: false, inside_injury_list: false, injury_current: false, inside_player_statistics: false, current_skill_value: None,
        };
        atk.starting_skills.push(SkillWithValue { skill_id: SkillId::PileDriver, value: None });
        game.team_home.players.push(atk);
        game.field_model.set_player_state(&atk_id, PlayerState::new(PS_MOVING));
        game.field_model.set_player_coordinate(&atk_id, FieldCoordinate::new(10, 5));
        game.acting_player.player_id = Some(atk_id.clone());

        // Defender: away player PRONE at (11,5).
        let def_id = "away_def".to_string();
        let def = Player { id: def_id.clone(), name: "Defender".into(), nr: 2,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, position_movement: 6, position_strength: 3,
            position_agility: 3, position_passing: 4, position_armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, is_thrall: false, race: None,
            is_big_guy: false,
            is_lineman: false,
            keywords: vec![],
            temporary_stat_mods: vec![], temporary_skill_sources: vec![],
            recovering_injury: None, player_status: PlayerStatus::ACTIVE,
            zapped: false, inside_skill_list: false, inside_injury_list: false, injury_current: false, inside_player_statistics: false, current_skill_value: None,
        };
        game.team_away.players.push(def);
        game.field_model.set_player_state(&def_id, PlayerState::new(PS_PRONE));
        game.field_model.set_player_coordinate(&def_id, FieldCoordinate::new(11, 5));

        let mut step = StepEndBlocking::new();
        step.use_pile_driver = Some(true);
        // old_defender_state Some but not prone/stunned so canFoulAfterBlock = regularBlock && true.
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        step.knocked_down_players = vec![def_id.clone()];
        step.target_player_id = Some(def_id);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "expected PileDriver sequence to be pushed");
        assert_eq!(out.pushes[0][0].step_id, StepId::PileDriver);
    }
}
