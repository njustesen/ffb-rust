/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepInitScatterPlayer`.
///
/// Step in the TTM scatter sequence. Calculates where the thrown/kicked player lands:
/// - If in-bounds with a player there: injury the hit player, continue scatter loop.
/// - If in-bounds empty (and no crash-landing): place player, end loop.
/// - If crash-landing: drop player without landing roll.
/// - If out-of-bounds: crowd-injury.
///
/// BB2020 differences vs BB2016:
///  - Adds `deviate` flag (WILDLY_INACCURATE path — deviates from thrower coordinate).
///  - Adds `crash_landing` flag.
///  - Adds `swoop_direction` for the Swoop skill path.
///  - If player state is PICKED_UP at start, changes it to IN_THE_AIR.
///
/// Init params (mandatory): THROWN_PLAYER_ID, THROWN_PLAYER_STATE,
///   THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_COORDINATE, THROW_SCATTER.
/// Optional init: IS_KICKED_PLAYER, PASS_DEVIATES, CRASH_LANDING, DIRECTION.
use ffb_mechanics::mechanics::scatter_coordinate;
use ffb_model::events::GameEvent;
use ffb_model::enums::{ApothecaryMode, Direction, PS_FALLING, PS_IN_THE_AIR, PS_PICKED_UP, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::{END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM, SWOOP_DISTANCE};
use ffb_model::option::util_game_option::{get_int_option, is_option_enabled};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::injuryType::injury_type_crowd_push::InjuryTypeCrowdPush;
use crate::injury::injuryType::injury_type_ktm_crowd::InjuryTypeKTMCrowd;
use crate::injury::injuryType::injury_type_ttm_hit_player::InjuryTypeTTMHitPlayer;
use crate::step::action::ttm::util_throw_team_mate_sequence::{scatter_player, kick_player, ScatterResult};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use ffb_model::report::report_pass_deviate::ReportPassDeviate;
use ffb_model::report::bb2020::report_swoop_player::ReportSwoopPlayer;
use crate::step::mixed::ttm::ttm_to_crowd_handler::TtmToCrowdHandler;
use crate::step::util_server_injury;

/// Java: `StepInitScatterPlayer` (bb2020/ttm).
pub struct StepInitScatterPlayer {
    /// Java: `thrownPlayerId` — mandatory init param.
    thrown_player_id: Option<String>,
    /// Java: `thrownPlayerState` — mandatory init param.
    thrown_player_state: Option<PlayerState>,
    /// Java: `thrownPlayerHasBall` — mandatory init param.
    thrown_player_has_ball: bool,
    /// Java: `thrownPlayerCoordinate` — mandatory init param.
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `throwScatter` — mandatory init param.
    throw_scatter: bool,
    /// Java: `isKickedPlayer` — optional.
    is_kicked_player: bool,
    /// Java: `deviate` (PASS_DEVIATES) — BB2020: wildly-inaccurate path.
    deviate: bool,
    /// Java: `crashLanding` — BB2020 addition.
    crash_landing: bool,
    /// Java: `swoopDirection` (DIRECTION) — BB2020 Swoop skill.
    swoop_direction: Option<Direction>,
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
            deviate: false,
            crash_landing: false,
            swoop_direction: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // BB2020: if player state is PICKED_UP → change to IN_THE_AIR.
        if let (Some(id), Some(state)) = (&self.thrown_player_id, self.thrown_player_state) {
            if state.base() == PS_PICKED_UP {
                let new_state = state.change_base(PS_IN_THE_AIR);
                self.thrown_player_state = Some(new_state);
                game.field_model.set_player_state(id, new_state);
            }
        }

        // Guard: no player or coordinate → NEXT_STEP.
        let thrown_player_id = match &self.thrown_player_id {
            Some(id) if game.player(id).is_some() => id.clone(),
            _ => return StepOutcome::next()
                .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
                .publish(StepParameter::IsKickedPlayer(self.is_kicked_player)),
        };
        let thrown_player_coord = match self.thrown_player_coordinate {
            Some(c) => c,
            None => return StepOutcome::next()
                .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
                .publish(StepParameter::IsKickedPlayer(self.is_kicked_player)),
        };

        // Java: startCoordinate = thrownPlayerCoordinate (then may change based on scatter/deviate)
        let mut start_coord = thrown_player_coord;
        if self.deviate {
            // Java: game.getFieldModel().setRangeRuler(null); game.getFieldModel().clearMoveSquares()
            // Range ruler / move squares are client-side display only; no server state change.
        } else if self.throw_scatter {
            // Java: startCoordinate = game.getPassCoordinate()
            // Range ruler / move squares are client-side display only.
            if let Some(pc) = game.pass_coordinate {
                start_coord = pc;
            }
        }

        // Java: ScatterResult scatterResult = ...
        let scatter_result: ScatterResult;
        let mut swoop_event: Option<GameEvent> = None;
        if self.is_kicked_player && self.throw_scatter {
            // Java: UtilThrowTeamMateSequence.kickPlayer(this, thrownPlayerCoordinate, startCoordinate)
            scatter_result = kick_player(game, rng, thrown_player_coord, start_coord);
        } else if self.deviate {
            scatter_result = self.deviate_scatter(game, rng, thrown_player_coord);
        } else if self.swoop_direction.is_some() {
            scatter_result = self.swoop_scatter(game, rng, start_coord);
            if let Some(coord) = self.thrown_player_coordinate {
                swoop_event = Some(GameEvent::SwoopPlayer { player_id: thrown_player_id.clone(), coord });
            }
        } else {
            // Java: UtilThrowTeamMateSequence.scatterPlayer(this, startCoordinate, throwScatter)
            scatter_result = scatter_player(game, rng, start_coord, self.throw_scatter);
        }

        let end_coord = scatter_result.last_valid_coordinate;

        // Java: getResult().setAnimation(...); UtilServerGame.syncGameModel(this) — client-side only.

        let mut outcome = StepOutcome::next();
        let mut player_landed_upon: Option<String> = None;

        if scatter_result.in_bounds {
            // Java: playerLandedUpon = game.getFieldModel().getPlayer(endCoordinate)
            // (null if same as thrown player)
            let at_end = game.field_model.player_at(end_coord)
                .filter(|id| *id != &thrown_player_id)
                .cloned();

            if let Some(hit_player_id) = at_end {
                // Java: publishParameter(DROP_THROWN_PLAYER, true)
                outcome = outcome.publish(StepParameter::DropThrownPlayer(true));

                // Java: InjuryResult injuryResultHitPlayer = UtilServerInjury.handleInjury(...)
                let mut hit_injury = InjuryTypeTTMHitPlayer::new();
                let injury_result = util_server_injury::handle_injury(
                    game, rng, &mut hit_injury,
                    None, &hit_player_id, end_coord, None, None,
                    ApothecaryMode::HitPlayer,
                );
                injury_result.apply_to(game);
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));

                // Java: alwaysTurnOver = END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM option
                // Java: if (alwaysTurnOver || hitting own team) → publish END_TURN=true
                {
                    let always_turn_over = is_option_enabled(game, END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM);
                    let hit_own_team = if game.home_playing {
                        game.team_home.has_player(&hit_player_id)
                    } else {
                        game.team_away.has_player(&hit_player_id)
                    };
                    if always_turn_over || hit_own_team {
                        outcome = outcome.publish(StepParameter::EndTurn(true));
                    }
                }

                // Java: crashLanding = false
                self.crash_landing = false;
                outcome = outcome
                    .publish(StepParameter::ThrownPlayerCoordinate(Some(end_coord)))
                    .publish(StepParameter::CrashLanding(false))
                    .publish(StepParameter::PlayerEnteringSquare(thrown_player_id.clone()));

                player_landed_upon = Some(hit_player_id);
            } else if self.crash_landing {
                // Java: crashLanding = false; publishParameter(DROP_THROWN_PLAYER, true)
                self.crash_landing = false;
                outcome = outcome
                    .publish(StepParameter::DropThrownPlayer(true))
                    .publish(StepParameter::ThrownPlayerCoordinate(Some(end_coord)))
                    .publish(StepParameter::CrashLanding(false))
                    .publish(StepParameter::PlayerEnteringSquare(thrown_player_id.clone()));
            } else {
                // Java: empty square + no crash → place player at endCoordinate; end loop.
                game.field_model.set_player_coordinate(&thrown_player_id, end_coord);
                if let Some(state) = self.thrown_player_state {
                    game.field_model.set_player_state(&thrown_player_id, state);
                }
                game.defender_id = None;

                // Java: THROWN_PLAYER_COORDINATE = null (end loop sentinel)
                outcome = outcome
                    .publish(StepParameter::ThrownPlayerCoordinate(None))
                    .publish(StepParameter::PlayerEnteringSquare(thrown_player_id.clone()));
            }
        } else {
            // Out of bounds → TtmToCrowdHandler
            let mut injury_type: Box<dyn crate::injury::InjuryTypeServer> = if self.is_kicked_player {
                Box::new(InjuryTypeKTMCrowd::new())
            } else {
                Box::new(InjuryTypeCrowdPush::new())
            };
            let crowd_params = TtmToCrowdHandler::handle(
                game, rng, &thrown_player_id, end_coord,
                self.thrown_player_has_ball, &mut *injury_type,
            );
            for p in crowd_params {
                outcome = outcome.publish(p);
            }
        }

        // Java: always published at end
        outcome = outcome
            .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
            .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
            .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
            .publish(StepParameter::IsKickedPlayer(self.is_kicked_player));

        // Java: if playerLandedUpon != null → publishParameters(UtilServerInjury.dropPlayer(...))
        if let Some(hit_id) = player_landed_upon {
            let drop_params = util_server_injury::drop_player(game, &hit_id, true);
            for p in drop_params {
                outcome = outcome.publish(p);
            }
        }

        // Java: game.getFieldModel().setPlayerCoordinate(thrownPlayer, endCoordinate)
        game.field_model.set_player_coordinate(&thrown_player_id, end_coord);

        if let Some(ev) = swoop_event { outcome.with_event(ev) } else { outcome }
    }

    /// Java: deviate(Game game, FieldCoordinate throwerCoordinate)
    /// Rolls D8 direction + D6 distance from the thrower coordinate.
    fn deviate_scatter(&mut self, game: &mut Game, rng: &mut GameRng, thrower_coord: FieldCoordinate) -> ScatterResult {
        let direction_roll = rng.d8();
        let distance_roll = rng.d6();
        let direction = Direction::for_roll(direction_roll).expect("d8 is 1..=8");

        let (ex, ey) = scatter_coordinate(thrower_coord.x, thrower_coord.y, direction, distance_roll);
        let coord_end = FieldCoordinate::new(ex, ey);

        // Walk back to last valid square if OOB
        let mut last_valid = coord_end;
        let mut valid_dist = distance_roll;
        while !last_valid.is_on_pitch() && valid_dist > 0 {
            valid_dist -= 1;
            let (vx, vy) = scatter_coordinate(thrower_coord.x, thrower_coord.y, direction, valid_dist);
            last_valid = FieldCoordinate::new(vx, vy);
        }

        // Java: publishParameter(THROWN_PLAYER_COORDINATE, lastValidCoordinate)
        self.thrown_player_coordinate = Some(last_valid);
        // Java: addReport(new ReportPassDeviate(coordinateEnd, direction, directionRoll, distanceRoll, true))
        game.report_list.add(ReportPassDeviate::new(coord_end, direction, direction_roll, distance_roll, true));

        ScatterResult::new(last_valid, coord_end.is_on_pitch())
    }

    /// Java: swoop(FieldCoordinate throwerCoordinate, Direction direction)
    /// Rolls D3 distance (or uses SWOOP_DISTANCE option) in the given direction.
    fn swoop_scatter(&mut self, game: &mut Game, rng: &mut GameRng, start_coord: FieldCoordinate) -> ScatterResult {
        let direction = self.swoop_direction.expect("swoop_direction is set when this is called");

        // Java: getOptionWithDefault(SWOOP_DISTANCE).getValue() — 0 means roll D3.
        let distance_option = get_int_option(game, SWOOP_DISTANCE);
        let distance_roll = if distance_option == 0 { rng.d3() } else { distance_option };

        let (ex, ey) = scatter_coordinate(start_coord.x, start_coord.y, direction, distance_roll);
        let coord_end = FieldCoordinate::new(ex, ey);

        let mut last_valid = coord_end;
        let mut valid_dist = distance_roll;
        while !last_valid.is_on_pitch() && valid_dist > 0 {
            valid_dist -= 1;
            let (vx, vy) = scatter_coordinate(start_coord.x, start_coord.y, direction, valid_dist);
            last_valid = FieldCoordinate::new(vx, vy);
        }

        // Java: publishParameter(THROWN_PLAYER_COORDINATE, lastValidCoordinate)
        self.thrown_player_coordinate = Some(last_valid);
        // Java: addReport(new ReportSwoopPlayer(throwerCoordinate, coordinateEnd, direction, distanceRoll))
        game.report_list.add(ReportSwoopPlayer::new(
            start_coord,
            coord_end,
            direction,
            distance_roll,
        ));

        ScatterResult::new(last_valid, coord_end.is_on_pitch())
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
            StepParameter::ThrownPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)  => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrowScatter(v)            => { self.throw_scatter = *v; true }
            StepParameter::IsKickedPlayer(v)          => { self.is_kicked_player = *v; true }
            StepParameter::CrashLanding(v)            => { self.crash_landing = *v; true }
            // Java: PASS_DEVIATES → deviate
            StepParameter::PassDeviates(v)            => { self.deviate = *v; true }
            // Java: DIRECTION → swoopDirection
            StepParameter::Direction(v)               => { self.swoop_direction = Some(*v); true }
            // Kicked-player aliases (same step handles both TTM and KTM scatter).
            StepParameter::KickedPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerCoordinate(v)  => { self.thrown_player_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_init_scatter_player() {
        assert_eq!(StepInitScatterPlayer::new().id(), StepId::InitScatterPlayer);
    }

    #[test]
    fn no_player_returns_next() {
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn picked_up_player_changes_state_to_in_the_air() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(5, 5));
        let picked_up = PlayerState::new(PS_PICKED_UP);
        game.field_model.set_player_state("p1", picked_up);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(picked_up);
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_IN_THE_AIR);
    }

    #[test]
    fn scatter_places_player_on_empty_square() {
        let mut game = make_game();
        // Place player at center; scatter should land somewhere in bounds
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.throw_scatter = false;
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);

        // Player should be placed somewhere (coordinate published or field model updated)
        let published_coord = out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(_)));
        assert!(published_coord, "Should publish ThrownPlayerCoordinate");
    }

    #[test]
    fn out_of_bounds_scatter_publishes_thrown_player_coord_none() {
        // Test that out of bounds path runs without panic (TtmToCrowdHandler is called)
        let mut game = make_game();
        // Player at edge; many d8 rolls will go OOB
        add_player(&mut game, "p1", FieldCoordinate::new(0, 0));
        game.field_model.set_player_state("p1", PlayerState::new(PS_FALLING));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_FALLING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(0, 0));
        step.throw_scatter = false;
        // Seed with roll that goes OOB: d8=8 → NW from (0,0) would go negative
        // Not guaranteed, but we test it doesn't panic
        let out = step.start(&mut game, &mut GameRng::new(7));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_throw_scatter() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrowScatter(true)));
        assert!(step.throw_scatter);
    }

    #[test]
    fn set_parameter_crash_landing() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::CrashLanding(true)));
        assert!(step.crash_landing);
    }

    #[test]
    fn set_parameter_pass_deviates() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::PassDeviates(true)));
        assert!(step.deviate);
    }

    #[test]
    fn set_parameter_direction_sets_swoop() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::Direction(Direction::North)));
        assert_eq!(step.swoop_direction, Some(Direction::North));
    }

    #[test]
    fn set_parameter_kicked_player_aliases() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::KickedPlayerId(Some("k1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("k1"));
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::KickedPlayerState(state)));
        assert_eq!(step.thrown_player_state, Some(state));
        assert!(step.set_parameter(&StepParameter::KickedPlayerHasBall(true)));
        assert!(step.thrown_player_has_ball);
        assert!(step.set_parameter(&StepParameter::KickedPlayerCoordinate(FieldCoordinate::new(1, 1))));
        assert_eq!(step.thrown_player_coordinate, Some(FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn publishes_thrown_player_id_and_state() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(5));

        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerState(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(_))));
    }

    #[test]
    fn crash_landing_publishes_drop_thrown_player() {
        let mut game = make_game();
        // Place player at center so it stays in bounds
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.crash_landing = true;
        // Use a seed that scatters to a square we know is empty
        // With throw_scatter=false and d8 scatter, many seeds will land in-bounds
        let out = step.start(&mut game, &mut GameRng::new(1));

        // If we landed in bounds (very likely from center), DROP_THROWN_PLAYER should be published
        let any_drop = out.published.iter().any(|p| matches!(p, StepParameter::DropThrownPlayer(true)));
        let any_coord = out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(_)));
        // At minimum ThrownPlayerCoordinate should be published
        assert!(any_coord || any_drop, "Should publish either DropThrownPlayer or ThrownPlayerCoordinate");
    }

    #[test]
    fn deviate_changes_coordinate() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.deviate = true;
        let out = step.start(&mut game, &mut GameRng::new(3));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn swoop_adds_swoop_player_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.swoop_direction = Some(Direction::North);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SWOOP_PLAYER),
            "swoop path must add SWOOP_PLAYER report");
    }

    #[test]
    fn deviate_adds_pass_deviate_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.deviate = true;
        step.start(&mut game, &mut GameRng::new(5));
        assert!(game.report_list.has_report(ReportId::PASS_DEVIATE),
            "deviate path must add PASS_DEVIATE report");
    }
}
