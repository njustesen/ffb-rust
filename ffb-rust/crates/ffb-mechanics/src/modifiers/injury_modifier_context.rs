use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.InjuryModifierContext.
pub struct InjuryModifierContext<'a> {
    pub game: &'a Game,
    pub attacker: Option<&'a Player>,
    pub defender: &'a Player,
    pub is_stab: bool,
    pub is_foul: bool,
    pub is_vomit_like: bool,
    pub is_chainsaw: bool,
    pub is_ttm: bool,
    attacker_mode: bool,
}

impl<'a> InjuryModifierContext<'a> {
    pub fn new(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, is_vomit_like: bool, is_chainsaw: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw, is_ttm: false, attacker_mode: true }
    }

    pub fn new_with_ttm(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, is_vomit_like: bool, is_chainsaw: bool, is_ttm: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw, is_ttm, attacker_mode: true }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_attacker(&self) -> Option<&Player> { self.attacker }
    pub fn get_defender(&self) -> &Player { self.defender }
    pub fn is_stab(&self) -> bool { self.is_stab }
    pub fn is_foul(&self) -> bool { self.is_foul }
    pub fn is_vomit_like(&self) -> bool { self.is_vomit_like }
    pub fn is_chainsaw(&self) -> bool { self.is_chainsaw }
    pub fn is_ttm(&self) -> bool { self.is_ttm }
    pub fn set_defender_mode(&mut self) { self.attacker_mode = false; }
    pub fn is_attacker_mode(&self) -> bool { self.attacker_mode }
    pub fn is_defender_mode(&self) -> bool { !self.attacker_mode }
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
        let ctx = InjuryModifierContext::new(&game, None, &defender, false, false, false, false);
        assert!(!ctx.is_stab());
        assert!(!ctx.is_foul());
        assert!(!ctx.is_vomit_like());
        assert!(!ctx.is_chainsaw());
        assert!(!ctx.is_ttm());
        assert!(ctx.is_attacker_mode());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let defender = Player::default();
        let ctx = InjuryModifierContext::new(&game, None, &defender, true, true, true, true);
        assert!(ctx.is_stab());
        assert!(ctx.is_foul());
        assert!(ctx.is_vomit_like());
        assert!(ctx.is_chainsaw());
        assert!(ctx.get_attacker().is_none());
    }

    #[test]
    fn variant_constructor_sets_ttm() {
        let game = make_game();
        let defender = Player::default();
        let ctx = InjuryModifierContext::new_with_ttm(&game, None, &defender, false, false, false, false, true);
        assert!(ctx.is_ttm());
        assert!(ctx.is_attacker_mode());
        assert!(!ctx.is_defender_mode());
    }

    #[test]
    fn attacker_none_when_not_provided() {
        let game = make_game();
        let defender = Player::default();
        let ctx = InjuryModifierContext::new(&game, None, &defender, false, false, false, false);
        assert!(ctx.get_attacker().is_none());
    }

}
