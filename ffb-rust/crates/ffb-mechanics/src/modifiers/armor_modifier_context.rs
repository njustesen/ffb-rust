use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.ArmorModifierContext.
pub struct ArmorModifierContext<'a> {
    pub game: &'a Game,
    pub attacker: Option<&'a Player>,
    pub defender: &'a Player,
    pub is_stab: bool,
    pub is_foul: bool,
    pub foul_assists: i32,
    pub is_ttm: bool,
}

impl<'a> ArmorModifierContext<'a> {
    pub fn new(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists: 0, is_ttm: false }
    }

    pub fn new_with_foul_assists(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, foul_assists: i32) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists, is_ttm: false }
    }

    pub fn new_full(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, foul_assists: i32, is_ttm: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists, is_ttm }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_attacker(&self) -> Option<&Player> { self.attacker }
    pub fn get_defender(&self) -> &Player { self.defender }
    pub fn is_stab(&self) -> bool { self.is_stab }
    pub fn is_foul(&self) -> bool { self.is_foul }
    pub fn get_foul_assists(&self) -> i32 { self.foul_assists }
    pub fn is_ttm(&self) -> bool { self.is_ttm }
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
        let defender = Player::default();
        let ctx = ArmorModifierContext::new(&game, None, &defender, false, false);
        assert!(!ctx.is_stab());
        assert!(!ctx.is_foul());
        assert_eq!(ctx.get_foul_assists(), 0);
        assert!(!ctx.is_ttm());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let defender = Player::default();
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, None, &defender, false, true, 3);
        assert!(ctx.is_foul());
        assert_eq!(ctx.get_foul_assists(), 3);
        assert!(!ctx.is_ttm());
    }

    #[test]
    fn variant_constructor_sets_ttm() {
        let game = make_game();
        let defender = Player::default();
        let ctx = ArmorModifierContext::new_full(&game, None, &defender, true, false, 2, true);
        assert!(ctx.is_stab());
        assert!(ctx.is_ttm());
        assert_eq!(ctx.get_foul_assists(), 2);
    }

    #[test]
    fn attacker_none_when_not_provided() {
        let game = make_game();
        let defender = Player::default();
        let ctx = ArmorModifierContext::new(&game, None, &defender, false, false);
        assert!(ctx.get_attacker().is_none());
    }

    #[test]
    fn get_game_returns_game_with_correct_home_id() {
        let game = make_game();
        let defender = Player::default();
        let ctx = ArmorModifierContext::new(&game, None, &defender, false, false);
        assert_eq!(ctx.get_game().team_home.id, "home");
    }
}
