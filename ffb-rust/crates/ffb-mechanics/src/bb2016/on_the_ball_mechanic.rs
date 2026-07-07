use std::collections::HashSet;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player, Team};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.OnTheBallMechanic.
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
    fn find_pass_blockers(&self, game: &Game, team: &Team, check_can_reach: bool) -> HashSet<String> {
        let mut pass_blockers = HashSet::new();
        // TODO: get JumpMechanic from game factory (FactoryType.Factory.MECHANIC)
        let can_jump = false; // TODO: mechanic.canJump(game, player, startPosition)
        let valid_pass_block_end_coordinates = util_passing_find_valid_pass_block_end_coordinates(game);
        for player in &team.players {
            if player.has_skill_property(NamedProperties::CAN_MOVE_WHEN_OPPONENT_PASSES) {
                let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
                let start_position = game.field_model.player_coordinate(&player.id);
                if !check_can_reach || (player_state.has_tacklezones()
                    && start_position.is_some()
                    && path_finder_allow_pass_block_move_is_provided(game, player, start_position.unwrap(), 3, can_jump, &valid_pass_block_end_coordinates)) {
                    pass_blockers.insert(player.id.clone());
                }
            }
        }
        pass_blockers
    }

    fn valid_pass_block_move(&self, game: &Game, acting_player: &ActingPlayer, _from_coordinate: FieldCoordinate, to_coordinate: FieldCoordinate, valid_pass_block_coordinates: &HashSet<FieldCoordinate>, can_still_jump: bool, distance: i32) -> bool {
        let player = acting_player.player_id.as_deref()
            .and_then(|id| game.player(id));
        valid_pass_block_coordinates.contains(&to_coordinate)
            || player.map(|p| path_finder_allow_pass_block_move_is_provided(
                game,
                p,
                to_coordinate,
                3 - distance - acting_player.current_move,
                can_still_jump,
                valid_pass_block_coordinates,
            )).unwrap_or(false)
    }

    fn display_string_pass_interference(&self) -> String {
        "Pass Block".to_string()
    }

    fn pass_interference_dialog_description(&self) -> Vec<String> {
        vec![
            "You may move your players with PASS BLOCK skill up to 3 squares.".to_string(),
            "The move must end in a square where the player can intercept or put a TZ on thrower or catcher.".to_string(),
        ]
    }

    fn pass_interference_status_description(&self) -> String {
        "Waiting for coach to move pass blockers.".to_string()
    }

    fn display_string_kick_off_interference(&self) -> String {
        "Kick-Off Return".to_string()
    }

    fn has_reached_valid_position(&self, game: &Game, player: &Player) -> bool {
        let valid_end_coordinates = util_passing_find_valid_pass_block_end_coordinates(game);
        let player_coordinate = game.field_model.player_coordinate(&player.id);
        player_coordinate.map(|c| valid_end_coordinates.contains(&c)).unwrap_or(false)
    }
}

fn util_passing_find_valid_pass_block_end_coordinates(game: &Game) -> HashSet<FieldCoordinate> {
    ffb_model::util::passing::find_valid_pass_block_end_coordinates(game)
}

/// Stub for ArrayTool.isProvided(PathFinderWithPassBlockSupport.INSTANCE.allowPassBlockMove(...)).
/// Returns true if the pathfinder found at least one valid move (non-empty result).
/// TODO: full implementation requires translating PathFinderWithPassBlockSupport.
fn path_finder_allow_pass_block_move_is_provided(
    _game: &Game,
    _player: &Player,
    _from: FieldCoordinate,
    _remaining_moves: i32,
    _can_jump: bool,
    _valid_end_coordinates: &HashSet<FieldCoordinate>,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallTrait;

    #[test]
    fn display_string_pass_interference_is_pass_block() {
        assert_eq!(OnTheBallMechanic.display_string_pass_interference(), "Pass Block");
    }

    #[test]
    fn pass_interference_dialog_has_two_entries() {
        let lines = OnTheBallMechanic.pass_interference_dialog_description();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn display_string_kick_off_interference_is_kickoff_return() {
        assert_eq!(OnTheBallMechanic.display_string_kick_off_interference(), "Kick-Off Return");
    }

    #[test]
    fn pass_interference_status_description_is_nonempty() {
        assert!(!OnTheBallMechanic.pass_interference_status_description().is_empty());
    }
}
