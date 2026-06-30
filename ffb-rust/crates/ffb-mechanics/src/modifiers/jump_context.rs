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
