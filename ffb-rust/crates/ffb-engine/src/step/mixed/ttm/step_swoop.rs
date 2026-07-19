/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.ttm.StepSwoop`.
///
/// Step in the TTM sequence to handle the Swoop skill (BB2016 + BB2020).
///
/// Structure mirrors the BB2025 version exactly except this edition is tagged
/// `@RulesCollection(BB2016)` and `@RulesCollection(BB2020)`.  The Java logic
/// is identical to BB2025 — the inner `StepState` fields and `executeStep` /
/// `executeSwoop` methods are the same.
///
/// See `crates/ffb-engine/src/step/bb2025/ttm/step_swoop.rs` for the full
/// commentary.
use ffb_model::enums::{Direction, PlayerAction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, StepAction};
use crate::util::util_server_player_swoop::UtilServerPlayerSwoop;

/// Java: `StepSwoop` (mixed/ttm, BB2016 + BB2020).
///
/// Inner class `StepState` translated as struct fields.
pub struct StepSwoop {
    /// Java: state.status
    pub status: Option<String>,
    /// Java: state.thrownPlayerId (mandatory init param)
    pub thrown_player_id: Option<String>,
    /// Java: state.thrownPlayerState (mandatory init param)
    pub thrown_player_state: Option<PlayerState>,
    /// Java: state.thrownPlayerHasBall
    pub thrown_player_has_ball: bool,
    /// Java: state.thrownPlayerCoordinate (mandatory init param)
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: state.throwScatter
    pub throw_scatter: bool,
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.coordinateTo (set by CLIENT_SWOOP)
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: state.goToLabelOnFallDown (mandatory init param)
    pub goto_label_on_fall_down: String,
    /// Java: (not present in mixed; present in bb2025 variant for usingSwoop dialog)
    pub using_swoop: Option<bool>,
    /// Java: (not present in mixed version's StepState)
    pub swoop_direction: Option<Direction>,
}

impl StepSwoop {
    pub fn new(goto_label_on_fall_down: String) -> Self {
        Self {
            status: None,
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            coordinate_from: None,
            coordinate_to: None,
            goto_label_on_fall_down,
            using_swoop: None,
            swoop_direction: None,
        }
    }
}

impl Default for StepSwoop {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepSwoop {
    fn id(&self) -> StepId { StepId::Swoop }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    /// Java: handleCommand dispatches:
    ///   CLIENT_SWOOP  → coordinateTo = transformed coord → executeSwoop()
    ///   EXECUTE_STEP  → executeStep()
    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                // CLIENT_SWOOP — target square selected
                // Java: if command NOT from home player, transform coordinate
                let is_home_player = self.thrown_player_id.as_deref()
                    .map(|id| game.team_home.player(id).is_some())
                    .unwrap_or(game.home_playing);
                self.coordinate_to = Some(if is_home_player { *coord } else { coord.transform() });
                // no-op: executeSwoop() SkillBehaviour hooks skipped in headless (registry not ported)
                return StepOutcome::next();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v)   => { self.coordinate_to = Some(*v); true }
            _ => false,
        }
    }
}

impl StepSwoop {
    /// Java: `StepSwoop.executeStep()` (mixed/ttm, BB2016 + BB2020).
    ///
    /// Note: unlike the BB2025 variant, this edition's `StepState` has no
    /// `usingSwoop` field — there is no "use the Swoop skill?" dialog gate here,
    /// the throw-scatter movement always runs unconditionally when `throwScatter`
    /// is set.
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: thrownPlayer = game.getPlayerById(state.thrownPlayerId)
        // if (thrownPlayer == null || state.thrownPlayerCoordinate == null) -> NEXT_STEP
        let player_id = match self.thrown_player_id.as_deref() {
            Some(id) if game.player(id).is_some() => id.to_string(),
            _ => return StepOutcome::next(),
        };
        if self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        let mut outcome = StepOutcome::next();

        // Java: if (state.throwScatter) { ... move thrown player to passCoordinate,
        //        change acting player to SWOOP, move ball if carried, set current
        //        move points, publish THROWN_PLAYER_ID/STATE/HAS_BALL ... }
        if self.throw_scatter {
            game.field_model.range_ruler = None;
            game.field_model.clear_move_squares();
            if let Some(pass_coord) = game.pass_coordinate {
                // client-only: animation(thrownPlayerCoordinate -> passCoordinate), syncGameModel
                game.field_model.set_player_coordinate(&player_id, pass_coord);
                // Java: UtilActingPlayer.changeActingPlayer(game, thrownPlayerId, SWOOP, false)
                game.acting_player.player_id = Some(player_id.clone());
                game.acting_player.player_action = Some(PlayerAction::Swoop);
                if let Some(ref mut bts) = game.blitz_turn_state {
                    bts.change_acting_player();
                }
                if self.thrown_player_has_ball {
                    game.field_model.ball_coordinate = Some(pass_coord);
                }
                // Java: actingPlayer.setCurrentMove(thrownPlayer.getMovementWithModifiers() - 3)
                let movement = game.player(&player_id)
                    .map(|p| p.movement_with_modifiers())
                    .unwrap_or(0);
                game.acting_player.current_move = movement - 3;
            }
            outcome = outcome
                .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball));
        }

        // Java: if (state.coordinateTo == null) → updateSwoopSquares → wait for CLIENT_SWOOP
        if self.coordinate_to.is_none() {
            UtilServerPlayerSwoop::update_swoop_squares(game, &player_id);
            outcome.action = StepAction::Continue;
            return outcome;
        }

        // coordinateTo is known; executeSwoop hook runs (TODO: step hooks)
        outcome
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn id_is_swoop() {
        assert_eq!(StepSwoop::new("fall".into()).id(), StepId::Swoop);
    }

    #[test]
    fn no_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        // thrown_player_id is None → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn unknown_player_id_returns_next_step() {
        // Java: `game.getPlayerById(state.thrownPlayerId) == null` -> NEXT_STEP
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("ghost".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_with_no_coord_to_waits() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate { x: 3, y: 3 });
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        // coordinate_to is None → Continue (wait for CLIENT_SWOOP)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn player_with_coord_to_returns_next() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate { x: 3, y: 3 });
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        step.coordinate_to = Some(FieldCoordinate { x: 5, y: 5 });
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn throw_scatter_moves_player_and_ball_to_pass_coordinate() {
        // Regression test: the mixed (BB2016/BB2020) StepSwoop.executeStep's
        // `if (state.throwScatter) { ... }` block — moving the thrown player and
        // ball to the pass coordinate, switching the acting player to SWOOP, and
        // setting current move points — was entirely missing from the Rust
        // translation. It must now actually move the player/ball.
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate { x: 3, y: 3 });
        game.pass_coordinate = Some(FieldCoordinate { x: 10, y: 10 });
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        step.thrown_player_has_ball = true;
        step.throw_scatter = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(FieldCoordinate { x: 10, y: 10 }));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate { x: 10, y: 10 }));
        assert_eq!(game.acting_player.player_id.as_deref(), Some("p1"));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Swoop));
        // movement (6) - 3 = 3
        assert_eq!(game.acting_player.current_move, 3);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(id)) if id == "p1")));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerHasBall(true))));
        // coordinate_to not set -> waits for CLIENT_SWOOP
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn throw_scatter_without_ball_does_not_move_ball() {
        let mut game = make_game();
        add_player(&mut game, "p1", FieldCoordinate { x: 3, y: 3 });
        game.pass_coordinate = Some(FieldCoordinate { x: 10, y: 10 });
        game.field_model.ball_coordinate = Some(FieldCoordinate { x: 1, y: 1 });
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        step.thrown_player_has_ball = false;
        step.throw_scatter = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate { x: 1, y: 1 }));
    }

    #[test]
    fn set_coordinate_from_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 1, y: 2 };
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(c)));
        assert_eq!(step.coordinate_from, Some(c));
    }

    #[test]
    fn set_coordinate_to_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 7, y: 4 };
        assert!(step.set_parameter(&StepParameter::CoordinateTo(c)));
        assert_eq!(step.coordinate_to, Some(c));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepSwoop::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn away_player_swoop_transforms_coordinate() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        use ffb_model::util::rng::GameRng;
        let mut game = make_game();
        // Add player to away team
        game.team_away.players.push(Player {
            id: "away_p".into(), name: "away_p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("away_p".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        let original_coord = FieldCoordinate { x: 3, y: 4 };
        let transformed = original_coord.transform();
        step.handle_command(&Action::Pass { coord: original_coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.coordinate_to, Some(transformed), "away player coord must be transformed");
    }

    #[test]
    fn home_player_swoop_does_not_transform_coordinate() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        use ffb_model::util::rng::GameRng;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "home_p".into(), name: "home_p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("home_p".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        let original_coord = FieldCoordinate { x: 3, y: 4 };
        step.handle_command(&Action::Pass { coord: original_coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.coordinate_to, Some(original_coord), "home player coord must not be transformed");
    }
}
