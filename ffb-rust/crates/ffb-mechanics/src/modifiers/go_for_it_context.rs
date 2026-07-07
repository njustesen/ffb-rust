use std::collections::HashSet;
use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.GoForItContext.
pub struct GoForItContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub teams_with_moles_under_pitch: HashSet<String>,
}

impl<'a> GoForItContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player) -> Self {
        Self {
            game,
            player,
            teams_with_moles_under_pitch: HashSet::new(),
        }
    }

    pub fn new_with_moles(
        game: &'a Game,
        player: &'a Player,
        teams_with_moles_under_pitch: HashSet<String>,
    ) -> Self {
        Self { game, player, teams_with_moles_under_pitch }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_teams_with_moles_under_pitch(&self) -> &HashSet<String> {
        &self.teams_with_moles_under_pitch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::Player;

    fn make_game() -> ffb_model::model::Game {
        use ffb_model::enums::Rules;
        ffb_model::model::Game::new(
            ffb_model::model::Team {
                id: "home".into(), name: "H".into(), race: "human".into(),
                roster_id: "human".into(), coach: "c".into(),
                rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
                cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
                team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
                vampire_lord: false, necromancer: false,
            },
            ffb_model::model::Team {
                id: "away".into(), name: "A".into(), race: "human".into(),
                roster_id: "human".into(), coach: "c".into(),
                rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
                cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
                team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
                vampire_lord: false, necromancer: false,
            },
            Rules::Bb2025,
        )
    }

    #[test]
    fn new_has_expected_fields() {
        let game = make_game();
        let player = Player::default();
        let ctx = GoForItContext::new(&game, &player);
        assert!(ctx.get_teams_with_moles_under_pitch().is_empty());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = GoForItContext::new(&game, &player);
        // get_player returns the same player reference
        assert_eq!(ctx.get_player().id, player.id);
    }

    #[test]
    fn variant_constructor_sets_moles() {
        let game = make_game();
        let player = Player::default();
        let mut moles = HashSet::new();
        moles.insert("home".to_string());
        let ctx = GoForItContext::new_with_moles(&game, &player, moles);
        assert!(ctx.get_teams_with_moles_under_pitch().contains("home"));
        assert_eq!(ctx.get_teams_with_moles_under_pitch().len(), 1);
    }
}
