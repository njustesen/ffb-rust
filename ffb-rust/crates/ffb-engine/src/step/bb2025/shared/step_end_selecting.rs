use ffb_model::enums::PlayerAction;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::auto_gaze_zoat::{AutoGazeZoat, AutoGazeZoatParams};
use crate::step::generator::bb2025::baleful_hex::{BalefulHex, BalefulHexParams};
use crate::step::generator::bb2025::black_ink::{BlackInk, BlackInkParams};
use crate::step::generator::bb2025::blitz_block::{BlitzBlock, BlitzBlockParams};
use crate::step::generator::bb2025::blitz_move::{BlitzMove, BlitzMoveParams};
use crate::step::generator::bb2025::block::{Block, BlockParams};
use crate::step::generator::bb2025::catch_of_the_day::{CatchOfTheDay, CatchOfTheDayParams};
use crate::step::generator::bb2025::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2025::foul::{Foul, FoulParams};
use crate::step::generator::bb2025::furious_outburst::FuriousOutburst;
use crate::step::generator::bb2025::look_into_my_eyes::{LookIntoMyEyes, LookIntoMyEyesParams};
use crate::step::generator::bb2025::move_::{Move, MoveParams};
use crate::step::generator::bb2025::multi_block::{MultiBlock, MultiBlockParams};
use crate::step::generator::bb2025::pass::{Pass, PassParams};
use crate::step::generator::bb2025::punt::Punt;
use crate::step::generator::bb2025::raiding_party::{RaidingParty, RaidingPartyParams};
use crate::step::generator::bb2025::select::{Select, SelectParams};
use crate::step::generator::bb2025::select_blitz_target::SelectBlitzTarget;
use crate::step::generator::bb2025::then_i_started_blastin::ThenIStartedBlastin as ThenIStartedBlastinGen;
use crate::step::generator::bb2025::throw_keg::{ThrowKeg, ThrowKegParams};
use crate::step::generator::bb2025::throw_team_mate::{ThrowTeamMate, ThrowTeamMateParams};
use crate::step::generator::bb2025::treacherous::{Treacherous, TreacherousParams};
use crate::step::generator::sequence::labels;
#[cfg(test)]
use crate::step::framework::StepAction;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepEndSelecting.
///
/// The last step in any SELECT sequence.  It reads all accumulated StepParameters
/// and dispatches to the appropriate action sequence by pushing generators onto
/// the step stack.
pub struct StepEndSelecting {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: bloodlustAction
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: fMoveStack (FieldCoordinate[])
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: fBlockDefenderId
    pub block_defender_id: Option<String>,
    /// Java: fUsingStab (Boolean tristate)
    pub using_stab: Option<bool>,
    /// Java: usingChainsaw
    pub using_chainsaw: bool,
    /// Java: usingVomit
    pub using_vomit: bool,
    /// Java: usingBreatheFire
    pub using_breathe_fire: bool,
    /// Java: usingChomp
    pub using_chomp: bool,
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
    /// Java: targetPlayerId
    pub target_player_id: Option<String>,
    /// Java: ballAndChainRrSetting
    pub ball_and_chain_rr_setting: Option<String>,
    /// Java: checkForgo
    pub check_forgo: bool,
    /// Java: blockTargets (List<BlockTarget> — here stored as defender IDs)
    pub block_targets: Vec<String>,
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
            using_chomp: false,
            foul_defender_id: None,
            target_coordinate: None,
            hail_mary_pass: false,
            kicked: false,
            thrown_player_id: None,
            kicked_player_id: None,
            num_dice: 0,
            target_player_id: None,
            ball_and_chain_rr_setting: None,
            check_forgo: false,
            block_targets: Vec::new(),
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
        // Java: no commands handled — only executeStep() in start()
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
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::TargetCoordinate(v) => { self.target_coordinate = Some(*v); true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::NrOfDice(v) => { self.num_dice = *v; true }
            StepParameter::UsingStab(v) => { self.using_stab = Some(*v); true }
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; true }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::UsingChomp(v) => { self.using_chomp = *v; true }
            StepParameter::BlockTargets(v) => { self.block_targets = v.clone(); true }
            StepParameter::IsKickedPlayer(v) => { self.kicked = *v; true }
            StepParameter::TargetPlayerId(v) => { self.target_player_id = v.clone(); true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            StepParameter::BallAndChainRrSetting(v) => { self.ball_and_chain_rr_setting = v.clone(); true }
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            _ => false,
        }
    }
}

impl StepEndSelecting {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())
        // DEFERRED(dialog-client): hide_dialog — dialog layer not yet translated

        // Java: if (fEndTurn || fEndPlayerAction) {
        //   game.getFieldModel().clearMultiBlockTargets()
        //   EndPlayerAction generator push (feeding_allowed=true, end_player_action=true, end_turn=fEndTurn, check_forgo=checkForgo)
        // }
        if self.end_turn || self.end_player_action {
            // Java: game.getFieldModel().clearMultiBlockTargets() — field model method not yet translated, skip
            let params = EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: self.check_forgo,
            };
            let seq = EndPlayerAction::build_sequence(&params);
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if (actingPlayer.isSufferingBloodLust()) { ... complex bloodlust dispatch ... }
        let suffering_blood_lust = game.acting_player.suffering_blood_lust;
        if suffering_blood_lust {
            if self.dispatch_player_action.is_some() || self.bloodlust_action.is_some() {
                let action = if let Some(bl) = self.bloodlust_action {
                    // Java: fDispatchPlayerAction = bloodlustAction
                    if bl == PlayerAction::Move {
                        // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), bloodlustAction, false)
                        if let Some(ref pid) = game.acting_player.player_id.clone() {
                            crate::step::util_server_steps::change_player_action(game, pid, bl, false);
                        }
                    }
                    bl
                } else {
                    let dpa = self.dispatch_player_action.unwrap();
                    if dpa == PlayerAction::Blitz {
                        // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), fDispatchPlayerAction, actingPlayer.isJumping())
                        let jumping = game.acting_player.jumping;
                        if let Some(ref pid) = game.acting_player.player_id.clone() {
                            crate::step::util_server_steps::change_player_action(game, pid, dpa, jumping);
                        }
                    } else if dpa == PlayerAction::Gaze {
                        // Java: fGazeVictimId = null
                        self.gaze_victim_id = None;
                    }
                    dpa
                };
                let with_param = self.bloodlust_action.is_none()
                    || !action.is_moving();
                return self.dispatch_player_action(game, action, with_param);
            } else {
                // Java: if (!actingPlayer.getPlayerAction().isMoving()) changePlayerAction to MOVE
                let current_action = game.acting_player.player_action;
                if let Some(act) = current_action {
                    if !act.is_moving() {
                        let jumping = game.acting_player.jumping;
                        if let Some(ref pid) = game.acting_player.player_id.clone() {
                            crate::step::util_server_steps::change_player_action(game, pid, PlayerAction::Move, jumping);
                        }
                    }
                }
                let act = game.acting_player.player_action;
                return self.dispatch_player_action(game, act.unwrap_or(PlayerAction::Move), false);
            }
        }

        // Java: else if (fDispatchPlayerAction != null) {
        //   dispatchPlayerAction(fDispatchPlayerAction, true)
        // }
        if let Some(dpa) = self.dispatch_player_action {
            return self.dispatch_player_action(game, dpa, true);
        }

        // Java: else {
        //   dispatchPlayerAction(actingPlayer.getPlayerAction(), false)
        // }
        let act = game.acting_player.player_action;
        self.dispatch_player_action(game, act.unwrap_or(PlayerAction::Move), false)
    }

    /// 1:1 translation of `private void dispatchPlayerAction(PlayerAction, boolean)`.
    ///
    /// Reads the acting player's state and pushes the appropriate sequence.
    /// `with_param` mirrors Java's `pWithParameter` — when true, the stored field values
    /// (block_defender_id, target_coordinate, move_stack, etc.) are forwarded as params.
    fn dispatch_player_action(
        &self,
        game: &mut Game,
        player_action: PlayerAction,
        with_param: bool,
    ) -> StepOutcome {
        // Java: PlayerState playerState = game.getFieldModel().getPlayerState(game.getActingPlayer().getPlayer())
        let player_state = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));

        // Java: if (pPlayerAction == null || (pPlayerAction == MOVE && playerState.isPinned() && UtilPlayer.canGaze(...)))
        //   → clearMultiBlockTargets; Select.push(false)
        let pinned_and_can_gaze = player_state
            .map(|ps| ps.is_pinned())
            .unwrap_or(false);
        let can_gaze = game.acting_player.player_id.as_deref()
            .map(|pid| UtilPlayer::can_gaze(game, pid))
            .unwrap_or(false);
        if player_action == PlayerAction::Move && pinned_and_can_gaze && can_gaze {
            // Java: game.getFieldModel().clearMultiBlockTargets()
            let seq = Select::build_sequence(&SelectParams { update_persistence: false, is_blitz_move: false });
            return StepOutcome::next().push_seq(seq);
        }

        // Common end-player-action params used by several cases below.
        let end_params = EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: false,
            check_forgo: false,
        };
        // Select params used by special-skill cases (Treacherous, RaidingParty, etc.).
        let select_params = SelectParams {
            update_persistence: true,
            is_blitz_move: false,
        };

        match player_action {
            // ── BLITZ_SELECT ──────────────────────────────────────────────────
            PlayerAction::BlitzSelect => {
                // Java: selectBlitzTarget.pushSequence(new SequenceGenerator.SequenceParams(getGameState()))
                let seq = SelectBlitzTarget::build_sequence();
                StepOutcome::next().push_seq(seq)
            }

            // ── PASS / HAIL_MARY_PASS / THROW_BOMB / HAIL_MARY_BOMB / HAND_OVER ──
            PlayerAction::Pass
            | PlayerAction::HailMaryPass
            | PlayerAction::ThrowBomb
            | PlayerAction::HailMaryBomb
            | PlayerAction::HandOver => {
                let params = PassParams {
                    target_coordinate: if with_param { self.target_coordinate } else { None },
                };
                let seq = Pass::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── THROW_TEAM_MATE / KICK_TEAM_MATE ─────────────────────────────
            PlayerAction::ThrowTeamMate | PlayerAction::KickTeamMate => {
                let params = if with_param {
                    ThrowTeamMateParams {
                        thrown_player_id: self.thrown_player_id.clone(),
                        is_kicked: self.kicked,
                        target_coordinate: self.target_coordinate,
                    }
                } else {
                    ThrowTeamMateParams::default()
                };
                let seq = ThrowTeamMate::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── BLITZ ─────────────────────────────────────────────────────────
            PlayerAction::Blitz => {
                let params = if with_param {
                    BlitzBlockParams {
                        block_defender_id: self.block_defender_id.clone(),
                        using_stab: self.using_stab.unwrap_or(false),
                        using_chainsaw: self.using_chainsaw,
                        using_vomit: self.using_vomit,
                        using_breathe_fire: self.using_breathe_fire,
                        using_chomp: self.using_chomp,
                        ..Default::default()
                    }
                } else {
                    BlitzBlockParams::default()
                };
                let seq = BlitzBlock::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── BLOCK ─────────────────────────────────────────────────────────
            PlayerAction::Block => {
                let params = if with_param {
                    BlockParams {
                        block_defender_id: self.block_defender_id.clone(),
                        using_stab: self.using_stab.unwrap_or(false),
                        using_chainsaw: self.using_chainsaw,
                        using_vomit: self.using_vomit,
                        using_breathe_fire: self.using_breathe_fire,
                        using_chomp: self.using_chomp,
                        ..Default::default()
                    }
                } else {
                    BlockParams::default()
                };
                let seq = Block::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── MULTIPLE_BLOCK ────────────────────────────────────────────────
            PlayerAction::MultipleBlock => {
                let params = if with_param {
                    MultiBlockParams { block_targets: self.block_targets.clone() }
                } else {
                    MultiBlockParams { block_targets: Vec::new() }
                };
                let seq = MultiBlock::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── FOUL ──────────────────────────────────────────────────────────
            PlayerAction::Foul => {
                let params = if with_param {
                    FoulParams {
                        fouled_defender_id: self.foul_defender_id.clone(),
                        using_chainsaw: self.using_chainsaw,
                    }
                } else {
                    FoulParams::default()
                };
                let seq = Foul::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── PUNT ──────────────────────────────────────────────────────────
            PlayerAction::Punt => {
                let seq = Punt::build_sequence();
                StepOutcome::next().push_seq(seq)
            }

            // ── MOVE (pinned) — falls into EndPlayerAction ────────────────────
            PlayerAction::Move if player_state.map(|ps| ps.is_pinned()).unwrap_or(false) => {
                let seq = EndPlayerAction::build_sequence(&end_params);
                StepOutcome::next().push_seq(seq)
            }

            // ── MOVE / FOUL_MOVE / PASS_MOVE / THROW_TEAM_MATE_MOVE / ─────────
            // ── KICK_TEAM_MATE_MOVE / HAND_OVER_MOVE / GAZE / PUNT_MOVE / ─────
            // ── SECURE_THE_BALL ──────────────────────────────────────────────
            PlayerAction::Move
            | PlayerAction::FoulMove
            | PlayerAction::PassMove
            | PlayerAction::ThrowTeamMateMove
            | PlayerAction::KickTeamMateMove
            | PlayerAction::HandOverMove
            | PlayerAction::Gaze
            | PlayerAction::PuntMove
            | PlayerAction::SecureTheBall => {
                let params = if with_param {
                    MoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        move_start: self.move_start,
                        ball_and_chain_rr_setting: self.ball_and_chain_rr_setting.clone(),
                        bloodlust_action: None,
                    }
                } else {
                    MoveParams::default()
                };
                let seq = Move::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── BLITZ_MOVE / KICK_EM_BLITZ ────────────────────────────────────
            PlayerAction::BlitzMove | PlayerAction::KickEmBlitz => {
                let params = if with_param {
                    BlitzMoveParams {
                        move_stack: self.move_stack.clone(),
                        gaze_victim_id: self.gaze_victim_id.clone(),
                        move_start: self.move_start,
                    }
                } else {
                    BlitzMoveParams::default()
                };
                let seq = BlitzMove::build_sequence(&params);
                StepOutcome::next().push_seq(seq)
            }

            // ── REMOVE_CONFUSION ──────────────────────────────────────────────
            PlayerAction::RemoveConfusion => {
                // Java: actingPlayer.setHasMoved(true); endGenerator.pushSequence(endParams)
                game.acting_player.has_moved = true;
                let seq = EndPlayerAction::build_sequence(&end_params);
                StepOutcome::next().push_seq(seq)
            }

            // ── STAND_UP ──────────────────────────────────────────────────────
            PlayerAction::StandUp => {
                let seq = EndPlayerAction::build_sequence(&end_params);
                StepOutcome::next().push_seq(seq)
            }

            // ── STAND_UP_BLITZ ────────────────────────────────────────────────
            PlayerAction::StandUpBlitz => {
                // Java: game.getTurnData().setBlitzUsed(true); endGenerator.pushSequence(endParams)
                game.turn_data_mut().blitz_used = true;
                let seq = EndPlayerAction::build_sequence(&end_params);
                StepOutcome::next().push_seq(seq)
            }

            // ── TREACHEROUS ───────────────────────────────────────────────────
            PlayerAction::Treacherous => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       treacherousGenerator.pushSequence(new Treacherous.SequenceParams(getGameState(), IStepLabel.END_SELECTING))
                // Both sequences pushed; driver pushes in reverse so Treacherous runs first.
                let select_seq = Select::build_sequence(&select_params);
                let treacherous_seq = Treacherous::build_sequence(&TreacherousParams {
                    failure_label: labels::END_SELECTING.into(),
                });
                StepOutcome::next()
                    .push_seq(treacherous_seq)
                    .push_seq(select_seq)
            }

            // ── RAIDING_PARTY ─────────────────────────────────────────────────
            PlayerAction::RaidingParty => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       radingParty.pushSequence(new RadingParty.SequenceParams(getGameState(), IStepLabel.END_SELECTING, null))
                let select_seq = Select::build_sequence(&select_params);
                let raiding_seq = RaidingParty::build_sequence(&RaidingPartyParams {
                    failure_label: labels::END_SELECTING.into(),
                    success_label: String::new(),
                });
                StepOutcome::next()
                    .push_seq(raiding_seq)
                    .push_seq(select_seq)
            }

            // ── WISDOM_OF_THE_WHITE_DWARF ─────────────────────────────────────
            PlayerAction::WisdomOfTheWhiteDwarf => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       Sequence sequence = new Sequence(getGameState()); sequence.add(StepId.WISDOM_OF_THE_WHITE_DWARF)
                //       getGameState().getStepStack().push(sequence.getSequence())
                let select_seq = Select::build_sequence(&select_params);
                let wisdom_seq = vec![
                    crate::step::framework::SequenceStep::new(StepId::WisdomOfTheWhiteDwarf),
                ];
                StepOutcome::next()
                    .push_seq(wisdom_seq)
                    .push_seq(select_seq)
            }

            // ── THROW_KEG ─────────────────────────────────────────────────────
            PlayerAction::ThrowKeg => {
                // Java: ThrowKeg.pushSequence(new ThrowKeg.SequenceParams(getGameState(), targetPlayerId))
                let seq = ThrowKeg::build_sequence(&ThrowKegParams {
                    player_id: self.target_player_id.clone(),
                });
                StepOutcome::next().push_seq(seq)
            }

            // ── LOOK_INTO_MY_EYES ─────────────────────────────────────────────
            PlayerAction::LookIntoMyEyes => {
                // Java: lookIntoMyEyes.pushSequence(new LookIntoMyEyes.SequenceParams(getGameState(), true, null))
                let seq = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams {
                    push_select: true,
                    goto_on_end: String::new(),
                });
                StepOutcome::next().push_seq(seq)
            }

            // ── BALEFUL_HEX ───────────────────────────────────────────────────
            PlayerAction::BalefulHex => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       balefulGenerator.pushSequence(new BalefulHex.SequenceParams(getGameState(), IStepLabel.END_SELECTING))
                let select_seq = Select::build_sequence(&select_params);
                let baleful_seq = BalefulHex::build_sequence(&BalefulHexParams {
                    failure_label: labels::END_SELECTING.into(),
                });
                StepOutcome::next()
                    .push_seq(baleful_seq)
                    .push_seq(select_seq)
            }

            // ── BLACK_INK ─────────────────────────────────────────────────────
            PlayerAction::BlackInk => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       blackInkGenerator.pushSequence(new BlackInk.SequenceParams(getGameState(), IStepLabel.END_SELECTING, playerState))
                let select_seq = Select::build_sequence(&select_params);
                let black_ink_seq = BlackInk::build_sequence(&BlackInkParams {
                    failure_label: labels::END_SELECTING.into(),
                    old_player_state: player_state,
                });
                StepOutcome::next()
                    .push_seq(black_ink_seq)
                    .push_seq(select_seq)
            }

            // ── CATCH_OF_THE_DAY ──────────────────────────────────────────────
            PlayerAction::CatchOfTheDay => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       cotdGenerator.pushSequence(new CatchOfTheDay.SequenceParams(getGameState(), IStepLabel.END_SELECTING))
                let select_seq = Select::build_sequence(&select_params);
                let cotd_seq = CatchOfTheDay::build_sequence(&CatchOfTheDayParams {
                    failure_label: labels::END_SELECTING.into(),
                });
                StepOutcome::next()
                    .push_seq(cotd_seq)
                    .push_seq(select_seq)
            }

            // ── THEN_I_STARTED_BLASTIN ────────────────────────────────────────
            PlayerAction::ThenIStartedBlastin => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       tisbGenerator.pushSequence(new ThenIStartedBlastin.SequenceParams(getGameState()))
                let select_seq = Select::build_sequence(&select_params);
                let tisb_seq = ThenIStartedBlastinGen::build_sequence();
                StepOutcome::next()
                    .push_seq(tisb_seq)
                    .push_seq(select_seq)
            }

            // ── FURIOUS_OUTBURST ──────────────────────────────────────────────
            PlayerAction::FuriousOutburst => {
                // Java: FuriousOutburst.pushSequence(new SequenceGenerator.SequenceParams(getGameState()))
                let seq = FuriousOutburst::build_sequence();
                StepOutcome::next().push_seq(seq)
            }

            // ── AUTO_GAZE_ZOAT ────────────────────────────────────────────────
            PlayerAction::AutoGazeZoat => {
                // Java: selectGenerator.pushSequence(selectParams)
                //       autoGazeZoatGenerator.pushSequence(new AutoGazeZoat.SequenceParams(getGameState(), IStepLabel.END_SELECTING, playerState))
                let select_seq = Select::build_sequence(&select_params);
                let agz_seq = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
                    failure_label: labels::END_SELECTING.into(),
                    old_player_state: player_state,
                });
                StepOutcome::next()
                    .push_seq(agz_seq)
                    .push_seq(select_seq)
            }

            // ── FORGO ─────────────────────────────────────────────────────────
            PlayerAction::Forgo => {
                // Java: actingPlayer.setForgone(true); endGenerator.pushSequence(endParams)
                game.acting_player.forgone = true;
                let seq = EndPlayerAction::build_sequence(&end_params);
                StepOutcome::next().push_seq(seq)
            }

            // ── unhandled — Java throws IllegalStateException ─────────────────
            other => {
                // Java: throw new IllegalStateException("Unhandled player action " + pPlayerAction.getName() + ".")
                panic!("StepEndSelecting: unhandled player action {:?}", other);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerAction, Rules};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_true_pushes_end_player_action_sequence() {
        // Java: fEndTurn → pushSequence(EndPlayerAction) → NEXT_STEP
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Should have pushed a sequence
        assert_eq!(out.pushes.len(), 1);
        // The EndPlayerAction sequence starts with RemoveTargetSelectionState
        let seq = &out.pushes[0];
        assert_eq!(seq[0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_player_action_true_pushes_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn check_forgo_propagated_to_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.end_turn = true;
        step.check_forgo = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        let seq = &out.pushes[0];
        // EndFeeding step (last) should have CheckForgo(true)
        let end_feeding = seq.iter().find(|s| s.step_id == StepId::EndFeeding).unwrap();
        assert!(end_feeding.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn dispatch_player_action_block_pushes_block_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Block);
        step.block_defender_id = Some("def1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // Block sequence starts with InitBlocking
        assert_eq!(out.pushes[0][0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn dispatch_player_action_pass_pushes_pass_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Pass);
        step.target_coordinate = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
        assert_eq!(out.pushes[0][0].step_id, StepId::InitPassing);
    }

    #[test]
    fn dispatch_player_action_move_pushes_move_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
        assert_eq!(out.pushes[0][0].step_id, StepId::InitMoving);
    }

    #[test]
    fn dispatch_player_action_foul_pushes_foul_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Foul);
        step.foul_defender_id = Some("def2".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFouling);
    }

    #[test]
    fn dispatch_player_action_blitz_pushes_blitz_block_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Blitz);
        step.block_defender_id = Some("def3".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
        // BlitzBlock sequence starts with InitBlocking
        assert_eq!(out.pushes[0][0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn dispatch_player_action_stand_up_pushes_end_player_action() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::StandUp);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn dispatch_player_action_stand_up_blitz_sets_blitz_used() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::StandUpBlitz);
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn dispatch_player_action_remove_confusion_sets_has_moved() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::RemoveConfusion);
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_moved);
    }

    #[test]
    fn dispatch_player_action_treacherous_pushes_two_sequences() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Treacherous);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Two sequences: Treacherous + Select
        assert_eq!(out.pushes.len(), 2);
    }

    #[test]
    fn dispatch_player_action_blitz_select_pushes_select_blitz_target() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::BlitzSelect);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::SelectBlitzTarget);
    }

    #[test]
    fn dispatch_player_action_blitz_move_pushes_blitz_move_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::BlitzMove);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitMoving);
    }

    #[test]
    fn dispatch_player_action_punt_pushes_punt_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::Punt);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitPunt);
    }

    #[test]
    fn dispatch_player_action_furious_outburst_pushes_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::FuriousOutburst);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFuriousOutburst);
    }

    #[test]
    fn dispatch_player_action_throw_keg_pushes_throw_keg_sequence() {
        let mut game = make_game();
        let mut step = StepEndSelecting::new();
        step.dispatch_player_action = Some(PlayerAction::ThrowKeg);
        step.target_player_id = Some("p99".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::ThrowKeg);
    }

    #[test]
    fn set_parameter_block_defender_id() {
        let mut step = StepEndSelecting::default();
        let accepted = step.set_parameter(&StepParameter::BlockDefenderId("def1".into()));
        assert!(accepted);
        assert_eq!(step.block_defender_id.as_deref(), Some("def1"));
    }

    #[test]
    fn set_parameter_move_stack() {
        let mut step = StepEndSelecting::default();
        let coords = vec![FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5)];
        let accepted = step.set_parameter(&StepParameter::MoveStack(coords.clone()));
        assert!(accepted);
        assert_eq!(step.move_stack, coords);
    }

    #[test]
    fn set_parameter_check_forgo() {
        let mut step = StepEndSelecting::default();
        let accepted = step.set_parameter(&StepParameter::CheckForgo(true));
        assert!(accepted);
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_dispatch_player_action() {
        let mut step = StepEndSelecting::default();
        let accepted = step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block)));
        assert!(accepted);
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn set_parameter_using_stab() {
        let mut step = StepEndSelecting::default();
        let accepted = step.set_parameter(&StepParameter::UsingStab(true));
        assert!(accepted);
        assert_eq!(step.using_stab, Some(true));
    }

    #[test]
    fn set_parameter_block_targets() {
        let mut step = StepEndSelecting::default();
        let targets = vec!["p1".into(), "p2".into()];
        let accepted = step.set_parameter(&StepParameter::BlockTargets(targets.clone()));
        assert!(accepted);
        assert_eq!(step.block_targets, targets);
    }
}
