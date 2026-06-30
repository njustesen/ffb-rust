use ffb_model::enums::{ApothecaryType, PlayerState};
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::apothecary_mechanic::ApothecaryMechanic as ApothecaryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.ApothecaryMechanic.
pub struct ApothecaryMechanic;

impl ApothecaryMechanic {
    pub fn new() -> Self { Self }
}

impl Default for ApothecaryMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for ApothecaryMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::APOTHECARY }
}

impl ApothecaryMechanicTrait for ApothecaryMechanic {
    fn apothecary_types(&self, _game: &Game, _defender: &Player, _player_state: PlayerState) -> Vec<ApothecaryType> {
        Vec::new()
    }
}
