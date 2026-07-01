use ffb_model::enums::{PassingDistance, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::step::util_server_steps::check_touchdown;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2020::bomb::{Bomb, BombParams};
use crate::step::generator::bb2020::move_::{Move, MoveParams};
// Pass generator (used for animosity re-try — not yet wired, isSufferingAnimosity not in ActingPlayer)
#[allow(unused_imports)]
use crate::step::generator::bb2020::pass::{Pass, PassParams};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.pass.StepEndPassing.
///
/// Final step of the pass sequence. Consumes all pass-related parameters and routes
/// to the correct continuation:
///
///  • EndPlayerAction + bomb → EndPlayerAction generator (feeding_allowed=true, end_player_action=true, end_turn).
///  • BloodLust action       → Move generator.
///  • Bomb turn              → Bomb generator (BB2020 Bomb).
///  • Animosity re-try       → Pass generator.
///  • Otherwise: compute statistics (TODO: SppMechanic), determine end_turn,
///    push EndPlayerAction or Move generator.
///
/// BB2020 vs BB2025: no `allowMoveAfterHandOff`; has `bloodlust_action` / `ball_snatcher_id`;
/// `allowMoveAfterBomb` check; uses BB2020 Bomb generator.
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
            // Java: REVERT_END_TURN → fEndTurn = false
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

        // Java path 1: failed confusion roll on throw bomb → end player action
        // Java: if (fEndPlayerAction && (isBomb || HAIL_MARY_BOMB))
        if self.end_player_action
            && (is_bomb || acting_action == Some(PlayerAction::HailMaryBomb))
        {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 2: BloodLust + bloodlustAction → reset hasPassed, pass_coordinate, change action, Move
        if game.acting_player.suffering_blood_lust && self.bloodlust_action.is_some() {
            // Java: actingPlayer.setHasPassed(false); game.setPassCoordinate(null);
            game.acting_player.has_passed = false;
            game.pass_coordinate = None;
            // Java: UtilServerSteps.changePlayerAction(..., bloodlustAction, false);
            if let Some(action) = self.bloodlust_action {
                game.acting_player.player_action = Some(action);
            }
            // Java: moveGenerator.pushSequence(...)
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: allowMoverAfterPass = QUICK_PASS distance + canMoveAfterQuickPass skill + !fPassFumble
        // Java: allowMoveAfterBomb = allowMoverAfterPass && !dontDropFumble && actingPlayerId == originalBombardier
        let thrower_has_quick_pass_skill = game.thrower_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_QUICK_PASS))
            .unwrap_or(false);
        let allow_mover_after_pass = self.passing_distance == Some(PassingDistance::QuickPass)
            && thrower_has_quick_pass_skill
            && !self.pass_fumble;
        // Java: allowMoveAfterBomb = allowMoverAfterPass && !dontDropFumble && actingPlayer == originalBombardier
        // (originalBombardier tracking not fully wired; approximate with allow_mover_after_pass)
        let _allow_move_after_bomb = allow_mover_after_pass && !self.dont_drop_fumble;

        // Java path 3: throw bomb mode → Bomb generator (BB2020)
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
                outcome = outcome.publish(StepParameter::BombOutOfBounds(true));
            }
            return outcome;
        }

        // Java path 4: animosity re-try → Pass generator
        // Java: actingPlayer.isSufferingAnimosity() && !fEndPlayerAction && passCoordinate == null
        // DEFERRED(animosity): isSufferingAnimosity not yet in ActingPlayer struct — waiting for ActingPlayer
        //   to grow a suffering_animosity field (BB2020 Animosity behaviour port).

        // Java: completions and passing statistics (SPP, deltaX)
        // DEFERRED(spp): SppMechanic.addCompletion + throwerResult.setPassing(deltaX) —
        //   requires: (a) PrayerState.additional_completion_spp_teams (PrayerState is a stub);
        //   (b) PlayerResult.passing field (not yet added to GameResult); (c) isSufferingAnimosity.

        // Java path 5: main branch — determine end_turn from thrower == actingPlayer
        let thrower_is_acting = game.thrower_id.is_some()
            && game.acting_player.player_id.is_some()
            && game.thrower_id == game.acting_player.player_id;

        if self.end_turn || self.end_player_action
            || (thrower_is_acting && game.acting_player.suffering_blood_lust)
        {
            // Java: fEndTurn |= checkTouchdown || (catcher==null && !animosity && !bloodlust && hasPassed)
            //   || otherTeam.hasPlayer(catcher) && !bloodlust || fPassFumble
            self.end_turn |= check_touchdown(game)
                || self.catcher_id.is_none()
                || self.pass_fumble;
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 6: interception / deflection handling (thrower is NOT the acting player — dump-off path)
        if !thrower_is_acting {
            // Java: passState.isDeflectionSuccessful() → set interceptor statistics, ball position
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

        // Java path 7: thrower is acting player — check deflection, determine move continuation
        // Java: passState.isDeflectionSuccessful() → interceptor statistics
        if let Some(ref interceptor_id) = self.interceptor_id.clone() {
            // Java: interceptorResult.setInterceptions(+1) / setDeflections(+1)
            // Java: if (!isBomb && isInterceptionSuccessful && !ballWasSnatched) → setBallCoordinate + setBallMoving(false)
            if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
                if !is_bomb {
                    game.field_model.ball_coordinate = Some(coord);
                    game.field_model.ball_moving = false;
                }
            }
        }

        // Java: fEndTurn |= checkTouchdown || (catcher==null) || otherTeam.hasPlayer(catcher) || (fPassFumble && !dontDropFumble)
        self.end_turn |= check_touchdown(game)
            || self.catcher_id.is_none()
            || (self.pass_fumble && !self.dont_drop_fumble);

        // Java: fEndPlayerAction |= !(allowMoverAfterPass && UtilPlayer.isNextMovePossible(game, false))
        if !(allow_mover_after_pass && UtilPlayer::is_next_move_possible(game, false)) {
            self.end_player_action = true;
        }

        if self.end_turn || self.end_player_action {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: changeActingPlayer → MOVE, updateMoveSquares, pushSequence Move
        if game.acting_player.player_id.is_some() {
            game.acting_player.player_action = Some(PlayerAction::Move);
        }
        let seq = Move::build_sequence(&MoveParams::default());
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::types::{FieldCoordinate, RangeRuler};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
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

    // ── end_turn path ─────────────────────────────────────────────────────────

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
        // has_passed should remain true when not suffering blood lust
        assert!(game.acting_player.has_passed);
    }
}
