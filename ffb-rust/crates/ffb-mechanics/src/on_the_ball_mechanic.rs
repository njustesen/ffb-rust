use std::collections::HashSet;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player, Team};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.OnTheBallMechanic.
pub trait OnTheBallMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::ON_THE_BALL }

    fn find_pass_blockers(&self, game: &Game, team: &Team, check_can_reach: bool) -> HashSet<String>;
    fn valid_pass_block_move(&self, game: &Game, acting_player: &ActingPlayer, from_coordinate: FieldCoordinate, to_coordinate: FieldCoordinate, valid_pass_block_coordinates: &HashSet<FieldCoordinate>, can_still_jump: bool, distance: i32) -> bool;
    fn display_string_pass_interference(&self) -> String;
    fn pass_interference_dialog_description(&self) -> Vec<String>;
    fn pass_interference_status_description(&self) -> String;
    fn display_string_kick_off_interference(&self) -> String;
    fn has_reached_valid_position(&self, game: &Game, player: &Player) -> bool;
}
