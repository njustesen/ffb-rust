use std::collections::HashSet;
use ffb_model::enums::PassingDistance;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{Game, Player, TurnData};
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::PassModifier;

/// 1:1 translation of com.fumbbl.ffb.mechanics.TtmMechanic.
pub trait TtmMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::TTM }

    fn find_throwable_team_mates<'a>(&self, game: &'a Game, thrower: &Player) -> Vec<&'a Player>;
    fn can_be_thrown(&self, game: &Game, player: &Player) -> bool;
    fn can_be_kicked(&self, game: &Game, player: &Player) -> bool;
    fn minimum_roll(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32;
    fn modifier_sum(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32;
    fn is_valid_end_scatter_coordinate(&self, game: &Game, coordinate: FieldCoordinate) -> bool;
    fn handle_kick_like_throw(&self) -> bool;
    fn is_ktm_available(&self, turn_data: &TurnData) -> bool;
    fn can_throw(&self, game: &Game, player: &Player) -> bool;
    fn is_ttm_available(&self, turn_data: &TurnData) -> bool;
    fn find_kickable_team_mates<'a>(&self, game: &'a Game, kicker: &Player) -> Vec<&'a Player>;
}
