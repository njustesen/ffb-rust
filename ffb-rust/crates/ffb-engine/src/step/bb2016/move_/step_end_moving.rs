use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::step::util_server_steps::{change_player_action, check_touchdown};
use crate::util::UtilServerPlayerMove;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::{
    Block, BlitzBlock, EndPlayerAction, Foul, Move, Pass, ThrowTeamMate, KickTeamMate,
};
use crate::step::generator::bb2016::block::BlockParams;
use crate::step::generator::bb2016::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2016::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2016::foul::FoulParams;
use crate::step::generator::bb2016::move_::MoveParams;
use crate::step::generator::bb2016::pass::PassParams;
use crate::step::generator::bb2016::throw_team_mate::ThrowTeamMateParams;
use crate::step::generator::bb2016::kick_team_mate::KickTeamMateParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepEndMoving.
///
/// Last step in the BB2016 move sequence. Consumes all expected stepParameters.
/// Decides which sequence to push next based on game state.
///
/// BB2016 differs from BB2025: no bloodlust_action, using_chainsaw, check_forgo,
/// thrown_player_id, move_start fields; GAZE (not GazeMove) → Move; KICK_TEAM_MATE → KickTeamMate
/// generator (not ThrowTeamMate); no Punt branch.
///
/// TODO(canHandOver): UtilPlayer::can_hand_over not yet ported.
/// TODO(canKickTeamMate): UtilPlayer::can_kick_team_mate not yet ported.
/// TODO(canThrowTeamMate): UtilPlayer::can_throw_team_mate not yet ported.
/// TODO(canGaze): UtilPlayer::can_gaze not yet ported (MOVE && canGaze branch).
pub struct StepEndMoving {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fFeedingAllowed (Boolean tristate)
    pub feeding_allowed: Option<bool>,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fBlockDefenderId
    pub block_defender_id: Option<String>,
}

impl StepEndMoving {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            feeding_allowed: None,
            move_stack: Vec::new(),
            dispatch_player_action: None,
            block_defender_id: None,
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
        // Java: CLIENT_BLOCK/FOUL/HAND_OVER/PASS/THROW_TEAM_MATE/KICK_TEAM_MATE
        //       → dispatchPlayerAction(fDispatchPlayerAction)
        match action {
            Action::Block { .. }
            | Action::Foul { .. }
            | Action::HandOff { .. }
            | Action::Pass { .. }
            | Action::ThrowTeamMate { .. }
            | Action::KickTeamMate { .. } => {
                return self.do_dispatch_player_action(game, rng);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockDefenderId(v) => { self.block_defender_id = Some(v.clone()); true }
            StepParameter::DispatchPlayerAction(v) => { self.dispatch_player_action = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::FeedingAllowed(v) => { self.feeding_allowed = Some(*v); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            _ => false,
        }
    }
}

impl StepEndMoving {
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
        // Java: UtilServerDialog.hideDialog(getGameState())
        // TODO: hide dialog — not yet ported

        self.end_turn |= check_touchdown(game);

        // Java: if (fFeedingAllowed == null) fFeedingAllowed = true
        let feeding_allowed = self.feeding_allowed.unwrap_or(true);

        // ── Branch 1: end turn or end player action ─────────────────────────────
        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 2: block defender set (ball-and-chain) ───────────────────────
        if let Some(ref defender_id) = self.block_defender_id.clone() {
            let seq = Block::build_sequence(&BlockParams {
                block_defender_id: Some(defender_id.clone()),
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        let player_action = game.acting_player.player_action;
        let player_id = game.acting_player.player_id.clone();
        let has_ball = player_id.as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false);

        // ── Branch 3: non-moving player action ──────────────────────────────────
        // Java: StringTool.isProvided(playerId) && playerAction != null && !isMoving()
        //       && !(PASS && !hasBall)
        if player_id.is_some() {
            if let Some(action) = player_action {
                let pass_no_ball = action == PlayerAction::Pass && !has_ball;
                if !action.is_moving() && !pass_no_ball {
                    if let Some(seq) = self.push_sequence_for_player_action(action) {
                        return StepOutcome::next().push_seq(seq);
                    }
                }
            }
        }

        // ── Branch 4: move stack provided ───────────────────────────────────────
        if !self.move_stack.is_empty() {
            let seq = Move::build_sequence(&MoveParams {
                move_stack: self.move_stack.clone(),
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 5: next move possible ────────────────────────────────────────
        // Java: isNextMovePossible || (HAND_OVER_MOVE && canHandOver) || (PASS_MOVE && hasBall)
        //       || (FOUL_MOVE && canFoul) || (MOVE && canGaze)
        //       || (KICK_TEAM_MATE_MOVE && canKickTeamMate(true))
        //       || (THROW_TEAM_MATE_MOVE && canThrowTeamMate(false))
        let pid = player_id.as_deref().unwrap_or("");
        let can_make_next_move = UtilPlayer::is_next_move_possible(game, false)
            // TODO(canHandOver): (HAND_OVER_MOVE && canHandOver) not yet ported
            || (player_action == Some(PlayerAction::PassMove) && has_ball)
            || (player_action == Some(PlayerAction::FoulMove) && UtilPlayer::can_foul(game, pid))
            // TODO(canGaze): (MOVE && canGaze) not yet ported
            // TODO(canKickTeamMate): (KICK_TEAM_MATE_MOVE && canKickTeamMate) not yet ported
            // TODO(canThrowTeamMate): (THROW_TEAM_MATE_MOVE && canThrowTeamMate) not yet ported
            ;

        if can_make_next_move {
            let jumping = game.acting_player.jumping;
            UtilServerPlayerMove::update_move_squares(game, jumping);
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 6 (else): end player action ──────────────────────────────────
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed,
            end_player_action: self.end_player_action,
            end_turn: self.end_turn,
        });
        StepOutcome::next().push_seq(seq)
    }

    fn push_sequence_for_player_action(
        &self,
        action: PlayerAction,
    ) -> Option<Vec<crate::step::framework::SequenceStep>> {
        match action {
            PlayerAction::Block => {
                Some(Block::build_sequence(&BlockParams::default()))
            }
            PlayerAction::Blitz | PlayerAction::BlitzMove => {
                Some(BlitzBlock::build_sequence(&BlitzBlockParams::default()))
            }
            PlayerAction::Foul | PlayerAction::FoulMove => {
                Some(Foul::build_sequence(&FoulParams::default()))
            }
            PlayerAction::HandOver
            | PlayerAction::HandOverMove
            | PlayerAction::Pass
            | PlayerAction::PassMove
            | PlayerAction::HailMaryPass => {
                Some(Pass::build_sequence(&PassParams::default()))
            }
            PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => {
                Some(ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default()))
            }
            PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
                Some(KickTeamMate::build_sequence(&KickTeamMateParams::default()))
            }
            // Java: GAZE → Move sequence (the BB2016 Gaze action falls into the Move path)
            PlayerAction::Gaze => {
                Some(Move::build_sequence(&MoveParams::default()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No move stack, no block defender → falls through to EndPlayerAction
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
        assert!(!out.pushes.is_empty());
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
        assert!(!out.pushes.is_empty(), "should push Move sequence");
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        let mut step = StepEndMoving::new();
        let stack = vec![FieldCoordinate::new(5, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack.clone())));
        assert_eq!(step.move_stack, stack);
    }

    #[test]
    fn set_parameter_block_defender_id_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::BlockDefenderId("d1".into())));
        assert_eq!(step.block_defender_id.as_deref(), Some("d1"));
    }

    #[test]
    fn set_parameter_feeding_allowed_accepted() {
        let mut step = StepEndMoving::new();
        assert!(step.set_parameter(&StepParameter::FeedingAllowed(false)));
        assert_eq!(step.feeding_allowed, Some(false));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndMoving::new();
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }

    #[test]
    fn block_action_not_moving_pushes_block_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "BLOCK should push Block sequence");
    }

    #[test]
    fn foul_action_not_moving_pushes_foul_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Foul);
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "FOUL should push Foul sequence");
    }
}
