use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.JumpMechanic.
pub trait JumpMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::JUMP }

    fn is_available_as_next_move(&self, game: &Game, player: &ActingPlayer, jumping: bool) -> bool;
    fn can_still_jump(&self, game: &Game, acting_player: &ActingPlayer) -> bool;
    fn can_jump(&self, game: &Game, player: &Player, coordinate: FieldCoordinate) -> bool;
    fn is_valid_jump(&self, game: &Game, player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool;
}
