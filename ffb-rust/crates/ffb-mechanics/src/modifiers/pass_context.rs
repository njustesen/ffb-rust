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
