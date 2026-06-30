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
