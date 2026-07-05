/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.OpponentPlayerSelector`.
/// Extends PlayerSelector; overrides determineTeam to return the opponent's team.
use ffb_model::model::game::Game;
use ffb_model::enums::SkillId;
use ffb_model::util::rng::GameRng;
use super::player_selector::PlayerSelector;
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
    /// Delegates to PlayerSelector but with the opponent team id.
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String> {
        let opponent_id = if game.team_home.id == team_id {
            &game.team_away.id
        } else {
            &game.team_home.id
        };
        PlayerSelector::new().select_players(game, opponent_id, nr_of_players, rng, added_skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE};
    use ffb_model::enums::PlayerState;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::PlayerType;
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
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn opponent_selector_default() {
        let _ = OpponentPlayerSelector::default();
    }

    #[test]
    fn selects_opponent_team_players() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1");
        add_player(&mut game, "away", "a1");
        // Team "home" passes → opponent is "away"
        let result = OpponentPlayerSelector::new().select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["a1".to_string()], "Should select from opponent (away)");
    }

    #[test]
    fn selects_no_own_team_players() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1");
        // no away players
        let result = OpponentPlayerSelector::new().select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty());
    }
}
