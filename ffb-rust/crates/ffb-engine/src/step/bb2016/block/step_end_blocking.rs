/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepEndBlocking`.
///
/// Last step in block sequence. Consumes all expected stepParameters.
///
/// Expects stepParameter DEFENDER_PUSHED to be set by a preceding step.
/// Expects stepParameter END_PLAYER_ACTION to be set by a preceding step.
/// Expects stepParameter END_TURN to be set by a preceding step.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
/// Expects stepParameter USING_STAB to be set by a preceding step.
///
/// May push a new sequence on the stack.
use ffb_model::enums::{PlayerAction, PlayerState, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_steps::check_touchdown;
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};
use crate::step::generator::bb2016::EndPlayerAction;
use crate::step::generator::bb2016::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2016::Block;
use crate::step::generator::bb2016::block::BlockParams;
use crate::step::generator::bb2016::BlitzBlock;
use crate::step::generator::bb2016::Move;
use crate::step::generator::bb2016::move_::MoveParams;
use crate::step::util_server_steps::change_player_action;

/// Java: `StepEndBlocking` (bb2016/block).
pub struct StepEndBlocking {
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
    /// Java: `fDefenderPushed`
    pub defender_pushed: bool,
    /// Java: `fUsingStab`
    pub using_stab: bool,
    /// Java: `fOldDefenderState`
    pub old_defender_state: Option<PlayerState>,
}

impl StepEndBlocking {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            defender_pushed: false,
            using_stab: false,
            old_defender_state: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState()) — not ported

        // Java: fEndTurn |= UtilServerSteps.checkTouchdown(getGameState())
        self.end_turn |= check_touchdown(game);

        let player_action = game.acting_player.player_action;
        let attacker_id = game.acting_player.player_id.clone();

        // Java: if (fEndTurn || fEndPlayerAction) {
        //   game.setDefenderId(null)
        //   endGenerator.pushSequence(...EndPlayerAction.SequenceParams(gs, true, true, fEndTurn)) }
        if self.end_turn || self.end_player_action {
            game.defender_id = None;
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: Revert strength gained from HORNS and DAUNTLESS to avoid interaction with tentacles.
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

        let defender_id = game.defender_id.clone();
        let is_blitz = player_action == Some(PlayerAction::Blitz);

        let attacker_position = attacker_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let defender_position = defender_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let defender_state = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));
        let attacker_state = attacker_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();

        // Java: Skill canBlockMultipleTimesSkill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canBlockMoreThanOnce)
        let can_block_multiple_times = attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_BLOCK_MORE_THAN_ONCE));

        // Java: if ((playerAction == MULTIPLE_BLOCK) && canBlockMultipleTimesSkill != null
        //   && !hasSkillToCancelProperty(canBlockMoreThanOnce)
        //   && hasTacklezones && !blocksLikeChainsaw && !confused && hasBlocked) {
        //   markSkillUsed; setHasBlocked(false); ... blockGenerator.pushSequence }
        let is_multiple_block = player_action == Some(PlayerAction::MultipleBlock);
        let has_chainsaw = attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);

        let has_skill_to_cancel_multi_block = attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::CAN_BLOCK_MORE_THAN_ONCE))
            .unwrap_or(false);

        if is_multiple_block
            && can_block_multiple_times.is_some()
            && !has_skill_to_cancel_multi_block
            && attacker_state.has_tacklezones()
            && !has_chainsaw
            && !attacker_state.is_confused()
            && game.acting_player.has_blocked
        {
            if let (Some(sid), Some(pid)) = (can_block_multiple_times, attacker_id.as_deref()) {
                let pid = pid.to_owned();
                let is_home = game.team_home.player(&pid).is_some();
                if is_home { game.team_home.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                else { game.team_away.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
            }
            game.acting_player.has_blocked = false;
            ServerUtilBlock::update_dice_decorations(game);

            let multi_defender_id = defender_id.clone();
            game.defender_id = None;

            if is_blitz {
                let seq = BlitzBlock::build_sequence(&BlockParams {
                    block_defender_id: multi_defender_id,
                    using_stab: self.using_stab,
                    ..Default::default()
                });
                return StepOutcome::next().push_seq(seq);
            } else {
                let seq = Block::build_sequence(&BlockParams {
                    multi_block_defender_id: multi_defender_id,
                    ..Default::default()
                });
                game.defender_id = None;
                return StepOutcome::next().push_seq(seq);
            }
        }

        // Java: Skill unusedPlayerMustMakeSecondBlockSkill =
        //   UtilCards.getUnusedSkillWithProperty(actingPlayer, forceSecondBlock)
        let unused_force_second_block = attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::FORCE_SECOND_BLOCK));

        // Java: if (activePlayer.hasSkillProperty(forceSecondBlock)) actingPlayer.setGoingForIt(true)
        if attacker_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::FORCE_SECOND_BLOCK))
            .unwrap_or(false)
        {
            game.acting_player.goes_for_it = true;
        }

        // Java: UtilPlayer.isNextMovePossible(game, false) — not hasMoveLeft; this also
        // checks actingPlayer.isHeldInPlace() (e.g. Tentacles), which hasMoveLeft alone omits.
        let next_move_possible_for_second_block = UtilPlayer::is_next_move_possible(game, false);
        let defender_can_be_blocked = defender_state.map(|s| s.can_be_blocked()).unwrap_or(false);
        let is_adjacent = attacker_position.zip(defender_position)
            .map(|(a, d)| a.is_adjacent(d))
            .unwrap_or(false);

        // Java: else if ((unusedPlayerMustMakeSecondBlockSkill != null)
        //   && defenderState.canBeBlocked() && attackerPosition.isAdjacent(defenderPosition)
        //   && hasTacklezones && fDefenderPushed && (playerAction != MULTIPLE_BLOCK)
        //   && UtilPlayer.isNextMovePossible(game, false)) {
        //   actingPlayer.setGoingForIt(true); markSkillUsed; blockGenerator.pushSequence(...frenzyBlock=true) }
        let force_second_block = unused_force_second_block.is_some()
            && defender_can_be_blocked
            && is_adjacent
            && attacker_state.has_tacklezones()
            && self.defender_pushed
            && !is_multiple_block
            && next_move_possible_for_second_block;

        if force_second_block {
            game.acting_player.goes_for_it = true;
            if let (Some(sid), Some(pid)) = (unused_force_second_block, attacker_id.as_deref()) {
                let pid = pid.to_owned();
                let is_home = game.team_home.player(&pid).is_some();
                if is_home { game.team_home.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                else { game.team_away.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
            }
            let seq = Block::build_sequence(&BlockParams {
                block_defender_id: defender_id.clone(),
                using_stab: self.using_stab,
                frenzy_block: true,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else { ServerUtilBlock.removePlayerBlockStates(game, null)
        //   fieldModel.clearDiceDecorations(); actingPlayer.setGoingForIt(isNextMoveGoingForIt)
        //   if (BLITZ && !usingStab && !blocksLikeChainsaw && hasTacklezones && isNextMovePossible) {
        //     changeActingPlayer to BLITZ_MOVE; updateMoveSquares; moveGenerator.pushSequence }
        //   else if (MOVE && isNextMovePossible) { updateMoveSquares; moveGenerator.pushSequence }
        //   else if (sufferingBloodLust && !hasBlocked) { revert; blockGenerator.pushSequence }
        //   else { endGenerator.pushSequence } }
        ServerUtilBlock::remove_player_block_states(game, None);
        game.field_model.clear_dice_decorations();
        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);

        let next_move_possible = UtilPlayer::is_next_move_possible(game, false);
        let can_move_on = !self.using_stab && !has_chainsaw;

        if is_blitz && can_move_on && attacker_state.has_tacklezones() && next_move_possible {
            // Java: changeActingPlayer(this, actingPlayerId, PlayerAction.BLITZ_MOVE, jumping)
            if let Some(pid) = attacker_id.as_deref() {
                let jumping = game.acting_player.jumping;
                change_player_action(game, pid, PlayerAction::BlitzMove, jumping);
            }
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            ServerUtilBlock::update_dice_decorations(game);
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if ((playerAction == MOVE) && isNextMovePossible(game, false)) — ball and chain
        if player_action == Some(PlayerAction::Move) && next_move_possible {
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            ServerUtilBlock::update_dice_decorations(game);
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if (actingPlayer.isSufferingBloodLust() && !actingPlayer.hasBlocked()) {
        //   revert defender state; game.setDefenderId(null); blockGenerator.pushSequence }
        if game.acting_player.suffering_blood_lust && !game.acting_player.has_blocked {
            if let (Some(old_state), Some(defender_id)) = (self.old_defender_state, defender_id.clone()) {
                game.field_model.set_player_state(&defender_id, old_state);
            }
            game.defender_id = None;
            ServerUtilBlock::update_dice_decorations(game);
            let seq = Block::build_sequence(&BlockParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else { game.setDefenderId(null); endGenerator.pushSequence }
        game.defender_id = None;
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: false,
        });
        StepOutcome::next().push_seq(seq)
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
        // Java: CLIENT_USE_SKILL → canAddBlockDie →
        //   getResult().addReport(new ReportSkillUse(commandUseSkill.getSkill(), true, SkillUse.ADD_BLOCK_DIE))
        if let Action::UseSkill { skill_id, use_skill: true } = action {
            game.report_list.add(ReportSkillUse::new(
                None, *skill_id, true, SkillUse::ADD_BLOCK_DIE,
            ));
            ServerUtilBlock::update_dice_decorations_with_frenzy(game, true);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: DEFENDER_PUSHED (consumed)
            StepParameter::DefenderPushed(v) => { self.defender_pushed = *v; true }
            // Java: END_PLAYER_ACTION (consumed)
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            // Java: END_TURN (consumed)
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            // Java: OLD_DEFENDER_STATE (consumed)
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            // Java: USING_STAB (consumed)
            StepParameter::UsingStab(v) => { self.using_stab = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_end_blocking() {
        assert_eq!(StepEndBlocking::new().id(), StepId::EndBlocking);
    }

    #[test]
    fn default_state_pushes_end_player_action_sequence() {
        // Java: normal path → endGenerator.pushSequence(EndPlayerAction.SequenceParams(gs, true, true, false))
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // EndPlayerAction sequence starts with InitFeeding
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding);
    }

    #[test]
    fn end_turn_clears_defender_and_pushes_sequence() {
        // Java: if (fEndTurn || fEndPlayerAction) { game.setDefenderId(null); endGenerator }
        let mut step = StepEndBlocking::new();
        step.end_turn = true;
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.defender_id.is_none());
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn end_player_action_clears_defender() {
        let mut step = StepEndBlocking::new();
        step.end_player_action = true;
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn set_parameter_all_flags_accepted() {
        let mut step = StepEndBlocking::new();
        assert!(step.set_parameter(&StepParameter::DefenderPushed(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::UsingStab(true)));
        assert!(step.defender_pushed);
        assert!(step.end_player_action);
        assert!(step.end_turn);
        assert!(step.using_stab);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepEndBlocking::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndBlocking::new();
        assert!(!step.set_parameter(&StepParameter::GotoLabel("x".into())));
    }

    // ── report_list: ADD_BLOCK_DIE skill use ─────────────────────────────────

    #[test]
    fn use_skill_adds_report_skill_use_add_block_die() {
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
    fn use_skill_false_does_not_add_report_skill_use() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepEndBlocking::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Block, use_skill: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(!game.report_list.has_report(ReportId::SKILL_USE),
            "no SKILL_USE report should be added when use_skill=false");
    }

    // ── force-second-block: must use isNextMovePossible, not hasMoveLeft ─────

    #[test]
    fn force_second_block_skipped_when_held_in_place() {
        // Java: the FORCE_SECOND_BLOCK branch requires UtilPlayer.isNextMovePossible(game, false),
        // which returns false whenever actingPlayer.isHeldInPlace() (e.g. Tentacles) — regardless
        // of whether the player still has movement left. A prior Rust translation used
        // UtilPlayer::has_move_left directly, which ignores held_in_place and would incorrectly
        // enter the FORCE_SECOND_BLOCK branch (pushing a frenzy Block sequence) instead of
        // falling through to the default EndPlayerAction sequence.
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;

        let mut step = StepEndBlocking::new();
        step.defender_pushed = true;
        let mut game = make_game();

        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Frenzy, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.acting_player.current_move = 0;
        game.acting_player.held_in_place = true; // e.g. Tentacles

        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());

        let out = step.start(&mut game, &mut GameRng::new(0));
        // Falls through to the default branch (endGenerator.pushSequence), whose sequence
        // starts with InitFeeding — not a frenzy Block sequence.
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding,
            "held_in_place must block the FORCE_SECOND_BLOCK branch even though hasMoveLeft is true");
    }
}
