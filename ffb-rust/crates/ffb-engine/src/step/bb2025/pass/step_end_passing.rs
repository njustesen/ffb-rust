use ffb_model::enums::{PassingDistance, PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::spp_calc::SppCalc;
use crate::step::util_server_steps::check_touchdown;
use crate::util::util_server_player_move::UtilServerPlayerMove;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2025::bomb::{Bomb, BombParams};
use crate::step::generator::bb2025::move_::{Move, MoveParams};
use crate::step::generator::bb2025::pass::{Pass, PassParams};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepEndPassing.
///
/// Final step of the pass sequence.  Consumes all pass-related parameters and routes
/// to the correct continuation:
///
///  • EndPlayerAction + bomb → EndPlayerAction generator (feeding_allowed=true, end_player_action=true, end_turn).
///  • BloodLust action       → Move generator (not yet fully wired — ActingPlayer.suffering_blood_lust not translated).
///  • Bomb turn              → Bomb generator.
///  • Animosity re-try       → Pass generator.
///  • Otherwise: compute SPP (completions/catches — TODO: SppMechanic), determine whether
///    turn/player-action ends, and push EndPlayerAction or Move generator.
pub struct StepEndPassing {
    /// Java: fInterceptorId
    pub interceptor_id: Option<String>,
    /// Java: PassState.interceptionSuccessful — for a BOMB, whether the interception was an
    /// OUTRIGHT catch (bb2020: only an easy intercept; bb2025: any success). Java's bb2020
    /// StepIntercept publishes INTERCEPTOR_ID only inside its easyIntercept branch, so a plain
    /// deflection never hands EndPassing an interceptor; Rust publishes the id unconditionally
    /// (its ResolvePass needs it in place of Java's shared PassState), so this flag carries the
    /// distinction instead.
    pub interception_successful: bool,
    /// Java bb2020: `PassState.deflectionSuccessful` (published by StepIntercept).
    pub deflection_successful: bool,
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: ballSnatcherId (from PlayerId parameter)
    pub ball_snatcher_id: Option<String>,
    /// Java: fPassAccurate
    pub pass_accurate: bool,
    /// Java: fPassFumble
    pub pass_fumble: bool,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fBombOutOfBounds
    pub bomb_out_of_bounds: bool,
    /// Java: dontDropFumble
    pub dont_drop_fumble: bool,
    /// Java: passingDistance
    pub passing_distance: Option<PassingDistance>,
    /// Java: bloodlustAction (from BLOOD_LUST_ACTION parameter)
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: REVERT_END_TURN → fEndTurn = false
    pub revert_end_turn: bool,
}

impl StepEndPassing {
    pub fn new() -> Self {
        Self {
            interceptor_id: None,
            interception_successful: false,
            deflection_successful: false,
            catcher_id: None,
            ball_snatcher_id: None,
            pass_accurate: false,
            pass_fumble: false,
            end_turn: false,
            end_player_action: false,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
            passing_distance: None,
            bloodlust_action: None,
            revert_end_turn: false,
        }
    }
}

impl Default for StepEndPassing {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndPassing {
    fn id(&self) -> StepId { StepId::EndPassing }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: consume(parameter) in all cases below
            StepParameter::InterceptionSuccessful(v) => {
                self.interception_successful = *v;
                true
            }
            StepParameter::InterceptorId(v) => { self.interceptor_id = v.clone(); true }
            // Java bb2020: `PassState.isDeflectionSuccessful()` gates the interception block.
            StepParameter::DeflectionSuccessful(v) => { self.deflection_successful = *v; true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::PassAccurate(v) => { self.pass_accurate = *v; true }
            StepParameter::PassFumble(v) => { self.pass_fumble = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::BombOutOfBounds(v) => { self.bomb_out_of_bounds = *v; true }
            StepParameter::DontDropFumble(v) => { self.dont_drop_fumble = *v; true }
            StepParameter::PassingDistance(v) => { self.passing_distance = Some(*v); true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            // Java: REVERT_END_TURN → fEndTurn = false (not a simple assignment)
            StepParameter::RevertEndTurn(_) => { self.end_turn = false; true }
            // Java: PLAYER_ID → ballSnatcherId
            StepParameter::PlayerId(v) => { self.ball_snatcher_id = Some(v.clone()); true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::CatcherId(_) | StepParameter::EndPlayerAction(_)
            | StepParameter::EndTurn(_) | StepParameter::InterceptorId(_)
            | StepParameter::PassAccurate(_) | StepParameter::PassFumble(_)
            | StepParameter::DontDropFumble(_) | StepParameter::BombOutOfBounds(_)
            | StepParameter::PassingDistance(_) | StepParameter::BloodLustAction(_)
            | StepParameter::RevertEndTurn(_) | StepParameter::PlayerId(_))
    }
}

impl StepEndPassing {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(gameState)
        crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);
        // Java: fieldModel.setRangeRuler(null)
        game.field_model.range_ruler = None;
        // Java: fieldModel.setOutOfBounds(false)
        game.field_model.out_of_bounds = false;

        let acting_action = game.acting_player.player_action;
        let is_bomb = acting_action == Some(PlayerAction::ThrowBomb);

        // Java path 1: EndPlayerAction + (bomb or hail-mary bomb) → EndPlayerAction generator
        // Java: endGenerator.pushSequence(new EndPlayerAction.SequenceParams(getGameState(), true, true, fEndTurn))
        if self.end_player_action
            && (is_bomb || acting_action == Some(PlayerAction::HailMaryBomb))
        {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: false,
                            rules: game.rules,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 2: BloodLust + bloodlustAction → Move generator
        if game.acting_player.suffering_blood_lust && self.bloodlust_action.is_some() {
            // Java: actingPlayer.setHasPassed(false); game.setPassCoordinate(null);
            game.acting_player.has_passed = false;
            game.pass_coordinate = None;
            // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), bloodlustAction, false);
            if let Some(action) = self.bloodlust_action {
                game.acting_player.player_action = Some(action);
            }
            // Java: moveGenerator.pushSequence(new Move.SequenceParams(getGameState()));
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: allowMoveAfterPass: QuickPass distance + canMoveAfterQuickPass skill + !pass_fumble
        // Java: allowMoveAfterHandOff: HAND_OVER action + canMoveAfterHandOff skill
        let thrower_has_quick_pass_skill = game.thrower_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_QUICK_PASS))
            .unwrap_or(false);
        let allow_move_after_pass = self.passing_distance == Some(PassingDistance::QuickPass)
            && thrower_has_quick_pass_skill
            && !self.pass_fumble;
        let thrower_has_hand_off_skill = game.thrower_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_HAND_OFF))
            .unwrap_or(false);
        let allow_move_after_hand_off =
            game.thrower_action == Some(PlayerAction::HandOver)
            && thrower_has_hand_off_skill;

        // Java PassState.setAllowMoveAfterBomb -- the Running Pass bombardier keeps moving after
        // a Quick Pass bomb (StepEndBomb reads it). The two editions write it differently:
        //   bb2020 :175-177  allowMoveAfterBomb = allowMoverAfterPass && !dontDropFumble
        //                      && actingPlayer == originalBombardier;   (always assigned)
        //   bb2025 :178-180  if (actingPlayer == originalBombardier)
        //                      setAllowMoveAfterBomb(allowMoverAfterPass && !dontDropFumble);
        // Never ported: a Running Pass bomb ended the bomber's activation in Rust where Java hands
        // it a MOVE (BACKLOG §M phase 2: goblin_sk bb2020 -- Java reported `playerAction move`).
        let acting_is_bombardier = game.acting_player.player_id.is_some()
            && game.acting_player.player_id == game.original_bombardier;
        match game.rules {
            ffb_model::enums::Rules::Bb2020 => {
                game.allow_move_after_bomb =
                    allow_move_after_pass && !self.dont_drop_fumble && acting_is_bombardier;
            }
            ffb_model::enums::Rules::Bb2025 if acting_is_bombardier => {
                game.allow_move_after_bomb = allow_move_after_pass && !self.dont_drop_fumble;
            }
            _ => {}
        }

        // Java path 3: bomb turn → Bomb generator
        // Java: if (StringTool.isProvided(fInterceptorId))
        //         bombGenerator.pushSequence(new Bomb.SequenceParams(gs, fInterceptorId, fPassFumble, dontDropFumble))
        //       else
        //         bombGenerator.pushSequence(new Bomb.SequenceParams(gs, fCatcherId, fPassFumble, dontDropFumble))
        if game.turn_mode.is_bomb_turn() {
            // Java: `if (StringTool.isProvided(fInterceptorId))` — and bb2020's StepIntercept only
            // publishes INTERCEPTOR_ID for an EASY (outright-catch) intercept of the bomb; a plain
            // deflection leaves it null, so the Bomb sequence runs catcher-less and the scattered
            // bomb explodes where it lies. Rust publishes the id for every success (ResolvePass
            // needs it), so gate on the outright-catch flag here instead (goblin bb2020 seed 45
            // step 104: a deflected-then-dropped bomb handed the deflector a free re-throw).
            let catcher_for_bomb = if self.interceptor_id.is_some() && self.interception_successful {
                self.interceptor_id.clone()
            } else {
                self.catcher_id.clone()
            };
            let seq = Bomb::build_sequence(&BombParams {
                catcher_id: catcher_for_bomb,
                pass_fumble: self.pass_fumble,
                dont_drop_fumble: self.dont_drop_fumble,
            });
            let mut outcome = StepOutcome::next().push_seq(seq);
            if self.bomb_out_of_bounds {
                // Java: publishParameter(new StepParameter(StepParameterKey.BOMB_OUT_OF_BOUNDS, true))
                outcome = outcome.publish(StepParameter::BombOutOfBounds(true));
            }
            return outcome;
        }

        // Java path 4: animosity re-try → Pass generator
        // Java: actingPlayer.isSufferingAnimosity() && !fEndPlayerAction && game.getPassCoordinate() == null
        if game.acting_player.suffering_animosity
            && !self.end_player_action
            && game.pass_coordinate.is_none()
        {
            let seq = Pass::build_sequence(&PassParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java "completions and passing statistic":
        //   if (thrower != null && hasBall(catcher)) spp.addCatch(catchTeams, catcherResult);
        //   if (thrower != null && (hasBall(catcher) || ballWasSnatched)
        //       && thrower.getTeam().hasPlayer(catcher) && endCoordinate.equals(passCoordinate)) {
        //     if (fPassAccurate) spp.addCompletion(completionTeams, throwerResult);
        //     throwerResult.passing += deltaX (catcher's square - thrower's, east = forward)
        //   }
        // The catcher must HOLD the ball on the target square -- an accurate pass he dropped, a
        // bomb, an interception are no completion; an inaccurate pass caught there still counts
        // its yards.
        let catcher_id = self.catcher_id.clone();
        let catcher_has_ball = catcher_id.as_deref()
            .map(|c| ffb_model::util::util_player::UtilPlayer::has_ball(game, c))
            .unwrap_or(false);
        if let (Some(_), Some(catcher)) = (game.thrower_id.clone(), catcher_id.as_deref()) {
            if catcher_has_ball {
                use ffb_mechanics::spp_mechanic::SppMechanic as _;
                let catcher_is_home = game.team_home.has_player(catcher);
                let team_id = if catcher_is_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
                let teams = game.prayer_state.get_additional_catches_spp_teams().clone();
                let rules = game.rules;
                let pr = game.game_result.team_result_mut(catcher_is_home).player_result_mut(catcher);
                match rules {
                    ffb_model::enums::Rules::Bb2016 => ffb_mechanics::bb2016::spp_mechanic::SppMechanic::new().add_catch(&teams, pr, &team_id),
                    ffb_model::enums::Rules::Bb2020 => ffb_mechanics::bb2020::spp_mechanic::SppMechanic::new().add_catch(&teams, pr, &team_id),
                    _ => ffb_mechanics::bb2025::spp_mechanic::SppMechanic::new().add_catch(&teams, pr, &team_id),
                }
            }
        }
        let ball_was_snatched = self.ball_snatcher_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
        if let (Some(thrower_id), Some(catcher)) = (game.thrower_id.clone(), catcher_id.as_deref()) {
            let is_home = game.team_home.has_player(&thrower_id);
            let same_team = if is_home { game.team_home.has_player(catcher) } else { game.team_away.has_player(catcher) };
            let end_coordinate = game.field_model.player_coordinate(catcher);
            if (catcher_has_ball || ball_was_snatched) && same_team
                && end_coordinate.is_some() && end_coordinate == game.pass_coordinate
            {
                let end_coord = end_coordinate.unwrap();
                let thrower_team_id = if is_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
                let has_prayer_bonus = game.prayer_state
                    .get_additional_completion_spp_teams()
                    .contains(&thrower_team_id);
                let start = game.field_model.player_coordinate(&thrower_id);
                let east_is_forward = if game.turn_mode == TurnMode::DumpOff { !game.home_playing } else { game.home_playing };
                let rules = game.rules;
                let pass_accurate = self.pass_accurate;
                let team_result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
                let pr = team_result.player_results.entry(thrower_id.clone()).or_default();
                if pass_accurate {
                    // Java: spp.addCompletion(prayerState.getAdditionalCompletionSppTeams(), throwerResult)
                    pr.completions += 1;
                    pr.spp_gained += SppCalc::completion_spp();
                    if has_prayer_bonus && rules != ffb_model::enums::Rules::Bb2016 {
                        pr.completions_with_additional_spp += 1;
                        pr.spp_gained += SppCalc::additional_spp(rules);
                    }
                }
                if let Some(start) = start {
                    let delta_x = if east_is_forward { end_coord.x - start.x } else { start.x - end_coord.x };
                    pr.passing += delta_x;
                }
            }
        }

        // Java path 5 (main branch): determine end_turn from thrower == actingPlayer
        // Java: boolean throwerIsActingPlayer = game.getThrower() == actingPlayer.getPlayer()
        // — reference equality, so null == null is TRUE. Requiring both ids to be Some sent the
        // both-None case down the dump-off path (path 6), which pushes no follow-up sequence and
        // silently ended the game whenever the stack was already empty.
        let thrower_is_acting = game.thrower_id == game.acting_player.player_id;

        // Java: fEndTurn || fEndPlayerAction || ((thrower == actingPlayer.getPlayer())
        //   && actingPlayer.isSufferingBloodLust() && !actingPlayer.hasFed())
        if self.end_turn
            || self.end_player_action
            || (thrower_is_acting
                && game.acting_player.suffering_blood_lust
                && !game.acting_player.has_fed)
        {
            // Java: fEndTurn |= (UtilServerSteps.checkTouchdown(getGameState())
            //   || (catcher == null && !actingPlayer.isSufferingAnimosity() && !actingPlayer.isSufferingBloodLust() && actingPlayer.hasPassed())
            //   || UtilPlayer.findOtherTeam(game, game.getThrower()).hasPlayer(catcher) && !actingPlayer.isSufferingBloodLust()
            //   || fPassFumble)
            let no_suffering = !game.acting_player.suffering_animosity
                && !game.acting_player.suffering_blood_lust;
            // Java: UtilPlayer.findOtherTeam(game, game.getThrower()).hasPlayer(catcher) — where
            // `catcher` is the FINAL catcher (fCatcherId, updated as the ball scatters/settles). A
            // pass/hand-off whose ball ends up with the thrower's OPPONENT is a turnover. This term
            // was documented in the comment above but omitted from the code, so a hand-off dropped by
            // the receiver that then bounced to an opponent (human seed 27 i=139: home_03 hands to the
            // Ogre, who fumbles the catch; the ball bounces to away_03) never turned over in Rust.
            let thrower_other_team_has_catcher = match (game.thrower_id.as_deref(), self.catcher_id.as_deref()) {
                (Some(tid), Some(cid)) => {
                    if game.team_home.has_player(tid) { game.team_away.has_player(cid) }
                    else { game.team_home.has_player(cid) }
                }
                _ => false,
            };
            self.end_turn |= check_touchdown(game)
                || (self.catcher_id.is_none() && no_suffering && game.acting_player.has_passed)
                || (thrower_other_team_has_catcher && !game.acting_player.suffering_blood_lust)
                || self.pass_fumble;
            // Java: endGenerator.pushSequence(new EndPlayerAction.SequenceParams(gs, true, fEndPlayerAction, fEndTurn))
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
                check_forgo: false,
                            rules: game.rules,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java else-branch: the interception statistic and ball placement run for EVERY pass that
        // gets here -- the acting player's own intercepted pass included (Rust ran them only on the
        // dump-off path, so a normal pass's interception never reached PlayerResult.interceptions);
        // then the thrower-is-acting split (path 7) or the dump-off reset.
        {
            if std::env::var_os("FFB_DRIVE_TRACE").is_some() {
                eprintln!("ENDPASSING path6 thrower={:?} acting={:?} action={:?}", game.thrower_id, game.acting_player.player_id, game.thrower_action);
            }
            // Java: catcher = game.getPlayerById(state.getInterceptorId())
            // Java: catcherResult.setInterceptions(+1)
            // Java: if (!isBomb && state.isInterceptionSuccessful() && !ballWasSnatched)
            //           field.setBallCoordinate(interceptorCoordinate); field.setBallMoving(false)
            // Java: game.setDefenderAction(null) -- reset dump-off action
            // Java (bb2020 + bb2025 else-branch), 1:1:
            //   [bb2020 only: if (state.isDeflectionSuccessful())] {
            //     catcher = interceptor;
            //     if (!isBomb) {
            //       if (state.isInterceptionSuccessful()) {
            //         interceptions += 1;
            //         if (!ballWasSnatched) ball = interceptorCoordinate;
            //         ballMoving = false;
            //       } else [bb2020 only] { deflections += 1; }
            //     }
            //   }
            // Rust placed the ball on the interceptor whenever an id was published, so a bb2020
            // DEFLECTION (interception NOT successful) of a dump-off still teleported the bounced
            // ball onto the deflector (dark_elf bb2020 seed 4 i=63: Java's ball stays moving at
            // 8,7, Rust's ended on home_01 at 8,9 and offered it a Pass).
            let is_bb2020 = game.rules == ffb_model::enums::Rules::Bb2020;
            let gate = !is_bb2020 || self.deflection_successful;
            if gate {
                if let Some(ref interceptor_id) = self.interceptor_id.clone() {
                    if !is_bomb {
                        let is_home = game.team_home.player(interceptor_id).is_some();
                        if self.interception_successful {
                            game.game_result.team_result_mut(is_home)
                                .player_result_mut(interceptor_id)
                                .interceptions += 1;
                            if self.ball_snatcher_id.is_none() {
                                if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
                                    game.field_model.ball_coordinate = Some(coord);
                                }
                            }
                            game.field_model.ball_moving = false;
                        } else if is_bb2020 {
                            game.game_result.team_result_mut(is_home)
                                .player_result_mut(interceptor_id)
                                .deflections += 1;
                        }
                    }
                }
            }
        }
        if !thrower_is_acting {
            // Java: game.setDefenderAction(null) -- reset dump-off action
            game.defender_action = None;
            return StepOutcome::next();
        }

        // Java path 7: thrower is acting player — determine move continuation.
        // Java recomputes `catcher` here as the player UNDER THE BALL (field.getPlayer(ballCoordinate)),
        // NOT the stored fCatcherId, then:
        //   fEndTurn |= checkTouchdown || (catcher == null) || otherTeam.hasPlayer(catcher)
        //              || (fPassFumble && !dontDropFumble)
        // The `otherTeam.hasPlayer(catcher)` term was previously omitted (and `catcher==null` was read
        // off the stored catcher_id), so an accurate pass that bounced to and was CAUGHT BY AN OPPONENT
        // never turned over (khemri seed 40 i=185: home_04's pass bounced to away_02, who caught it;
        // Java turned the ball over, Rust let home keep its turn).
        // Java: if (ballWasSnatched) catcher = getPlayerById(fCatcherId) else the player under the ball
        let ball_was_snatched = self.ball_snatcher_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
        let ball_catcher_id: Option<String> = if ball_was_snatched {
            self.catcher_id.clone().filter(|c| game.player(c).is_some())
        } else {
            game.field_model.ball_coordinate.and_then(|c| game.field_model.player_at(c).cloned())
        };
        let ball_catcher_is_opponent = match (game.thrower_id.as_deref(), ball_catcher_id.as_deref()) {
            (Some(tid), Some(cid)) => {
                if game.team_home.has_player(tid) { game.team_away.has_player(cid) }
                else { game.team_home.has_player(cid) }
            }
            _ => false,
        };
        self.end_turn |= check_touchdown(game)
            || ball_catcher_id.is_none()
            || ball_catcher_is_opponent
            || (self.pass_fumble && !self.dont_drop_fumble);

        // Java: fEndPlayerAction |= !((allowMoveAfterPass || allowMoveAfterHandOff) && UtilPlayer.isNextMovePossible(game, false))
        if !((allow_move_after_pass || allow_move_after_hand_off)
            && UtilPlayer::is_next_move_possible(game, false))
        {
            self.end_player_action = true;
        }

        if self.end_turn || self.end_player_action {
            // Java: endGenerator.pushSequence(new EndPlayerAction.SequenceParams(gs, true, fEndPlayerAction, fEndTurn))
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
                check_forgo: false,
                            rules: game.rules,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: UtilServerGame.changeActingPlayer(this, actingPlayerId, PlayerAction.MOVE, actingPlayer.isJumping())
        // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), actingPlayer.isJumping())
        // Java: moveGenerator.pushSequence(new Move.SequenceParams(getGameState()))
        if game.acting_player.player_id.is_some() {
            game.acting_player.player_action = Some(PlayerAction::Move);
        }
        UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
        let seq = Move::build_sequence(&MoveParams::default());
        StepOutcome::next().push_seq(seq)
    }

    /// Build a Pass sequence (Java: Pass generator pushSequence) for animosity re-try.
    #[allow(dead_code)]
    fn push_pass_retry() -> StepOutcome {
        let seq = Pass::build_sequence(&PassParams::default());
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, SkillId, TurnMode};
    use ffb_model::types::{FieldCoordinate, RangeRuler};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ── set_parameter ─────────────────────────────────────────────────────────

    #[test]
    fn set_parameter_interceptor_id_accepted() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::InterceptorId(Some("i1".into())));
        assert_eq!(step.interceptor_id.as_deref(), Some("i1"));
    }

    #[test]
    fn set_parameter_pass_accurate_accepted() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::PassAccurate(true));
        assert!(step.pass_accurate);
    }

    #[test]
    fn set_parameter_revert_end_turn_sets_end_turn_false() {
        let mut step = StepEndPassing::new();
        step.end_turn = true;
        step.set_parameter(&StepParameter::RevertEndTurn(true));
        assert!(!step.end_turn);
    }

    #[test]
    fn set_parameter_pass_fumble() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::PassFumble(true));
        assert!(step.pass_fumble);
    }

    #[test]
    fn ball_ending_with_thrower_opponent_is_a_turnover() {
        // Regression (human seed 27 i=139): a hand-off/pass whose ball ends with the thrower's
        // OPPONENT (dropped by the receiver, bounced to an opponent) is a turnover —
        // findOtherTeam(thrower).hasPlayer(catcher). A same-team catcher must NOT turn over.
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::util::rng::GameRng;
        let mk = |id: &str| Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default()
        };

        // Opponent recovers the ball → turnover.
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(mk("t1"));
        game.team_away.players.push(mk("opp"));
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::HandOver);
        game.acting_player.set_player("t1".into(), PlayerAction::HandOver);
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("opp".into());
        step.end_player_action = true; // hand-off publishes EndPlayerAction
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.end_turn, "ball ending with the thrower's opponent must be a turnover");

        // Same-team recovery → NOT a turnover from this term.
        let mut game2 = make_game();
        game2.home_playing = true;
        game2.team_home.players.push(mk("t1"));
        game2.team_home.players.push(mk("mate"));
        game2.thrower_id = Some("t1".into());
        game2.thrower_action = Some(PlayerAction::HandOver);
        game2.acting_player.set_player("t1".into(), PlayerAction::HandOver);
        let mut step2 = StepEndPassing::new();
        step2.catcher_id = Some("mate".into());
        step2.end_player_action = true;
        step2.start(&mut game2, &mut GameRng::new(0));
        assert!(!step2.end_turn, "a teammate recovering the ball is not a turnover from the opponent term");
    }

    #[test]
    fn set_parameter_dont_drop_fumble() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::DontDropFumble(true));
        assert!(step.dont_drop_fumble);
    }

    #[test]
    fn set_parameter_passing_distance() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::PassingDistance(PassingDistance::QuickPass));
        assert_eq!(step.passing_distance, Some(PassingDistance::QuickPass));
    }

    #[test]
    fn set_parameter_bomb_out_of_bounds() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::BombOutOfBounds(true));
        assert!(step.bomb_out_of_bounds);
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }

    #[test]
    fn set_parameter_player_id_is_ball_snatcher() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::PlayerId("snatcher".into()));
        assert_eq!(step.ball_snatcher_id.as_deref(), Some("snatcher"));
    }

    #[test]
    fn set_parameter_bloodlust_action() {
        let mut step = StepEndPassing::new();
        step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move)));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    // ── clears range ruler ────────────────────────────────────────────────────

    #[test]
    fn clears_range_ruler_on_start() {
        let mut game = make_game();
        game.field_model.range_ruler = Some(RangeRuler::new(
            "t1".into(),
            Some(FieldCoordinate::new(5, 5)),
            -1,
            false,
        ));
        let mut step = StepEndPassing::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_none());
    }

    // ── bomb turn path ────────────────────────────────────────────────────────

    #[test]
    fn bomb_turn_pushes_bomb_sequence() {
        let mut game = make_game();
        game.set_turn_mode(TurnMode::BombHome);
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // Bomb sequence starts with InitBomb
        assert_eq!(out.pushes[0][0].step_id, StepId::InitBomb);
    }

    #[test]
    fn bomb_turn_uses_interceptor_id_when_set() {
        let mut game = make_game();
        game.set_turn_mode(TurnMode::BombHome);
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some("i1".into());
        // Java hands the Bomb sequence the interceptor only when the interception was an
        // OUTRIGHT catch (bb2020 publishes INTERCEPTOR_ID only for an easy intercept; bb2025 for
        // any success) — mirrored by the interception_successful gate.
        step.interception_successful = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // bomb sequence init params should contain catcher CatcherId("i1")
        let init_step = &out.pushes[0][0];
        let has_i1 = init_step.params.iter().any(|p| {
            matches!(p, StepParameter::CatcherId(Some(id)) if id == "i1")
        });
        assert!(has_i1, "bomb sequence should carry interceptor id");
    }

    #[test]
    fn bomb_turn_publishes_bomb_out_of_bounds_when_set() {
        let mut game = make_game();
        game.set_turn_mode(TurnMode::BombHome);
        let mut step = StepEndPassing::new();
        step.bomb_out_of_bounds = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        let oob = out.published.iter().any(|p| {
            matches!(p, StepParameter::BombOutOfBounds(true))
        });
        assert!(oob);
    }

    // ── end_player_action + bomb path ─────────────────────────────────────────

    #[test]
    fn end_player_action_with_throw_bomb_pushes_end_player_action() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::ThrowBomb);
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // EndPlayerAction sequence starts with RemoveTargetSelectionState
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_player_action_with_hail_mary_bomb_pushes_end_player_action() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::HailMaryBomb);
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    // ── normal end_turn / end_player_action path ──────────────────────────────

    #[test]
    fn end_turn_true_pushes_end_player_action_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("t1".into());
        game.thrower_id = Some("t1".into());
        let mut step = StepEndPassing::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_player_action_true_pushes_end_player_action_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("t1".into());
        game.thrower_id = Some("t1".into());
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    // ── pass_fumble forces end_turn ───────────────────────────────────────────

    #[test]
    fn pass_fumble_and_thrower_is_acting_forces_end_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("t1".into());
        game.thrower_id = Some("t1".into());
        let mut step = StepEndPassing::new();
        step.pass_fumble = true;
        step.catcher_id = Some("c1".into()); // catcher present to avoid catcher==None path
        let out = step.start(&mut game, &mut GameRng::new(0));
        // end_turn is set by pass_fumble && !dont_drop_fumble → EndPlayerAction pushed
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    // ── move continuation when allowed ───────────────────────────────────────

    #[test]
    fn quick_pass_accurate_allows_move_continuation() {
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{PlayerState, PS_MOVING};
        let mut game = make_game();
        // Give the thrower a GiveAndGo skill (canMoveAfterQuickPass) and MA=6
        let mut thrower = Player::default();
        thrower.id = "t1".into();
        thrower.movement = 6; // moves available
        thrower.starting_skills.push(SkillWithValue::new(SkillId::GiveAndGo));
        game.team_home.players.push(thrower);
        game.field_model.set_player_state("t1", PlayerState::new(PS_MOVING).change_active(true));
        game.acting_player.player_id = Some("t1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.thrower_id = Some("t1".into());
        // Place the catcher c1 (a home TEAMMATE) on the field UNDER THE BALL — Java's path-7 turnover
        // check recomputes the catcher as the player under the ball, and a same-team catcher (not null,
        // not an opponent) is NOT a turnover, so the thrower may continue moving (GiveAndGo).
        let mut catcher = Player::default();
        catcher.id = "c1".into();
        game.team_home.players.push(catcher);
        let ball_coord = ffb_model::types::FieldCoordinate::new(10, 7);
        game.field_model.set_player_coordinate("c1", ball_coord);
        game.field_model.set_player_state("c1", PlayerState::new(PS_MOVING).change_active(true));
        game.field_model.ball_coordinate = Some(ball_coord);
        let mut step = StepEndPassing::new();
        // quick pass accurate: passing_distance=QuickPass, pass_fumble=false
        step.passing_distance = Some(PassingDistance::QuickPass);
        step.pass_fumble = false;
        step.catcher_id = Some("c1".into());
        // dont_drop_fumble=false so fumble would end turn, but pass_fumble=false
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should push a Move sequence (not EndPlayerAction)
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitMoving);
    }

    // ── start always returns NextStep ──────────────────────────────────────────

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndPassing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bloodlust_action_pushes_move_sequence_and_resets_has_passed() {
        let mut game = make_game();
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_passed = true;
        game.pass_coordinate = Some(ffb_model::types::FieldCoordinate::new(5, 5));
        let mut step = StepEndPassing::new();
        step.bloodlust_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.has_passed);
        assert!(game.pass_coordinate.is_none());
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Move));
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn bloodlust_not_suffering_does_not_reset_has_passed() {
        let mut game = make_game();
        game.acting_player.suffering_blood_lust = false;
        game.acting_player.has_passed = true;
        let mut step = StepEndPassing::new();
        step.bloodlust_action = Some(PlayerAction::Move);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_passed);
    }

    fn make_game_with_home_thrower(thrower_id: &str) -> Game {
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: thrower_id.into(), nr: 1, name: thrower_id.into(),
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.thrower_id = Some(thrower_id.into());
        game.acting_player.player_id = Some(thrower_id.into());
        game
    }

    /// A home team-mate "c1" holding the ball on the pass target (Java's completion condition).
    fn catch_at(game: &mut Game, step: &mut StepEndPassing, coord: ffb_model::types::FieldCoordinate) {
        let mut catcher = game.team_home.players[0].clone();
        catcher.id = "c1".into();
        catcher.nr = 2;
        game.team_home.players.push(catcher);
        game.field_model.set_player_coordinate("c1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;
        game.pass_coordinate = Some(coord);
        step.catcher_id = Some("c1".into());
    }

    #[test]
    fn accurate_pass_awards_completion_spp() {
        let mut game = make_game_with_home_thrower("t1");
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        catch_at(&mut game, &mut step, ffb_model::types::FieldCoordinate::new(9, 7));
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("t1").unwrap();
        assert_eq!(pr.completions, 1);
        assert_eq!(pr.spp_gained, 1);
    }

    /// Java: an accurate pass the catcher did not hold is no completion (and no yards).
    #[test]
    fn accurate_pass_not_held_awards_nothing() {
        let mut game = make_game_with_home_thrower("t1");
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        catch_at(&mut game, &mut step, ffb_model::types::FieldCoordinate::new(9, 7));
        game.field_model.ball_coordinate = Some(ffb_model::types::FieldCoordinate::new(10, 7));
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("t1").cloned().unwrap_or_default();
        assert_eq!((pr.completions, pr.passing), (0, 0));
    }

    #[test]
    fn prayer_spp_grants_extra_spp_and_increments_counter() {
        let mut game = make_game_with_home_thrower("t1");
        game.prayer_state.add_get_additional_completion_spp(&game.team_home.id.clone());
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        catch_at(&mut game, &mut step, ffb_model::types::FieldCoordinate::new(9, 7));
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("t1").unwrap();
        assert_eq!(pr.completions_with_additional_spp, 1);
        assert_eq!(pr.spp_gained, 2);
    }

    #[test]
    fn passing_yards_calculated_on_accurate_pass() {
        let mut game = make_game_with_home_thrower("t1");
        game.field_model.set_player_coordinate("t1", ffb_model::types::FieldCoordinate::new(5, 7));
        game.home_playing = true;
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        catch_at(&mut game, &mut step, ffb_model::types::FieldCoordinate::new(12, 7));
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("t1").unwrap();
        assert_eq!(pr.passing, 7);
    }

    #[test]
    fn bloodlust_suffering_without_feeding_forces_end_player_action_path() {
        // Bug fix regression: Java's outer condition is
        //   fEndTurn || fEndPlayerAction || (thrower==actingPlayer && sufferingBloodLust && !hasFed())
        // Previously the third disjunct was dropped entirely, so a player suffering
        // BloodLust who hadn't fed (and has no bloodlust_action set, so the earlier
        // bloodlust-retarget branch doesn't fire) would fall through to the move-continuation
        // path instead of ending the player's action via EndPlayerAction.
        let mut game = make_game_with_home_thrower("t1");
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = false;
        let mut step = StepEndPassing::new();
        // No bloodlust_action set, so the early bloodlust-retarget branch doesn't trigger.
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn interception_increments_player_result() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        // Thrower on home team; interceptor on away team (inactive)
        let mut game = make_game_with_home_thrower("thrower");
        game.home_playing = true;
        // Distinct acting player (thrower) — set up so thrower_is_acting is false
        // For path 6 (interception dump-off): thrower != acting player
        game.acting_player.player_id = Some("other".into());
        let interceptor_id = "interceptor".to_string();
        game.team_away.players.push(Player {
            id: interceptor_id.clone(), name: "ic".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some(interceptor_id.clone());
        // Java counts an interception only when `PassState.isInterceptionSuccessful()`.
        step.interception_successful = true;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.away.player_results.get(&interceptor_id).unwrap();
        assert_eq!(pr.interceptions, 1);

        // Java's else-branch counts it for the ACTING player's own pass too (then path 7).
        let mut game = make_game_with_home_thrower("thrower");
        game.home_playing = true;
        game.acting_player.player_id = Some("thrower".into());
        game.team_away.players.push(Player { id: interceptor_id.clone(), nr: 2, ..Default::default() });
        game.field_model.set_player_coordinate(&interceptor_id, FieldCoordinate::new(9, 9));
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some(interceptor_id.clone());
        step.interception_successful = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.player_results.get(&interceptor_id).map(|r| r.interceptions), Some(1));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(9, 9)));
        assert!(step.end_turn, "the ball ends with an opponent: turnover");
    }

    /// Java bb2020 else-branch: a DEFLECTION (deflectionSuccessful, interception not successful)
    /// counts a deflection and leaves the ball where it is — it is NOT placed on the deflector.
    #[test]
    fn bb2020_dump_off_deflection_counts_a_deflection_and_leaves_the_ball() {
        let mut game = make_game_with_home_thrower("thrower");
        game.rules = ffb_model::enums::Rules::Bb2020;
        game.home_playing = false;
        game.acting_player.player_id = Some("other".into());
        let mut ic = ffb_model::model::player::Player::default();
        ic.id = "interceptor".into();
        game.team_away.players.push(ic);
        game.field_model.set_player_coordinate("interceptor", ffb_model::types::FieldCoordinate::new(8, 9));
        game.field_model.ball_coordinate = Some(ffb_model::types::FieldCoordinate::new(8, 7));
        game.field_model.ball_moving = true;
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        step.interception_successful = false;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.away.player_results.get("interceptor").unwrap();
        assert_eq!((pr.interceptions, pr.deflections), (0, 1));
        assert_eq!(game.field_model.ball_coordinate, Some(ffb_model::types::FieldCoordinate::new(8, 7)));
        assert!(game.field_model.ball_moving);
    }
}
