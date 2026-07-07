use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::bb2025::ttm_mechanic::TtmMechanic as Bb2025TtmMechanic;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use crate::step::util_server_steps::{change_player_action, check_touchdown};
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::{
    EndPlayerAction, BlitzBlock, BlitzMove, Block, Foul, Move, Pass, Punt, ThrowTeamMate,
};
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::block::BlockParams;
use crate::step::generator::bb2025::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2025::blitz_move::BlitzMoveParams;
use crate::step::generator::bb2025::foul::FoulParams;
use crate::step::generator::bb2025::move_::MoveParams;
use crate::step::generator::bb2025::pass::PassParams;
use crate::step::generator::bb2025::throw_team_mate::ThrowTeamMateParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepEndMoving.
///
/// Finalises the move action.  Decides which sequence to push next:
/// 1. endTurn / endPlayerAction → EndPlayerAction sequence.
/// 2. fBlockDefenderId provided → Block sequence.
/// 3. PlayerAction not moving (PASS, BLOCK, FOUL, etc.) → push that action's sequence.
/// 4. fMoveStack provided → push Move / BlitzMove sequence.
/// 5. nextMovePossible / canHandOver / hasBall etc. → push Move / BlitzMove.
/// 6. Otherwise → EndPlayerAction sequence.
///
/// Also handles: CLIENT_BLOCK / CLIENT_FOUL / CLIENT_HAND_OVER / CLIENT_PASS /
///               CLIENT_THROW_TEAM_MATE / CLIENT_KICK_TEAM_MATE / CLIENT_ACTING_PLAYER commands.
///
/// checkTouchdown is wired via util_server_steps::check_touchdown.
/// allowSpecialBlocksWithBallAndChain option → askForBlockKind wired in ball-and-chain branch.
pub struct StepEndMoving {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: usingChainsaw
    pub using_chainsaw: bool,
    /// Java: checkForgo
    pub check_forgo: bool,
    /// Java: fFeedingAllowed (Boolean tristate — None = not yet set)
    pub feeding_allowed: Option<bool>,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: dispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: bloodlustAction
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: fBlockDefenderId
    pub block_defender_id: Option<String>,
    /// Java: thrownPlayerId
    pub thrown_player_id: Option<String>,
}

impl StepEndMoving {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            using_chainsaw: false,
            check_forgo: false,
            feeding_allowed: None,
            move_stack: Vec::new(),
            move_start: None,
            dispatch_player_action: None,
            bloodlust_action: None,
            block_defender_id: None,
            thrown_player_id: None,
        }
    }
}

impl Default for StepEndMoving {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndMoving {
    fn id(&self) -> StepId { StepId::EndMoving }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_BLOCK / CLIENT_FOUL / CLIENT_HAND_OVER / CLIENT_PASS /
        //       CLIENT_THROW_TEAM_MATE / CLIENT_KICK_TEAM_MATE → dispatchPlayerAction(dispatchPlayerAction)
        // Java: CLIENT_USE_SKILL (canAddBlockDie + isSkillUsed) → dispatchPlayerAction(dispatchPlayerAction)
        match action {
            Action::Block { .. }
            | Action::Foul { .. }
            | Action::HandOff { .. }
            | Action::Pass { .. }
            | Action::ThrowTeamMate { .. }
            | Action::KickTeamMate { .. } => {
                return self.do_dispatch_player_action(game, rng);
            }
            Action::UseSkill { skill_id, use_skill: true } => {
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_BLOCK_DIE) {
                    return self.do_dispatch_player_action(game, rng);
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            StepParameter::FeedingAllowed(v) => { self.feeding_allowed = Some(*v); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            StepParameter::BlockDefenderId(v) => { self.block_defender_id = Some(v.clone()); true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::DispatchPlayerAction(v) => {
                self.dispatch_player_action = *v; true
            }
            _ => false,
        }
    }
}

impl StepEndMoving {
    /// Java: dispatchPlayerAction(pPlayerAction).
    /// Called from handle_command when a redirected client command arrives.
    fn do_dispatch_player_action(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Some(dispatch_action) = self.dispatch_player_action {
            if let Some(ref pid) = game.acting_player.player_id.clone() {
                let jumping = game.acting_player.jumping;
                change_player_action(game, pid, dispatch_action, jumping);
            }
            if let Some(seq) = self.push_sequence_for_player_action(dispatch_action) {
                return StepOutcome::next().push_seq(seq);
            }
        }
        self.execute_step(game, rng)
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(gameState)
        // client-only: hide dialog — dialog is client-side

        // Java: triesToSecureBall = playerAction == SECURE_THE_BALL && !(isSufferingBloodLust && bloodlustAction == MOVE)
        // Java: secureTheBallFailed = endPlayerAction && triesToSecureBall && !UtilPlayer.hasBall(...)
        let player_action = game.acting_player.player_action;
        let tries_to_secure_ball = player_action == Some(PlayerAction::SecureTheBall)
            && !(game.acting_player.suffering_blood_lust
                && self.bloodlust_action == Some(PlayerAction::Move));
        let has_ball = game.acting_player.player_id.as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false);
        let secure_the_ball_failed = self.end_player_action && tries_to_secure_ball && !has_ball;

        // Java: if (secureTheBallFailed) getResult().addReport(new ReportPlayerEvent(...))
        if secure_the_ball_failed {
            game.report_list.add(ReportPlayerEvent::new(
                game.acting_player.player_id.clone(),
                Some("could not secure the ball, causing a turnover".into()),
            ));
        }

        // Java: fEndTurn |= checkTouchdown(gameState) || secureTheBallFailed
        self.end_turn |= check_touchdown(game) || secure_the_ball_failed;

        // Java: if (fFeedingAllowed == null) fFeedingAllowed = true
        let feeding_allowed = self.feeding_allowed.unwrap_or(true);

        // ── Branch 1: end turn or end player action ─────────────────────────────
        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
                check_forgo: self.check_forgo,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 2: block defender set (ball-and-chain) ───────────────────────
        if let Some(ref defender_id) = self.block_defender_id.clone() {
            // Java: askForBlockKind check (GameOptionBoolean ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN)
            let ask_for_block_kind = if game.options.is_enabled("allowSpecialBlocksWithBallAndChain") {
                let defender_state = game.field_model.player_state(defender_id);
                let acting_has_alt = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::PROVIDES_BLOCK_ALTERNATIVE))
                    .unwrap_or(false);
                let defender_not_prone_stunned = defender_state
                    .map(|s| !s.is_stunned() && !s.is_prone_or_stunned())
                    .unwrap_or(false);
                if acting_has_alt && defender_not_prone_stunned {
                    game.defender_id = Some(defender_id.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            };
            let seq = Block::build_sequence(&BlockParams {
                block_defender_id: Some(defender_id.clone()),
                using_chainsaw: self.using_chainsaw,
                ask_for_block_kind,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 3: non-moving player action ──────────────────────────────────
        let player_action = game.acting_player.player_action;
        let player_id = game.acting_player.player_id.clone();
        let has_ball = player_id.as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false);

        if player_id.is_some() {
            if let Some(action) = player_action {
                // Java: !playerAction.isMoving() && !(PASS/HAND_OVER without ball)
                let pass_or_handover_no_ball = (action == PlayerAction::Pass
                    || action == PlayerAction::HandOver)
                    && !has_ball;
                if !action.is_moving() && !pass_or_handover_no_ball {
                    if let Some(seq) = self.push_sequence_for_player_action(action) {
                        return StepOutcome::next().push_seq(seq);
                    }
                }
            }
        }

        // ── Branch 4: move stack provided ───────────────────────────────────────
        if !self.move_stack.is_empty() {
            let seq = if player_action == Some(PlayerAction::BlitzMove) {
                BlitzMove::build_sequence(&BlitzMoveParams {
                    move_stack: self.move_stack.clone(),
                    move_start: self.move_start,
                    ..Default::default()
                })
            } else {
                Move::build_sequence(&MoveParams {
                    move_stack: self.move_stack.clone(),
                    move_start: self.move_start,
                    bloodlust_action: self.bloodlust_action,
                    ..Default::default()
                })
            };
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 5: next move possible ────────────────────────────────────────
        // Java: isNextMovePossible || canHandOver || (PASS_MOVE && hasBall) || (FOUL_MOVE && canFoul)
        //       || (GAZE_MOVE && hasAdjacentGazeTarget) || canKickTeamMate || canThrowTeamMate
        //       || (blitzMove && adjacentTarget) || (PUNT_MOVE && hasBall)
        let pid = player_id.as_deref().unwrap_or("");

        // Java: adjacentTarget from TargetSelectionState.getSelectedPlayerId()
        let adjacent_target = game.field_model.target_selection_state.as_ref()
            .and_then(|tss| tss.get_selected_player_id())
            .and_then(|target_id| {
                let target_coord = game.field_model.player_coordinate(target_id)?;
                let acting_coord = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id))?;
                Some(target_coord.is_adjacent(acting_coord))
            })
            .unwrap_or(false);

        let is_blitz_move = player_action.map(|a| a.is_blitz_move()).unwrap_or(false);
        let can_make_next_move = UtilPlayer::is_next_move_possible(game, false)
            || (player_action == Some(PlayerAction::HandOverMove) && UtilPlayer::can_hand_over(game, pid))
            || (player_action == Some(PlayerAction::PassMove) && has_ball)
            || (player_action == Some(PlayerAction::FoulMove) && UtilPlayer::can_foul(game, pid))
            || (player_action == Some(PlayerAction::GazeMove)
                && UtilPlayer::has_adjacent_gaze_target(game, pid))
            || (player_action == Some(PlayerAction::KickTeamMateMove) && can_kick_team_mate(game, pid, true))
            || (player_action == Some(PlayerAction::ThrowTeamMateMove) && can_throw_team_mate(game, pid, false))
            || (is_blitz_move && adjacent_target)
            || (player_action == Some(PlayerAction::PuntMove) && has_ball);
        if can_make_next_move {
            // Java: UtilServerPlayerMove.updateMoveSquares(gameState, actingPlayer.isJumping())
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            let seq = if is_blitz_move {
                // Java: ServerUtilBlock.updateDiceDecorations(gameState) — only for blitz move
                ServerUtilBlock::update_dice_decorations(game);
                BlitzMove::build_sequence(&BlitzMoveParams::default())
            } else {
                Move::build_sequence(&MoveParams {
                    bloodlust_action: self.bloodlust_action,
                    ..Default::default()
                })
            };
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 6 (else): end player action ──────────────────────────────────
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed,
            end_player_action: self.end_player_action,
            end_turn: self.end_turn,
            check_forgo: self.check_forgo,
        });
        StepOutcome::next().push_seq(seq)
    }

    /// Java: pushSequenceForPlayerAction(pPlayerAction).
    /// Returns the sequence to push, or None if the action isn't handled.
    fn push_sequence_for_player_action(
        &self,
        action: PlayerAction,
    ) -> Option<Vec<crate::step::framework::SequenceStep>> {
        match action {
            // Java: VICIOUS_VINES | BLOCK → Block sequence
            PlayerAction::ViciousVines | PlayerAction::Block => {
                Some(Block::build_sequence(&BlockParams {
                    using_chainsaw: self.using_chainsaw,
                    ..Default::default()
                }))
            }
            // Java: BLITZ | BLITZ_MOVE | PUTRID_REGURGITATION_MOVE | KICK_EM_BLITZ → BlitzBlock
            PlayerAction::Blitz
            | PlayerAction::BlitzMove
            | PlayerAction::PutridRegurgitationMove
            | PlayerAction::KickEmBlitz => {
                Some(BlitzBlock::build_sequence(&BlitzBlockParams {
                    using_chainsaw: self.using_chainsaw,
                    ..Default::default()
                }))
            }
            // Java: FOUL | FOUL_MOVE → Foul
            PlayerAction::Foul | PlayerAction::FoulMove => {
                Some(Foul::build_sequence(&FoulParams::default()))
            }
            // Java: HAND_OVER | HAND_OVER_MOVE | PASS | PASS_MOVE | HAIL_MARY_PASS → Pass
            PlayerAction::HandOver
            | PlayerAction::HandOverMove
            | PlayerAction::Pass
            | PlayerAction::PassMove
            | PlayerAction::HailMaryPass => {
                Some(Pass::build_sequence(&PassParams::default()))
            }
            // Java: THROW_TEAM_MATE | THROW_TEAM_MATE_MOVE → ThrowTeamMate(thrown, false)
            PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => {
                Some(ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    is_kicked: false,
                    ..Default::default()
                }))
            }
            // Java: KICK_TEAM_MATE | KICK_TEAM_MATE_MOVE → ThrowTeamMate(thrown, true)
            PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
                Some(ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    is_kicked: true,
                    ..Default::default()
                }))
            }
            // Java: GAZE → Move sequence (empty params)
            PlayerAction::Gaze => {
                Some(Move::build_sequence(&MoveParams {
                    bloodlust_action: self.bloodlust_action,
                    ..Default::default()
                }))
            }
            // Java: PUNT → Punt sequence
            PlayerAction::Punt => {
                Some(Punt::build_sequence())
            }
            _ => None,
        }
    }
}

/// Java: UtilPlayer.canKickTeamMate(game, kicker, checkBlitzUsed).
fn can_kick_team_mate(game: &Game, player_id: &str, check_blitz_used: bool) -> bool {
    let player = match game.player(player_id) { Some(p) => p, None => return false };
    if check_blitz_used && game.turn_data().blitz_used { return false; }
    use ffb_model::model::property::named_properties::NamedProperties;
    if !player.has_skill_property(NamedProperties::CAN_KICK_TEAM_MATES) { return false; }
    let mechanic = Bb2025TtmMechanic::new();
    !mechanic.find_kickable_team_mates(game, player).is_empty()
}

/// Java: UtilPlayer.canThrowTeamMate(game, thrower, checkPassUsed).
fn can_throw_team_mate(game: &Game, player_id: &str, check_pass_used: bool) -> bool {
    let player = match game.player(player_id) { Some(p) => p, None => return false };
    let mechanic = Bb2025TtmMechanic::new();
    if check_pass_used && !mechanic.is_ttm_available(game.turn_data()) { return false; }
    if !mechanic.can_throw(game, player) { return false; }
    !mechanic.find_throwable_team_mates(game, player).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No move stack, no block defender → falls through to EndPlayerAction sequence push
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push EndPlayerAction sequence");
    }

    #[test]
    fn end_player_action_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push EndPlayerAction sequence");
    }

    #[test]
    fn block_defender_id_pushes_block_sequence() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        step.block_defender_id = Some("def1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push Block sequence");
    }

    #[test]
    fn move_stack_pushes_move_sequence() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        step.move_stack = vec![FieldCoordinate::new(5, 5)];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push Move sequence for move stack");
    }

    #[test]
    fn blitz_move_action_with_stack_pushes_blitz_move_sequence() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        let mut step = StepEndMoving::new();
        step.move_stack = vec![FieldCoordinate::new(5, 5)];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push BlitzMove sequence");
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_feeding_allowed_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::FeedingAllowed(false)));
        assert_eq!(step.feeding_allowed, Some(false));
    }

    #[test]
    fn set_parameter_block_defender_id_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::BlockDefenderId("def1".into())));
        assert_eq!(step.block_defender_id.as_deref(), Some("def1"));
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        let mut step = StepEndMoving::new();
        let stack = vec![FieldCoordinate::new(5, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack.clone())));
        assert_eq!(step.move_stack, stack);
    }

    #[test]
    fn set_parameter_dispatch_player_action_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block))));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndMoving::new();
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }

    #[test]
    fn feeding_allowed_defaults_to_true_when_not_set() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        // feeding_allowed not set → defaults to true in EndPlayerAction sequence
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // should push sequence (non-empty pushes)
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn block_action_when_not_moving_pushes_block_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "BLOCK action should push Block sequence");
    }

    #[test]
    fn foul_action_when_not_moving_pushes_foul_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Foul);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "FOUL action should push Foul sequence");
    }

    #[test]
    fn pass_action_when_not_moving_pushes_pass_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "PASS action should push Pass sequence");
    }

    #[test]
    fn punt_action_when_not_moving_pushes_punt_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Punt);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "PUNT action should push Punt sequence");
    }

    // ── handle_command dispatch tests ────────────────────────────────────────

    #[test]
    fn handle_command_block_dispatches_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::Block);
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "Block dispatch should push Block sequence");
    }

    #[test]
    fn handle_command_foul_dispatches_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Foul);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::Foul);
        let action = crate::action::Action::Foul { target_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "Foul dispatch should push Foul sequence");
    }

    #[test]
    fn handle_command_pass_dispatches_player_action() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::Pass);
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(5, 5) };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "Pass dispatch should push Pass sequence");
    }

    #[test]
    fn handle_command_hand_off_dispatches_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::HandOver);
        let action = crate::action::Action::HandOff { receiver_id: "rcv1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "HandOff dispatch should push Pass sequence");
    }

    #[test]
    fn handle_command_throw_team_mate_dispatches_player_action() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::ThrowTeamMate);
        let action = crate::action::Action::ThrowTeamMate {
            player_id: "ttm1".into(),
            coord: FieldCoordinate::new(5, 5),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "ThrowTeamMate dispatch should push TTM sequence");
    }

    #[test]
    fn handle_command_kick_team_mate_dispatches_player_action() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        let mut step = StepEndMoving::new();
        step.dispatch_player_action = Some(PlayerAction::KickTeamMate);
        let action = crate::action::Action::KickTeamMate {
            player_id: "ktm1".into(),
            coord: FieldCoordinate::new(5, 5),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "KickTeamMate dispatch should push TTM(kicked) sequence");
    }

    #[test]
    fn handle_command_use_skill_non_block_die_falls_through_to_execute_step() {
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepEndMoving::new();
        // Dodge does NOT have CAN_ADD_BLOCK_DIE — should fall through to execute_step
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // execute_step falls through to EndPlayerAction sequence
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_no_dispatch_action_falls_through() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepEndMoving::new();
        // dispatch_player_action is None → falls through to execute_step
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        step.dispatch_player_action = None;
        // do_dispatch_player_action with None falls through to execute_step
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn adjacent_target_blitz_move_pushes_blitz_move_sequence() {
        use ffb_model::types::FieldCoordinate;
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        // Place acting player at (5,5) and target at (6,5) — adjacent
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("target1", FieldCoordinate::new(6, 5));
        let mut tss = TargetSelectionState::new("target1");
        tss.select();
        game.field_model.target_selection_state = Some(tss);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "adjacent blitz target should push BlitzMove sequence");
    }

    fn add_player_at(game: &mut Game, team_is_home: bool, id: &str, coord: FieldCoordinate) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if team_is_home { game.team_home.players.push(p) } else { game.team_away.players.push(p) }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn hand_over_move_with_can_hand_over_pushes_move_sequence() {
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        add_player_at(&mut game, true, "p2", FieldCoordinate::new(5, 6));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::HandOverMove);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "HandOverMove with can_hand_over=true should push Move");
    }

    #[test]
    fn hand_over_move_without_can_hand_over_falls_through_to_end_player_action() {
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::HandOverMove);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push EndPlayerAction as fallback");
    }

    #[test]
    fn secure_the_ball_failed_triggers_end_turn() {
        // Java: secureTheBallFailed = endPlayerAction && triesSecure && !hasBall
        // Player has SecureTheBall action but does NOT have the ball → endTurn is forced
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::SecureTheBall);
        // Ball is elsewhere — player does not have it
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepEndMoving::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // secureTheBallFailed → end_turn → push EndPlayerAction with end_turn=true
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn secure_the_ball_succeeded_does_not_trigger_end_turn() {
        // Player has SecureTheBall and has the ball → normal end_player_action path
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::SecureTheBall);
        // Ball IS at player's square
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game.field_model.ball_in_play = true;
        let mut step = StepEndMoving::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn block_defender_without_option_does_not_set_defender_id() {
        let mut game = make_game();
        // option disabled (default)
        let mut step = StepEndMoving::new();
        step.block_defender_id = Some("def1".into());
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none(), "defender_id must not be set when option is disabled");
    }

    #[test]
    fn block_defender_with_option_enabled_but_player_lacks_alt_skill_does_not_set_defender_id() {
        let mut game = make_game();
        game.options.set("allowSpecialBlocksWithBallAndChain", "true");
        // acting player has NO PROVIDES_BLOCK_ALTERNATIVE skill
        add_player_at(&mut game, true, "atk", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("atk".into());
        add_player_at(&mut game, false, "def1", FieldCoordinate::new(6, 5));
        let mut step = StepEndMoving::new();
        step.block_defender_id = Some("def1".into());
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none(), "defender_id must not be set when player lacks providesBlockAlternative");
    }

    #[test]
    fn secure_the_ball_failed_adds_player_event_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::SecureTheBall);
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepEndMoving::new();
        step.end_player_action = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT), "ReportPlayerEvent must be added when secure_the_ball_failed");
    }

    #[test]
    fn no_player_event_report_when_secure_the_ball_succeeds() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player_at(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::SecureTheBall);
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game.field_model.ball_in_play = true;
        let mut step = StepEndMoving::new();
        step.end_player_action = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PLAYER_EVENT), "no ReportPlayerEvent when player has the ball");
    }
}
