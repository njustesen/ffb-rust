use ffb_model::enums::ApothecaryType;
use ffb_model::enums::PlayerState;
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.ApothecaryMechanic.
pub trait ApothecaryMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::APOTHECARY }

    fn apothecary_types(&self, game: &Game, defender: &Player, player_state: PlayerState) -> Vec<ApothecaryType>;
}
