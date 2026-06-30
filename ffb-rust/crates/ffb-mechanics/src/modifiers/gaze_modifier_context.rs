use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.GazeModifierContext.
pub struct GazeModifierContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
}

impl<'a> GazeModifierContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player) -> Self {
        Self { game, player }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
}
