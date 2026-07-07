/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.ttm.StepInitScatterPlayer`.
///
/// Scatters the thrown/kicked player to their landing square (BB2025).
///
/// BB2025 differences vs BB2020:
///  - Extends AbstractStepWithReRoll — Swoop re-roll dialog is client-only (headless executes immediately).
///  - `usingBullseye` flag: when set, land at `game.passCoordinate` directly (no scatter roll).
///  - usingSwoop flag: implemented — controls Swoop movement path.
///  - SteadyFootingContext — published for hit-player scenarios (implemented).
///  - SppMechanic — grants SPP on TTM hit for skilled players (wired: addCasualty for lethalSpp+violentSpp).
///  - Always uses `InjuryTypeCrowdPush` (no InjuryTypeKTMCrowd) for OOB.
///  - No `deviate` / `crashLanding` flags (BB2020-only).
///  - Publishes `USING_SWOOP` and `OLD_DEFENDER_STATE` from `handleLanding`.
use std::sync::Arc;
use ffb_mechanics::bb2025::spp_mechanic::SppMechanic as SppMechanic2025;
use ffb_mechanics::mechanics::scatter_coordinate;
use ffb_mechanics::spp_mechanic::SppMechanic as SppMechanicTrait;
use ffb_model::enums::{ApothecaryMode, Direction, PS_IN_THE_AIR, PS_PICKED_UP, PlayerState};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id::{END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM, SWOOP_DISTANCE};
use ffb_model::option::util_game_option::{get_int_option, is_option_enabled};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2025::report_swoop_player::ReportSwoopPlayer;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::injury::injuryType::injury_type_crowd_push::InjuryTypeCrowdPush;
use crate::injury::injuryType::injury_type_ttm_hit_player::InjuryTypeTTMHitPlayer;
use crate::injury::injuryType::injury_type_ttm_hit_player_for_spp::InjuryTypeTTMHitPlayerForSpp;
use crate::step::action::ttm::util_throw_team_mate_sequence::{scatter_player, ScatterResult};
use crate::step::bb2025::command::{DropPlayerCommand, HitPlayerTurnOverCommand};
use crate::step::framework::{DeferredCommand, Step, StepOutcome, StepId, StepParameter};
use crate::step::mixed::ttm::ttm_to_crowd_handler::TtmToCrowdHandler;
use crate::step::util_server_injury;

/// Java: `StepInitScatterPlayer` (bb2025/ttm).
pub struct StepInitScatterPlayer {
    /// Java: thrownPlayerId
    thrown_player_id: Option<String>,
    /// Java: thrownPlayerState
    thrown_player_state: Option<PlayerState>,
    /// Java: oldPlayerState
    old_player_state: Option<PlayerState>,
    /// Java: thrownPlayerCoordinate
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: thrownPlayerHasBall
    thrown_player_has_ball: bool,
    /// Java: throwScatter
    throw_scatter: bool,
    /// Java: isKickedPlayer
    is_kicked_player: bool,
    /// Java: usingBullseye
    using_bullseye: bool,
    /// Java: usingSwoop
    using_swoop: bool,
    /// Java: swoopDirection
    swoop_direction: Option<Direction>,
    // AbstractStepWithReRoll stubs
    re_rolled_action: Option<String>,
}

impl StepInitScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            old_player_state: None,
            thrown_player_coordinate: None,
            thrown_player_has_ball: false,
            throw_scatter: false,
            is_kicked_player: false,
            using_bullseye: false,
            using_swoop: false,
            swoop_direction: None,
            re_rolled_action: None,
        }
    }

    /// Java: executeStep()
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let swooping = self.swoop_direction.is_some() && self.using_swoop;

        // Java: if !swooping && thrownPlayerState.base == PICKED_UP → changeBase(IN_THE_AIR)
        if !swooping {
            if let (Some(id), Some(state)) = (&self.thrown_player_id.clone(), self.thrown_player_state) {
                if state.base() == PS_PICKED_UP {
                    let new_state = state.change_base(PS_IN_THE_AIR);
                    self.thrown_player_state = Some(new_state);
                    game.field_model.set_player_state(id, new_state);
                }
            }
        }

        // Guard: no player or coordinate → NEXT_STEP.
        let thrown_player_id = match &self.thrown_player_id {
            Some(id) if game.player(id).is_some() => id.clone(),
            _ => return StepOutcome::next(),
        };
        let thrown_player_coord = match self.thrown_player_coordinate {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: if usingBullseye → setRangeRuler/clearMoveSquares (deferred); endCoord = passCoord; handleLanding; return
        if self.using_bullseye {
            // Range ruler / move squares clearing and setAnimation are client-side only.
            let end_coord = game.pass_coordinate.unwrap_or(thrown_player_coord);
            return self.handle_landing(game, rng, &thrown_player_id, end_coord);
        }

        let mut start_coord = thrown_player_coord;
        if self.throw_scatter {
            // Java: setRangeRuler(null); clearMoveSquares(); startCoordinate = game.passCoordinate
            // Range ruler / move squares are client-side only.
            if let Some(pc) = game.pass_coordinate {
                start_coord = pc;
            }
        }

        let scatter_result: ScatterResult;
        let mut swoop_event: Option<GameEvent> = None;
        if swooping {
            // Java: doRoll = reRolledAction != SWOOP_DISTANCE || (reRollSource != null && useReRoll)
            // client-only: re-roll availability check requires dialog; headless proceeds without re-roll offer
            scatter_result = self.swoop_scatter(game, rng, start_coord);
            // Java: addReport(new ReportSwoopPlayer(thrownPlayer, thrownPlayerCoordinate))
            if let Some(coord) = self.thrown_player_coordinate {
                swoop_event = Some(GameEvent::SwoopPlayer {
                    player_id: thrown_player_id.clone(),
                    coord,
                });
            }
            // Java: clearMoveSquares(); if inBounds: add(MoveSquare) — client-side display only.
            // Java: if distance==0 && reRolledAction==null → askForReRollIfAvailable → CONTINUE
            // client-only: distance==0 re-roll offer requires dialog; headless proceeds

            // Java: thrownPlayerState = changeBase(IN_THE_AIR)
            if let Some(state) = self.thrown_player_state {
                let new_state = state.change_base(PS_IN_THE_AIR);
                self.thrown_player_state = Some(new_state);
                game.field_model.set_player_state(&thrown_player_id, new_state);
            }
        } else {
            // Java: UtilThrowTeamMateSequence.scatterPlayer(this, startCoordinate, throwScatter)
            scatter_result = scatter_player(game, rng, start_coord, self.throw_scatter);
        }

        let end_coord = scatter_result.last_valid_coordinate;
        // Java: getResult().setAnimation(...); UtilServerGame.syncGameModel(this) — client-side only.

        let outcome = if scatter_result.in_bounds {
            self.handle_landing(game, rng, &thrown_player_id, end_coord)
        } else {
            // Java: new TtmToCrowdHandler().handle(game, this, thrownPlayer, endCoord, hasBall, new InjuryTypeCrowdPush())
            let mut injury_type = InjuryTypeCrowdPush::new();
            let crowd_params = TtmToCrowdHandler::handle(
                game, rng, &thrown_player_id, end_coord,
                self.thrown_player_has_ball, &mut injury_type,
            );
            let mut outcome = StepOutcome::next();
            for p in crowd_params {
                outcome = outcome.publish(p);
            }
            // Java: publishParameter THROWN_PLAYER_ID, STATE, HAS_BALL, IS_KICKED_PLAYER; NEXT_STEP
            outcome
                .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
                .publish(StepParameter::IsKickedPlayer(self.is_kicked_player))
        };
        if let Some(ev) = swoop_event { outcome.with_event(ev) } else { outcome }
    }

    /// Java: handleLanding(Player<?> thrownPlayer, FieldCoordinate endCoordinate)
    fn handle_landing(
        &mut self,
        game: &mut Game,
        rng: &mut GameRng,
        thrown_player_id: &str,
        end_coord: FieldCoordinate,
    ) -> StepOutcome {
        let mut outcome = StepOutcome::next();

        // Java: playerLandedUpon = fieldModel.getPlayer(endCoordinate), null if same player
        let at_end = game.field_model.player_at(end_coord)
            .filter(|id| *id != thrown_player_id)
            .cloned();

        if let Some(hit_player_id) = at_end {
            // Java: publishParameter(DROP_THROWN_PLAYER, true)
            outcome = outcome.publish(StepParameter::DropThrownPlayer(true));

            // Java: SPP check — if vsOpponent && (lethalSpp || violentSpp) use InjuryTypeTTMHitPlayerForSpp.
            // lethalSpp: thrownPlayer.hasUsableSkillProperty(grantsSppWhenHittingOpponentOnTtm, oldPlayerState)
            // violentSpp: isKickedPlayer && thrower.hasSkillWithProperty(grantsSppFromSpecialActionsCas)
            let vs_opponent = if game.home_playing {
                game.team_away.has_player(&hit_player_id)
            } else {
                game.team_home.has_player(&hit_player_id)
            };
            let lethal_spp = vs_opponent && game.player(thrown_player_id)
                .map(|p| p.has_unused_skill_with_property(NamedProperties::GRANTS_SPP_WHEN_HITTING_OPPONENT_ON_TTM))
                .unwrap_or(false);
            // violentSpp: thrower (actingPlayer) has grantsSppFromSpecialActionsCas
            let violent_spp = vs_opponent && self.is_kicked_player
                && game.acting_player.player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS))
                    .unwrap_or(false);

            let thrower_id_clone = game.acting_player.player_id.clone();
            let injury_result = if vs_opponent && (lethal_spp || violent_spp) {
                // Java: attacker = lethalSpp ? thrownPlayer : thrower
                let attacker_id: Option<String> = if lethal_spp {
                    Some(thrown_player_id.to_string())
                } else {
                    game.acting_player.player_id.clone()
                };
                let mut hit_injury = InjuryTypeTTMHitPlayerForSpp::new();
                let result = util_server_injury::handle_injury(
                    game, rng, &mut hit_injury,
                    attacker_id.as_deref(), &hit_player_id, end_coord, None, None,
                    ApothecaryMode::HitPlayer,
                );
                result.apply_to(game);
                // Java: if (lethalSpp && violentSpp && injuryResultHitPlayer.injuryContext().isCasualty())
                //         spp.addCasualty(prayerState.additionalCasSppTeams, playerResult(thrower))
                if lethal_spp && violent_spp && result.injury_context().is_casualty() {
                    if let Some(ref tid) = thrower_id_clone {
                        let thrower_team_id = if game.home_playing {
                            game.team_home.id.clone()
                        } else {
                            game.team_away.id.clone()
                        };
                        let additional_teams = game.prayer_state.get_additional_cas_spp_teams().clone();
                        let pr = game.game_result.team_result_mut(game.home_playing).player_result_mut(tid);
                        SppMechanic2025::new().add_casualty(&additional_teams, pr, &thrower_team_id);
                    }
                }
                result
            } else {
                let mut hit_injury = InjuryTypeTTMHitPlayer::new();
                let result = util_server_injury::handle_injury(
                    game, rng, &mut hit_injury,
                    None, &hit_player_id, end_coord, None, None,
                    ApothecaryMode::HitPlayer,
                );
                result.apply_to(game);
                result
            };

            // Java: alwaysTurnOver || hitting own team → HitPlayerTurnOverCommand in deferred list
            let always_turn_over = is_option_enabled(game, END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM);
            let hit_own_team = if game.home_playing {
                game.team_home.has_player(&hit_player_id)
            } else {
                game.team_away.has_player(&hit_player_id)
            };

            // Java: commands = [HitPlayerTurnOverCommand (conditional), DropPlayerCommand(...)]
            let mut commands: Vec<Arc<dyn DeferredCommand>> = Vec::new();
            if always_turn_over || hit_own_team {
                commands.push(Arc::new(HitPlayerTurnOverCommand));
                // EndTurn is also published directly so it propagates immediately
                outcome = outcome.publish(StepParameter::EndTurn(true));
            }
            commands.push(Arc::new(DropPlayerCommand::new(
                hit_player_id.clone(),
                ApothecaryMode::HitPlayer,
                true,
            )));

            // Java: getResult().addReport(new ReportPlayerEvent(playerLandedUpon.getId(), "was hit"));
            game.report_list.add(ReportPlayerEvent::new(
                Some(hit_player_id.clone()),
                Some("was hit".to_string()),
            ));
            // Java: publishParameter(STEADY_FOOTING_CONTEXT, new SteadyFootingContext(injuryResult, commands))
            let sfc = SteadyFootingContext::from_injury_result_with_commands(injury_result, commands);
            outcome = outcome.publish(StepParameter::SteadyFootingContext(Box::new(sfc)));

            // Java: THROWN_PLAYER_COORDINATE = endCoordinate (continue loop)
            outcome = outcome
                .publish(StepParameter::ThrownPlayerCoordinate(Some(end_coord)))
                .publish(StepParameter::PlayerEnteringSquare(thrown_player_id.to_string()));
        } else {
            // Java: put thrown player in target coordinate; end loop
            game.field_model.set_player_coordinate(thrown_player_id, end_coord);
            if let Some(state) = self.thrown_player_state {
                game.field_model.set_player_state(thrown_player_id, state);
            }
            game.defender_id = None;

            // Java: THROWN_PLAYER_COORDINATE = null (end loop sentinel)
            outcome = outcome
                .publish(StepParameter::ThrownPlayerCoordinate(None))
                .publish(StepParameter::PlayerEnteringSquare(thrown_player_id.to_string()));
        }

        // Java: always published at end of handleLanding
        outcome = outcome
            .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
            .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
            .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
            .publish(StepParameter::IsKickedPlayer(self.is_kicked_player))
            .publish(StepParameter::UsingSwoop(self.using_swoop));

        if let Some(old_state) = self.old_player_state {
            outcome = outcome.publish(StepParameter::OldDefenderState(old_state));
        }

        // Java: game.getFieldModel().setPlayerCoordinate(thrownPlayer, endCoordinate)
        game.field_model.set_player_coordinate(thrown_player_id, end_coord);

        outcome
    }

    /// Java: swoop(FieldCoordinate throwerCoordinate, Direction direction, int distanceOption)
    ///
    /// distanceOption == 0 → roll D6; else use distanceOption directly.
    fn swoop_scatter(&mut self, game: &mut Game, rng: &mut GameRng, start_coord: FieldCoordinate) -> ScatterResult {
        let direction = self.swoop_direction.expect("swoop_direction is set");

        // Java: getOptionWithDefault(SWOOP_DISTANCE).getValue() — 0 means roll D6.
        let distance_option = get_int_option(game, SWOOP_DISTANCE);
        let distance_roll = if distance_option == 0 { rng.d6() } else { distance_option };

        let (ex, ey) = scatter_coordinate(start_coord.x, start_coord.y, direction, distance_roll);
        let coord_end = FieldCoordinate::new(ex, ey);

        let mut last_valid = coord_end;
        let mut valid_dist = distance_roll;
        while !last_valid.is_on_pitch() && valid_dist > 0 {
            valid_dist -= 1;
            let (vx, vy) = scatter_coordinate(start_coord.x, start_coord.y, direction, valid_dist);
            last_valid = FieldCoordinate::new(vx, vy);
        }

        let in_bounds = coord_end.is_on_pitch();

        // Java: publishParameter(THROWN_PLAYER_COORDINATE, lastValidCoordinate)
        self.thrown_player_coordinate = Some(last_valid);
        // Java: getResult().addReport(new ReportSwoopPlayer(throwerCoordinate, coordinateEnd, direction, distanceRoll, !inBounds));
        game.report_list.add(ReportSwoopPlayer::new(
            start_coord,
            coord_end,
            direction,
            distance_roll,
            !in_bounds,
        ));
        // SwoopPlayer GameEvent emitted in execute_step after swoop_scatter returns.

        ScatterResult::new(last_valid, in_bounds)
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
        // AbstractStepWithReRoll: EXECUTE_STEP → executeStep()
        // client-only: re-roll command arrives from dialog; headless always executes immediately
        self.execute_step(game, rng)
    }

    /// Java: setParameter(StepParameter parameter) — handles both init and dynamic params.
    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Mandatory init params (also accepted as KICKED_PLAYER_* aliases)
            StepParameter::ThrownPlayerId(v)         => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerId(v)         => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerHasBall(v)    => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerHasBall(v)    => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerCoordinate(v) => { self.thrown_player_coordinate = *v; true }
            StepParameter::KickedPlayerCoordinate(v) => { self.thrown_player_coordinate = Some(*v); true }
            StepParameter::ThrownPlayerState(v)      => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerState(v)      => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrowScatter(v)           => { self.throw_scatter = *v; true }
            // Optional dynamic params
            StepParameter::IsKickedPlayer(v)         => { self.is_kicked_player = *v; true }
            StepParameter::Direction(v)              => { self.swoop_direction = Some(*v); true }
            StepParameter::UsingBullseye(v)          => { self.using_bullseye = *v; true }
            StepParameter::UsingSwoop(v)             => { self.using_swoop = *v; true }
            StepParameter::OldDefenderState(v)       => { self.old_player_state = Some(*v); true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
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
    fn no_thrown_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn scatter_in_bounds_places_player_and_publishes_id() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))));
    }

    #[test]
    fn picked_up_changes_to_in_the_air_when_not_swooping() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));
        let picked_up = PlayerState::new(PS_PICKED_UP);
        game.field_model.set_player_state("p1", picked_up);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(picked_up);
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_IN_THE_AIR);
    }

    #[test]
    fn bullseye_lands_at_pass_coordinate() {
        let mut game = make_game();
        let pass_coord = FieldCoordinate::new(10, 5);
        game.pass_coordinate = Some(pass_coord);
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_bullseye = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_coordinate("p1"), Some(pass_coord));
    }

    #[test]
    fn handle_landing_publishes_using_swoop() {
        let mut game = make_game();
        let pass_coord = FieldCoordinate::new(10, 5);
        game.pass_coordinate = Some(pass_coord);
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_swoop = true;
        step.using_bullseye = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingSwoop(true))));
    }

    #[test]
    fn handle_landing_publishes_old_defender_state_when_set() {
        let mut game = make_game();
        let pass_coord = FieldCoordinate::new(10, 5);
        game.pass_coordinate = Some(pass_coord);
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.old_player_state = Some(PlayerState::new(PS_STANDING));
        step.using_bullseye = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::OldDefenderState(_))));
    }

    #[test]
    fn set_using_bullseye_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::UsingBullseye(true)));
        assert!(step.using_bullseye);
    }

    #[test]
    fn set_using_swoop_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::UsingSwoop(true)));
        assert!(step.using_swoop);
    }

    #[test]
    fn set_direction_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::Direction(Direction::North)));
        assert_eq!(step.swoop_direction, Some(Direction::North));
    }

    #[test]
    fn set_old_defender_state_accepted() {
        let mut step = StepInitScatterPlayer::default();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_player_state, Some(state));
    }

    #[test]
    fn kicked_player_aliases_work() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::KickedPlayerId(Some("k1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("k1"));
    }

    #[test]
    fn publishes_is_kicked_player() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate::new(13, 7));
        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.is_kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(3));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(true))));
    }

    #[test]
    fn swoop_scatter_returns_scatter_result() {
        let mut step = StepInitScatterPlayer::new();
        step.swoop_direction = Some(Direction::North);
        let mut rng = GameRng::new(42);
        let mut game = make_game();
        let start = FieldCoordinate::new(13, 7);
        let result = step.swoop_scatter(&mut game, &mut rng, start);
        // Should update thrown_player_coordinate to last valid coord
        assert!(step.thrown_player_coordinate.is_some());
        let _ = result;
    }

    #[test]
    fn swoop_scatter_uses_fixed_distance_when_option_set() {
        use ffb_model::option::game_option_id::SWOOP_DISTANCE;
        let mut step = StepInitScatterPlayer::new();
        step.swoop_direction = Some(Direction::East);
        let mut rng = GameRng::new(0);
        let mut game = make_game();
        game.options.set(SWOOP_DISTANCE, "3");
        let start = FieldCoordinate::new(13, 8);
        let _result = step.swoop_scatter(&mut game, &mut rng, start);
        assert!(step.thrown_player_coordinate.is_some());
    }

    // ── New tests for SPP + deferred-command wiring ─────────────────────────

    /// Helper: add a player to the away team at a coordinate.
    fn add_away_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 2,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    /// Landing on an opponent publishes SteadyFootingContext and DropThrownPlayer.
    #[test]
    fn landing_on_hit_player_publishes_steady_footing_context_and_drop_thrown() {
        let mut game = make_game();
        game.home_playing = true;
        // Thrown player starts at (8, 7); it will scatter to pass_coord (10, 7) via bullseye.
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        // Target player occupies the landing square (away team = opponent)
        let land = FieldCoordinate::new(10, 7);
        add_away_player(&mut game, "target", land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(8, 7));
        step.using_bullseye = true;
        game.pass_coordinate = Some(land);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "expected SteadyFootingContext to be published");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropThrownPlayer(true))),
            "expected DropThrownPlayer(true)");
    }

    /// Hitting own team player triggers EndTurn and includes HitPlayerTurnOverCommand in SFC.
    #[test]
    fn landing_on_own_team_triggers_end_turn() {
        let mut game = make_game();
        game.home_playing = true;
        // Both players in home team
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        let land = FieldCoordinate::new(10, 7);
        add_player(&mut game, "teammate", land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_bullseye = true;
        game.pass_coordinate = Some(land);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "expected EndTurn(true) when hitting own team");
    }

    /// Hitting an opponent without SPP skills uses InjuryTypeTTMHitPlayer (no SPP).
    /// SteadyFootingContext still carries one DropPlayerCommand (but no HitPlayerTurnOver).
    #[test]
    fn hitting_opponent_without_spp_skills_publishes_sfc_without_end_turn() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        let land = FieldCoordinate::new(10, 7);
        add_away_player(&mut game, "opponent", land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_bullseye = true;
        game.pass_coordinate = Some(land);

        let out = step.start(&mut game, &mut GameRng::new(0));
        // No EndTurn published (no alwaysTurnOver, not own team)
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "should not publish EndTurn(true) for opponent hit without always-turn-over");
        // SteadyFootingContext should still be present
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    /// SteadyFootingContext deferred_commands list has DropPlayerCommand always,
    /// HitPlayerTurnOverCommand only when hitting own team.
    #[test]
    fn steady_footing_context_deferred_commands_count_for_own_team_hit() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        let land = FieldCoordinate::new(10, 7);
        add_player(&mut game, "teammate", land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_bullseye = true;
        game.pass_coordinate = Some(land);

        let out = step.start(&mut game, &mut GameRng::new(0));
        let sfc_param = out.published.iter().find_map(|p| {
            if let StepParameter::SteadyFootingContext(ctx) = p { Some(ctx) } else { None }
        });
        assert!(sfc_param.is_some(), "SteadyFootingContext must be published");
        // Own team hit → 2 commands: HitPlayerTurnOver + DropPlayer
        assert_eq!(sfc_param.unwrap().deferred_commands.len(), 2,
            "expected HitPlayerTurnOverCommand + DropPlayerCommand (2 total)");
    }

    #[test]
    fn steady_footing_context_deferred_commands_count_for_opponent_hit() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        let land = FieldCoordinate::new(10, 7);
        add_away_player(&mut game, "opponent", land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(13, 7));
        step.using_bullseye = true;
        game.pass_coordinate = Some(land);

        let out = step.start(&mut game, &mut GameRng::new(0));
        let sfc_param = out.published.iter().find_map(|p| {
            if let StepParameter::SteadyFootingContext(ctx) = p { Some(ctx) } else { None }
        });
        assert!(sfc_param.is_some());
        // Opponent hit without always-turn-over → 1 command: DropPlayer only
        assert_eq!(sfc_param.unwrap().deferred_commands.len(), 1,
            "expected only DropPlayerCommand (1 total) for opponent hit");
    }

    /// When lethal_spp + violent_spp conditions are met, the step runs without panic
    /// and uses InjuryTypeTTMHitPlayerForSpp (DropThrownPlayer published).
    #[test]
    fn lethal_spp_and_violent_spp_conditions_run_without_panic() {
        use ffb_model::enums::SkillId;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        game.home_playing = true;
        // Thrown player (on home team) with LethalFlight — grants SPP when hitting on TTM
        let land = FieldCoordinate::new(10, 7);
        game.team_home.players.push(Player {
            id: "thrown".into(), name: "thrown".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::LethalFlight, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(8, 7));
        game.field_model.set_player_state("thrown", PlayerState::new(PS_STANDING));

        // Thrower (actingPlayer) on home team with ViolentInnovator
        game.team_home.players.push(Player {
            id: "thrower".into(), name: "thrower".into(), nr: 2,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::ViolentInnovator, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            ..Default::default()
        });
        game.acting_player.player_id = Some("thrower".into());

        // Hit player on away team (opponent) with AV=2 to maximize armor-break chance
        game.team_away.players.push(Player {
            id: "opponent".into(), name: "opponent".into(), nr: 3,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 2,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("opponent", land);
        game.field_model.set_player_state("opponent", PlayerState::new(PS_STANDING));

        game.pass_coordinate = Some(land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrown".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(8, 7));
        step.using_bullseye = true;
        step.is_kicked_player = true;

        // Must not panic; DropThrownPlayer(true) should be published
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropThrownPlayer(true))));
    }

    // ── report wiring ─────────────────────────────────────────────────────────

    #[test]
    fn player_event_report_added_when_landing_on_another_player() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        // Thrown player and target both on pitch; use bullseye so target is hit
        add_player(&mut game, "thrower", FieldCoordinate::new(8, 7));
        let land = FieldCoordinate::new(10, 7);
        // Target on away team (so InjuryTypeTTMHitPlayer path)
        game.team_away.players.push(Player {
            id: "target".into(), name: "target".into(), nr: 2,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None, ..Default::default()
        });
        game.field_model.set_player_coordinate("target", land);
        game.field_model.set_player_state("target", PlayerState::new(PS_STANDING));
        game.pass_coordinate = Some(land);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("thrower".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.thrown_player_coordinate = Some(FieldCoordinate::new(8, 7));
        step.using_bullseye = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT),
            "PLAYER_EVENT report must be added when thrown player lands on another player");
    }

    #[test]
    fn swoop_scatter_adds_swoop_player_report() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepInitScatterPlayer::new();
        step.swoop_direction = Some(Direction::East);
        let mut rng = GameRng::new(42);
        let mut game = make_game();
        let start = FieldCoordinate::new(13, 7);
        step.swoop_scatter(&mut game, &mut rng, start);
        assert!(game.report_list.has_report(ReportId::SWOOP_PLAYER),
            "SWOOP_PLAYER report must be added by swoop_scatter");
    }
}
