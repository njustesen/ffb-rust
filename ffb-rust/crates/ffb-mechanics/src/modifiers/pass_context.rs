use ffb_model::enums::PassingDistance;
use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.PassContext.
pub struct PassContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub distance: PassingDistance,
    pub ttm: bool,
}

impl<'a> PassContext<'a> {
    pub fn new(
        game: &'a Game,
        player: &'a Player,
        distance: PassingDistance,
        ttm: bool,
    ) -> Self {
        Self { game, player, distance, ttm }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_distance(&self) -> PassingDistance { self.distance }
    pub fn is_ttm(&self) -> bool { self.ttm }
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
        let ctx = PassContext::new(&game, &player, PassingDistance::ShortPass, false);
        assert_eq!(ctx.get_distance(), PassingDistance::ShortPass);
        assert!(!ctx.is_ttm());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = PassContext::new(&game, &player, PassingDistance::LongBomb, true);
        assert_eq!(ctx.get_distance(), PassingDistance::LongBomb);
        assert!(ctx.is_ttm());
    }

    #[test]
    fn flag_toggles_ttm_false_vs_true() {
        let game = make_game();
        let player = Player::default();
        let ctx_no_ttm = PassContext::new(&game, &player, PassingDistance::QuickPass, false);
        let ctx_ttm = PassContext::new(&game, &player, PassingDistance::QuickPass, true);
        assert!(!ctx_no_ttm.is_ttm());
        assert!(ctx_ttm.is_ttm());
    }
    #[test]
    fn player_stored_is_same_as_input() {
        let game = make_game();
        let mut player = Player::default();
        player.id = "thrower".into();
        let ctx = PassContext::new(&game, &player, PassingDistance::QuickPass, false);
        assert_eq!(ctx.get_player().id, "thrower");
    }
}
