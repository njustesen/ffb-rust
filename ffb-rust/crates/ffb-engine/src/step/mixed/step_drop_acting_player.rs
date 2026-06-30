/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepDropActingPlayer`.
///
/// Drops the acting player prone if they are on the field and not stunned.
/// Publishes parameters from `drop_player` (ball scatter, end-turn flag).
use ffb_model::enums::PS_STUNNED;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury::drop_player;

/// Java: `StepDropActingPlayer` (mixed, BB2020 + BB2025).
pub struct StepDropActingPlayer;

impl StepDropActingPlayer {
    pub fn new() -> Self { Self }
}

impl Default for StepDropActingPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropActingPlayer {
    fn id(&self) -> StepId { StepId::DropActingPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepDropActingPlayer {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let coord = game.field_model.player_coordinate(&player_id);
        let in_bounds = coord.map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c)).unwrap_or(false);
        let state = game.field_model.player_state(&player_id);
        let not_stunned = state.map(|s| s.base() != PS_STUNNED).unwrap_or(false);

        let mut outcome = StepOutcome::next();
        if in_bounds && not_stunned {
            let params = drop_player(game, &player_id, true);
            for p in params {
                outcome = outcome.publish(p);
            }
        }
        outcome
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING, PS_STUNNED, PS_PRONE, PlayerAction};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) -> FieldCoordinate {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        pos
    }

    #[test]
    fn id_is_drop_acting_player() {
        assert_eq!(StepDropActingPlayer::new().id(), StepId::DropActingPlayer);
    }

    #[test]
    fn standing_acting_player_becomes_prone() {
        let mut step = StepDropActingPlayer::new();
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("att").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }

    #[test]
    fn stunned_acting_player_not_dropped() {
        let mut step = StepDropActingPlayer::new();
        let mut game = make_game();
        add_player(&mut game, "att", PS_STUNNED);
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("att").unwrap();
        assert_eq!(state.base(), PS_STUNNED, "stunned player should not be dropped");
    }

    #[test]
    fn no_acting_player_is_noop() {
        let mut step = StepDropActingPlayer::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
    }

    #[test]
    fn ball_carrier_triggers_scatter_param() {
        let mut step = StepDropActingPlayer::new();
        let mut game = make_game();
        let pos = add_player(&mut game, "att", PS_STANDING);
        game.field_model.ball_coordinate = Some(pos);
        game.field_model.ball_in_play = true;
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(game.field_model.ball_moving);
        let has_scatter = outcome.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(crate::step::CatchScatterThrowInMode::ScatterBall))
        });
        assert!(has_scatter);
    }
}
