use std::collections::HashSet;
use ffb_model::enums::PassingDistance;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{Game, Player, TurnData};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::PassModifier;
use crate::ttm_mechanic::TtmMechanic as TtmMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.TtmMechanic.
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
        let team = game.active_team();
        UtilPlayer::find_adjacent_players_with_tacklezones(game, team, thrower_coord, false)
            .into_iter()
            .filter_map(|id| game.player(id))
            .filter(|p| p.can_be_thrown())
            .collect()
    }

    fn can_be_thrown(&self, game: &Game, player: &Player) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        player.can_be_thrown()
            && player_state.has_tacklezones()
            && game.is_active_team_player(&player.id)
    }

    fn can_be_kicked(&self, game: &Game, player: &Player) -> bool {
        player.has_skill_property(NamedProperties::CAN_BE_KICKED)
            && game.is_active_team_player(&player.id)
    }

    fn minimum_roll(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32 {
        (2 + self.modifier_sum(distance, modifiers)).max(2)
    }

    fn modifier_sum(&self, distance: PassingDistance, modifiers: &HashSet<PassModifier>) -> i32 {
        let modifier_total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        modifier_total - distance.modifier_2016()
    }

    fn is_valid_end_scatter_coordinate(&self, game: &Game, coordinate: FieldCoordinate) -> bool {
        game.field_model.player_at(coordinate).is_none()
    }

    fn handle_kick_like_throw(&self) -> bool {
        false
    }

    fn is_ktm_available(&self, turn_data: &TurnData) -> bool {
        !turn_data.blitz_used
    }

    fn can_throw(&self, _game: &Game, player: &Player) -> bool {
        player.has_skill_property(NamedProperties::CAN_THROW_TEAM_MATES)
    }

    fn is_ttm_available(&self, turn_data: &TurnData) -> bool {
        !turn_data.pass_used
    }

    fn find_kickable_team_mates<'a>(&self, game: &'a Game, kicker: &Player) -> Vec<&'a Player> {
        let kicker_coord = match game.field_model.player_coordinate(&kicker.id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let team = game.active_team();
        UtilPlayer::find_adjacent_players_with_tacklezones(game, team, kicker_coord, false)
            .into_iter()
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
        // QuickPass modifier_2016=1, modifier_sum=0-1=-1, max(2,2-1)=2
        assert_eq!(TtmMechanic.minimum_roll(PassingDistance::QuickPass, &HashSet::new()), 2);
    }

    #[test]
    fn minimum_roll_long_bomb_no_modifiers() {
        // LongBomb modifier_2016=-2, modifier_sum=0-(-2)=2, max(2,4)=4
        assert_eq!(TtmMechanic.minimum_roll(PassingDistance::LongBomb, &HashSet::new()), 4);
    }

    #[test]
    fn handle_kick_like_throw_is_false() {
        assert!(!TtmMechanic.handle_kick_like_throw());
    }

    #[test]
    fn is_ttm_available_when_pass_not_used() {
        assert!(TtmMechanic.is_ttm_available(&TurnData::new()));
    }

    #[test]
    fn is_ttm_not_available_when_pass_used() {
        let mut td = TurnData::new();
        td.pass_used = true;
        assert!(!TtmMechanic.is_ttm_available(&td));
    }

    #[test]
    fn is_ktm_available_when_blitz_not_used() {
        assert!(TtmMechanic.is_ktm_available(&TurnData::new()));
    }

    #[test]
    fn is_ktm_not_available_when_blitz_used() {
        let mut td = TurnData::new();
        td.blitz_used = true;
        assert!(!TtmMechanic.is_ktm_available(&td));
    }
}
