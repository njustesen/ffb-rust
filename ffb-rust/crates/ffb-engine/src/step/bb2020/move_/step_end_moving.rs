use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::step::util_server_steps::{change_player_action, check_touchdown};
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::{
    EndPlayerAction, BlitzBlock, BlitzMove, Block, Foul, Move, Pass, ThrowTeamMate,
};
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2020::block::BlockParams;
use crate::step::generator::bb2020::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2020::blitz_move::BlitzMoveParams;
use crate::step::generator::bb2020::foul::FoulParams;
use crate::step::generator::bb2020::move_::MoveParams;
use crate::step::generator::bb2020::pass::PassParams;
use crate::step::generator::bb2020::throw_team_mate::ThrowTeamMateParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepEndMoving.
///
/// Finalises the move action. BB2020 differs from BB2025 only in using BB2020 generators.
///
/// TODO(secureTheBall): secureTheBallFailed logic not yet ported.
/// TODO(canHandOver): UtilPlayer::can_hand_over not yet ported (HAND_OVER_MOVE branch stub).
/// TODO(ktm): canKickTeamMate / canThrowTeamMate checks not yet ported.
/// TODO(askBlockKind): GameOption ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN check not yet ported.
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
        // Java: fEndTurn |= checkTouchdown(gameState)
        self.end_turn |= check_touchdown(game);

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
                using_chainsaw: self.using_chainsaw,
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
                    ..Default::default()
                })
            };
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 5: next move possible ────────────────────────────────────────
        let pid = player_id.as_deref().unwrap_or("");

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
            || (player_action == Some(PlayerAction::PassMove) && has_ball)
            || (player_action == Some(PlayerAction::FoulMove) && UtilPlayer::can_foul(game, pid))
            || (player_action == Some(PlayerAction::GazeMove)
                && UtilPlayer::has_adjacent_gaze_target(game, pid))
            || (is_blitz_move && adjacent_target)
            || (player_action == Some(PlayerAction::PuntMove) && has_ball);
        if can_make_next_move {
            UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
            let seq = if is_blitz_move {
                ServerUtilBlock::update_dice_decorations(game);
                BlitzMove::build_sequence(&BlitzMoveParams::default())
            } else {
                Move::build_sequence(&MoveParams::default())
            };
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
            PlayerAction::ViciousVines | PlayerAction::Block => {
                Some(Block::build_sequence(&BlockParams {
                    using_chainsaw: self.using_chainsaw,
                    ..Default::default()
                }))
            }
            PlayerAction::Blitz
            | PlayerAction::BlitzMove
            | PlayerAction::PutridRegurgitationMove
            | PlayerAction::KickEmBlitz => {
                Some(BlitzBlock::build_sequence(&BlitzBlockParams {
                    using_chainsaw: self.using_chainsaw,
                    ..Default::default()
                }))
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
                Some(ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    is_kicked: false,
                    ..Default::default()
                }))
            }
            PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
                Some(ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    is_kicked: true,
                    ..Default::default()
                }))
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
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndMoving::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
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
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepEndMoving::new();
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }
}
