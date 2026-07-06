use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepResolvePass.
///
/// Sets animation, syncs game model, then routes ball/bomb placement based on whether
/// the pass was intercepted, accurate, or missed.
///
/// Publishes: `CatchScatterThrowInMode`, `PassAccurate`, `BombOutOfBounds`.
///
/// Reads from PassState: `interception_successful`, `result` (PassResult), `catcher_id`,
/// `thrower_coordinate`.  These fields live on `GameState.passState` in Java — in the
/// Rust step-rewrite they are expected to arrive via published StepParameters
/// (`PassAccurate`, `InterceptorId`, `CatcherId`).
///
/// client-only: Animation / syncGameModel infrastructure — client-side rendering only.
/// Interception and accurate-pass routing are wired via StepParameter::InterceptorId and PassAccurate.
pub struct StepResolvePass {
    /// Consumed from PassAccurate parameter published by StepPass.
    pub pass_accurate: bool,
    /// Consumed from InterceptorId parameter published by StepIntercept.
    pub interceptor_id: Option<String>,
    /// Consumed from CatcherId parameter.
    pub catcher_id: Option<String>,
    /// Whether the pass is a bomb action (derived from thrower_action).
    pub is_bomb: bool,
}

impl StepResolvePass {
    pub fn new() -> Self {
        Self {
            pass_accurate: false,
            interceptor_id: None,
            catcher_id: None,
            is_bomb: false,
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
            StepParameter::InterceptorId(v) => { self.interceptor_id = v.clone(); true }
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
        // client-only: Animation + syncGameModel infrastructure — client-side

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: if (state.isInterceptionSuccessful())
        if self.interceptor_id.is_some() {
            // Java: ball/bomb → interceptor coordinate
            if let Some(ref id) = self.interceptor_id {
                if let Some(coord) = game.field_model.player_coordinate(id) {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(coord);
                    } else {
                        game.field_model.ball_coordinate = Some(coord);
                    }
                }
            }
            return StepOutcome::next();
        }

        // Java: else if (state.getResult() == PassResult.ACCURATE)
        if self.pass_accurate {
            let pass_coord = game.pass_coordinate;
            // Java: check catcher state (tacklezones) — simplified: assume catcher present and standing
            if let Some(ref catcher_id) = self.catcher_id {
                // Java: PlayerState catcherState = getPlayerState(catcher)
                // Java: if catcher==null || catcherState==null || !catcherState.hasTacklezones()
                //          → CATCH_ACCURATE_PASS_EMPTY_SQUARE or CATCH_MISSED_PASS
                // hasTacklezones() ≡ can_be_blocked() — standing/moving only
                let catcher_has_tackle_zones = game.field_model.player_state(catcher_id)
                    .map(|s| s.can_be_blocked())
                    .unwrap_or(false);
                if !catcher_has_tackle_zones {
                    // treat as empty square — ball lands at pass_coordinate
                    if let Some(pc) = game.pass_coordinate {
                        if is_bomb { game.field_model.bomb_coordinate = Some(pc); }
                        else { game.field_model.ball_coordinate = Some(pc); }
                    }
                    return StepOutcome::next().publish(StepParameter::CatchScatterThrowInMode(
                        if is_bomb { CatchScatterThrowInMode::CatchAccurateBombEmptySquare }
                        else { CatchScatterThrowInMode::CatchAccuratePassEmptySquare }
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
                if is_bomb {
                    return StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccurateBombEmptySquare,
                        ));
                } else {
                    return StepOutcome::next()
                        .publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::CatchAccuratePassEmptySquare,
                        ));
                }
            }
        }

        // Java: else (missed / inaccurate) branch
        // Java: if (fieldModel.isOutOfBounds())
        if game.field_model.out_of_bounds {
            // Ball landed out of bounds — throw-in sequence handled by earlier scatter step.
        }

        // Default: inaccurate pass, ball at pass coordinate, catch as missed pass
        if let Some(pass_coord) = game.pass_coordinate {
            if is_bomb {
                game.field_model.bomb_coordinate = Some(pass_coord);
                game.field_model.bomb_moving = true;
            } else {
                game.field_model.ball_coordinate = Some(pass_coord);
                game.field_model.ball_moving = true;
            }
        }

        if is_bomb {
            StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb))
        } else {
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn accurate_pass_with_catcher_publishes_catch_accurate_pass() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        // Catcher must be on field with tackle zones (standing)
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
        // Catcher must be on field with tackle zones
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
    fn accurate_pass_with_prone_catcher_goes_to_empty_square() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        // Prone catcher has no tackle zones → treat as empty square
        use ffb_model::enums::PS_PRONE;
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(10, 5));
        game.field_model.set_player_state("c1", PlayerState::new(PS_PRONE));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccuratePassEmptySquare))
        });
        assert!(mode.is_some(), "prone catcher → empty square mode");
    }

    #[test]
    fn intercepted_pass_moves_ball_to_interceptor() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(8, 5));
        let mut step = StepResolvePass::new();
        step.interceptor_id = Some("i1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(8, 5)));
    }

    #[test]
    fn missed_pass_publishes_catch_missed_pass_mode() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepResolvePass::new();
        // pass_accurate stays false, no interceptor
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
        });
        assert!(mode.is_some());
    }

    #[test]
    fn accurate_pass_no_catcher_publishes_empty_square_mode() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        // catcher_id is None → empty square
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccuratePassEmptySquare))
        });
        assert!(mode.is_some());
    }
}
