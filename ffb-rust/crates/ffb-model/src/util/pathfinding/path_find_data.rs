use std::collections::HashMap;
use crate::types::{FieldCoordinate, FIELD_WIDTH, FIELD_HEIGHT};
use crate::util::pathfinding::path_find_state::PathFindState;
use crate::util::pathfinding::path_find_node::PathFindNode;

/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFindData`.
///
/// Holds the per-state node grids used during A* path finding.
pub struct PathFindData {
    /// Java: `Hashtable<PathFindState, PathFindNode[][]>`
    nodes: HashMap<PathFindState, Vec<Vec<Option<Box<PathFindNode>>>>>,
}

impl PathFindData {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        for state in [PathFindState::NORMAL, PathFindState::HAS_JUMPED] {
            let grid = vec![vec![None::<Box<PathFindNode>>; FIELD_HEIGHT as usize]; FIELD_WIDTH as usize];
            nodes.insert(state, grid);
        }
        Self { nodes }
    }

    /// Java: `blockNode(FieldCoordinate)` — mark a coordinate as blocked in all states.
    pub fn block_node(&mut self, coord: FieldCoordinate) {
        let blocked = PathFindNode::new(PathFindState::GLOBAL, coord, 1000, false, vec![], None);
        let b = Box::new(blocked);
        for grid in self.nodes.values_mut() {
            grid[coord.x as usize][coord.y as usize] = Some(b.clone());
        }
    }

    /// Java: `isProcessed(PathFindState, int x, int y)`.
    pub fn is_processed(&self, state: PathFindState, x: i32, y: i32) -> bool {
        self.nodes.get(&state)
            .and_then(|g| g.get(x as usize))
            .and_then(|col| col.get(y as usize))
            .map(|n| n.is_some())
            .unwrap_or(false)
    }

    /// Java: `setNode(FieldCoordinate, PathFindNode)`.
    pub fn set_node(&mut self, coord: FieldCoordinate, node: PathFindNode) {
        let state = node.state;
        if let Some(grid) = self.nodes.get_mut(&state) {
            grid[coord.x as usize][coord.y as usize] = Some(Box::new(node));
        }
    }

    /// Java: `getNeighbour(PathFindState, FieldCoordinate)`.
    pub fn get_neighbour(&self, state: PathFindState, coord: FieldCoordinate) -> Option<&PathFindNode> {
        self.nodes.get(&state)
            .and_then(|g| g.get(coord.x as usize))
            .and_then(|col| col.get(coord.y as usize))
            .and_then(|n| n.as_deref())
    }
}

impl Default for PathFindData {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_processed_nodes() {
        let data = PathFindData::new();
        assert!(!data.is_processed(PathFindState::NORMAL, 0, 0));
    }

    #[test]
    fn set_and_get_node() {
        let mut data = PathFindData::new();
        let coord = FieldCoordinate::new(3, 4);
        let node = PathFindNode::new(PathFindState::NORMAL, coord, 2, false, vec![], None);
        data.set_node(coord, node);
        assert!(data.is_processed(PathFindState::NORMAL, 3, 4));
        let n = data.get_neighbour(PathFindState::NORMAL, coord).unwrap();
        assert_eq!(n.get_distance(), 2);
    }
}
