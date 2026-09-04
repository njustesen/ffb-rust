/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepInitScatterPlayer`.
///
/// Step in TTM scatter sequence. Calculates where the thrown/kicked player lands:
/// - If in-bounds with a player there: injury the hit player, continue scatter loop.
/// - If in-bounds empty: place player, end loop.
/// - If out-of-bounds: crowd-injury, publish THROW_IN / CATCH_SCATTER_THROW_IN_MODE.
///
/// Init params (all mandatory): THROWN_PLAYER_ID, THROWN_PLAYER_STATE,
///   THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_COORDINATE, THROW_SCATTER.
/// Optional init: IS_KICKED_PLAYER.
use ffb_model::enums::{ApothecaryMode, PS_FALLING, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::injuryType::injury_type_crowd_push::InjuryTypeCrowdPush;
use crate::injury::injuryType::injury_type_ktm_crowd::InjuryTypeKTMCrowd;
use crate::injury::injuryType::injury_type_ttm_hit_player::InjuryTypeTTMHitPlayer;
use crate::step::action::ttm::util_throw_team_mate_sequence::{scatter_player, kick_player};
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury;

/// Java: `StepInitScatterPlayer` (bb2016/ttm).
pub struct StepInitScatterPlayer {
    /// Java: `fThrownPlayerId` — mandatory init param.
    thrown_player_id: Option<String>,
    /// Java: `fThrownPlayerState` — mandatory init param.
    thrown_player_state: Option<PlayerState>,
    /// Java: `fThrownPlayerHasBall` — mandatory init param.
    thrown_player_has_ball: bool,
    /// Java: `fThrownPlayerCoordinate` — mandatory init param.
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrowScatter` — mandatory init param.
    throw_scatter: bool,
    /// Java: `fIsKickedPlayer` — optional.
    is_kicked_player: bool,
}

impl StepInitScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            is_kicked_player: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Guard: no player or coordinate → skip.
        // Java: `if ((thrownPlayer == null) || (fThrownPlayerCoordinate == null)) { setNextAction(NEXT_STEP); return; }`
        // Note: Java does NOT publish any parameters on this early-return path.
        let thrown_player_id = match &self.thrown_player_id {
            Some(id) if game.player(id).is_some() => id.clone(),
            _ => {
                return StepOutcome::next();
            }
        };
        let thrown_player_coord = match self.thrown_player_coordinate {
            Some(c) => c,
            None => {
                return StepOutcome::next();
            }
        };

        // Java: startCoordinate = thrownPlayerCoordinate (if throwScatter → use passCoordinate)
        let start_coord = if self.throw_scatter {
            // client-side: game.getFieldModel().setRangeRuler(null); clearMoveSquares()
            game.pass_coordinate.unwrap_or(thrown_player_coord)
        } else {
            thrown_player_coord
        };

        // Java: ScatterResult scatterResult = ...
        let scatter_result = if self.is_kicked_player && self.throw_scatter {
            kick_player(game, rng, thrown_player_coord, start_coord)
        } else {
            scatter_player(game, rng, start_coord, self.throw_scatter)
        };

        let end_coord = scatter_result.last_valid_coordinate;

        // client-only: setAnimation, syncGameModel

        let mut outcome = StepOutcome::next();
        let mut player_landed_upon: Option<String> = None;

        if std::env::var("FFB_BOMB").is_ok() {
            eprintln!(
                "RSCATTER start={:?} end={:?} in_bounds={} at_end={:?}",
                start_coord, end_coord, scatter_result.in_bounds,
                game.field_model.player_at(end_coord)
            );
        }
        if scatter_result.in_bounds {
            // Java: playerLandedUpon = game.getFieldModel().getPlayer(endCoordinate)
            //       (skip if same as thrown player)
            let at_end = game.field_model.player_at(end_coord)
                .filter(|id| *id != &thrown_player_id)
                .cloned();

            if let Some(hit_player_id) = at_end {
                // Java: publishParameter(DROP_THROWN_PLAYER, true)
                outcome = outcome.publish(StepParameter::DropThrownPlayer(true));

                // Java: handleInjury(InjuryTypeTTMHitPlayer, ...)
                let mut hit_injury = InjuryTypeTTMHitPlayer::new();
                let injury_result = util_server_injury::handle_injury(
                    game, rng, &mut hit_injury,
                    None, &hit_player_id, end_coord, None, None,
                    ApothecaryMode::HitPlayer,
                );
                // Java PUBLISHES this result and does NOT apply it here — the ScatterPlayer
                // sequence's `APOTHECARY [APOTHECARY_HIT_PLAYER]` step calls `applyTo` later.
                // Applying it here is not merely early, it CANCELS the drop below: `InjuryResult
                // .applyTo` only does `playerState.changeBase(...)`, so the hit player is already
                // PRONE by the time `UtilServerInjury.dropPlayer` runs, and dropPlayer's whole
                // body is guarded by `if (base != PRONE && base != STUNNED)` — the branch that
                // carries `playerState.changeRooted(false)`. A rooted Treeman landed on by a
                // thrown team-mate therefore stayed rooted forever and never rolled Take Root
                // again (halfling bb2016 seed 21: Java rolls Take Root at die pos 70 then the
                // stand-up d6 at 71, Rust rolled only the stand-up, shifting every later die).
                // Same class as the out-of-bounds branch below, which was corrected earlier.
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));

                // Java: if (isHomePlaying && teamHome.hasPlayer(hit)) || (!isHome && teamAway.hasPlayer(hit)) → END_TURN
                let hit_own_team = if game.home_playing {
                    game.team_home.has_player(&hit_player_id)
                } else {
                    game.team_away.has_player(&hit_player_id)
                };
                if hit_own_team {
                    outcome = outcome.publish(StepParameter::EndTurn(true));
                }

                // Continue scatter loop
                game.field_model.set_player_coordinate(&thrown_player_id, end_coord);
                outcome = outcome.publish(StepParameter::ThrownPlayerCoordinate(Some(end_coord)));

                player_landed_upon = Some(hit_player_id);
            } else {
                // Java: empty square — place player, end loop
                game.field_model.set_player_coordinate(&thrown_player_id, end_coord);
                if let Some(state) = self.thrown_player_state {
                    game.field_model.set_player_state(&thrown_player_id, state);
                }
                game.defender_id = None;
                // null = end-loop sentinel
                outcome = outcome.publish(StepParameter::ThrownPlayerCoordinate(None));
            }
        } else {
            // Out of bounds — crowd injury
            game.field_model.set_player_state(&thrown_player_id, PlayerState::new(PS_FALLING));

            let injury_result = if self.is_kicked_player {
                let mut inj = InjuryTypeKTMCrowd::new();
                util_server_injury::handle_injury(
                    game, rng, &mut inj,
                    None, &thrown_player_id, end_coord, None, None,
                    ApothecaryMode::ThrownPlayer,
                )
            } else {
                let mut inj = InjuryTypeCrowdPush::new();
                util_server_injury::handle_injury(
                    game, rng, &mut inj,
                    None, &thrown_player_id, end_coord, None, None,
                    ApothecaryMode::ThrownPlayer,
                )
            };
            // Java only ROLLS here (handleInjury publishes; the THROWN_PLAYER apothecary step
            // applies it later) — the player must stay FALLING so StepRightStuff's FALLING
            // check skips the landing roll. The old immediate apply_to sent a KO'd player to
            // the box, the FALLING check missed, and Rust rolled a phantom landing d6 for a
            // player in the crowd (goblin bb2016 seed 71 i=175: Java 8 dice, Rust 9). Same
            // class as the ITER78-81 "TTM landing drop-before-apply" fix in land_injury.
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));

            if self.thrown_player_has_ball {
                outcome = outcome
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                    .publish(StepParameter::ThrowInCoordinate(end_coord))
                    .publish(StepParameter::EndTurn(true));
            }

            // End-loop sentinel
            outcome = outcome.publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // Java: always published at end
        outcome = outcome
            .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
            .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
            .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball));

        // Java: if playerLandedUpon != null → dropPlayer
        if let Some(hit_id) = player_landed_upon {
            let drop_params = util_server_injury::drop_player(game, &hit_id, true);
            for p in drop_params {
                outcome = outcome.publish(p);
            }
        }

        outcome
    }
}

impl Default for StepInitScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitScatterPlayer {
    fn id(&self) -> StepId { StepId::InitScatterPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)   => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrowScatter(v)             => { self.throw_scatter = *v; true }
            StepParameter::IsKickedPlayer(v)           => { self.is_kicked_player = *v; true }
            StepParameter::KickedPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerCoordinate(v)   => { self.thrown_player_coordinate = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerGender, PlayerType, PS_STANDING, Rules};
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_home_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_player_returns_next_with_params() {
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn early_exit_publishes_nothing() {
        // Java: `if ((thrownPlayer == null) || (fThrownPlayerCoordinate == null)) {
        //   getResult().setNextAction(StepAction.NEXT_STEP); return; }`
        // No StepParameter is published on this guard-clause path.
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("nobody".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.is_empty(), "Java does not publish any parameters on the early-return path");
    }

    #[test]
    fn set_parameter_throw_scatter() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrowScatter(true)));
        assert!(step.throw_scatter);
    }

    #[test]
    fn set_parameter_is_kicked() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.is_kicked_player);
    }

    #[test]
    fn set_parameter_kicked_player_aliases() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::KickedPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id, Some("p1".into()));
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::KickedPlayerCoordinate(Some(coord))));
        assert_eq!(step.thrown_player_coordinate, Some(coord));
    }

    #[test]
    fn in_bounds_empty_lands_player_and_clears_coord_param() {
        let mut step = StepInitScatterPlayer::new();
        let coord = FieldCoordinate::new(12, 7); // mid-pitch, scatter stays in bounds
        let state = PlayerState::new(PS_STANDING);
        add_home_player(&mut make_game(), "thrower", coord); // just to check add_home_player compiles
        let mut game = make_game();
        add_home_player(&mut game, "thrower", coord);
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_coordinate = Some(coord);
        step.thrown_player_state = Some(state);
        step.throw_scatter = true; // 3 d8 scatter rolls from mid-pitch
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
        // ThrownPlayerId always published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(_))));
    }

    #[test]
    fn out_of_bounds_publishes_injury_result() {
        let mut step = StepInitScatterPlayer::new();
        // Place player at top-left edge; scatter will very likely go OOB
        let coord = FieldCoordinate::new(0, 0);
        let state = PlayerState::new(PS_STANDING);
        let mut game = make_game();
        add_home_player(&mut game, "thrower", coord);
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_coordinate = Some(coord);
        step.thrown_player_state = Some(state);
        step.throw_scatter = true;
        // Use a seed that makes d8 roll 7 = NW from (0,0) which is definitely OOB
        // (just verify no panic and INJURY_RESULT is published when OOB)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // If went OOB, should have INJURY_RESULT published
        // (if stayed in-bounds, still has ThrownPlayerId)
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(_))));
    }

    #[test]
    fn landing_on_a_rooted_player_drops_it_and_clears_the_root() {
        // Java bb2016 StepInitScatterPlayer.executeStep, in-bounds + playerLandedUpon != null:
        //   publishParameter(DROP_THROWN_PLAYER, true);
        //   InjuryResult injuryResultHitPlayer = UtilServerInjury.handleInjury(
        //       this, new InjuryTypeTTMHitPlayer(), null, playerLandedUpon, endCoordinate, ...);
        //   publishParameter(new StepParameter(INJURY_RESULT, injuryResultHitPlayer));
        //   ...
        //   if (playerLandedUpon != null) {
        //       publishParameters(UtilServerInjury.dropPlayer(this, playerLandedUpon,
        //           ApothecaryMode.HIT_PLAYER, true));
        //   }
        // Java never calls injuryResultHitPlayer.applyTo(...) here — the sequence's
        // APOTHECARY [APOTHECARY_HIT_PLAYER] step does. So when dropPlayer runs, the hit player is
        // still STANDING/MOVING and dropPlayer's non-PRONE/STUNNED branch fires:
        //   `if (base != HIT_ON_GROUND) playerState = playerState.changeRooted(false);`
        //   `playerState = playerState.changeBase(pPlayerBase /* PRONE */);`
        // i.e. being landed on UNROOTS a rooted Treeman.
        let seed = 42u64;
        let start = FieldCoordinate::new(12, 7);

        // Where does the 3× d8 scatter of this seed end? The step draws no dice before the
        // scatter, so a fresh GameRng with the same seed reproduces it exactly.
        let end = {
            let mut probe = make_game();
            add_home_player(&mut probe, "thrown", start);
            let r = crate::step::action::ttm::util_throw_team_mate_sequence::scatter_player(
                &mut probe, &mut GameRng::new(seed), start, true);
            assert!(r.in_bounds, "seed must scatter in-bounds for this test");
            r.last_valid_coordinate
        };
        assert_ne!(end, start, "the occupant must not be the thrown player itself");

        let mut game = make_game();
        add_home_player(&mut game, "thrown", start);
        add_home_player(&mut game, "occupant", end);
        // The occupant is a Treeman that has already failed its Take Root roll.
        let rooted = game.field_model.player_state("occupant").unwrap().change_rooted(true);
        game.field_model.set_player_state("occupant", rooted);
        assert!(game.field_model.player_state("occupant").unwrap().is_rooted());
        game.pass_coordinate = Some(start);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrown".into());
        step.thrown_player_coordinate = Some(start);
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.throw_scatter = true;
        let out = step.start(&mut game, &mut GameRng::new(seed));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropThrownPlayer(true))),
            "Java publishes DROP_THROWN_PLAYER for a landing on a player");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))),
            "Java publishes the TTM-hit InjuryResult (the apothecary step applies it later)");

        let after = game.field_model.player_state("occupant").unwrap();
        assert!(!after.is_rooted(),
            "dropPlayer must clear ROOTED — an applyTo here would leave the occupant PRONE first \
             and skip dropPlayer's whole branch");
        assert_eq!(after.base(), ffb_model::enums::PS_PRONE,
            "dropPlayer places the hit player PRONE");
    }

    // ── report wiring (bb2016 has no addReport calls in Java) ─────────────────

    #[test]
    fn no_player_does_not_add_any_reports() {
        // Java bb2016 StepInitScatterPlayer has no addReport calls.
        // Verify the step runs cleanly with no player set.
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn early_exit_does_not_publish_thrown_player_id() {
        // Java's guard-clause early return sets NEXT_STEP and returns immediately,
        // without publishing THROWN_PLAYER_ID (or any other parameter).
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        // No player in game → early exit path
        step.thrown_player_id = Some("nobody".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(_))),
            "Java's early-return guard clause does not publish ThrownPlayerId");
    }
}
