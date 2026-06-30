use ffb_model::model::{Game, Player};
use crate::pass_result::PassResult;

/// 1:1 translation of com.fumbbl.ffb.modifiers.InterceptionContext.
pub struct InterceptionContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub pass_result: PassResult,
    pub bomb: bool,
    /// Number of tackle zones the interceptor exerts on the pass corridor (0-8).
    pub nr_of_tacklezones: i32,
}

impl<'a> InterceptionContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player, pass_result: PassResult, bomb: bool) -> Self {
        Self { game, player, pass_result, bomb, nr_of_tacklezones: 0 }
    }

    pub fn new_with_tacklezones(game: &'a Game, player: &'a Player, pass_result: PassResult, bomb: bool, nr_of_tacklezones: i32) -> Self {
        Self { game, player, pass_result, bomb, nr_of_tacklezones }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_pass_result(&self) -> PassResult { self.pass_result }
    pub fn is_bomb(&self) -> bool { self.bomb }
    pub fn get_nr_of_tacklezones(&self) -> i32 { self.nr_of_tacklezones }
}
