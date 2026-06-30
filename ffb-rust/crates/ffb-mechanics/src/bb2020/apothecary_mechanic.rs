use ffb_model::enums::{ApothecaryType, PlayerState, PlayerType};
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::apothecary_mechanic::ApothecaryMechanic as ApothecaryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.ApothecaryMechanic.
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
    fn apothecary_types(&self, game: &Game, defender: &Player, player_state: PlayerState) -> Vec<ApothecaryType> {
        let mut valid_types = Vec::new();
        if defender.is_zapped() || defender.player_type == PlayerType::Star {
            return valid_types;
        }
        let turn_data = if game.team_home.player(&defender.id).is_some() {
            &game.turn_data_home
        } else {
            &game.turn_data_away
        };
        let team_has_wandering_apo = turn_data.wandering_apothecaries > 0;
        let team_can_use_plague_doctor = turn_data.plague_doctors > 0 && player_state.is_ko();
        if defender.player_type == PlayerType::Mercenary {
            if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
        } else if defender.is_journeyman() {
            if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
            if team_can_use_plague_doctor {
                valid_types.push(ApothecaryType::Plague);
            }
        } else {
            if turn_data.apothecaries > turn_data.wandering_apothecaries {
                valid_types.push(ApothecaryType::Team);
            } else if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
            if team_can_use_plague_doctor {
                valid_types.push(ApothecaryType::Plague);
            }
        }
        valid_types
    }
}
