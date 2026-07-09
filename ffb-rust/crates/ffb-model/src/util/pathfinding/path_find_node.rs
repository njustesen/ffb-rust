use crate::types::FieldCoordinate;
use crate::util::pathfinding::path_find_state::PathFindState;

/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFindNode`.
#[derive(Debug, Clone)]
pub struct PathFindNode {
    /// Java: state
    pub state: PathFindState,
    /// Java: distance
    pub distance: i32,
    /// Java: estimate (to target)
    pub estimate: i32,
    /// Java: tz (has tackle zone)
    pub tz: bool,
    /// Java: coord
    pub coord: FieldCoordinate,
    /// Java: target (set of target squares)
    pub target: Vec<FieldCoordinate>,
    /// Java: parent
    pub parent: Option<Box<PathFindNode>>,
}

impl PathFindNode {
    pub fn new(
        state: PathFindState,
        coord: FieldCoordinate,
        distance: i32,
        tz: bool,
        target: Vec<FieldCoordinate>,
        parent: Option<Box<PathFindNode>>,
    ) -> Self {
        let estimate = if target.is_empty() {
            1000
        } else {
            target.iter().map(|t| coord.distance_in_steps(*t)).min().unwrap_or(1000)
        };
        Self { state, distance, estimate, tz, coord, target, parent }
    }

    /// Java: `getWeight()`.
    pub fn get_weight(&self) -> i32 { self.distance + self.estimate }

    fn get_non_diagonal_weight(&self) -> i32 {
        if self.target.is_empty() { return self.distance + 10000; }
        let best = self.target.iter()
            .map(|t| (self.coord.x - t.x).abs() + (self.coord.y - t.y).abs())
            .min().unwrap_or(10000);
        self.distance + best
    }

    /// Java: `setSource(PathFindNode, int)`.
    pub fn set_source(&mut self, source: &PathFindNode, length: i32) {
        self.distance = source.distance + length;
    }

    pub fn get_distance(&self) -> i32 { self.distance }
    pub fn get_coord(&self) -> FieldCoordinate { self.coord }
    pub fn get_state(&self) -> PathFindState { self.state }
    pub fn is_tz(&self) -> bool { self.tz }
}

impl PartialEq for PathFindNode {
    fn eq(&self, other: &Self) -> bool {
        self.get_weight() == other.get_weight()
            && self.get_non_diagonal_weight() == other.get_non_diagonal_weight()
    }
}

impl Eq for PathFindNode {}

impl PartialOrd for PathFindNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathFindNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let tw = self.get_weight();
        let ow = other.get_weight();
        if tw != ow { return tw.cmp(&ow); }
        self.get_non_diagonal_weight().cmp(&other.get_non_diagonal_weight())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_is_distance_plus_estimate() {
        let node = PathFindNode::new(
            PathFindState::NORMAL,
            FieldCoordinate::new(0, 0),
            3,
            false,
            vec![FieldCoordinate::new(5, 0)],
            None,
        );
        assert_eq!(node.get_weight(), 3 + 5);
    }

    #[test]
    fn node_order_shorter_first() {
        let a = PathFindNode::new(PathFindState::NORMAL, FieldCoordinate::new(0,0), 1, false, vec![FieldCoordinate::new(5,0)], None);
        let b = PathFindNode::new(PathFindState::NORMAL, FieldCoordinate::new(0,0), 3, false, vec![FieldCoordinate::new(5,0)], None);
        assert!(a < b);
    }
}
