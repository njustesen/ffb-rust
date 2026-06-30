use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::{
    Block, BlitzBlock, BlitzMove, EndPlayerAction, Foul, Move, Pass, ThrowTeamMate, KickTeamMate,
    Select,
};
use crate::step::generator::bb2016::block::BlockParams;
use crate::step::generator::bb2016::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2016::blitz_move::BlitzMoveParams;
use crate::step::generator::bb2016::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2016::foul::FoulParams;
use crate::step::generator::bb2016::move_::MoveParams;
use crate::step::generator::bb2016::pass::PassParams;
use crate::step::generator::bb2016::throw_team_mate::ThrowTeamMateParams;
use crate::step::generator::bb2016::kick_team_mate::KickTeamMateParams;
use crate::step::generator::bb2016::select::SelectParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepEndSelecting.
///
/// Last step in the BB2016 select sequence. Consumes all expected stepParameters.
/// Dispatches to the appropriate action sequence based on the player action.
///
/// Expects: BLOCK_DEFENDER_ID, DISPATCH_PLAYER_ACTION, END_PLAYER_ACTION, END_TURN,
///          FOUL_DEFENDER_ID, GAZE_VICTIM_ID, HAIL_MARY_PASS, MOVE_STACK,
///          TARGET_COORDINATE, THROWN_PLAYER_ID, KICKED_PLAYER_ID, NR_OF_DICE, USING_STAB.
///
/// TODO(bloodlust): isSufferingBloodLust path not yet ported.
/// TODO(isRooted): playerState.isRooted() check not yet ported.
/// TODO(removeConfusion): REMOVE_CONFUSION path not yet ported.
/// TODO(standUp): STAND_UP / STAND_UP_BLITZ paths not yet ported.
/// TODO(setHasMoved): actingPlayer.setHasMoved(true) for REMOVE_CONFUSION not yet ported.
pub struct StepEndSelecting {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: fBlockDefenderId
    pub block_defender_id: Option<String>,
    /// Java: fUsingStab
    pub using_stab: Option<bool>,
    /// Java: fFoulDefenderId
    pub foul_defender_id: Option<String>,
    /// Java: fTargetCoordinate
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: fHailMaryPass
    pub hail_mary_pass: bool,
    /// Java: fThrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: fKickedPlayerId
    pub kicked_player_id: Option<String>,
    /// Java: fNumDice
    pub num_dice: i32,
}

impl StepEndSelecting {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            dispatch_player_action: None,
            move_stack: Vec::new(),
            gaze_victim_id: None,
            block_defender_id: None,
            using_stab: None,
            foul_defender_id: None,
            target_coordinate: None,
            hail_mary_pass: false,
            thrown_player_id: None,
            kicked_player_id: None,
            num_dice: 0,
        }
    }
}

impl Default for StepEndSelecting {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndSelecting {
    fn id(&self) -> StepId { StepId::EndSelecting }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockDefenderId(v) => { self.block_defender_id = Some(v.clone()); true }
            StepParameter::DispatchPlayerAction(v) => { self.dispatch_player_action = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::FoulDefenderId(v) => { self.foul_defender_id = Some(v.clone()); true }
            StepParameter::GazeVictimId(v) => { self.gaze_victim_id = v.clone(); true }
            StepParameter::HailMaryPassFlag(v) => { self.hail_mary_pass = *v; true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::TargetCoordinate(v) => { self.target_coordinate = Some(*v); true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::NumDice(v) => { self.num_dice = *v; true }
            StepParameter::UsingStab(v) => { self.using_stab = Some(*v); true }
            _ => false,
        }
    }
}

impl StepEndSelecting {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())
        // TODO: hide dialog — not yet ported

        // ── Branch 1: end turn or end player action ─────────────────────────────
        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // TODO(bloodlust): actingPlayer.isSufferingBloodLust() branch not yet ported

        // ── Dispatch ─────────────────────────────────────────────────────────────
        let player_action = if let Some(da) = self.dispatch_player_action {
            da
        } else {
            match game.acting_player.player_action {
                Some(a) => a,
                None => {
                    // Java: dispatchPlayerAction(null, false) → Select sequence
                    let seq = Select::build_sequence(&SelectParams { update_persistence: false });
                    return StepOutcome::next().push_seq(seq);
                }
            }
        };

        let with_parameter = self.dispatch_player_action.is_some();

        match player_action {
            PlayerAction::Pass
            | PlayerAction::HailMaryPass
            | PlayerAction::HandOver => {
                let seq = if with_parameter && self.target_coordinate.is_some() {
                    // Java: Pass.SequenceParams(gameState, fTargetCoordinate)
                    // Stub: pass with target coord
                    Pass::build_sequence(&PassParams::default())
                } else {
                    Pass::build_sequence(&PassParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::ThrowTeamMate => {
                let seq = ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    ..Default::default()
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::KickTeamMate => {
                let seq = KickTeamMate::build_sequence(&KickTeamMateParams::default());
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Blitz => {
                let seq = if with_parameter {
                    BlitzBlock::build_sequence(&BlitzBlockParams {
                        block_defender_id: self.block_defender_id.clone(),
                        using_stab: self.using_stab.unwrap_or(false),
                        ..Default::default()
                    })
                } else {
                    BlitzBlock::build_sequence(&BlitzBlockParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Block => {
                let seq = if with_parameter {
                    Block::build_sequence(&BlockParams {
                        block_defender_id: self.block_defender_id.clone(),
                        using_stab: self.using_stab.unwrap_or(false),
                        ..Default::default()
                    })
                } else {
                    Block::build_sequence(&BlockParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Foul => {
                let seq = Foul::build_sequence(&FoulParams::default());
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Move
            | PlayerAction::FoulMove
            | PlayerAction::PassMove
            | PlayerAction::ThrowTeamMateMove
            | PlayerAction::KickTeamMateMove
            | PlayerAction::HandOverMove
            | PlayerAction::Gaze => {
                // TODO(isRooted): MOVE + isRooted → EndPlayerAction not yet ported
                let seq = if with_parameter {
                    Move::build_sequence(&MoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        ..Default::default()
                    })
                } else {
                    Move::build_sequence(&MoveParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::BlitzMove => {
                let seq = if with_parameter {
                    BlitzMove::build_sequence(&BlitzMoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                    })
                } else {
                    BlitzMove::build_sequence(&BlitzMoveParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            // TODO(removeConfusion): REMOVE_CONFUSION + STAND_UP + STAND_UP_BLITZ paths not yet ported
            _ => {
                // Fallback: EndPlayerAction
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: false,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
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
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push EndPlayerAction sequence");
    }

    #[test]
    fn end_player_action_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn dispatch_block_pushes_block_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Block);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "BLOCK should push Block sequence");
    }

    #[test]
    fn dispatch_foul_pushes_foul_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Foul);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "FOUL should push Foul sequence");
    }

    #[test]
    fn dispatch_pass_pushes_pass_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Pass);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "PASS should push Pass sequence");
    }

    #[test]
    fn dispatch_move_pushes_move_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "MOVE should push Move sequence");
    }

    #[test]
    fn dispatch_blitz_move_pushes_blitz_move_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::BlitzMove);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "BLITZ_MOVE should push BlitzMove sequence");
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndSelecting::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_dispatch_player_action_accepted() {
        let mut step = StepEndSelecting::new();
        assert!(step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block))));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        let mut step = StepEndSelecting::new();
        let stack = vec![FieldCoordinate::new(5, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack.clone())));
        assert_eq!(step.move_stack, stack);
    }

    #[test]
    fn no_action_fallback_returns_next_step() {
        let mut game = make_game();
        // No dispatch_player_action and no acting_player.player_action → Select sequence
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
