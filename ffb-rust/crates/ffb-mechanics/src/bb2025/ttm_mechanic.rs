use std::collections::HashSet;
use ffb_model::enums::PassingDistance;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{Game, Player, TurnData};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::PassModifier;
use crate::ttm_mechanic::TtmMechanic as TtmMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.TtmMechanic.
pub struct TtmMechanic;

impl TtmMechanic {
    pub fn new() -> Self { Self }
}

impl Default for TtmMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for TtmMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::TTM }
}

impl TtmMechanicTrait for TtmMechanic {
    fn find_throwable_team_mates<'a>(&self, game: &'a Game, thrower: &Player) -> Vec<&'a Player> {
        let thrower_coord = match game.field_model.player_coordinate(&thrower.id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        thrower_coord.neighbours().iter()
            .filter_map(|&adj| game.field_model.player_at(adj))
            .filter_map(|id| game.player(id))
            .filter(|p| self.can_be_thrown(game, p))
            .collect()
    }

    fn can_be_thrown(&self, game: &Game, player: &Player) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        player.can_be_thrown()
            && !player_state.is_pinned()
            && game.is_active_team_player(&player.id)
    }

    fn can_be_kicked(&self, game: &Game, player: &Player) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        player.can_be_thrown()
            && !player_state.is_pinned()
            && game.is_active_team_player(&player.id)
    }

    fn minimum_roll(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32 {
        2 + self.modifier_sum(distance, modifiers)
    }

    fn modifier_sum(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32 {
        let modifier_total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        modifier_total + distance.modifier_2020()
    }

    fn is_valid_end_scatter_coordinate(&self, _game: &Game, _coordinate: FieldCoordinate) -> bool {
        true
    }

    fn handle_kick_like_throw(&self) -> bool {
        true
    }

    fn is_ktm_available(&self, turn_data: &TurnData) -> bool {
        !turn_data.ktm_used
    }

    fn can_throw(&self, game: &Game, player: &Player) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        let can_declare = !player_state.is_prone()
            || game.options.is_enabled("allowSpecialActionsFromProne");
        can_declare
            && player.has_skill_property(NamedProperties::CAN_THROW_TEAM_MATES)
            && player.strength_with_modifiers() >= 5
    }

    fn is_ttm_available(&self, turn_data: &TurnData) -> bool {
        !turn_data.ttm_used
    }

    fn find_kickable_team_mates<'a>(&self, game: &'a Game, kicker: &Player) -> Vec<&'a Player> {
        let kicker_coord = match game.field_model.player_coordinate(&kicker.id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        kicker_coord.neighbours().iter()
            .filter_map(|&adj| game.field_model.player_at(adj))
            .filter_map(|id| game.player(id))
            .filter(|p| self.can_be_kicked(game, p))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::enums::PassingDistance;
    use ffb_model::model::TurnData;
    use crate::ttm_mechanic::TtmMechanic as TtmTrait;

    #[test]
    fn minimum_roll_quick_pass_no_modifiers() {
        // QuickPass modifier_2020=0, modifier_sum=0, min=2
        assert_eq!(TtmMechanic.minimum_roll(PassingDistance::QuickPass, &HashSet::new()), 2);
    }

    #[test]
    fn minimum_roll_long_bomb_no_modifiers() {
        // LongBomb modifier_2020=3, modifier_sum=3, min=5
        assert_eq!(TtmMechanic.minimum_roll(PassingDistance::LongBomb, &HashSet::new()), 5);
    }

    #[test]
    fn handle_kick_like_throw_is_true() {
        assert!(TtmMechanic.handle_kick_like_throw());
    }

    #[test]
    fn is_ttm_available_when_ttm_not_used() {
        assert!(TtmMechanic.is_ttm_available(&TurnData::new()));
    }

    #[test]
    fn is_ttm_not_available_when_ttm_used() {
        let mut td = TurnData::new();
        td.ttm_used = true;
        assert!(!TtmMechanic.is_ttm_available(&td));
    }

    #[test]
    fn is_ktm_available_when_ktm_not_used() {
        assert!(TtmMechanic.is_ktm_available(&TurnData::new()));
    }

    #[test]
    fn is_ktm_not_available_when_ktm_used() {
        let mut td = TurnData::new();
        td.ktm_used = true;
        assert!(!TtmMechanic.is_ktm_available(&td));
    }
}
