use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::jump_mechanic::JumpMechanic as JumpMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.JumpMechanic.
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

    fn can_still_jump(&self, _game: &Game, acting_player: &ActingPlayer) -> bool {
        // TODO: UtilCards::has_unused_skill_with_property(acting_player, NamedProperties.canLeap)
        let _ = acting_player;
        false
    }

    fn can_jump(&self, _game: &Game, player: &Player, _coordinate: FieldCoordinate) -> bool {
        player.has_skill_property(NamedProperties::CAN_LEAP)
    }

    fn is_valid_jump(&self, _game: &Game, _player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        from != to && to.distance_in_steps(from) < 3
    }
}
