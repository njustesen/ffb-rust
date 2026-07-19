/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepInitLookIntoMyEyes`.
///
/// Finds the first adjacent opponent that is carrying the ball and sets them as
/// the defender. Clears `game.defender_id` first so earlier state does not linger.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitLookIntoMyEyes` (mixed, BB2020 + BB2025).
pub struct StepInitLookIntoMyEyes;

impl StepInitLookIntoMyEyes {
    pub fn new() -> Self { Self }

    fn execute_step(game: &mut Game) -> StepOutcome {
        game.defender_id = None;

        let acting_id = game.acting_player.player_id.clone();
        let acting_id = match acting_id {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let coord = match game.field_model.player_coordinate(&acting_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        let opposing_team_ids: std::collections::HashSet<String> = if game.home_playing {
            game.team_away.players.iter().map(|p| p.id.clone()).collect()
        } else {
            game.team_home.players.iter().map(|p| p.id.clone()).collect()
        };

        let ball_coord = game.field_model.ball_coordinate;
        for adj in game.field_model.adjacent_on_pitch(coord) {
            if let Some(player_id) = game.field_model.player_at(adj) {
                // Java: UtilPlayer.findAdjacentBlockablePlayers → findBlockablePlayers filters
                // on `playerState.canBeBlocked()` (STANDING or MOVING) in addition to team
                // membership — a prone/stunned/downed opponent must not be targetable.
                let can_be_blocked = game.field_model.player_state(player_id.as_str())
                    .map(|s| s.can_be_blocked())
                    .unwrap_or(false);
                if opposing_team_ids.contains(player_id.as_str()) && can_be_blocked && ball_coord == Some(adj) {
                    game.defender_id = Some(player_id.clone());
                    break;
                }
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepInitLookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitLookIntoMyEyes {
    fn id(&self) -> StepId { StepId::InitLookIntoMyEyes }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING, PlayerAction};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_home_player(game: &mut Game, id: &str, pos: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    fn add_away_player(game: &mut Game, id: &str, pos: FieldCoordinate) {
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_init_look_into_my_eyes() {
        assert_eq!(StepInitLookIntoMyEyes::new().id(), StepId::InitLookIntoMyEyes);
    }

    #[test]
    fn no_adjacent_ball_carrier_clears_defender() {
        let mut step = StepInitLookIntoMyEyes::new();
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "att", FieldCoordinate::new(5, 5));
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        game.defender_id = Some("old".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn adjacent_ball_carrier_becomes_defender() {
        let mut step = StepInitLookIntoMyEyes::new();
        let mut game = make_game();
        game.home_playing = true;
        let att_pos = FieldCoordinate::new(5, 5);
        let def_pos = FieldCoordinate::new(5, 6);
        add_home_player(&mut game, "att", att_pos);
        add_away_player(&mut game, "def_with_ball", def_pos);
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        game.field_model.ball_coordinate = Some(def_pos);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.defender_id, Some("def_with_ball".into()));
    }

    #[test]
    fn prone_adjacent_ball_carrier_not_chosen() {
        // Java: findAdjacentBlockablePlayers filters on `playerState.canBeBlocked()`
        // (STANDING or MOVING only) — a prone opponent must not become the defender
        // even if adjacent and carrying the ball.
        let mut step = StepInitLookIntoMyEyes::new();
        let mut game = make_game();
        game.home_playing = true;
        let att_pos = FieldCoordinate::new(5, 5);
        let def_pos = FieldCoordinate::new(5, 6);
        add_home_player(&mut game, "att", att_pos);
        add_away_player(&mut game, "def_with_ball", def_pos);
        game.field_model.set_player_state(
            "def_with_ball",
            ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE),
        );
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        game.field_model.ball_coordinate = Some(def_pos);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none(), "prone opponent cannot be blocked, so must not be targeted");
    }

    #[test]
    fn non_adjacent_ball_carrier_not_chosen() {
        let mut step = StepInitLookIntoMyEyes::new();
        let mut game = make_game();
        game.home_playing = true;
        let att_pos = FieldCoordinate::new(5, 5);
        let def_pos = FieldCoordinate::new(10, 10);
        add_home_player(&mut game, "att", att_pos);
        add_away_player(&mut game, "far_carrier", def_pos);
        game.acting_player.set_player("att".into(), PlayerAction::Block);
        game.field_model.ball_coordinate = Some(def_pos);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn no_acting_player_is_noop() {
        let mut step = StepInitLookIntoMyEyes::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
    }
}
