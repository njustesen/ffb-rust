use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::jump_mechanic::JumpMechanic as JumpMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.JumpMechanic.
pub struct JumpMechanic;

impl JumpMechanic {
    pub fn new() -> Self { Self }
}

impl Default for JumpMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for JumpMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::JUMP }
}

impl JumpMechanicTrait for JumpMechanic {
    fn is_available_as_next_move(&self, game: &Game, acting_player: &ActingPlayer, jumping: bool) -> bool {
        self.can_still_jump(game, acting_player) && {
            // TODO: UtilPlayer::is_next_move_possible(game, jumping)
            let _ = (game, jumping);
            false
        }
    }

    fn can_still_jump(&self, game: &Game, acting_player: &ActingPlayer) -> bool {
        let player_id = match &acting_player.player_id {
            Some(id) => id,
            None => return false,
        };
        let player = match game.player(player_id) {
            Some(p) => p,
            None => return false,
        };
        let coord = game.field_model.player_coordinate(player_id);
        let can_leap = player.has_skill_property(NamedProperties::CAN_LEAP);
        let has_prone_adjacent = coord.map(|c| self.has_prone_or_stunned_players_adjacent(game, c)).unwrap_or(false);
        (can_leap || has_prone_adjacent) && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn can_jump(&self, game: &Game, player: &Player, coordinate: FieldCoordinate) -> bool {
        let can_leap = player.has_skill_property(NamedProperties::CAN_LEAP);
        let has_prone_adjacent = self.has_prone_or_stunned_players_adjacent(game, coordinate);
        (can_leap || has_prone_adjacent) && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn is_valid_jump(&self, game: &Game, player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        from != to
            && to.distance_in_steps(from) == 2
            && (player.has_skill_property(NamedProperties::CAN_LEAP) || {
                // TODO: PathFinderExtension::has_prone_or_stunned_player_on_path(game, from, to)
                let _ = game;
                false
            })
    }
}

impl JumpMechanic {
    fn has_prone_or_stunned_players_adjacent(&self, game: &Game, coordinate: FieldCoordinate) -> bool {
        // TODO: FieldCoordinateBounds::FIELD adjacency filtering — uses full findAdjacentCoordinates
        coordinate.neighbours().iter().any(|&adj| {
            if let Some(id) = game.field_model.player_at(adj) {
                if let Some(state) = game.field_model.player_state(id) {
                    return state.is_prone_or_stunned() || state.is_stunned();
                }
            }
            false
        })
    }
}
