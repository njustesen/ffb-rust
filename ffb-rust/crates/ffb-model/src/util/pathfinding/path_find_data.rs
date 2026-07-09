use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{FieldCoordinate, FIELD_WIDTH, FIELD_HEIGHT};
use super::path_find_state::PathFindState;
use super::path_find_node::{PathFindNode, NodeRef};

/// Per-coordinate node cache for both movement states.
/// Java: PathFindData with Hashtable<PathFindState, PathFindNode[][]>
pub struct PathFindData {
    normal: Vec<Vec<Option<NodeRef>>>,
    has_jumped: Vec<Vec<Option<NodeRef>>>,
}

impl PathFindData {
    pub fn new() -> Self {
        let w = FIELD_WIDTH as usize;
        let h = FIELD_HEIGHT as usize;
        PathFindData {
            normal: vec![vec![None; h]; w],
            has_jumped: vec![vec![None; h]; w],
        }
    }

    /// Creates a blocked node with distance=1000 and places it in both state slots.
    pub fn block_node(&mut self, coordinate: FieldCoordinate) -> NodeRef {
        let blocked = Rc::new(RefCell::new(PathFindNode::new(
            PathFindState::GLOBAL,
            coordinate,
            1000,
            false,
            None,
            None,
        )));
        let x = coordinate.x as usize;
        let y = coordinate.y as usize;
        self.normal[x][y] = Some(Rc::clone(&blocked));
        self.has_jumped[x][y] = Some(Rc::clone(&blocked));
        blocked
    }

    pub fn is_processed(&self, state: PathFindState, x: i32, y: i32) -> bool {
        match state {
            PathFindState::NORMAL | PathFindState::GLOBAL => {
                self.normal[x as usize][y as usize].is_some()
            }
            PathFindState::HAS_JUMPED => {
                self.has_jumped[x as usize][y as usize].is_some()
            }
        }
    }

    pub fn set_node(&mut self, coord: FieldCoordinate, node: NodeRef) {
        let state = node.borrow().get_state();
        let x = coord.x as usize;
        let y = coord.y as usize;
        match state {
            PathFindState::NORMAL | PathFindState::GLOBAL => self.normal[x][y] = Some(node),
            PathFindState::HAS_JUMPED => self.has_jumped[x][y] = Some(node),
        }
    }

    pub fn get_neighbour(&self, state: PathFindState, coord: FieldCoordinate) -> Option<NodeRef> {
        let x = coord.x as usize;
        let y = coord.y as usize;
        match state {
            PathFindState::NORMAL | PathFindState::GLOBAL => self.normal[x][y].clone(),
            PathFindState::HAS_JUMPED => self.has_jumped[x][y].clone(),
        }
    }
}

impl Default for PathFindData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fc(x: i32, y: i32) -> FieldCoordinate {
        FieldCoordinate::new(x, y)
    }

    fn make_node(state: PathFindState, coord: FieldCoordinate, dist: i32) -> NodeRef {
        Rc::new(RefCell::new(PathFindNode::new(state, coord, dist, false, None, None)))
    }

    #[test]
    fn new_data_is_empty() {
        let data = PathFindData::new();
        assert!(!data.is_processed(PathFindState::NORMAL, 0, 0));
        assert!(!data.is_processed(PathFindState::HAS_JUMPED, 5, 5));
    }

    #[test]
    fn set_node_normal_marks_processed() {
        let mut data = PathFindData::new();
        let node = make_node(PathFindState::NORMAL, fc(3, 4), 1);
        data.set_node(fc(3, 4), node);
        assert!(data.is_processed(PathFindState::NORMAL, 3, 4));
        assert!(!data.is_processed(PathFindState::HAS_JUMPED, 3, 4));
    }

    #[test]
    fn set_node_jumped_marks_processed_in_jumped_slot() {
        let mut data = PathFindData::new();
        let node = make_node(PathFindState::HAS_JUMPED, fc(1, 1), 2);
        data.set_node(fc(1, 1), node);
        assert!(!data.is_processed(PathFindState::NORMAL, 1, 1));
        assert!(data.is_processed(PathFindState::HAS_JUMPED, 1, 1));
    }

    #[test]
    fn block_node_marks_both_states() {
        let mut data = PathFindData::new();
        let _blocked = data.block_node(fc(5, 5));
        assert!(data.is_processed(PathFindState::NORMAL, 5, 5));
        assert!(data.is_processed(PathFindState::HAS_JUMPED, 5, 5));
    }

    #[test]
    fn get_neighbour_returns_set_node() {
        let mut data = PathFindData::new();
        let node = make_node(PathFindState::NORMAL, fc(2, 3), 1);
        data.set_node(fc(2, 3), Rc::clone(&node));
        let found = data.get_neighbour(PathFindState::NORMAL, fc(2, 3));
        assert!(found.is_some());
        assert_eq!(found.unwrap().borrow().get_coord(), fc(2, 3));
    }

    #[test]
    fn get_neighbour_returns_none_for_unset_coord() {
        let data = PathFindData::new();
        assert!(data.get_neighbour(PathFindState::NORMAL, fc(10, 7)).is_none());
    }

    #[test]
    fn block_node_returns_same_ref_in_both_slots() {
        let mut data = PathFindData::new();
        let blocked = data.block_node(fc(4, 4));
        let from_normal = data.get_neighbour(PathFindState::NORMAL, fc(4, 4)).unwrap();
        let from_jumped = data.get_neighbour(PathFindState::HAS_JUMPED, fc(4, 4)).unwrap();
        assert!(Rc::ptr_eq(&blocked, &from_normal));
        assert!(Rc::ptr_eq(&blocked, &from_jumped));
    }
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
