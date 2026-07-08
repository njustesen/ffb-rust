use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use crate::types::FieldCoordinate;
use super::path_find_state::PathFindState;

/// Shared-ownership handle — mirrors Java object references.
pub type NodeRef = Rc<RefCell<PathFindNode>>;

/// A node in the A* pathfinding search.
/// Java: PathFindNode implements Comparable<PathFindNode>
#[derive(Debug)]
pub struct PathFindNode {
    pub(super) state: PathFindState,
    pub(super) distance: i32,
    estimate: i32,
    tz: bool,
    coord: FieldCoordinate,
    target: Option<HashSet<FieldCoordinate>>,
    parent: Option<NodeRef>,
}

impl PathFindNode {
    pub fn new(
        state: PathFindState,
        coord: FieldCoordinate,
        distance: i32,
        tz: bool,
        target: Option<HashSet<FieldCoordinate>>,
        parent: Option<NodeRef>,
    ) -> Self {
        let estimate = if let Some(ref targets) = target {
            let mut est = 1000;
            for t in targets {
                est = est.min(coord.distance_in_steps(*t));
            }
            est
        } else {
            1000
        };
        PathFindNode { state, coord, parent, target, tz, distance, estimate }
    }

    pub fn get_weight(&self) -> i32 {
        self.distance + self.estimate
    }

    pub fn get_non_diagonal_weight(&self) -> i32 {
        let mut best_weight = 10000;
        if let Some(ref targets) = self.target {
            for t in targets {
                let weight = (self.coord.x - t.x).abs() + (self.coord.y - t.y).abs();
                if weight < best_weight {
                    best_weight = weight;
                }
            }
        }
        self.distance + best_weight
    }

    pub fn compare_to(&self, other: &PathFindNode) -> std::cmp::Ordering {
        let mut this_weight = self.get_weight();
        let mut other_weight = other.get_weight();
        if this_weight == other_weight {
            this_weight = self.get_non_diagonal_weight();
            other_weight = other.get_non_diagonal_weight();
        }
        this_weight.cmp(&other_weight)
    }

    pub fn set_source(&mut self, source: NodeRef, length: i32) {
        let state = self.state;
        self.set_source_with_state(source, length, state);
    }

    pub fn set_source_with_state(&mut self, source: NodeRef, length: i32, new_state: PathFindState) {
        self.distance = source.borrow().distance + length;
        self.parent = Some(source);
        self.state = new_state;
    }

    pub fn get_coord(&self) -> FieldCoordinate {
        self.coord
    }

    pub fn get_state(&self) -> PathFindState {
        self.state
    }

    pub fn get_distance(&self) -> i32 {
        self.distance
    }

    pub fn is_tz(&self) -> bool {
        self.tz
    }

    pub fn get_parent(&self) -> Option<NodeRef> {
        self.parent.clone()
    }
}

impl PartialEq for PathFindNode {
    fn eq(&self, other: &Self) -> bool {
        self.get_weight() == other.get_weight()
            && self.get_non_diagonal_weight() == other.get_non_diagonal_weight()
    }
}

impl Eq for PathFindNode {}

impl std::hash::Hash for PathFindNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_weight().hash(state);
        self.get_non_diagonal_weight().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(x: i32, y: i32) -> FieldCoordinate {
        FieldCoordinate::new(x, y)
    }

    fn target_set(coords: &[(i32, i32)]) -> HashSet<FieldCoordinate> {
        coords.iter().map(|&(x, y)| coord(x, y)).collect()
    }

    fn node_ref(state: PathFindState, c: FieldCoordinate, dist: i32) -> NodeRef {
        Rc::new(RefCell::new(PathFindNode::new(state, c, dist, false, None, None)))
    }

    #[test]
    fn get_weight_is_distance_plus_estimate() {
        let targets = target_set(&[(5, 5)]);
        let node = PathFindNode::new(PathFindState::NORMAL, coord(3, 5), 2, false, Some(targets), None);
        // estimate = distance_in_steps((3,5), (5,5)) = max(|2|, |0|) = 2
        assert_eq!(node.get_weight(), 4);
    }

    #[test]
    fn estimate_is_1000_when_no_targets() {
        let node = PathFindNode::new(PathFindState::NORMAL, coord(0, 0), 0, false, None, None);
        assert_eq!(node.get_weight(), 1000);
    }

    #[test]
    fn compare_to_orders_by_weight() {
        let targets = target_set(&[(10, 10)]);
        let close = PathFindNode::new(PathFindState::NORMAL, coord(9, 10), 1, false, Some(targets.clone()), None);
        let far = PathFindNode::new(PathFindState::NORMAL, coord(5, 5), 1, false, Some(targets), None);
        assert_eq!(close.compare_to(&far), std::cmp::Ordering::Less);
    }

    #[test]
    fn set_source_updates_distance_and_preserves_state() {
        let targets = target_set(&[(5, 5)]);
        let parent = node_ref(PathFindState::NORMAL, coord(0, 0), 0);
        let mut child = PathFindNode::new(PathFindState::NORMAL, coord(1, 0), 1, false, Some(targets), None);
        child.set_source(parent, 2);
        assert_eq!(child.get_distance(), 2);
        assert_eq!(child.get_state(), PathFindState::NORMAL);
    }

    #[test]
    fn set_source_with_state_changes_state() {
        let targets = target_set(&[(5, 5)]);
        let parent = node_ref(PathFindState::NORMAL, coord(0, 0), 0);
        let mut child = PathFindNode::new(PathFindState::NORMAL, coord(2, 0), 0, false, Some(targets), None);
        child.set_source_with_state(parent, 2, PathFindState::HAS_JUMPED);
        assert_eq!(child.get_state(), PathFindState::HAS_JUMPED);
        assert_eq!(child.get_distance(), 2);
    }

    #[test]
    fn getters_return_constructor_values() {
        let targets = target_set(&[(3, 3)]);
        let node = PathFindNode::new(PathFindState::HAS_JUMPED, coord(1, 2), 5, true, Some(targets), None);
        assert_eq!(node.get_coord(), coord(1, 2));
        assert_eq!(node.get_state(), PathFindState::HAS_JUMPED);
        assert_eq!(node.get_distance(), 5);
        assert!(node.is_tz());
        assert!(node.get_parent().is_none());
    }

    #[test]
    fn eq_based_on_weights() {
        let targets = target_set(&[(5, 5)]);
        let a = PathFindNode::new(PathFindState::NORMAL, coord(4, 5), 1, false, Some(targets.clone()), None);
        let b = PathFindNode::new(PathFindState::NORMAL, coord(4, 5), 1, false, Some(targets), None);
        assert_eq!(a, b);
    }

    #[test]
    fn get_parent_returns_none_for_root() {
        let node = PathFindNode::new(PathFindState::NORMAL, coord(0, 0), 0, false, None, None);
        assert!(node.get_parent().is_none());
    }
}
