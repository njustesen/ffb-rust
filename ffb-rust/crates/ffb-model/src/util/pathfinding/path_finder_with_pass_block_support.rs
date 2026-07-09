use std::collections::{BinaryHeap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Reverse;
use crate::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::enums::TurnMode;
use crate::model::game::Game;
use crate::model::player::Player;
use crate::model::property::named_properties::NamedProperties;
use super::path_find_context::{Builder as ContextBuilder, PathFindContext};
use super::path_find_data::PathFindData;
use super::path_find_node::{PathFindNode, NodeRef};
use super::path_find_state::PathFindState;

/// Min-heap entry wrapping a NodeRef; ordering by weight at insertion time.
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
        // Min-heap: smaller weight → higher priority → reverse comparison
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

pub struct PathFinderWithPassBlockSupport {
    normal_move_context: PathFindContext,
    pass_block_context: PathFindContext,
}

impl PathFinderWithPassBlockSupport {
    pub fn new() -> Self {
        PathFinderWithPassBlockSupport {
            normal_move_context: ContextBuilder::new().block_tacklezones().build(),
            pass_block_context: ContextBuilder::new().allow_exit_endzone_with_ball().allow_jump().build(),
        }
    }

    /// Gets the shortest path from start to any of the target squares.
    /// Java: private getShortestPath(Game, FieldCoordinate, Set<FieldCoordinate>, int, Team, PathFindContext, boolean)
    fn get_shortest_path_impl(
        &self,
        game: &Game,
        start: FieldCoordinate,
        end_coords: &HashSet<FieldCoordinate>,
        max_distance: i32,
        moving_team_id: &str,
        context: &PathFindContext,
        can_jump: bool,
    ) -> Option<Vec<FieldCoordinate>> {
        if !self.is_on_field(game, start) || !self.is_on_field_set(game, end_coords) {
            return None;
        }

        let field_model = &game.field_model;

        let endzone_bounds = if moving_team_id == game.team_home.id {
            FieldCoordinateBounds::ENDZONE_AWAY
        } else {
            FieldCoordinateBounds::ENDZONE_HOME
        };

        let mut open_set: BinaryHeap<HeapEntry> = BinaryHeap::new();
        let mut data = PathFindData::new();
        let mut closed_set: HashSet<FieldCoordinate> = HashSet::new();

        // Initialise with start node
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

        // Handle ball square (mark as TZ so player can enter but not pass through)
        if let Some(ball_coord) = field_model.ball_coordinate {
            if self.is_on_field(game, ball_coord)
                && context.is_block_tacklezones()
                && field_model.ball_in_play
            {
                let ball_node = Rc::new(RefCell::new(PathFindNode::new(
                    PathFindState::NORMAL,
                    ball_coord,
                    1000,
                    true,
                    Some(end_coords.clone()),
                    None,
                )));
                data.set_node(ball_coord, ball_node);
            }
        }

        // Treat trapdoors as TZ
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

        let has_ball = field_model.ball_coordinate.map_or(false, |bc| bc == start);

        // Block off squares with players
        let player_coords: Vec<FieldCoordinate> = field_model
            .player_coordinates
            .values()
            .copied()
            .collect();
        for p_coord in player_coords {
            if !self.is_on_field(game, p_coord) {
                continue;
            }
            let blocked = data.block_node(p_coord);
            closed_set.insert(blocked.borrow().get_coord());

            if context.is_block_tacklezones() {
                // Mark tackle zones of opponents
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

                // Don't allow an initial dodge from the start square
                if p_coord.is_adjacent(start) {
                    return None;
                }

                let tz_coords =
                    self.find_adjacent_coordinates(p_coord, FieldCoordinateBounds::FIELD, 1);
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
            // Lazy deletion: skip stale entries
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

            let search_distance = if can_jump
                && current_state != PathFindState::HAS_JUMPED
                && context.is_allow_jump()
                && max_distance - current_dist > 1
            {
                2
            } else {
                1
            };

            let neighbours =
                self.find_adjacent_coordinates(current_coord, FieldCoordinateBounds::FIELD, search_distance);

            for neighbour_coord in neighbours {
                let distance = current_coord.distance_in_steps(neighbour_coord);

                // Skip invalid jumps
                if distance > 1
                    && (max_distance - current_dist - distance < 0
                        || current_state == PathFindState::HAS_JUMPED
                        || !context.is_allow_jump()
                        || !can_jump)
                {
                    continue;
                }

                let neighbour_state = if distance == 1 {
                    current_state
                } else {
                    PathFindState::HAS_JUMPED
                };

                let neighbour = data.get_neighbour(neighbour_state, neighbour_coord);

                // Skip closed or TZ (non-target) nodes
                if let Some(ref nb) = neighbour {
                    let nb_ref = nb.borrow();
                    let is_closed = closed_set.contains(&nb_ref.get_coord());
                    let is_tz_block = nb_ref.is_tz() && !end_coords.contains(&neighbour_coord);
                    if is_closed || is_tz_block {
                        continue;
                    }
                }

                // Don't exit endzone with ball
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
                            // Found a shorter path: update and re-add (lazy deletion handles old entry)
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

    /// Java: public FieldCoordinate[] getShortestPath(Game, FieldCoordinate)
    pub fn get_shortest_path_to_coord(
        &self,
        game: &Game,
        end_coord: FieldCoordinate,
    ) -> Option<Vec<FieldCoordinate>> {
        let acting_player_id = game.acting_player.player_id.as_deref()?;
        let player = game.player(acting_player_id)?;

        let mut end_coords = HashSet::new();
        end_coords.insert(end_coord);

        self.get_shortest_path_for_player(game, &end_coords, player, game.acting_player.current_move)
    }

    /// Java: public FieldCoordinate[] getShortestPathToPlayer(Game, Player)
    pub fn get_shortest_path_to_player(
        &self,
        game: &Game,
        target_player: &Player,
    ) -> Option<Vec<FieldCoordinate>> {
        let target_coord = game.field_model.player_coordinate(&target_player.id)?;
        let adjacent = self.find_adjacent_coordinates(target_coord, FieldCoordinateBounds::FIELD, 1);
        let end_coords: HashSet<FieldCoordinate> = adjacent
            .into_iter()
            .filter(|c| game.field_model.player_at(*c).is_none())
            .collect();

        let acting_player_id = game.acting_player.player_id.as_deref()?;
        let acting_player = game.player(acting_player_id)?;
        let start = game.field_model.player_coordinate(&acting_player.id)?;
        let max_distance =
            acting_player.movement_with_modifiers() - game.acting_player.current_move;
        let moving_team_id = game.player_team_id(&acting_player.id)?.to_string();

        self.get_shortest_path_impl(game, start, &end_coords, max_distance, &moving_team_id, &self.normal_move_context, false)
    }

    /// Java: public FieldCoordinate[] getShortestPath(Game, Set<FieldCoordinate>, Player, int)
    pub fn get_shortest_path_for_player(
        &self,
        game: &Game,
        end_coords: &HashSet<FieldCoordinate>,
        player: &Player,
        current_move: i32,
    ) -> Option<Vec<FieldCoordinate>> {
        let moving_team_id = game.player_team_id(&player.id)?.to_string();
        let max_distance = player.movement_with_modifiers() - current_move;
        let start = game.field_model.player_coordinate(&player.id)?;
        self.get_shortest_path_impl(game, start, end_coords, max_distance, &moving_team_id, &self.normal_move_context, false)
    }

    /// Java: public FieldCoordinate[] allowPassBlockMove(Game, Player, FieldCoordinate, int, boolean, Set)
    pub fn allow_pass_block_move(
        &self,
        game: &Game,
        pass_blocker: &Player,
        start_position: FieldCoordinate,
        distance: i32,
        can_jump: bool,
        valid_end_coordinates: &HashSet<FieldCoordinate>,
    ) -> Vec<FieldCoordinate> {
        if !pass_blocker.has_skill_property(NamedProperties::CAN_MOVE_WHEN_OPPONENT_PASSES) {
            return Vec::new();
        }
        let moving_team_id = match game.player_team_id(&pass_blocker.id) {
            Some(tid) => tid.to_string(),
            None => return Vec::new(),
        };
        self.get_shortest_path_impl(
            game,
            start_position,
            valid_end_coordinates,
            distance,
            &moving_team_id,
            &self.pass_block_context,
            can_jump,
        )
        .unwrap_or_default()
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

    /// Java: fieldModel.findAdjacentCoordinates(coord, bounds, distance, false)
    /// Returns all squares within Chebyshev distance 1..=distance of coord within bounds.
    fn find_adjacent_coordinates(
        &self,
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

impl Default for PathFinderWithPassBlockSupport {
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
        let _pf = PathFinderWithPassBlockSupport::new();
    }

    #[test]
    fn find_adjacent_distance_1_returns_up_to_8() {
        let pf = PathFinderWithPassBlockSupport::new();
        let adj = pf.find_adjacent_coordinates(fc(5, 5), FieldCoordinateBounds::FIELD, 1);
        assert_eq!(adj.len(), 8);
    }

    #[test]
    fn find_adjacent_distance_2_returns_up_to_24() {
        let pf = PathFinderWithPassBlockSupport::new();
        let adj = pf.find_adjacent_coordinates(fc(5, 5), FieldCoordinateBounds::FIELD, 2);
        assert_eq!(adj.len(), 24);
    }

    #[test]
    fn find_adjacent_clips_at_field_edge() {
        let pf = PathFinderWithPassBlockSupport::new();
        let adj = pf.find_adjacent_coordinates(fc(0, 0), FieldCoordinateBounds::FIELD, 1);
        // Corner square: only 3 neighbours are in bounds
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn get_shortest_path_to_coord_returns_none_without_acting_player() {
        let pf = PathFinderWithPassBlockSupport::new();
        let game = empty_game();
        let result = pf.get_shortest_path_to_coord(&game, fc(5, 5));
        assert!(result.is_none());
    }

    #[test]
    fn allow_pass_block_move_returns_empty_without_skill() {
        let pf = PathFinderWithPassBlockSupport::new();
        let game = empty_game();
        let player = crate::model::player::Player {
            id: "p1".into(),
            movement: 6,
            ..Default::default()
        };
        let result = pf.allow_pass_block_move(&game, &player, fc(5, 5), 3, false, &HashSet::new());
        assert!(result.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_move_blocks_tacklezones() {
        let pf = PathFinderWithPassBlockSupport::new();
        assert!(pf.normal_move_context.is_block_tacklezones());
        assert!(!pf.normal_move_context.is_allow_jump());
    }

    #[test]
    fn pass_block_allows_jump_and_endzone_exit() {
        let pf = PathFinderWithPassBlockSupport::new();
        assert!(pf.pass_block_context.is_allow_jump());
        assert!(pf.pass_block_context.is_allow_exit_endzone_with_ball());
    }
}
