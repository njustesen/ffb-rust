/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.OpponentPlayerSelector`.
/// Selects players from the OPPOSING team for prayer effects (e.g. Bad Habits, Greasy Cleats).
///
/// Java: overrides `determineTeam(team, game)` to return `game.getOtherTeam(team)`.
use ffb_model::model::game::Game;
use ffb_model::enums::{TurnMode, PS_RESERVE, SkillId};
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

pub struct OpponentPlayerSelector;

impl OpponentPlayerSelector {
    pub const INSTANCE: OpponentPlayerSelector = OpponentPlayerSelector;

    pub fn new() -> Self { Self }
}

impl Default for OpponentPlayerSelector {
    fn default() -> Self { Self::new() }
}

impl PlayerSelectorTrait for OpponentPlayerSelector {
    /// Same eligibility rules as PlayerSelector but applied to the opposing team.
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String> {
        let other_team_id = if game.team_home.id == team_id {
            game.team_away.id.as_str()
        } else {
            game.team_home.id.as_str()
        };

        let other_team = if game.team_home.id == other_team_id {
            &game.team_home
        } else {
            &game.team_away
        };

        let is_start_game = game.turn_mode == TurnMode::StartGame;

        let mut eligible: Vec<&str> = other_team.players.iter()
            .filter(|p| {
                if is_start_game {
                    game.field_model.player_state(&p.id)
                        .map_or(false, |s| s.base() == PS_RESERVE)
                } else {
                    game.field_model.player_coordinate(&p.id)
                        .map_or(false, |c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                }
            })
            .filter(|p| added_skills.is_empty() || !added_skills.iter().all(|s| p.has_skill(*s)))
            .map(|p| p.id.as_str())
            .collect();

        // Fisher-Yates shuffle for random selection.
        let n = eligible.len();
        for i in (1..n).rev() {
            let j = rng.range(i + 1);
            eligible.swap(i, j);
        }
        eligible.truncate(nr_of_players as usize);
        eligible.iter().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str, state: PlayerState) {
        use ffb_model::model::player::Player;
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn opponent_selector_default() {
        let _ = OpponentPlayerSelector::default();
    }

    #[test]
    fn selects_from_opposing_team() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "away", "a1", PlayerState::new(PS_RESERVE));
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        // Passing "home" as the praying team → should select from "away"
        let sel = OpponentPlayerSelector::new();
        let result = sel.select_players(&game, "home", 2, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["a1".to_string()]);
    }

    #[test]
    fn opponent_selector_on_pitch_regular() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, "away", "a1", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(13, 7));
        let sel = OpponentPlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["a1".to_string()]);
    }

    #[test]
    fn opponent_selector_respects_count() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        for i in 0..4 {
            let id = format!("a{i}");
            add_player(&mut game, "away", &id, PlayerState::new(PS_RESERVE));
        }
        let sel = OpponentPlayerSelector::new();
        let result = sel.select_players(&game, "home", 2, &mut GameRng::new(0), &[]);
        assert_eq!(result.len(), 2);
    }
}
