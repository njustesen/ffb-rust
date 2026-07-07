/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.PlayerSelector`.
/// Selects eligible players for prayer enhancements (own team).
/// Extends mixed::prayers::PlayerSelector.
///
/// Eligibility: state == RESERVE, not a Star player, not already having all addedSkills.
use ffb_model::enums::{PS_RESERVE, PlayerType, SkillId};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

pub struct PlayerSelector;

impl PlayerSelector {
    pub const INSTANCE: PlayerSelector = PlayerSelector;

    pub fn new() -> Self { Self }
}

impl Default for PlayerSelector {
    fn default() -> Self { Self::new() }
}

impl PlayerSelectorTrait for PlayerSelector {
    /// Java: `eligiblePlayers` filtered via `selectPlayers`.
    /// Eligible: RESERVE state, not a Star, not already having all addedSkills.
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String> {
        let team = if game.team_home.id == team_id {
            &game.team_home
        } else {
            &game.team_away
        };

        let mut eligible: Vec<&str> = team.players.iter()
            .filter(|p| {
                game.field_model.player_state(&p.id)
                    .map_or(false, |s| s.base() == PS_RESERVE)
            })
            .filter(|p| p.player_type != PlayerType::Star)
            .filter(|p| added_skills.is_empty() || !added_skills.iter().all(|s| p.has_skill(*s)))
            .map(|p| p.id.as_str())
            .collect();

        // Java: shuffle then remove first for each slot — Fisher-Yates shuffle.
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
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str, state: PlayerState) {
        use ffb_model::model::player::Player;
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            is_big_guy: false,
            player_status: ffb_model::model::player_status::PlayerStatus::ACTIVE,
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, state);
    }

    fn add_star_player(game: &mut Game, team_id: &str, id: &str, state: PlayerState) {
        use ffb_model::model::player::Player;
        let p = Player {
            id: id.into(), name: id.into(), nr: 99,
            position_id: "star-pos".into(),
            player_type: PlayerType::Star,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 4, agility: 2, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            is_big_guy: false,
            player_status: ffb_model::model::player_status::PlayerStatus::ACTIVE,
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
    fn player_selector_default() {
        let _ = PlayerSelector::default();
    }

    #[test]
    fn selects_reserve_regular_player() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        let result = PlayerSelector::new().select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["h1".to_string()]);
    }

    #[test]
    fn excludes_non_reserve_player() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", PlayerState::new(PS_STANDING));
        let result = PlayerSelector::new().select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn excludes_star_players() {
        let mut game = make_game();
        add_star_player(&mut game, "home", "star1", PlayerState::new(PS_RESERVE));
        let result = PlayerSelector::new().select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty(), "Star player should be excluded");
    }

    #[test]
    fn respects_count_limit() {
        let mut game = make_game();
        for i in 0..5 {
            add_player(&mut game, "home", &format!("h{i}"), PlayerState::new(PS_RESERVE));
        }
        let result = PlayerSelector::new().select_players(&game, "home", 3, &mut GameRng::new(0), &[]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn selects_from_correct_team() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        add_player(&mut game, "away", "a1", PlayerState::new(PS_RESERVE));
        let result = PlayerSelector::new().select_players(&game, "away", 2, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["a1".to_string()]);
    }
}
