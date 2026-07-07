use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.modifiers.DodgeContext.
pub struct DodgeContext<'a> {
    pub game: &'a Game,
    pub acting_player: &'a ActingPlayer,
    pub source_coordinate: FieldCoordinate,
    pub target_coordinate: FieldCoordinate,
    pub use_break_tackle: bool,
}

impl<'a> DodgeContext<'a> {
    pub fn new(
        game: &'a Game,
        acting_player: &'a ActingPlayer,
        source: FieldCoordinate,
        target: FieldCoordinate,
    ) -> Self {
        Self {
            game,
            acting_player,
            source_coordinate: source,
            target_coordinate: target,
            use_break_tackle: false,
        }
    }

    pub fn new_with_break_tackle(
        game: &'a Game,
        acting_player: &'a ActingPlayer,
        source: FieldCoordinate,
        target: FieldCoordinate,
        use_break_tackle: bool,
    ) -> Self {
        Self {
            game,
            acting_player,
            source_coordinate: source,
            target_coordinate: target,
            use_break_tackle,
        }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_acting_player(&self) -> &ActingPlayer { self.acting_player }
    pub fn get_source_coordinate(&self) -> FieldCoordinate { self.source_coordinate }
    pub fn get_target_coordinate(&self) -> FieldCoordinate { self.target_coordinate }
    pub fn is_use_break_tackle(&self) -> bool { self.use_break_tackle }

    /// Java getPlayer() returns actingPlayer.getPlayer() — resolves via game.
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
    use ffb_model::types::FieldCoordinate;

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
        let src = FieldCoordinate { x: 3, y: 4 };
        let tgt = FieldCoordinate { x: 5, y: 6 };
        let ctx = DodgeContext::new(&game, &acting_player, src, tgt);
        assert_eq!(ctx.source_coordinate, src);
        assert_eq!(ctx.target_coordinate, tgt);
        assert!(!ctx.use_break_tackle);
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let acting_player = ActingPlayer::default();
        let src = FieldCoordinate { x: 1, y: 2 };
        let tgt = FieldCoordinate { x: 7, y: 8 };
        let ctx = DodgeContext::new(&game, &acting_player, src, tgt);
        assert_eq!(ctx.get_source_coordinate(), src);
        assert_eq!(ctx.get_target_coordinate(), tgt);
        assert!(!ctx.is_use_break_tackle());
    }

    #[test]
    fn variant_constructor_sets_break_tackle() {
        let game = make_game();
        let acting_player = ActingPlayer::default();
        let src = FieldCoordinate { x: 0, y: 0 };
        let tgt = FieldCoordinate { x: 1, y: 1 };
        let ctx = DodgeContext::new_with_break_tackle(&game, &acting_player, src, tgt, true);
        assert!(ctx.is_use_break_tackle());
    }

    #[test]
    fn break_tackle_false_by_default() {
        let game = make_game();
        let acting_player = ActingPlayer::default();
        use ffb_model::types::FieldCoordinate;
        let src = FieldCoordinate { x: 0, y: 0 };
        let tgt = FieldCoordinate { x: 1, y: 1 };
        let ctx = DodgeContext::new(&game, &acting_player, src, tgt);
        assert!(!ctx.is_use_break_tackle());
    }


    #[test]
    fn get_game_away_team_id_is_accessible() {
        use ffb_model::model::ActingPlayer;
        use ffb_model::types::FieldCoordinate;
        let game = make_game();
        let ap = ActingPlayer::default();
        let ctx = DodgeContext::new(&game, &ap, FieldCoordinate::new(1,1), FieldCoordinate::new(2,2));
        assert_eq!(ctx.get_game().team_away.id, "away");
    }
}
