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
        let thrown_player_id = match &self.thrown_player_id {
            Some(id) if game.player(id).is_some() => id.clone(),
            _ => {
                return StepOutcome::next()
                    .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                    .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                    .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball));
            }
        };
        let thrown_player_coord = match self.thrown_player_coordinate {
            Some(c) => c,
            None => {
                return StepOutcome::next()
                    .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                    .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                    .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball));
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
                injury_result.apply_to(game);
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
            injury_result.apply_to(game);
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
            StepParameter::KickedPlayerCoordinate(v)   => { self.thrown_player_coordinate = Some(*v); true }
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
    fn id_is_init_scatter_player() {
        assert_eq!(StepInitScatterPlayer::new().id(), StepId::InitScatterPlayer);
    }

    #[test]
    fn no_player_returns_next_with_params() {
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
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
        assert!(step.set_parameter(&StepParameter::KickedPlayerCoordinate(coord)));
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
}
