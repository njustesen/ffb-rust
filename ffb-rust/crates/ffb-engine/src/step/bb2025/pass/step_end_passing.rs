use ffb_model::enums::{PassingDistance, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::step::util_server_steps::check_touchdown;
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
///  • Animosity re-try       → Pass generator (not yet fully wired — ActingPlayer.suffering_animosity not translated).
///  • Otherwise: compute SPP (completions/catches — TODO: SppMechanic), determine whether
///    turn/player-action ends, and push EndPlayerAction or Move generator.
pub struct StepEndPassing {
    /// Java: fInterceptorId
    pub interceptor_id: Option<String>,
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
            StepParameter::InterceptorId(v) => { self.interceptor_id = v.clone(); true }
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
}

impl StepEndPassing {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(gameState)
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
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 2: BloodLust + bloodlustAction → Move generator
        // Java: actingPlayer.setHasPassed(false); game.setPassCoordinate(null);
        // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), bloodlustAction, false);
        // Java: moveGenerator.pushSequence(new Move.SequenceParams(getGameState()));

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

        // Java path 3: bomb turn → Bomb generator
        // Java: if (StringTool.isProvided(fInterceptorId))
        //         bombGenerator.pushSequence(new Bomb.SequenceParams(gs, fInterceptorId, fPassFumble, dontDropFumble))
        //       else
        //         bombGenerator.pushSequence(new Bomb.SequenceParams(gs, fCatcherId, fPassFumble, dontDropFumble))
        if game.turn_mode.is_bomb_turn() {
            let catcher_for_bomb = if self.interceptor_id.is_some() {
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
        // TODO: actingPlayer.is_suffering_animosity() — not yet translated in ActingPlayer
        // Stub: if pass_coordinate is None and end_player_action is false, treat as animosity re-try
        // This matches Java's guard: "failed animosity may try to choose a new target"
        // Conservative: only trigger if no coordinate set (pass was never aimed)
        // (Full implementation requires ActingPlayer.is_suffering_animosity)

        // Java: SPP completions / catches
        // TODO: SppMechanic.addCatch, SppMechanic.addCompletion, throwerResult.setPassing(deltaX)
        // These require GameResult/PlayerResult + SppMechanic translation.

        // Java path 5 (main branch): determine end_turn from thrower == actingPlayer
        // Java: boolean throwerIsActingPlayer = game.getThrower() == actingPlayer.getPlayer()
        // Proxy: thrower_id == acting_player.player_id
        let thrower_is_acting = game.thrower_id.is_some()
            && game.acting_player.player_id.is_some()
            && game.thrower_id == game.acting_player.player_id;

        if self.end_turn || self.end_player_action {
            // Java: fEndTurn |= (UtilServerSteps.checkTouchdown(getGameState())
            //   || (catcher == null && !actingPlayer.isSufferingAnimosity() && !actingPlayer.isSufferingBloodLust() && actingPlayer.hasPassed())
            //   || UtilPlayer.findOtherTeam(game, game.getThrower()).hasPlayer(catcher) && !actingPlayer.isSufferingBloodLust()
            //   || fPassFumble)
            // Java also: isSufferingAnimosity, isSufferingBloodLust, hasPassed — not yet wired
            self.end_turn |= check_touchdown(game)
                || self.catcher_id.is_none()
                || self.pass_fumble;
            // Java: endGenerator.pushSequence(new EndPlayerAction.SequenceParams(gs, true, fEndPlayerAction, fEndTurn))
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
                check_forgo: false,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 6: interception handling (thrower is NOT the acting player — dump-off path)
        if !thrower_is_acting {
            // Java: catcher = game.getPlayerById(state.getInterceptorId())
            // Java: catcherResult.setInterceptions(+1)
            // Java: if (!isBomb && state.isInterceptionSuccessful() && !ballWasSnatched)
            //           field.setBallCoordinate(interceptorCoordinate); field.setBallMoving(false)
            // Java: game.setDefenderAction(null) -- reset dump-off action
            // TODO: GameResult/PlayerResult.set_interceptions — not yet wired
            if let Some(ref interceptor_id) = self.interceptor_id.clone() {
                if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
                    if !is_bomb {
                        game.field_model.ball_coordinate = Some(coord);
                        game.field_model.ball_moving = false;
                    }
                }
            }
            // Java: game.setDefenderAction(null)
            game.defender_action = None;
            return StepOutcome::next();
        }

        // Java path 7: thrower is acting player — determine move continuation
        // Java: fEndTurn |= checkTouchdown || (catcher==null) || otherTeam.hasPlayer(catcher) || (fPassFumble && !dontDropFumble)
        // Simplified: check catcher presence and fumble
        self.end_turn |= check_touchdown(game)
            || self.catcher_id.is_none()
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
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: UtilServerGame.changeActingPlayer(this, actingPlayerId, PlayerAction.MOVE, actingPlayer.isJumping())
        // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), actingPlayer.isJumping())
        // Java: moveGenerator.pushSequence(new Move.SequenceParams(getGameState()))
        // TODO: changeActingPlayer, updateMoveSquares — require more Game infrastructure
        if game.acting_player.player_id.is_some() {
            game.acting_player.player_action = Some(PlayerAction::Move);
        }
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
        game.turn_mode = TurnMode::BombHome;
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
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some("i1".into());
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
        game.turn_mode = TurnMode::BombHome;
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
}
