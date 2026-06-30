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
