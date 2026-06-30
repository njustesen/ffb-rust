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
