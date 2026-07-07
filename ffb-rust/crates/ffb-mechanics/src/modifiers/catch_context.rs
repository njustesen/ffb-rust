use ffb_model::model::{CatchScatterThrowInMode, Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.CatchContext.
pub struct CatchContext<'a> {
    pub game: &'a Game,
    pub player: Option<&'a Player>,
    pub catch_mode: CatchScatterThrowInMode,
    pub using_blast_it: bool,
}

impl<'a> CatchContext<'a> {
    pub fn new(game: &'a Game, player: Option<&'a Player>, catch_mode: CatchScatterThrowInMode, using_blast_it: Option<bool>) -> Self {
        Self {
            game,
            player,
            catch_mode,
            using_blast_it: using_blast_it.unwrap_or(false),
        }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> Option<&Player> { self.player }
    pub fn get_catch_mode(&self) -> CatchScatterThrowInMode { self.catch_mode }
    pub fn get_using_blast_it(&self) -> bool { self.using_blast_it }
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
        let ctx = CatchContext::new(&game, None, CatchScatterThrowInMode::CatchAccuratePass, None);
        assert_eq!(ctx.get_catch_mode(), CatchScatterThrowInMode::CatchAccuratePass);
        assert!(!ctx.get_using_blast_it());
        assert!(ctx.get_player().is_none());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchHandOff, Some(true));
        assert_eq!(ctx.get_catch_mode(), CatchScatterThrowInMode::CatchHandOff);
        assert!(ctx.get_using_blast_it());
        assert!(ctx.get_player().is_some());
    }

    #[test]
    fn flag_toggles_blast_it_none_defaults_false() {
        let game = make_game();
        let ctx_default = CatchContext::new(&game, None, CatchScatterThrowInMode::CatchBomb, None);
        let ctx_explicit = CatchContext::new(&game, None, CatchScatterThrowInMode::CatchBomb, Some(true));
        assert!(!ctx_default.get_using_blast_it());
        assert!(ctx_explicit.get_using_blast_it());
    }

    #[test]
    fn player_none_when_not_provided() {
        use ffb_model::model::CatchScatterThrowInMode;
        let game = make_game();
        let ctx = CatchContext::new(&game, None, CatchScatterThrowInMode::CatchHandOff, None);
        assert!(ctx.get_player().is_none());
    }

}
