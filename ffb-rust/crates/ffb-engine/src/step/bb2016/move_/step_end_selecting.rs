use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
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
use crate::step::util_server_steps::change_player_action;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepEndSelecting.
///
/// Last step in the BB2016 select sequence. Consumes all expected stepParameters.
/// Dispatches to the appropriate action sequence based on the player action.
///
/// Expects: BLOCK_DEFENDER_ID, DISPATCH_PLAYER_ACTION, END_PLAYER_ACTION, END_TURN,
///          FOUL_DEFENDER_ID, GAZE_VICTIM_ID, HAIL_MARY_PASS, MOVE_STACK,
///          TARGET_COORDINATE, THROWN_PLAYER_ID, KICKED_PLAYER_ID, NR_OF_DICE, USING_STAB.
///
/// isSufferingBloodLust, REMOVE_CONFUSION (with setHasMoved), STAND_UP / STAND_UP_BLITZ are all wired.
/// client-only: UtilServerDialog.hideDialog
/// no-op: gameCache.queueDbUpdate — headless engine has no DB layer (confirmed intentional).
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
        // client-only: UtilServerDialog.hideDialog

        // ── Branch 1: end turn or end player action ─────────────────────────────
        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if (actingPlayer.isSufferingBloodLust()) → force action to MOVE if not moving
        if game.acting_player.suffering_blood_lust {
            let effective_action = if let Some(da) = self.dispatch_player_action {
                if !da.is_moving() { PlayerAction::Move } else { da }
            } else {
                match game.acting_player.player_action {
                    Some(a) if !a.is_moving() => {
                        let pid = game.acting_player.player_id.clone();
                        let jumping = game.acting_player.jumping;
                        if let Some(id) = pid.as_deref() {
                            change_player_action(game, id, PlayerAction::Move, jumping);
                        }
                        PlayerAction::Move
                    }
                    Some(a) => a,
                    None => PlayerAction::Move,
                }
            };
            // dispatch without with_parameter
            let player_action = effective_action;
            let with_parameter = false;
            return self.dispatch_to_sequence(game, player_action, with_parameter);
        }

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
        self.dispatch_to_sequence(game, player_action, with_parameter)
    }

    fn dispatch_to_sequence(&self, game: &mut Game, player_action: PlayerAction, with_parameter: bool) -> StepOutcome {
        // Java: if (pPlayerAction == null || (pPlayerAction == MOVE && playerState.isRooted()
        //           && UtilPlayer.canGaze(game, actingPlayer.getPlayer()))) → Select sequence
        // (the null case is handled by the caller before dispatch_to_sequence is invoked)
        if player_action == PlayerAction::Move {
            let player_id = game.acting_player.player_id.clone();
            let is_rooted = player_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .map(|s| s.is_rooted())
                .unwrap_or(false);
            let can_gaze = player_id.as_deref()
                .map(|id| UtilPlayer::can_gaze(game, id))
                .unwrap_or(false);
            if is_rooted && can_gaze {
                let seq = Select::build_sequence(&SelectParams { update_persistence: false });
                return StepOutcome::next().push_seq(seq);
            }
        }
        match player_action {
            PlayerAction::Pass
            | PlayerAction::HailMaryPass
            | PlayerAction::HandOver => {
                StepOutcome::next().push_seq(Pass::build_sequence(&PassParams::default()))
            }
            PlayerAction::ThrowTeamMate => {
                let seq = ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    ..Default::default()
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::KickTeamMate => {
                StepOutcome::next().push_seq(KickTeamMate::build_sequence(&KickTeamMateParams::default()))
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
                StepOutcome::next().push_seq(Foul::build_sequence(&FoulParams::default()))
            }
            PlayerAction::Move
            | PlayerAction::FoulMove
            | PlayerAction::PassMove
            | PlayerAction::ThrowTeamMateMove
            | PlayerAction::KickTeamMateMove
            | PlayerAction::HandOverMove
            | PlayerAction::Gaze => {
                // Java: case MOVE: if (playerState.isRooted()) → EndPlayerAction; else fall through
                if player_action == PlayerAction::Move {
                    let is_rooted = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.field_model.player_state(id))
                        .map(|s| s.is_rooted())
                        .unwrap_or(false);
                    if is_rooted {
                        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                            feeding_allowed: true,
                            end_player_action: self.end_player_action,
                            end_turn: self.end_turn,
                        });
                        return StepOutcome::next().push_seq(seq);
                    }
                }
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
            PlayerAction::RemoveConfusion => {
                // Java: actingPlayer.setHasMoved(true); endGenerator.pushSequence(endParams)
                game.acting_player.has_moved = true;
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: false,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::StandUp => {
                // Java: if inflictsConfusion → move seq; else → end seq
                let has_gaze = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::INFLICTS_CONFUSION))
                    .unwrap_or(false);
                let seq = if has_gaze {
                    Move::build_sequence(&MoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        ..Default::default()
                    })
                } else {
                    EndPlayerAction::build_sequence(&EndPlayerActionParams {
                        feeding_allowed: true,
                        end_player_action: false,
                        end_turn: false,
                    })
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::StandUpBlitz => {
                // Java: game.getTurnData().setBlitzUsed(true); endGenerator.pushSequence(endParams)
                game.turn_data_mut().blitz_used = true;
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: false,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
            _ => {
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
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
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

    #[test]
    fn move_action_when_rooted_pushes_end_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let rooted_state = PlayerState::new(PS_STANDING).change_rooted(true);
        game.field_model.set_player_state("p1", rooted_state);
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "rooted Move player should push EndPlayerAction, not Move");
    }

    #[test]
    fn move_action_when_rooted_and_can_gaze_pushes_select_sequence() {
        // Java: dispatchPlayerAction() guard —
        //   if (pPlayerAction == MOVE && playerState.isRooted() && UtilPlayer.canGaze(...))
        //       → Select.pushSequence(...); return;
        // A rooted player who still has a gaze option available must be routed back to
        // Select (not straight to EndPlayerAction) so they can choose to gaze instead.
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;

        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "gazer".into(), name: "gazer".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::HypnoticGaze)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("gazer", FieldCoordinate::new(5, 5));
        let rooted_state = PlayerState::new(PS_STANDING).change_active(true).change_rooted(true);
        game.field_model.set_player_state("gazer", rooted_state);
        // adjacent target with a tackle zone, required for canGaze() to be true
        game.team_away.players.push(ffb_model::model::player::Player {
            id: "target".into(), name: "target".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("target", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("target", PlayerState::new(PS_STANDING));

        game.acting_player.player_id = Some("gazer".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.first().and_then(|seq| seq.first()).map(|s| s.step_id), Some(StepId::InitSelecting),
            "rooted player who canGaze must be routed back to Select, not EndPlayerAction");
    }

    #[test]
    fn move_action_when_not_rooted_pushes_move_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn blood_lust_non_moving_action_redirected_to_move_sequence() {
        // isSufferingBloodLust=true with a non-moving action (Block) → redirected to MOVE sequence
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "blood lust should push Move sequence for non-moving action");
    }

    #[test]
    fn blood_lust_already_moving_keeps_move_sequence() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "blood lust should still push Move sequence for moving action");
    }

    #[test]
    fn remove_confusion_sets_has_moved_and_pushes_end_player_action() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::RemoveConfusion);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "RemoveConfusion should push EndPlayerAction sequence");
        assert!(game.acting_player.has_moved, "RemoveConfusion must set has_moved=true");
    }

    #[test]
    fn stand_up_without_gaze_pushes_end_player_action() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::StandUp);
        // acting_player has no player_id set → no gaze skill → EndPlayerAction path
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "StandUp without HypnoticGaze should push EndPlayerAction");
    }

    #[test]
    fn stand_up_blitz_sets_blitz_used_and_pushes_end_player_action() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::StandUpBlitz);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "StandUpBlitz should push EndPlayerAction sequence");
        assert!(game.turn_data().blitz_used, "StandUpBlitz must set blitz_used=true");
    }
}
