use crate::client::action_key::ActionKey;
use ffb_model::enums::Direction;
use ffb_model::model::game::Game;
use ffb_model::model::player::PlayerId;
use ffb_model::types::{FieldCoordinate, FIELD_HEIGHT, FIELD_WIDTH};

/// 1:1 translation of com.fumbbl.ffb.client.util.UtilClientActionKeys (Java class).
pub struct UtilClientActionKeys;

impl UtilClientActionKeys {
    pub fn find_move_coordinate(
        start_coordinate: FieldCoordinate,
        action_key: ActionKey,
    ) -> Option<FieldCoordinate> {
        let move_direction = Self::find_move_direction(action_key)?;
        Some(match move_direction {
            Direction::North => start_coordinate.add(0, -1),
            Direction::Northeast => start_coordinate.add(1, -1),
            Direction::East => start_coordinate.add(1, 0),
            Direction::Southeast => start_coordinate.add(1, 1),
            Direction::South => start_coordinate.add(0, 1),
            Direction::Southwest => start_coordinate.add(-1, 1),
            Direction::West => start_coordinate.add(-1, 0),
            Direction::Northwest => start_coordinate.add(-1, -1),
        })
    }

    pub fn find_move_direction(action_key: ActionKey) -> Option<Direction> {
        match action_key {
            ActionKey::PLAYER_MOVE_NORTH => Some(Direction::North),
            ActionKey::PLAYER_MOVE_NORTHEAST => Some(Direction::Northeast),
            ActionKey::PLAYER_MOVE_EAST => Some(Direction::East),
            ActionKey::PLAYER_MOVE_SOUTHEAST => Some(Direction::Southeast),
            ActionKey::PLAYER_MOVE_SOUTH => Some(Direction::South),
            ActionKey::PLAYER_MOVE_SOUTHWEST => Some(Direction::Southwest),
            ActionKey::PLAYER_MOVE_WEST => Some(Direction::West),
            ActionKey::PLAYER_MOVE_NORTHWEST => Some(Direction::Northwest),
            _ => None,
        }
    }

    pub fn cycle_player(
        game: &Game,
        start_player: Option<&PlayerId>,
        right: bool,
    ) -> Option<PlayerId> {
        let mut next_player: Option<PlayerId> = None;

        if let Some(start_player) = start_player {
            let start_player_position = game.field_model.player_coordinate(start_player)?;
            if right {
                let mut y = 0;
                while next_player.is_none() && y < FIELD_HEIGHT - start_player_position.y {
                    let mut x = 0;
                    while next_player.is_none() && x < FIELD_WIDTH - start_player_position.x - 2 {
                        if x != 0 || y != 0 {
                            next_player =
                                Self::find_selectable_home_player(game, start_player_position.add(x, y));
                        }
                        x += 1;
                    }
                    if y > 0 {
                        let mut x = -1;
                        while next_player.is_none() && x > 1 - start_player_position.x {
                            next_player =
                                Self::find_selectable_home_player(game, start_player_position.add(x, y));
                            x -= 1;
                        }
                    }
                    y += 1;
                }
            } else {
                let mut y = 0;
                while next_player.is_none() && y > -start_player_position.y {
                    let mut x = 0;
                    while next_player.is_none() && x > 1 - start_player_position.x {
                        if x != 0 || y != 0 {
                            next_player =
                                Self::find_selectable_home_player(game, start_player_position.add(x, y));
                        }
                        x -= 1;
                    }
                    if y < 0 {
                        let mut x = 1;
                        while next_player.is_none() && x < FIELD_WIDTH - start_player_position.x - 2 {
                            next_player =
                                Self::find_selectable_home_player(game, start_player_position.add(x, y));
                            x += 1;
                        }
                    }
                    y -= 1;
                }
            }
        } else {
            for player in &game.team_home.players {
                let Some(player_state) = game.field_model.player_state(&player.id) else {
                    continue;
                };
                if !player_state.is_active() {
                    continue;
                }
                match &next_player {
                    None => next_player = Some(player.id.clone()),
                    Some(current_next) => {
                        let player_coordinate = game.field_model.player_coordinate(&player.id);
                        let next_player_coordinate = game.field_model.player_coordinate(current_next);
                        if let (Some(pc), Some(npc)) = (player_coordinate, next_player_coordinate) {
                            let takes_over = if right { pc.x > npc.x } else { pc.x < npc.x };
                            if takes_over {
                                next_player = Some(player.id.clone());
                            }
                        }
                    }
                }
            }
        }

        next_player.or_else(|| start_player.cloned())
    }

    fn find_selectable_home_player(game: &Game, coordinate: FieldCoordinate) -> Option<PlayerId> {
        let player_id = game.field_model.player_at(coordinate)?;
        if !game.team_home.has_player(player_id) {
            return None;
        }
        let player_state = game.field_model.player_state(player_id)?;
        if player_state.is_active() {
            Some(player_id.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_move_direction_maps_all_eight() {
        assert_eq!(
            UtilClientActionKeys::find_move_direction(ActionKey::PLAYER_MOVE_NORTH),
            Some(Direction::North)
        );
        assert_eq!(
            UtilClientActionKeys::find_move_direction(ActionKey::PLAYER_MOVE_SOUTHWEST),
            Some(Direction::Southwest)
        );
    }

    #[test]
    fn find_move_direction_non_move_key_is_none() {
        assert_eq!(
            UtilClientActionKeys::find_move_direction(ActionKey::PLAYER_ACTION_BLOCK),
            None
        );
    }

    #[test]
    fn find_move_coordinate_north_decrements_y() {
        let start = FieldCoordinate::new(5, 5);
        let result =
            UtilClientActionKeys::find_move_coordinate(start, ActionKey::PLAYER_MOVE_NORTH).unwrap();
        assert_eq!(result, FieldCoordinate::new(5, 4));
    }

    #[test]
    fn find_move_coordinate_southeast_increments_both() {
        let start = FieldCoordinate::new(5, 5);
        let result =
            UtilClientActionKeys::find_move_coordinate(start, ActionKey::PLAYER_MOVE_SOUTHEAST).unwrap();
        assert_eq!(result, FieldCoordinate::new(6, 6));
    }

    #[test]
    fn find_move_coordinate_non_move_key_is_none() {
        let start = FieldCoordinate::new(5, 5);
        assert_eq!(
            UtilClientActionKeys::find_move_coordinate(start, ActionKey::PLAYER_ACTION_PASS),
            None
        );
    }
}
