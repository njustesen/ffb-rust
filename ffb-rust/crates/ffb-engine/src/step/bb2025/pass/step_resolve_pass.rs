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
    /// Java BB2020 `PassState.deflectionSuccessful` — a successful interception ROLL.
    pub deflection_successful: bool,
    /// Java BB2020 `PassState.interceptionSuccessful` — set only by an *easy* intercept, i.e. the
    /// deflection was also an outright catch.
    pub interception_successful: bool,
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
            deflection_successful: false,
            interception_successful: false,
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
            // BB2020 PassState.deflectionSuccessful / interceptionSuccessful (see execute_step).
            StepParameter::DeflectionSuccessful(v) => { self.deflection_successful = *v; true }
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
        // client-only: Animation + syncGameModel infrastructure — client-side

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: `if (state.isInterceptionSuccessful())` — the SUCCESS FLAG, not the mere presence
        // of an interceptor id. Rust gated on `interceptor_id.is_some()`, so a FAILED interception
        // (whose id is still around) teleported the ball onto the player who missed it, and a
        // published id could even outlive its own pass sequence — Java re-creates `PassState` per
        // pass, Rust threads step parameters that persist (dwarf bb2025 seed 45 i=17: Rust put the
        // ball on away_01 at 13,7 where Java bounced it to 12,4, with identical dice).
        // The gate is PER-EDITION. `bb2020/StepResolvePass:43` opens on
        // `state.isDeflectionSuccessful()` (the deflected ball still has to be caught or scattered
        // from the interceptor's square), whereas `bb2025:43` opens on
        // `state.isInterceptionSuccessful()` (a successful interception simply IS a catch).
        // Both are the SUCCESS FLAG, never the mere presence of an interceptor id: an id left over
        // from a failed interception — or from an earlier pass, since Rust's step parameters outlive
        // the sequence Java re-creates per pass — teleported the ball onto the player who missed it.
        let took_the_ball = if game.rules == ffb_model::enums::Rules::Bb2020 {
            self.deflection_successful
        } else {
            self.interception_successful
        };
        if took_the_ball {
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
            // BB2020 (`bb2020/pass/StepResolvePass:43-53`): a DEFLECTION that was not an outright
            // catch puts the ball on the interceptor's square and publishes
            // CATCH_SCATTER_THROW_IN_MODE = DEFLECTED, so the deflected ball is caught-or-scattered
            // from there. BB2025 has no deflection concept — every successful interception is a
            // catch — and this shared step runs for both, so the ball simply stopped dead on the
            // interceptor in BB2020 and the whole scatter chain was skipped (high_elf bb2020
            // seed 61 i=3: the ball ended at 12,7 in Rust and 14,7 in Java).
            if game.rules == ffb_model::enums::Rules::Bb2020
                && self.deflection_successful
                && !self.interception_successful
            {
                return StepOutcome::next().publish(StepParameter::CatchScatterThrowInMode(
                    if is_bomb {
                        CatchScatterThrowInMode::DeflectedBomb
                    } else {
                        CatchScatterThrowInMode::Deflected
                    },
                ));
            }
            return StepOutcome::next();
        }

        // Java: else if (state.getResult() == PassResult.ACCURATE)
        if self.pass_accurate {
            let pass_coord = game.pass_coordinate;
            // Java shares one PassState across every pass step, so StepResolvePass reads the
            // catcherId set back in StepInitPassing. Rust delivers CatcherId as a step parameter, but
            // delivery STOPS at the first consumer (StepDispatchPassing), so it never reaches here and
            // self.catcher_id stays None. For an ACCURATE pass the ball lands on pass_coordinate, so
            // the catcher is exactly the player standing there — fall back to that. Without this a
            // present receiver was treated as an empty square → CatchScatter (+1 "Inaccurate Pass or
            // Scatter") instead of CatchAccuratePass, so a catchable accurate pass failed and turned
            // over (seed 23 i=158: away_04 at (21,4) needed a 4+ catch instead of 3+ and missed d6=3).
            let effective_catcher_id = self.catcher_id.clone()
                .or_else(|| pass_coord.and_then(|pc| game.field_model.player_at(pc).cloned()));
            // Java: check catcher state (tacklezones) — simplified: assume catcher present and standing
            if let Some(ref catcher_id) = effective_catcher_id {
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
            if is_bomb {
                // Java: fieldModel.setBombCoordinate(null); publish BOMB_OUT_OF_BOUNDS=true
                game.field_model.bomb_coordinate = None;
                return StepOutcome::next().publish(StepParameter::BombOutOfBounds(true));
            } else {
                // Java: publish CATCH_SCATTER_THROW_IN_MODE=THROW_IN; publish
                // THROW_IN_COORDINATE=passCoordinate; setBallMoving(true)
                // Note: ball_coordinate is deliberately left as set by the earlier scatter step.
                game.field_model.ball_moving = true;
                let mut outcome = StepOutcome::next().publish(StepParameter::CatchScatterThrowInMode(
                    CatchScatterThrowInMode::ThrowIn,
                ));
                if let Some(pc) = game.pass_coordinate {
                    outcome = outcome.publish(StepParameter::ThrowInCoordinate(pc));
                }
                return outcome;
            }
        }

        // Default (not out of bounds): inaccurate pass, ball at pass coordinate, catch as missed pass
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

    /// `bb2020/pass/StepResolvePass:43-53`: a DEFLECTION that was not an outright catch puts the
    /// ball on the interceptor's square and publishes CATCH_SCATTER_THROW_IN_MODE = DEFLECTED, so
    /// the deflected ball is caught-or-scattered from there. BB2025 has no deflection concept and
    /// this shared step serves both editions, so without the gate the ball stopped dead on the
    /// interceptor and the whole scatter chain was skipped (high_elf bb2020 seed 61 i=3).
    /// The ball-to-interceptor branch opens on the PER-EDITION SUCCESS FLAG, never on the mere
    /// presence of an interceptor id. Java re-creates `PassState` per pass, so a failed (or earlier)
    /// interception simply has no flag set; Rust threads step parameters that OUTLIVE the sequence,
    /// so gating on the id teleported the ball onto a player who had missed it — or onto the
    /// interceptor of a previous pass entirely (dwarf bb2025 seed 45 i=17).
    #[test]
    fn a_stale_interceptor_id_without_a_success_flag_does_not_move_the_ball() {
        use ffb_model::model::player::Player;
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let mut away = test_team("away", 0);
            let mut opp = Player::default();
            opp.id = "opp1".into();
            away.players.push(opp);
            let mut game = Game::new(test_team("home", 0), away, rules);
            game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
            game.field_model.ball_coordinate = Some(FieldCoordinate::new(2, 2));

            let mut step = StepResolvePass::new();
            step.interceptor_id = Some("opp1".into());   // id present …
            step.deflection_successful = false;          // … but nothing succeeded
            step.interception_successful = false;

            step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(2, 2)),
                "{rules:?}: an interceptor id alone must not move the ball");
        }
    }

    #[test]
    fn a_bb2020_deflection_publishes_the_deflected_mode_and_bb2025_does_not() {
        use ffb_model::model::player::Player;
        for (rules, expect_deflected) in [(Rules::Bb2020, true), (Rules::Bb2025, false)] {
            let mut away = test_team("away", 0);
            let mut opp = Player::default();
            opp.id = "opp1".into();
            away.players.push(opp);
            let mut game = Game::new(test_team("home", 0), away, rules);
            game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));

            let mut step = StepResolvePass::new();
            step.interceptor_id = Some("opp1".into());
            // Each edition opens the branch on its OWN flag: BB2020 on deflection, BB2025 on
            // interception (Java `bb2020/StepResolvePass:43` vs `bb2025:43`).
            step.deflection_successful = rules == Rules::Bb2020;
            step.interception_successful = rules != Rules::Bb2020;

            let out = step.start(&mut game, &mut GameRng::new(0));
            let deflected = out.published.iter().any(|p| matches!(p,
                StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::Deflected)));
            assert_eq!(deflected, expect_deflected,
                "{rules:?}: DEFLECTED mode is BB2020-only");
            assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(7, 7)),
                "{rules:?}: either way the ball lands on the interceptor's square");
        }
    }

    /// An *easy* intercept (`interceptionSuccessful`) IS an outright catch even in BB2020, so it
    /// must NOT publish DEFLECTED — Java guards the branch with `!state.isInterceptionSuccessful()`.
    #[test]
    fn a_bb2020_easy_intercept_is_a_catch_not_a_deflection() {
        use ffb_model::model::player::Player;
        let mut away = test_team("away", 0);
        let mut opp = Player::default();
        opp.id = "opp1".into();
        away.players.push(opp);
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2020);
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));

        let mut step = StepResolvePass::new();
        step.interceptor_id = Some("opp1".into());
        step.deflection_successful = true;
        step.interception_successful = true;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p,
            StepParameter::CatchScatterThrowInMode(_))),
            "an easy intercept keeps the ball — no catch/scatter mode is published");
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
    fn accurate_pass_falls_back_to_player_at_pass_coordinate_when_catcher_id_missing() {
        // The CatcherId parameter is consumed by StepDispatchPassing before it reaches StepResolvePass,
        // so catcher_id stays None. For an accurate pass the receiver stands on pass_coordinate, so the
        // step must fall back to that player and route CatchAccuratePass — NOT the empty-square path
        // (which the catch step turns into CatchScatter, +1 to catch, failing catchable passes).
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        let target = FieldCoordinate::new(21, 4);
        game.pass_coordinate = Some(target);
        game.field_model.set_player_coordinate("recv", target);
        game.field_model.set_player_state("recv", PlayerState::new(PS_STANDING));
        let mut step = StepResolvePass::new();
        step.pass_accurate = true;
        step.catcher_id = None; // parameter never arrived
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccuratePass))),
            "falls back to the standing player at pass_coordinate → CatchAccuratePass");
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
        // The branch opens on Java's success flag (`isInterceptionSuccessful()` in BB2025), not on
        // the presence of an id — an id alone can be left over from a failed or earlier pass.
        step.interception_successful = true;
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
    fn missed_pass_out_of_bounds_publishes_throw_in_not_catch_missed_pass() {
        // Bug fix regression: Java's else-branch checks fieldModel.isOutOfBounds() first;
        // out-of-bounds routes to THROW_IN (not CATCH_MISSED_PASS), and must NOT
        // overwrite ball_coordinate to pass_coordinate (it stays wherever the earlier
        // scatter step left it).
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.out_of_bounds = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 0));
        let mut step = StepResolvePass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let throw_in_mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
        });
        assert!(throw_in_mode.is_some(), "out-of-bounds missed pass should publish ThrowIn mode");
        let missed_mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
        });
        assert!(missed_mode.is_none(), "should not publish CatchMissedPass when out of bounds");
        let throw_in_coord = out.published.iter().find(|p| matches!(p, StepParameter::ThrowInCoordinate(_)));
        assert!(throw_in_coord.is_some(), "expected ThrowInCoordinate published");
        // ball_coordinate must be left untouched (not overwritten to pass_coordinate)
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(0, 0)));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn missed_bomb_out_of_bounds_clears_bomb_coordinate_and_publishes_bomb_out_of_bounds() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.out_of_bounds = true;
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(3, 3));
        let mut step = StepResolvePass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.bomb_coordinate.is_none(), "bomb_coordinate should be cleared to null");
        let oob = out.published.iter().find(|p| matches!(p, StepParameter::BombOutOfBounds(true)));
        assert!(oob.is_some(), "expected BombOutOfBounds(true) published");
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
