use ffb_model::model::{Game, Player};
use crate::pass_result::PassResult;

/// 1:1 translation of com.fumbbl.ffb.modifiers.RightStuffContext.
/// KickTeamMateRange is not yet fully translated — represented as Option<String> placeholder.
pub struct RightStuffContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub pass_result: Option<PassResult>,
    /// Placeholder for KickTeamMateRange until that type is fully translated.
    pub ktm_range: Option<String>,
}

impl<'a> RightStuffContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player) -> Self {
        Self { game, player, pass_result: None, ktm_range: None }
    }

    pub fn new_full(
        game: &'a Game,
        player: &'a Player,
        pass_result: Option<PassResult>,
        ktm_range: Option<String>,
    ) -> Self {
        Self { game, player, pass_result, ktm_range }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_pass_result(&self) -> Option<PassResult> { self.pass_result }
    pub fn get_ktm_range(&self) -> Option<&str> { self.ktm_range.as_deref() }
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
        let ctx = RightStuffContext::new(&game, &player);
        assert!(ctx.get_pass_result().is_none());
        assert!(ctx.get_ktm_range().is_none());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = RightStuffContext::new_full(&game, &player, Some(PassResult::ACCURATE), Some("Short".into()));
        assert_eq!(ctx.get_pass_result(), Some(PassResult::ACCURATE));
        assert_eq!(ctx.get_ktm_range(), Some("Short"));
    }

    #[test]
    fn variant_constructor_with_none_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = RightStuffContext::new_full(&game, &player, None, None);
        assert!(ctx.get_pass_result().is_none());
        assert!(ctx.get_ktm_range().is_none());
    }

    #[test]
    fn player_stored_is_same_as_input() {
        let game = make_game();
        let player = Player::default();
        let ctx = RightStuffContext::new(&game, &player);
        assert_eq!(ctx.get_player().id, player.id);
    }


    #[test]
    fn get_game_away_team_id_is_accessible() {
        let game = make_game();
        let player = Player::default();
        let ctx = RightStuffContext::new(&game, &player);
        assert_eq!(ctx.get_game().team_away.id, "away");
    }
}
