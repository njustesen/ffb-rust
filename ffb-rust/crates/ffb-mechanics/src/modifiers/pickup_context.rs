use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.PickupContext.
pub struct PickupContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
}

impl<'a> PickupContext<'a> {
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
        player.id = "picker".into();
        let ctx = PickupContext::new(&game, &player);
        assert_eq!(ctx.get_player().id, "picker");
    }

    #[test]
    fn get_game_returns_game() {
        let game = make_game();
        let player = Player::default();
        let ctx = PickupContext::new(&game, &player);
        assert_eq!(ctx.get_game().team_home.id, "home");
    }
}
