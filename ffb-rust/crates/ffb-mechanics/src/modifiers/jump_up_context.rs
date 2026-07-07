use ffb_model::model::{ActingPlayer, Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpUpContext.
/// Java getPlayer() returns actingPlayer.getPlayer() — resolved via game.
pub struct JumpUpContext<'a> {
    pub game: &'a Game,
    pub acting_player: &'a ActingPlayer,
}

impl<'a> JumpUpContext<'a> {
    pub fn new(game: &'a Game, acting_player: &'a ActingPlayer) -> Self {
        Self { game, acting_player }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_acting_player(&self) -> &ActingPlayer { self.acting_player }

    pub fn get_player<'b>(&self, game: &'b Game) -> Option<&'b Player>
    where
        'a: 'b,
    {
        self.acting_player.player_id.as_deref().and_then(|id| game.player(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ActingPlayer;

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
        let acting_player = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting_player);
        assert!(ctx.get_acting_player().player_id.is_none());
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let mut acting_player = ActingPlayer::default();
        acting_player.player_id = Some("p1".into());
        let ctx = JumpUpContext::new(&game, &acting_player);
        assert_eq!(ctx.get_acting_player().player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn get_player_returns_none_when_id_absent() {
        let game = make_game();
        let acting_player = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting_player);
        assert!(ctx.get_player(&game).is_none());
    }

    #[test]
    fn acting_player_id_none_by_default() {
        let game = make_game();
        let acting_player = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting_player);
        assert!(ctx.get_acting_player().player_id.is_none());
    }


    #[test]
    fn get_game_away_team_id_is_accessible() {
        let game = make_game();
        let ap = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &ap);
        assert_eq!(ctx.get_game().team_away.id, "away");
    }
}
