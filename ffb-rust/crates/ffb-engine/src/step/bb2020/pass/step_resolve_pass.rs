use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.pass.StepResolvePass.
///
/// Sets animation, syncs game model, then routes ball/bomb placement based on whether
/// the pass was deflected, intercepted, accurate, out-of-bounds, or missed.
///
/// BB2020 vs BB2025 additions:
/// - Deflection path: `isDeflectionSuccessful() && !isInterceptionSuccessful()` →
///   `DEFLECTED` / `DEFLECTED_BOMB` modes.
/// - Full interception path: interceptor coordinate set for bomb too.
///
/// Publishes: `CatchScatterThrowInMode`, `PassAccurate`, `BombOutOfBounds`.
///
/// Animation type + UtilServerGame.syncGameModel are client-side only; no server game-state effect.
pub struct StepResolvePass {
    /// Consumed from PassAccurate parameter published by StepPass.
    pub pass_accurate: bool,
    /// Consumed from InterceptorId parameter (set = deflection/interception succeeded).
    pub interceptor_id: Option<String>,
    /// Consumed from CatcherId parameter.
    pub catcher_id: Option<String>,
    /// Whether deflection was successful (passState.isDeflectionSuccessful()).
    /// In BB2020 this is set when the intercept step sets the interceptor_id.
    /// Note: both deflection (partial) and interception (full) go through this flag.
    pub deflection_successful: bool,
    /// Whether full interception (not just deflection) was successful.
    /// Java: passState.isInterceptionSuccessful() — true only for a successful *easy*
    /// interception; consumed from the `InterceptionSuccessful` parameter published by
    /// `StepIntercept`.
    pub interception_successful: bool,
}

impl StepResolvePass {
    pub fn new() -> Self {
        Self {
            pass_accurate: false,
            interceptor_id: None,
            catcher_id: None,
            deflection_successful: false,
            interception_successful: false,
        }
    }
}

impl Default for StepResolvePass {
    fn default() -> Self { Self::new() }
}

impl Step for StepResolvePass {
    fn id(&self) -> StepId { StepId::ResolvePass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PassAccurate(v) => { self.pass_accurate = *v; true }
            StepParameter::InterceptorId(v) => {
                if v.is_some() {
                    // An interceptor was set → deflection occurred.
                    // Java: state.setDeflectionSuccessful(doIntercept) is set whenever the
                    // interception attempt succeeds (easy or not); interceptionSuccessful is
                    // a separate flag (see InterceptionSuccessful parameter below) that is
                    // only true for an *easy* interception.
                    self.deflection_successful = true;
                }
                self.interceptor_id = v.clone();
                true
            }
            // Java: passState.isInterceptionSuccessful() — published by StepIntercept only
            // when the successful interception was an "easy" one (Yoink-style skill).
            StepParameter::InterceptionSuccessful(v) => { self.interception_successful = *v; true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepResolvePass {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: AnimationType animationType = getAnimationType(throwerAction)
        // Java: getResult().setAnimation(new Animation(animationType, throwerCoord, passCoord, interceptorCoord))
        // Java: UtilServerGame.syncGameModel(this)
        // Animation + syncGameModel: client-side visual only, no server state change.

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: if (state.isDeflectionSuccessful())
        if self.deflection_successful {
            let interceptor_coord = self.interceptor_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));

            // Java: if (!state.isInterceptionSuccessful()) → DEFLECTED / DEFLECTED_BOMB
            if !self.interception_successful {
                if is_bomb {
                    if let Some(coord) = interceptor_coord {
                        game.field_model.bomb_coordinate = Some(coord);
                    }
                    return StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::DeflectedBomb
                        ));
                } else {
                    if let Some(coord) = interceptor_coord {
                        game.field_model.ball_coordinate = Some(coord);
                    }
                    return StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::Deflected
                        ));
                }
            } else {
                // Java: interception successful — set bomb coordinate for bomb case
                if is_bomb {
                    if let Some(coord) = interceptor_coord {
                        game.field_model.bomb_coordinate = Some(coord);
                    }
                }
                // Java: for non-bomb interception, ball coordinate is set in StepEndPassing
                return StepOutcome::next();
            }
        }

        // Java: else if (state.getResult() == PassResult.ACCURATE)
        if self.pass_accurate {
            let pass_coord = game.pass_coordinate;
            if let Some(ref catcher_id) = self.catcher_id {
                // Java: check catcher state — if no tackle zones → empty square path
                // Java: `Player<?> catcher = ...; PlayerState catcherState = ...;
                //        if (catcher == null || catcherState == null || !catcherState.hasTacklezones())`
                // hasTacklezones() (not canBeBlocked()) is the correct predicate here: it also
                // excludes confused/hypnotized/eye-gouged players and includes the BLOCKED base
                // state, both of which canBeBlocked() gets wrong.
                let catcher_has_tackle_zones = game.field_model.player_state(catcher_id)
                    .map(|s| s.has_tacklezones())
                    .unwrap_or(false);
                if !catcher_has_tackle_zones {
                    // Java: CATCH_ACCURATE_PASS_EMPTY_SQUARE / CATCH_ACCURATE_BOMB_EMPTY_SQUARE
                    // Java: for bomb with no TZ catcher → CATCH_BOMB (not empty square)
                    if let Some(pc) = pass_coord {
                        if is_bomb {
                            game.field_model.bomb_coordinate = Some(pc);
                        } else {
                            game.field_model.ball_coordinate = Some(pc);
                        }
                    }
                    return StepOutcome::next().publish(StepParameter::CatchScatterThrowInMode(
                        if is_bomb {
                            CatchScatterThrowInMode::CatchBomb
                        } else {
                            CatchScatterThrowInMode::CatchMissedPass
                        }
                    ));
                }
                if is_bomb {
                    return StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccurateBomb,
                        ));
                } else {
                    return StepOutcome::next()
                        .publish(StepParameter::PassAccurate(true))
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccuratePass,
                        ));
                }
            } else {
                // No catcher (empty square)
                if let Some(pc) = pass_coord {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(pc);
                    } else {
                        game.field_model.ball_coordinate = Some(pc);
                    }
                }
                return if is_bomb {
                    StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccurateBombEmptySquare,
                        ))
                } else {
                    StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccuratePassEmptySquare,
                        ))
                };
            }
        }

        // Java: else (missed / inaccurate) branch
        // Java: if (fieldModel.isOutOfBounds())
        if game.field_model.out_of_bounds {
            if is_bomb {
                // Java: setBombCoordinate(null); publishParameter(BOMB_OUT_OF_BOUNDS, true)
                game.field_model.bomb_coordinate = None;
                return StepOutcome::next()
                    .publish(StepParameter::BombOutOfBounds(true));
            } else {
                // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, THROW_IN)
                // Java: publishParameter(THROW_IN_COORDINATE, game.getPassCoordinate())
                // Java: setBallMoving(true)
                game.field_model.ball_moving = true;
                let mut outcome = StepOutcome::next()
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn));
                if let Some(pc) = game.pass_coordinate {
                    outcome = outcome.publish(StepParameter::ThrowInCoordinate(pc));
                }
                return outcome;
            }
        }

        // Java: else (in bounds, not accurate, not intercepted)
        if is_bomb {
            // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_BOMB)
            // Java: setBombCoordinate(game.getPassCoordinate()); setBombMoving(true)
            if let Some(pass_coord) = game.pass_coordinate {
                game.field_model.bomb_coordinate = Some(pass_coord);
                game.field_model.bomb_moving = true;
            }
            StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb))
        } else {
            // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_MISSED_PASS)
            // Java: setBallCoordinate(game.getPassCoordinate()); setBallMoving(true)
            if let Some(pass_coord) = game.pass_coordinate {
                game.field_model.ball_coordinate = Some(pass_coord);
                game.field_model.ball_moving = true;
            }
            StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerState, Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn accurate_pass_with_catcher_publishes_catch_accurate_pass() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(10, 5));
        game.field_model.set_player_state("c1", PlayerState::new(PS_STANDING));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccuratePass))
        });
        assert!(mode.is_some());
    }

    #[test]
    fn accurate_bomb_with_catcher_publishes_catch_accurate_bomb() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(10, 5));
        game.field_model.set_player_state("c1", PlayerState::new(PS_STANDING));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccurateBomb))
        });
        assert!(mode.is_some());
    }

    #[test]
    fn missed_pass_publishes_catch_missed_pass_mode() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepResolvePass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
        });
        assert!(mode.is_some());
    }

    #[test]
    fn intercepted_pass_returns_next_step() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(8, 5));
        let mut step = StepResolvePass::new();
        step.deflection_successful = true;
        step.interception_successful = true;
        step.interceptor_id = Some("i1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn deflection_without_interception_publishes_deflected_mode() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(8, 5));
        let mut step = StepResolvePass::new();
        step.deflection_successful = true;
        step.interception_successful = false;
        step.interceptor_id = Some("i1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::Deflected))
        });
        assert!(mode.is_some(), "expected Deflected mode for deflection without interception");
    }

    #[test]
    fn bomb_deflection_publishes_deflected_bomb() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(8, 5));
        let mut step = StepResolvePass::new();
        step.deflection_successful = true;
        step.interception_successful = false;
        step.interceptor_id = Some("i1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::DeflectedBomb))
        });
        assert!(mode.is_some(), "expected DeflectedBomb mode");
    }

    #[test]
    fn accurate_pass_no_catcher_publishes_empty_square_mode() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccuratePassEmptySquare))
        });
        assert!(mode.is_some());
    }

    #[test]
    fn interceptor_id_alone_does_not_imply_interception_successful() {
        // Regression test: a successful (non-easy) interception publishes InterceptorId
        // and (via the caller of set_parameter, e.g. StepIntercept) sets deflection_successful,
        // but must NOT be treated as a clean "interception successful" catch unless
        // StepIntercept also published InterceptionSuccessful(true) — that only happens for
        // an *easy* interception (Yoink-style skill). Absent that signal, the pass should
        // route through the DEFLECTED path (still needs a catch roll), not skip straight
        // to NextStep as if fully caught.
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(8, 5));
        let mut step = StepResolvePass::new();
        step.set_parameter(&StepParameter::InterceptorId(Some("i1".into())));
        assert!(step.deflection_successful, "InterceptorId alone should set deflection_successful");
        assert!(
            !step.interception_successful,
            "InterceptorId alone must NOT set interception_successful"
        );
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::Deflected))
        });
        assert!(mode.is_some(), "expected Deflected mode when InterceptionSuccessful was never published");
    }

    #[test]
    fn interception_successful_parameter_is_consumed() {
        let mut step = StepResolvePass::new();
        assert!(step.set_parameter(&StepParameter::InterceptionSuccessful(true)));
        assert!(step.interception_successful);
    }

    #[test]
    fn out_of_bounds_bomb_publishes_bomb_out_of_bounds() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.field_model.out_of_bounds = true;
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepResolvePass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let oob = out.published.iter().any(|p| matches!(p, StepParameter::BombOutOfBounds(true)));
        assert!(oob, "expected BombOutOfBounds(true) for bomb OOB");
    }

    #[test]
    fn hypnotized_standing_catcher_is_treated_as_having_no_tacklezones() {
        // Regression: Java's condition is `!catcherState.hasTacklezones()`, not
        // `!catcherState.canBeBlocked()`. hasTacklezones() additionally excludes
        // confused/hypnotized/eye-gouged players, which canBeBlocked() does not check.
        // A STANDING-but-hypnotized catcher must be routed through the "no tacklezones"
        // (empty-square-equivalent) accurate-pass branch, not the normal catch branch.
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(10, 5));
        let hypnotized = PlayerState::new(PS_STANDING).change_hypnotized(true);
        game.field_model.set_player_state("c1", hypnotized);
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let empty_square_mode = out.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
        });
        assert!(
            empty_square_mode,
            "expected a hypnotized (no-tacklezones) standing catcher to route through the \
             CatchMissedPass branch, not the normal CatchAccuratePass branch"
        );
    }
}
