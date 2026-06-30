use std::collections::HashSet;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player, Team};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.mixed.OnTheBallMechanic.
pub struct OnTheBallMechanic;

impl OnTheBallMechanic {
    pub fn new() -> Self { Self }
}

impl Default for OnTheBallMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for OnTheBallMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::ON_THE_BALL }
}

impl OnTheBallMechanicTrait for OnTheBallMechanic {
    fn find_pass_blockers(&self, game: &Game, team: &Team, _check_can_reach: bool) -> HashSet<String> {
        let mut pass_blockers = HashSet::new();
        for player in &team.players {
            let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
            if player.has_skill_property(NamedProperties::CAN_MOVE_WHEN_OPPONENT_PASSES) && player_state.has_tacklezones() {
                pass_blockers.insert(player.id.clone());
            }
        }
        pass_blockers
    }

    fn valid_pass_block_move(&self, _game: &Game, acting_player: &ActingPlayer, _from_coordinate: FieldCoordinate, _to_coordinate: FieldCoordinate, _valid_pass_block_coordinates: &HashSet<FieldCoordinate>, _can_still_jump: bool, distance: i32) -> bool {
        distance + acting_player.current_move <= 3
    }

    fn display_string_pass_interference(&self) -> String {
        "On The Ball".to_string()
    }

    fn pass_interference_dialog_description(&self) -> Vec<String> {
        vec!["You may move your players with ON THE BALL skill up to 3 squares.".to_string()]
    }

    fn pass_interference_status_description(&self) -> String {
        "Waiting for coach to move players with \"On The Ball\".".to_string()
    }

    fn display_string_kick_off_interference(&self) -> String {
        self.display_string_pass_interference()
    }

    fn has_reached_valid_position(&self, _game: &Game, _player: &Player) -> bool {
        true
    }
}
