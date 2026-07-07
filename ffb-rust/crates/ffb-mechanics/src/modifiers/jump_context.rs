use ffb_model::model::{Game, Player};
use ffb_model::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpContext.
pub struct JumpContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub from: FieldCoordinate,
    pub to: FieldCoordinate,
    pub accumulated_modifiers: i32,
    pub modifier_count: i32,
}

impl<'a> JumpContext<'a> {
    pub fn new(
        game: &'a Game,
        player: &'a Player,
        from: FieldCoordinate,
        to: FieldCoordinate,
    ) -> Self {
        Self {
            game,
            player,
            from,
            to,
            accumulated_modifiers: 0,
            modifier_count: 0,
        }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_from(&self) -> FieldCoordinate { self.from }
    pub fn get_to(&self) -> FieldCoordinate { self.to }
    pub fn get_accumulated_modifiers(&self) -> i32 { self.accumulated_modifiers }
    pub fn get_modifier_count(&self) -> i32 { self.modifier_count }

    pub fn add_modifier_value(&mut self, value: i32) {
        self.accumulated_modifiers += value;
    }

    pub fn add_modifier_count(&mut self, count: i32) {
        self.modifier_count += count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::Player;
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
        let player = Player::default();
        let from = FieldCoordinate { x: 2, y: 3 };
        let to = FieldCoordinate { x: 4, y: 5 };
        let ctx = JumpContext::new(&game, &player, from, to);
        assert_eq!(ctx.from, from);
        assert_eq!(ctx.to, to);
        assert_eq!(ctx.accumulated_modifiers, 0);
        assert_eq!(ctx.modifier_count, 0);
    }

    #[test]
    fn getters_return_set_values() {
        let game = make_game();
        let player = Player::default();
        let from = FieldCoordinate { x: 1, y: 1 };
        let to = FieldCoordinate { x: 9, y: 9 };
        let ctx = JumpContext::new(&game, &player, from, to);
        assert_eq!(ctx.get_from(), from);
        assert_eq!(ctx.get_to(), to);
        assert_eq!(ctx.get_accumulated_modifiers(), 0);
        assert_eq!(ctx.get_modifier_count(), 0);
    }

    #[test]
    fn flag_toggles_accumulate_modifiers() {
        let game = make_game();
        let player = Player::default();
        let from = FieldCoordinate { x: 0, y: 0 };
        let to = FieldCoordinate { x: 1, y: 1 };
        let mut ctx = JumpContext::new(&game, &player, from, to);
        ctx.add_modifier_value(-2);
        ctx.add_modifier_count(1);
        assert_eq!(ctx.get_accumulated_modifiers(), -2);
        assert_eq!(ctx.get_modifier_count(), 1);
    }
}
