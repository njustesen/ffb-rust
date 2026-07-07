use ffb_model::model::{Game, Player};
use crate::pass_result::PassResult;

/// 1:1 translation of com.fumbbl.ffb.modifiers.InterceptionContext.
pub struct InterceptionContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub pass_result: PassResult,
    pub bomb: bool,
    /// Number of tackle zones the interceptor exerts on the pass corridor (0-8).
    pub nr_of_tacklezones: i32,
}

impl<'a> InterceptionContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player, pass_result: PassResult, bomb: bool) -> Self {
        Self { game, player, pass_result, bomb, nr_of_tacklezones: 0 }
    }

    pub fn new_with_tacklezones(game: &'a Game, player: &'a Player, pass_result: PassResult, bomb: bool, nr_of_tacklezones: i32) -> Self {
        Self { game, player, pass_result, bomb, nr_of_tacklezones }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_pass_result(&self) -> PassResult { self.pass_result }
    pub fn is_bomb(&self) -> bool { self.bomb }
    pub fn get_nr_of_tacklezones(&self) -> i32 { self.nr_of_tacklezones }
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
        let ctx = InterceptionContext::new(&game, &player, PassResult::ACCURATE, false);
        assert_eq!(ctx.get_pass_result(), PassResult::ACCURATE);
        assert!(!ctx.is_bomb());
        assert_eq!(ctx.get_nr_of_tacklezones(), 0);
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = InterceptionContext::new(&game, &player, PassResult::INACCURATE, true);
        assert_eq!(ctx.get_pass_result(), PassResult::INACCURATE);
        assert!(ctx.is_bomb());
    }

    #[test]
    fn variant_constructor_sets_tacklezones() {
        let game = make_game();
        let player = Player::default();
        let ctx = InterceptionContext::new_with_tacklezones(&game, &player, PassResult::FUMBLE, false, 3);
        assert_eq!(ctx.get_nr_of_tacklezones(), 3);
        assert_eq!(ctx.get_pass_result(), PassResult::FUMBLE);
    }
}
