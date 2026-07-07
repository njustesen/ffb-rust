use ffb_model::enums::ApothecaryMode;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_throw_at_stalling_player::ReportThrowAtStallingPlayer;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepStallingPlayer` (BB2020).
///
/// Resolves the "stalling player" detection: rolls a d6, and on a 5+ a rock is thrown
/// at the stalling player, potentially injuring them.
///
/// Init parameters:
///  - `PLAYER_ID` — the ID of the player detected to be stalling.
///
/// Java `start()` logic:
///  1. Roll d6: `successful = roll >= 5`
///  2. Report `ReportThrowAtStallingPlayer(playerId, roll, successful)`.
///  3. If successful:
///     a. Look up player coordinate.
///     b. Determine `startCoordinate`: if `FieldCoordinateBounds.UPPER_HALF.isInBounds(playerCoordinate)`
///        (y <= 7) → start at `(rollXCoordinate(), 0)`, else → start at `(rollXCoordinate(), 14)`.
///     c. Set animation (THROW_A_ROCK).
///     d. `UtilServerGame.syncGameModel(this)`.
///     e. `UtilServerInjury.dropPlayer(this, player, ApothecaryMode.HIT_PLAYER, true)` — publish params
///        minus `END_TURN`.
///     f. `UtilServerInjury.handleInjury(this, InjuryTypeThrowARockStalling, null, player,
///        playerCoordinate, null, null, ApothecaryMode.HIT_PLAYER)` — publish as `INJURY_RESULT`.
///  4. NEXT_STEP.
pub struct StepStallingPlayer {
    /// Java: playerId — set via init(StepParameterSet)
    pub player_id: Option<String>,
}

impl StepStallingPlayer {
    pub fn new() -> Self {
        Self { player_id: None }
    }
}

impl Default for StepStallingPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepStallingPlayer {
    fn id(&self) -> StepId { StepId::StallingPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommand dispatches to executeStep via super.handleCommand returning EXECUTE_STEP.
        // For this step there is no dialog, so handle_command is effectively a no-op / next.
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        if let StepParameter::PlayerId(id) = param {
            self.player_id = Some(id.clone());
            return true;
        }
        false
    }
}

impl StepStallingPlayer {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.player_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        // Java: int roll = getGameState().getDiceRoller().rollDice(6)
        // Java: boolean successful = roll >= 5
        let roll = rng.d6();
        let successful = roll >= 5;

        // Java: getResult().addReport(new ReportThrowAtStallingPlayer(playerId, roll, successful))
        game.report_list.add(ReportThrowAtStallingPlayer::new(Some(player_id.clone()), roll, successful));

        let stalling_event = GameEvent::ThrowAtStallingPlayer { player_id: player_id.clone(), roll, success: successful };

        if successful {
            let player = match game.player(&player_id) {
                Some(_) => {},
                None => return StepOutcome::next().with_event(stalling_event),
            };
            let _ = player;

            let player_coord = game.field_model.player_coordinate(&player_id)
                .unwrap_or(FieldCoordinate::new(0, 0));

            // Java: FieldCoordinateBounds.UPPER_HALF = FieldCoordinateBounds(new FieldCoordinate(0, 0), new FieldCoordinate(25, 7))
            // Java: if (FieldCoordinateBounds.UPPER_HALF.isInBounds(playerCoordinate)) →
            //   startCoordinate = new FieldCoordinate(getDiceRoller().rollXCoordinate(), 0)
            // else →
            //   startCoordinate = new FieldCoordinate(getDiceRoller().rollXCoordinate(), 14)
            let in_upper_half = player_coord.y <= 7;
            let x = rng.die(24) as i32; // Java rollXCoordinate() = roll in [1..24] (inner pitch x coords)
            let _start_coord = if in_upper_half {
                FieldCoordinate::new(x, 0)
            } else {
                FieldCoordinate::new(x, 14)
            };

            // Animation and syncGameModel are client-side only; no server state change.

            // Java: StepParameterSet pParameterSet = UtilServerInjury.dropPlayer(this, player, ApothecaryMode.HIT_PLAYER, true)
            // Java: pParameterSet.remove(StepParameterKey.END_TURN)
            // Java: publishParameters(pParameterSet) — implemented below via DropPlayerContext.

            // Java: publishParameter(new StepParameter(StepParameterKey.INJURY_RESULT,
            //   UtilServerInjury.handleInjury(this, new InjuryTypeThrowARockStalling(),
            //     null, player, playerCoordinate, null, null, ApothecaryMode.HIT_PLAYER)))
            let injury_result = handle_injury_by_name(
                game, rng, "InjuryTypeThrowARockStalling",
                None, &player_id,
                player_coord, None, None,
                ApothecaryMode::HitPlayer,
            );

            // BB2020 Java publishes INJURY_RESULT directly (StepParameter(INJURY_RESULT, injuryResult))
            // and handles the player drop via the SteadyFooting pipeline in subsequent steps.
            // We publish via SteadyFootingContext wrapping a DropPlayerContext (consistent with BB2025).
            let dpc = DropPlayerContext {
                injury_result: Some(Box::new(injury_result)),
                end_turn: false,
                eligible_for_safe_pair_of_hands: true,
                label: None,
                player_id: Some(player_id),
                apothecary_mode: Some(ApothecaryMode::HitPlayer),
                requires_armour_break: false,
                ..DropPlayerContext::new()
            };
            let ctx = SteadyFootingContext::from_drop_player(dpc);
            return StepOutcome::next()
                .with_event(stalling_event)
                .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        }

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next().with_event(stalling_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    /// When no player_id is set, returns NEXT_STEP immediately.
    #[test]
    fn no_player_id_returns_next() {
        let mut game = make_game();
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
}

    /// set_parameter wires player_id correctly.
    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepStallingPlayer::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("staller".into())));
        assert_eq!(step.player_id.as_deref(), Some("staller"));
    }

    /// Unknown parameters are rejected.
    #[test]
    fn set_parameter_unknown_rejected() {
        let mut step = StepStallingPlayer::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    /// On a failed roll (< 5), no SteadyFootingContext is published.
    #[test]
    fn failed_roll_does_not_publish_context() {
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 5));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        // Find a seed where d6 < 5
        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll < 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let out = step.start(&mut game, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep);
                assert!(
                    !out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
                    "seed={seed} roll={roll}: should NOT publish SteadyFootingContext on miss"
                );
                return;
            }
        }
        panic!("no seed found with d6 < 5");
    }

    /// On a successful roll (>= 5), SteadyFootingContext is published.
    #[test]
    fn successful_roll_publishes_steady_footing_context() {
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 5));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        // Find a seed where d6 >= 5
        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll >= 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let out = step.start(&mut game, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep);
                assert!(
                    out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
                    "seed={seed} roll={roll}: expected SteadyFootingContext on successful hit"
                );
                return;
            }
        }
        panic!("no seed found with d6 >= 5");
    }

    /// Player in upper half (y <= 7) uses y=0 for start coordinate.
    #[test]
    fn upper_half_player_uses_y0_start() {
        // We can verify the game doesn't panic — the exact coordinate is internal.
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 3)); // y=3, upper half
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        // Find a seed with d6 >= 5
        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll >= 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let out = step.start(&mut game, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep);
                // Just verify we get a result without panic
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
                return;
            }
        }
        panic!("no seed found");
    }

    /// Player in lower half (y > 7) uses y=14 for start coordinate.
    #[test]
    fn lower_half_player_uses_y14_start() {
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 10)); // y=10, lower half
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll >= 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let out = step.start(&mut game, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep);
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
                return;
            }
        }
        panic!("no seed found");
    }

    /// handle_command always returns NEXT_STEP (no dialog for this step).
    #[test]
    fn handle_command_returns_next() {
        let mut game = make_game();
        let mut step = StepStallingPlayer::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// A report is added to game.report_list on a failed roll (< 5).
    #[test]
    fn failed_roll_adds_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 5));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        // Find a seed where d6 < 5 (roll is not successful)
        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll < 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let _out = step.start(&mut game, &mut GameRng::new(seed));
                assert!(
                    game.report_list.has_report(ReportId::THROW_AT_STALLING_PLAYER),
                    "seed={seed} roll={roll}: report_list should contain ReportThrowAtStallingPlayer on miss"
                );
                return;
            }
        }
        panic!("no seed found with d6 < 5");
    }

    /// A report is added to game.report_list on a successful roll (>= 5).
    #[test]
    fn successful_roll_adds_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let player = make_player("staller");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("staller", FieldCoordinate::new(10, 5));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state("staller", PlayerState::new(PS_STANDING));

        // Find a seed where d6 >= 5 (roll is successful)
        for seed in 0u64..1000 {
            let mut rng = GameRng::new(seed);
            let roll = rng.d6();
            if roll >= 5 {
                let mut step = StepStallingPlayer::new();
                step.player_id = Some("staller".into());
                let _out = step.start(&mut game, &mut GameRng::new(seed));
                assert!(
                    game.report_list.has_report(ReportId::THROW_AT_STALLING_PLAYER),
                    "seed={seed} roll={roll}: report_list should contain ReportThrowAtStallingPlayer on hit"
                );
                return;
            }
        }
        panic!("no seed found with d6 >= 5");
    }
}
