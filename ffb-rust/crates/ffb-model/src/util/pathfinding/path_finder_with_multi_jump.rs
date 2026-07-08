use std::collections::{BinaryHeap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::enums::TurnMode;
use crate::model::game::Game;
use crate::model::player::Player;
use crate::model::property::named_properties::NamedProperties;
use super::path_find_context::{Builder as ContextBuilder, PathFindContext};
use super::path_find_data::PathFindData;
use super::path_find_node::{PathFindNode, NodeRef};
use super::path_find_state::PathFindState;
use super::path_finder_extension::PathFinderExtension;

struct HeapEntry {
    distance_at_insert: i32,
    node: NodeRef,
}

impl HeapEntry {
    fn current_weight(&self) -> i32 {
        self.node.borrow().get_weight()
    }
    fn current_non_diag(&self) -> i32 {
        self.node.borrow().get_non_diagonal_weight()
    }
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.current_weight() == other.current_weight()
            && self.current_non_diag() == other.current_non_diag()
    }
}
impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.current_weight()
            .cmp(&self.current_weight())
            .then(other.current_non_diag().cmp(&self.current_non_diag()))
    }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PathFinderWithMultiJump {
    theoretical_range_context: PathFindContext,
    extension: PathFinderExtension,
}

impl PathFinderWithMultiJump {
    pub fn new() -> Self {
        PathFinderWithMultiJump {
            theoretical_range_context: ContextBuilder::new().allow_jump().build(),
            extension: PathFinderExtension::new(),
        }
    }

    /// Java: private getShortestPath(Game, FieldCoordinate, Set, int, PathFindContext, Player)
    fn get_shortest_path_impl(
        &self,
        game: &Game,
        start: FieldCoordinate,
        end_coords: &HashSet<FieldCoordinate>,
        max_distance: i32,
        context: &PathFindContext,
        player: &Player,
    ) -> Option<Vec<FieldCoordinate>> {
        if !self.is_on_field(game, start) || !self.is_on_field_set(game, end_coords) {
            return None;
        }

        let can_jump_over_standing = player.has_skill_property(NamedProperties::CAN_LEAP);
        let moving_team_id = game.player_team_id(&player.id)?.to_string();

        let field_model = &game.field_model;

        let endzone_bounds = if moving_team_id == game.team_home.id {
            FieldCoordinateBounds::ENDZONE_AWAY
        } else {
            FieldCoordinateBounds::ENDZONE_HOME
        };

        let mut open_set: BinaryHeap<HeapEntry> = BinaryHeap::new();
        let mut data = PathFindData::new();
        // Java: Set<FieldCoordinate> closedSet — tracked by coordinate
        let mut closed_set: HashSet<FieldCoordinate> = HashSet::new();

        let start_node = Rc::new(RefCell::new(PathFindNode::new(
            PathFindState::NORMAL,
            start,
            0,
            false,
            Some(end_coords.clone()),
            None,
        )));
        data.set_node(start, Rc::clone(&start_node));
        open_set.push(HeapEntry { distance_at_insert: 0, node: start_node });

        let ball_coord = field_model.ball_coordinate;

        if context.is_block_ball() {
            if let Some(bc) = ball_coord {
                if self.is_on_field(game, bc) && context.is_block_tacklezones() && field_model.ball_in_play {
                    let ball_node = Rc::new(RefCell::new(PathFindNode::new(
                        PathFindState::NORMAL,
                        bc,
                        1000,
                        true,
                        Some(end_coords.clone()),
                        None,
                    )));
                    data.set_node(bc, ball_node);
                }
            }
        }

        if context.is_block_trapdoors() {
            for &trap_coord in &field_model.trap_doors {
                let trap_node = Rc::new(RefCell::new(PathFindNode::new(
                    PathFindState::NORMAL,
                    trap_coord,
                    1000,
                    true,
                    Some(end_coords.clone()),
                    None,
                )));
                data.set_node(trap_coord, trap_node);
            }
        }

        let has_ball = ball_coord.map_or(false, |bc| bc == start);

        let player_coords: Vec<FieldCoordinate> =
            field_model.player_coordinates.values().copied().collect();
        for p_coord in player_coords {
            if !self.is_on_field(game, p_coord) {
                continue;
            }
            let blocked = data.block_node(p_coord);
            closed_set.insert(blocked.borrow().get_coord());

            if context.is_block_tacklezones() {
                let player_id = match field_model.player_at(p_coord) {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let player_team_id = match game.player_team_id(&player_id) {
                    Some(tid) => tid.to_string(),
                    None => continue,
                };
                if player_team_id == moving_team_id {
                    continue;
                }

                let state = match field_model.player_state(&player_id) {
                    Some(s) => s,
                    None => continue,
                };
                if !state.has_tacklezones() {
                    continue;
                }

                if p_coord.is_adjacent(start) {
                    return None;
                }

                let tz_coords =
                    Self::find_adjacent_coordinates(p_coord, FieldCoordinateBounds::FIELD, 1);
                for tz_coord in tz_coords {
                    if data.is_processed(PathFindState::NORMAL, tz_coord.x, tz_coord.y) {
                        continue;
                    }
                    let tz_node = Rc::new(RefCell::new(PathFindNode::new(
                        PathFindState::NORMAL,
                        tz_coord,
                        1000,
                        true,
                        Some(end_coords.clone()),
                        None,
                    )));
                    data.set_node(tz_coord, tz_node);
                }
            }
        }

        while let Some(entry) = open_set.pop() {
            if entry.node.borrow().get_distance() != entry.distance_at_insert {
                continue;
            }

            let current = entry.node;
            let current_dist = current.borrow().get_distance();
            let current_coord = current.borrow().get_coord();
            let current_state = current.borrow().get_state();

            if current_dist > max_distance {
                return None;
            }

            if end_coords.contains(&current_coord) {
                return Some(Self::reconstruct_path(&current.borrow()));
            }

            closed_set.insert(current_coord);

            let is_in_endzone = endzone_bounds.is_in_bounds(current_coord);

            let search_distance =
                if context.is_allow_jump() && max_distance - current_dist > 1 { 2 } else { 1 };

            let neighbours =
                Self::find_adjacent_coordinates(current_coord, FieldCoordinateBounds::FIELD, search_distance);

            for neighbour_coord in neighbours {
                let distance = current_coord.distance_in_steps(neighbour_coord);

                if distance > 1 {
                    let downed_on_path = self.extension.has_prone_or_stunned_player_on_path(
                        game,
                        current_coord,
                        neighbour_coord,
                    );
                    let mut path_squares = self.extension.find_possible_path_squares(current_coord, neighbour_coord);
                    path_squares.retain(|c| !closed_set.contains(c));

                    if max_distance - current_dist - distance < 0
                        || !context.is_allow_jump()
                        || (!can_jump_over_standing && !downed_on_path)
                        || !path_squares.is_empty()
                    {
                        continue;
                    }
                }

                let neighbour_state = if distance == 1 {
                    current_state
                } else {
                    PathFindState::HAS_JUMPED
                };

                let neighbour = data.get_neighbour(neighbour_state, neighbour_coord);

                if let Some(ref nb) = neighbour {
                    let nb_ref = nb.borrow();
                    let is_closed = closed_set.contains(&nb_ref.get_coord());
                    let is_tz_block = nb_ref.is_tz() && !end_coords.contains(&neighbour_coord);
                    if is_closed || is_tz_block {
                        continue;
                    }
                }

                if !context.is_allow_exit_endzone_with_ball()
                    && has_ball
                    && is_in_endzone
                    && !endzone_bounds.is_in_bounds(neighbour_coord)
                {
                    continue;
                }

                let new_dist = current_dist + distance;

                match neighbour {
                    None => {
                        let nb_node = Rc::new(RefCell::new(PathFindNode::new(
                            neighbour_state,
                            neighbour_coord,
                            new_dist,
                            false,
                            Some(end_coords.clone()),
                            Some(Rc::clone(&current)),
                        )));
                        data.set_node(neighbour_coord, Rc::clone(&nb_node));
                        open_set.push(HeapEntry { distance_at_insert: new_dist, node: nb_node });
                    }
                    Some(nb) => {
                        if new_dist < nb.borrow().get_distance() {
                            nb.borrow_mut().set_source_with_state(Rc::clone(&current), distance, neighbour_state);
                            open_set.push(HeapEntry { distance_at_insert: new_dist, node: nb });
                        }
                    }
                }
            }
        }

        None
    }

    fn reconstruct_path(end: &PathFindNode) -> Vec<FieldCoordinate> {
        let distance = end.get_distance() as usize;
        let mut list = Vec::with_capacity(distance);
        let mut current_parent = end.get_parent();
        let mut coord = end.get_coord();

        loop {
            match current_parent {
                None => break,
                Some(parent_ref) => {
                    list.push(coord);
                    let parent = parent_ref.borrow();
                    current_parent = parent.get_parent();
                    coord = parent.get_coord();
                }
            }
        }

        list.reverse();
        list
    }

    /// Java: public FieldCoordinate[] getPathToBlitzTarget(Game, Player targetPlayer)
    pub fn get_path_to_blitz_target(
        &self,
        game: &Game,
        target_player: &Player,
    ) -> Option<Vec<FieldCoordinate>> {
        let target_coord = game.field_model.player_coordinate(&target_player.id)?;
        let adjacent = Self::find_adjacent_coordinates(target_coord, FieldCoordinateBounds::FIELD, 1);
        let end_coords: HashSet<FieldCoordinate> = adjacent
            .into_iter()
            .filter(|c| game.field_model.player_at(*c).is_none())
            .collect();

        let acting_player_id = game.acting_player.player_id.as_deref()?;
        let acting_player = game.player(acting_player_id)?;
        let start = game.field_model.player_coordinate(&acting_player.id)?;

        let mut max_distance =
            acting_player.movement_with_modifiers() - game.acting_player.current_move;

        // Extra rush allowance: +2 for Sprint/SureFeet, +1 otherwise
        if acting_player.has_skill_property(NamedProperties::CAN_MAKE_AN_EXTRA_GFI) {
            max_distance += 2;
        } else {
            max_distance += 1;
        }

        self.get_shortest_path_impl(
            game,
            start,
            &end_coords,
            max_distance,
            &self.theoretical_range_context,
            acting_player,
        )
    }

    fn is_on_field(&self, game: &Game, coord: FieldCoordinate) -> bool {
        if game.turn_mode == TurnMode::KickoffReturn {
            FieldCoordinateBounds::HALF_HOME.is_in_bounds(coord)
        } else {
            FieldCoordinateBounds::FIELD.is_in_bounds(coord)
        }
    }

    fn is_on_field_set(&self, game: &Game, coords: &HashSet<FieldCoordinate>) -> bool {
        coords.iter().all(|&c| self.is_on_field(game, c))
    }

    fn find_adjacent_coordinates(
        coord: FieldCoordinate,
        bounds: FieldCoordinateBounds,
        distance: i32,
    ) -> Vec<FieldCoordinate> {
        let mut result = Vec::new();
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nc = FieldCoordinate::new(coord.x + dx, coord.y + dy);
                if bounds.is_in_bounds(nc) {
                    result.push(nc);
                }
            }
        }
        result
    }
}

impl Default for PathFinderWithMultiJump {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game::Game;
    use crate::model::team::Team;
    use crate::enums::Rules;

    fn empty_game() -> Game {
        let home = Team {
            id: "home".into(),
            name: "Home".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach1".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = Team { id: "away".into(), name: "Away".into(), ..home.clone() };
        Game::new(home, away, Rules::Bb2025)
    }

    fn fc(x: i32, y: i32) -> FieldCoordinate {
        FieldCoordinate::new(x, y)
    }

    #[test]
    fn new_creates_instance() {
        let _pf = PathFinderWithMultiJump::new();
    }

    #[test]
    fn find_adjacent_distance_1_returns_up_to_8() {
        let adj = PathFinderWithMultiJump::find_adjacent_coordinates(fc(5, 5), FieldCoordinateBounds::FIELD, 1);
        assert_eq!(adj.len(), 8);
    }

    #[test]
    fn find_adjacent_distance_2_returns_up_to_24() {
        let adj = PathFinderWithMultiJump::find_adjacent_coordinates(fc(5, 5), FieldCoordinateBounds::FIELD, 2);
        assert_eq!(adj.len(), 24);
    }

    #[test]
    fn find_adjacent_clips_at_field_edge() {
        let adj = PathFinderWithMultiJump::find_adjacent_coordinates(fc(0, 0), FieldCoordinateBounds::FIELD, 1);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn get_path_to_blitz_target_returns_none_without_acting_player() {
        let pf = PathFinderWithMultiJump::new();
        let game = empty_game();
        let target = crate::model::player::Player {
            id: "target".into(),
            ..Default::default()
        };
        let result = pf.get_path_to_blitz_target(&game, &target);
        assert!(result.is_none());
    }

    #[test]
    fn default_uses_allow_jump_context() {
        let pf = PathFinderWithMultiJump::default();
        assert!(pf.theoretical_range_context.is_allow_jump());
    }
}
