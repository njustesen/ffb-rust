/// 1:1 translation of com.fumbbl.ffb.server.step.action.ttm.StepEatTeamMate (COMMON).
///
/// Step in TTM sequence to eat a team-mate (Always Hungry failure).
///
/// Expected preceding params: THROWN_PLAYER_COORDINATE, THROWN_PLAYER_ID.
/// Publishes: INJURY_RESULT, THROWN_PLAYER_COORDINATE(null), and if ball was at that coord:
///   END_TURN(true), CATCH_SCATTER_THROW_IN_MODE(ScatterBall).
/// Always returns NEXT_STEP.
///
/// Injury: InjuryTypeEatPlayer → armorBroken=true, injury=RIP (no dice).
use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::{InjuryContext, InjuryResult};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

pub struct StepEatTeamMate {
    /// Java: fThrownPlayerCoordinate — set by preceding step parameter.
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: fThrownPlayerId — set by preceding step parameter.
    pub thrown_player_id: Option<String>,
}

impl StepEatTeamMate {
    pub fn new() -> Self {
        Self {
            thrown_player_coordinate: None,
            thrown_player_id: None,
        }
    }
}

impl Default for StepEatTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepEatTeamMate {
    fn id(&self) -> StepId { StepId::EatTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerCoordinate(v) => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepEatTeamMate {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let thrown_player_exists = self.thrown_player_id.as_deref()
            .and_then(|id| game.player(id))
            .is_some();

        let mut out = StepOutcome::next();

        if thrown_player_exists {
            if let Some(coord) = self.thrown_player_coordinate {
                // Java: if (coord.equals(game.getFieldModel().getBallCoordinate()))
                if game.field_model.ball_coordinate == Some(coord) {
                    game.field_model.ball_moving = true;
                    out = out
                        .publish(StepParameter::EndTurn(true))
                        .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
                }

                // Java: InjuryTypeEatPlayer → armorBroken=true, injury=RIP
                let mut ctx = InjuryContext::new(ApothecaryMode::ThrownPlayer);
                ctx.armor_broken = true;
                let mut injury = InjuryResult::new(ApothecaryMode::ThrownPlayer);
                injury.injury_context = ctx;
                injury.rip = true;

                out = out
                    .publish(StepParameter::InjuryResult(Box::new(injury)))
                    // Java: publishParameter(THROWN_PLAYER_COORDINATE, null) — avoid reset in end step
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{CatchScatterThrowInMode, StepAction, StepParameter};
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.to_string(), name: id.to_string(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
}

    fn make_game_with_player(id: &str) -> Game {
        let mut home = test_team("home", 0);
        home.players.push(make_player(id));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn no_thrown_player_does_nothing_returns_next() {
        let mut home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        // thrown_player_id not set → game.player() returns None → no publish
        let out = StepEatTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn valid_player_publishes_injury_and_clears_coord() {
        let mut game = make_game_with_player("eaten");
        game.field_model.set_player_coordinate("eaten", FieldCoordinate::new(5, 5));
        let mut step = StepEatTeamMate::new();
        step.thrown_player_id = Some("eaten".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);

        let has_injury = out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(ir) if ir.rip && ir.injury_context.armor_broken));
        assert!(has_injury, "should publish InjuryResult with rip=true and armor_broken=true");

        let has_null_coord = out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None)));
        assert!(has_null_coord, "should publish ThrownPlayerCoordinate(None) sentinel");
    }

    #[test]
    fn ball_at_thrown_coord_triggers_scatter_and_end_turn() {
        let mut game = make_game_with_player("eaten");
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("eaten", coord);
        game.field_model.ball_coordinate = Some(coord);
        let mut step = StepEatTeamMate::new();
        step.thrown_player_id = Some("eaten".into());
        step.thrown_player_coordinate = Some(coord);
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(game.field_model.ball_moving);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
        let has_scatter = out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)));
        assert!(has_scatter);
    }

    #[test]
    fn ball_not_at_thrown_coord_does_not_scatter() {
        let mut game = make_game_with_player("eaten");
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("eaten", coord);
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(3, 3));
        let mut step = StepEatTeamMate::new();
        step.thrown_player_id = Some("eaten".into());
        step.thrown_player_coordinate = Some(coord);
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(!game.field_model.ball_moving);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(_)));
        assert!(!has_end_turn);
        let has_scatter = out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(_)));
        assert!(!has_scatter);
    }

    #[test]
    fn thrown_player_id_parameter_accepted() {
        let mut step = StepEatTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn thrown_player_coordinate_parameter_accepted() {
        let mut step = StepEatTeamMate::new();
        let coord = FieldCoordinate::new(3, 4);
        step.set_parameter(&StepParameter::ThrownPlayerCoordinate(Some(coord)));
        assert_eq!(step.thrown_player_coordinate, Some(coord));
    }
}
