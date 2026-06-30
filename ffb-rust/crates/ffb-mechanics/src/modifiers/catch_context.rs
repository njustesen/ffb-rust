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
