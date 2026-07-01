use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::change_player_action;
use crate::step::generator::bb2020::{
    Block, BlitzBlock, BlitzMove, EndPlayerAction, Foul, Move, Pass, ThrowTeamMate,
    Select, SelectBlitzTarget, SelectGazeTarget, MultiBlock,
    BalefulHex, BlackInk, CatchOfTheDay, ThenIStartedBlastin, FuriousOutburst,
    LookIntoMyEyes, RaidingParty, ThrowKeg, Treacherous,
};
use crate::step::generator::bb2020::block::BlockParams;
use crate::step::generator::bb2020::blitz_block::BlitzBlockParams;
use crate::step::generator::bb2020::blitz_move::BlitzMoveParams;
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2020::foul::FoulParams;
use crate::step::generator::bb2020::move_::MoveParams;
use crate::step::generator::bb2020::pass::PassParams;
use crate::step::generator::bb2020::throw_team_mate::ThrowTeamMateParams;
use crate::step::generator::bb2020::select::SelectParams;
use crate::step::generator::bb2020::multi_block::MultiBlockParams;
use crate::step::generator::bb2020::baleful_hex::BalefulHexParams;
use crate::step::generator::bb2020::black_ink::BlackInkParams;
use crate::step::generator::bb2020::catch_of_the_day::CatchOfTheDayParams;
use crate::step::generator::bb2020::look_into_my_eyes::LookIntoMyEyesParams;
use crate::step::generator::bb2020::raiding_party::RaidingPartyParams;
use crate::step::generator::bb2020::throw_keg::ThrowKegParams;
use crate::step::generator::bb2020::treacherous::TreacherousParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepEndSelecting.
///
/// Last step in the BB2020 select sequence. Dispatches to the appropriate action sequence.
///
/// Expects: BLOCK_DEFENDER_ID, DISPATCH_PLAYER_ACTION, END_PLAYER_ACTION, END_TURN,
///          FOUL_DEFENDER_ID, GAZE_VICTIM_ID, HAIL_MARY_PASS, MOVE_STACK,
///          TARGET_COORDINATE, THROWN_PLAYER_ID, KICKED_PLAYER_ID, NR_OF_DICE, USING_STAB,
///          USING_CHAINSAW, USING_VOMIT, USING_BREATHE_FIRE, BLOCK_TARGETS, BLOOD_LUST_ACTION,
///          BALL_AND_CHAIN_RE_ROLL_SETTING, TARGET_PLAYER_ID.
///
pub struct StepEndSelecting {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: bloodlustAction
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: fBlockDefenderId
    pub block_defender_id: Option<String>,
    /// Java: fUsingStab
    pub using_stab: Option<bool>,
    /// Java: usingChainsaw
    pub using_chainsaw: bool,
    /// Java: usingVomit
    pub using_vomit: bool,
    /// Java: usingBreatheFire
    pub using_breathe_fire: bool,
    /// Java: fFoulDefenderId
    pub foul_defender_id: Option<String>,
    /// Java: fTargetCoordinate
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: fHailMaryPass
    pub hail_mary_pass: bool,
    /// Java: kicked
    pub kicked: bool,
    /// Java: fThrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: fKickedPlayerId
    pub kicked_player_id: Option<String>,
    /// Java: fNumDice
    pub num_dice: i32,
    /// Java: blockTargets
    pub block_targets: Vec<String>,
    /// Java: targetPlayerId
    pub target_player_id: Option<String>,
    /// Java: ballAndChainRrSetting
    pub ball_and_chain_rr_setting: Option<String>,
}

impl StepEndSelecting {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            dispatch_player_action: None,
            bloodlust_action: None,
            move_stack: Vec::new(),
            move_start: None,
            gaze_victim_id: None,
            block_defender_id: None,
            using_stab: None,
            using_chainsaw: false,
            using_vomit: false,
            using_breathe_fire: false,
            foul_defender_id: None,
            target_coordinate: None,
            hail_mary_pass: false,
            kicked: false,
            thrown_player_id: None,
            kicked_player_id: None,
            num_dice: 0,
            block_targets: Vec::new(),
            target_player_id: None,
            ball_and_chain_rr_setting: None,
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
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::TargetCoordinate(v) => { self.target_coordinate = Some(*v); true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::NumDice(v) => { self.num_dice = *v; true }
            StepParameter::UsingStab(v) => { self.using_stab = Some(*v); true }
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; true }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::BlockTargets(v) => { self.block_targets = v.clone(); true }
            StepParameter::IsKickedPlayer(v) => { self.kicked = *v; true }
            StepParameter::TargetPlayerId(v) => { self.target_player_id = v.clone(); true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            StepParameter::BallAndChainRrSetting(v) => { self.ball_and_chain_rr_setting = v.clone(); true }
            _ => false,
        }
    }
}

impl StepEndSelecting {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())

        // ── Branch 1: end turn or end player action ─────────────────────────────
        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // ── Branch 2: isSufferingBloodLust ──────────────────────────────────────
        // Java: } else if (actingPlayer.isSufferingBloodLust()) {
        if game.acting_player.suffering_blood_lust {
            if self.dispatch_player_action.is_some() || self.bloodlust_action.is_some() {
                if let Some(ba) = self.bloodlust_action {
                    self.dispatch_player_action = Some(ba);
                    if ba == PlayerAction::Move {
                        if let Some(ref pid) = game.acting_player.player_id.clone() {
                            change_player_action(game, pid, ba, false);
                        }
                    }
                } else if self.dispatch_player_action == Some(PlayerAction::Blitz) {
                    let jumping = game.acting_player.jumping;
                    if let Some(ref pid) = game.acting_player.player_id.clone() {
                        change_player_action(game, pid, PlayerAction::Blitz, jumping);
                    }
                }
                // Java: dispatchPlayerAction(fDispatchPlayerAction, bloodlustAction == null || !fDispatchPlayerAction.isMoving())
                let with_parameter = self.bloodlust_action.is_none()
                    || !self.dispatch_player_action.map(|a| a.is_moving()).unwrap_or(false);
                return self.dispatch_action(game, self.dispatch_player_action, with_parameter);
            } else {
                // Java: if (actingPlayer.getPlayerAction() != null && !actingPlayer.getPlayerAction().isMoving())
                //           → changePlayerAction(MOVE, jumping)
                //       dispatchPlayerAction(actingPlayer.getPlayerAction(), false)
                if let Some(action) = game.acting_player.player_action {
                    if !action.is_moving() {
                        let jumping = game.acting_player.jumping;
                        if let Some(ref pid) = game.acting_player.player_id.clone() {
                            change_player_action(game, pid, PlayerAction::Move, jumping);
                        }
                    }
                }
                let current_action = game.acting_player.player_action;
                return self.dispatch_action(game, current_action, false);
            }
        }

        // ── Branch 3: dispatch_player_action set (non-bloodlust) ────────────────
        if self.dispatch_player_action.is_some() {
            return self.dispatch_action(game, self.dispatch_player_action, true);
        }

        // ── Branch 4: fall through to acting player's current action ─────────────
        let current_action = game.acting_player.player_action;
        self.dispatch_action(game, current_action, false)
    }

    fn dispatch_action(&self, game: &mut Game, player_action: Option<PlayerAction>, with_parameter: bool) -> StepOutcome {
        // ── Null / isRooted + canGaze guard ──────────────────────────────────────
        // Java: if (pPlayerAction == null ||
        //           (pPlayerAction == MOVE && playerState.isRooted() && UtilPlayer.canGaze(...)))
        let player_state = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));

        let rooted_and_can_gaze = player_action == Some(PlayerAction::Move)
            && player_state.map(|s| s.is_rooted()).unwrap_or(false)
            && game.acting_player.player_id.as_deref()
                .map(|id| UtilPlayer::can_gaze(game, id))
                .unwrap_or(false);

        if player_action.is_none() || rooted_and_can_gaze {
            let seq = Select::build_sequence(&SelectParams {
                update_persistence: false,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(seq);
        }

        let player_action = player_action.unwrap();

        match player_action {
            PlayerAction::BlitzSelect => {
                StepOutcome::next().push_seq(SelectBlitzTarget::build_sequence())
            }
            PlayerAction::GazeSelect => {
                StepOutcome::next().push_seq(SelectGazeTarget::build_sequence())
            }
            PlayerAction::Pass
            | PlayerAction::HailMaryPass
            | PlayerAction::ThrowBomb
            | PlayerAction::HailMaryBomb
            | PlayerAction::HandOver => {
                StepOutcome::next().push_seq(Pass::build_sequence(&PassParams::default()))
            }
            PlayerAction::ThrowTeamMate
            | PlayerAction::KickTeamMate => {
                let seq = if with_parameter {
                    ThrowTeamMate::build_sequence(&ThrowTeamMateParams {
                        thrown_player_id: self.thrown_player_id.clone(),
                        target_coordinate: self.target_coordinate,
                        is_kicked: self.kicked,
                    })
                } else {
                    ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Blitz => {
                let seq = if with_parameter {
                    BlitzBlock::build_sequence(&BlitzBlockParams {
                        block_defender_id: self.block_defender_id.clone(),
                        using_stab: self.using_stab.unwrap_or(false),
                        using_chainsaw: self.using_chainsaw,
                        using_vomit: self.using_vomit,
                        using_breathe_fire: self.using_breathe_fire,
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
                        using_chainsaw: self.using_chainsaw,
                        using_vomit: self.using_vomit,
                        using_breathe_fire: self.using_breathe_fire,
                        ..Default::default()
                    })
                } else {
                    Block::build_sequence(&BlockParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::MultipleBlock => {
                let seq = if with_parameter {
                    MultiBlock::build_sequence(&MultiBlockParams {
                        block_targets: self.block_targets.clone(),
                    })
                } else {
                    MultiBlock::build_sequence(&MultiBlockParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Foul => {
                let seq = if with_parameter {
                    Foul::build_sequence(&FoulParams {
                        foul_defender_id: self.foul_defender_id.clone(),
                        using_chainsaw: self.using_chainsaw,
                    })
                } else {
                    Foul::build_sequence(&FoulParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Move
            | PlayerAction::FoulMove
            | PlayerAction::PassMove
            | PlayerAction::ThrowTeamMateMove
            | PlayerAction::KickTeamMateMove
            | PlayerAction::HandOverMove
            | PlayerAction::Gaze => {
                // Java: case MOVE: if (playerState.isRooted()) { endGenerator.pushSequence(endParams); break; }
                // (fall through to FOUL_MOVE etc. if not rooted)
                if player_action == PlayerAction::Move {
                    let is_rooted = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.field_model.player_state(id))
                        .map(|s| s.is_rooted())
                        .unwrap_or(false);
                    if is_rooted {
                        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                            feeding_allowed: true,
                            end_player_action: true,
                            end_turn: false,
                        });
                        return StepOutcome::next().push_seq(seq);
                    }
                }
                let seq = if with_parameter {
                    Move::build_sequence(&MoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        move_start: self.move_start,
                        ball_and_chain_rr_setting: self.ball_and_chain_rr_setting.clone(),
                        bloodlust_action: None,
                    })
                } else {
                    Move::build_sequence(&MoveParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::BlitzMove | PlayerAction::KickEmBlitz => {
                let seq = if with_parameter {
                    BlitzMove::build_sequence(&BlitzMoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        move_start: self.move_start,
                    })
                } else {
                    BlitzMove::build_sequence(&BlitzMoveParams::default())
                };
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::RemoveConfusion => {
                // Java: actingPlayer.setHasMoved(true); endGenerator.pushSequence(endParams)
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: true,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::StandUp => {
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: true,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::StandUpBlitz => {
                // Java: game.getTurnData().setBlitzUsed(true); endGenerator.pushSequence(endParams)
                game.turn_data_mut().blitz_used = true;
                let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: true,
                    end_turn: false,
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::Treacherous => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let treacherous_seq = Treacherous::build_sequence(&TreacherousParams {
                    failure_label: "END_SELECTING".into(),
                });
                StepOutcome::next().push_seq(select_seq).push_seq(treacherous_seq)
            }
            PlayerAction::RaidingParty => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let raiding_seq = RaidingParty::build_sequence(&RaidingPartyParams {
                    failure_label: "END_SELECTING".into(),
                    success_label: String::new(),
                });
                StepOutcome::next().push_seq(select_seq).push_seq(raiding_seq)
            }
            PlayerAction::ThrowKeg => {
                let seq = ThrowKeg::build_sequence(&ThrowKegParams {
                    player_id: self.target_player_id.clone(),
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::LookIntoMyEyes => {
                let seq = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams {
                    push_select: true,
                    goto_on_end: String::new(),
                });
                StepOutcome::next().push_seq(seq)
            }
            PlayerAction::BalefulHex => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let bh_seq = BalefulHex::build_sequence(&BalefulHexParams {
                    failure_label: "END_SELECTING".into(),
                });
                StepOutcome::next().push_seq(select_seq).push_seq(bh_seq)
            }
            PlayerAction::BlackInk => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let bi_seq = BlackInk::build_sequence(&BlackInkParams {
                    failure_label: "END_SELECTING".into(),
                    old_player_state: None,
                });
                StepOutcome::next().push_seq(select_seq).push_seq(bi_seq)
            }
            PlayerAction::CatchOfTheDay => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let cotd_seq = CatchOfTheDay::build_sequence(&CatchOfTheDayParams {
                    failure_label: "END_SELECTING".into(),
                });
                StepOutcome::next().push_seq(select_seq).push_seq(cotd_seq)
            }
            PlayerAction::ThenIStartedBlastin => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: true, ..Default::default() });
                let tisb_seq = ThenIStartedBlastin::build_sequence();
                StepOutcome::next().push_seq(select_seq).push_seq(tisb_seq)
            }
            PlayerAction::FuriousOutburst => {
                let seq = FuriousOutburst::build_sequence();
                StepOutcome::next().push_seq(seq)
            }
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
        Game::new(home, away, Rules::Bb2020)
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
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
