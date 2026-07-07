use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.GazeModifierContext.
pub struct GazeModifierContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
}

impl<'a> GazeModifierContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player) -> Self {
        Self { game, player }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
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
    fn new_creates_context_with_player() {
        let game = make_game();
        let mut player = Player::default();
        player.id = "gazer".into();
        let ctx = GazeModifierContext::new(&game, &player);
        assert_eq!(ctx.get_player().id, "gazer");
    }

    #[test]
    fn get_game_returns_game() {
        let game = make_game();
        let player = Player::default();
        let ctx = GazeModifierContext::new(&game, &player);
        assert_eq!(ctx.get_game().team_home.id, "home");
    }

    #[test]
    fn get_game_away_team_id_is_accessible() {
        let game = make_game();
        let player = Player::default();
        let ctx = GazeModifierContext::new(&game, &player);
        assert_eq!(ctx.get_game().team_away.id, "away");
    }

    #[test]
    fn player_name_is_accessible_via_get_player() {
        let game = make_game();
        let mut player = Player::default();
        player.name = "Gazer".into();
        let ctx = GazeModifierContext::new(&game, &player);
        assert_eq!(ctx.get_player().name, "Gazer");
    }

    #[test]
    fn two_contexts_can_share_same_game() {
        let game = make_game();
        let p1 = Player::default();
        let p2 = Player::default();
        let ctx1 = GazeModifierContext::new(&game, &p1);
        let ctx2 = GazeModifierContext::new(&game, &p2);
        assert_eq!(ctx1.get_game().team_home.id, ctx2.get_game().team_home.id);
    }
}
