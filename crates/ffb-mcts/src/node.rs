//! Arena-allocated MCTS node pool.
//!
//! All nodes live in a flat `Vec`; `NodeId(u32)` is an index into that vec.
//! `reset()` clears the pool without reallocating — critical for repeated searches.

use ffb_core::actions::BbAction;

// ── Node id ───────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

// ── Node ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    /// Action that led to this node from parent.
    pub action: Option<BbAction>,
    pub visits: u32,
    pub value_sum: f64,
    /// Cached candidate actions (expanded on first visit).
    pub candidates: Option<Vec<BbAction>>,
    /// Prior probability (for PUCT mode; uniform = 1/n_siblings if unset).
    pub prior: f64,
}

impl Node {
    pub fn new(parent: Option<NodeId>, action: Option<BbAction>, prior: f64) -> Self {
        Self {
            parent,
            children: Vec::new(),
            action,
            visits: 0,
            value_sum: 0.0,
            candidates: None,
            prior,
        }
    }

    pub fn value(&self) -> f64 {
        if self.visits == 0 { 0.0 } else { self.value_sum / self.visits as f64 }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

// ── Arena ─────────────────────────────────────────────────────────────────────

pub struct NodeArena {
    nodes: Vec<Node>,
    free: Vec<NodeId>,
}

impl NodeArena {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(cap),
            free: Vec::new(),
        }
    }

    pub fn alloc(&mut self, node: Node) -> NodeId {
        if let Some(id) = self.free.pop() {
            self.nodes[id.0 as usize] = node;
            id
        } else {
            let id = NodeId(self.nodes.len() as u32);
            self.nodes.push(node);
            id
        }
    }

    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0 as usize]
    }

    pub fn free_node(&mut self, id: NodeId) {
        self.nodes[id.0 as usize] = Node::new(None, None, 1.0);
        self.free.push(id);
    }

    /// Reset the arena for a new search without heap reallocation.
    pub fn reset(&mut self) {
        self.nodes.clear();
        self.free.clear();
    }

    pub fn len(&self) -> usize {
        self.nodes.len() - self.free.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── UCB / PUCT scores ────────────────────────────────────────────────────────

/// UCB1 score: `value/visits + c * sqrt(ln(parent_visits) / visits)`.
/// Unexplored nodes return `f64::INFINITY`.
pub fn ucb_score(node: &Node, parent_visits: u32, c: f64) -> f64 {
    if node.visits == 0 {
        return f64::INFINITY;
    }
    node.value() + c * ((parent_visits as f64).ln() / node.visits as f64).sqrt()
}

/// PUCT score: `value/visits + c * prior * sqrt(parent_visits) / (1 + visits)`.
pub fn puct_score(node: &Node, parent_visits: u32, c: f64) -> f64 {
    node.value() + c * node.prior * (parent_visits as f64).sqrt() / (1.0 + node.visits as f64)
}

/// Select the child of `parent_id` with the highest UCB score.
pub fn select_child_ucb(arena: &NodeArena, parent_id: NodeId, c: f64) -> Option<NodeId> {
    let parent = arena.get(parent_id);
    let parent_visits = parent.visits;
    parent
        .children
        .iter()
        .copied()
        .max_by(|&a, &b| {
            let sa = ucb_score(arena.get(a), parent_visits, c);
            let sb = ucb_score(arena.get(b), parent_visits, c);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_alloc_and_get() {
        let mut arena = NodeArena::with_capacity(64);
        let id = arena.alloc(Node::new(None, None, 1.0));
        assert_eq!(id.0, 0);
        assert_eq!(arena.get(id).visits, 0);
    }

    #[test]
    fn arena_reset_clears_nodes() {
        let mut arena = NodeArena::with_capacity(64);
        for _ in 0..100 {
            arena.alloc(Node::new(None, None, 1.0));
        }
        arena.reset();
        assert_eq!(arena.len(), 0);
        let id = arena.alloc(Node::new(None, None, 1.0));
        assert_eq!(id.0, 0);
    }

    #[test]
    fn arena_free_and_reuse() {
        let mut arena = NodeArena::with_capacity(64);
        let _id0 = arena.alloc(Node::new(None, None, 1.0));
        let id1 = arena.alloc(Node::new(None, None, 1.0));
        arena.free_node(id1);
        let id2 = arena.alloc(Node::new(None, None, 1.0));
        assert_eq!(id2, id1);
    }

    #[test]
    fn ucb_unexplored_is_infinity() {
        let node = Node::new(None, None, 1.0);
        assert_eq!(ucb_score(&node, 10, 1.41), f64::INFINITY);
    }

    #[test]
    fn ucb_explored_node_finite() {
        let mut node = Node::new(None, None, 1.0);
        node.visits = 5;
        node.value_sum = 3.0;
        let score = ucb_score(&node, 20, 1.41);
        assert!(score.is_finite());
        assert!(score > 0.0);
    }

    #[test]
    fn select_child_prefers_unexplored() {
        let mut arena = NodeArena::with_capacity(16);
        let root = arena.alloc(Node::new(None, None, 1.0));
        arena.get_mut(root).visits = 10;

        let c1 = arena.alloc(Node::new(Some(root), None, 1.0));
        arena.get_mut(c1).visits = 5;
        arena.get_mut(c1).value_sum = 4.0;
        arena.get_mut(root).children.push(c1);

        let c2 = arena.alloc(Node::new(Some(root), None, 1.0));
        arena.get_mut(root).children.push(c2);

        let selected = select_child_ucb(&arena, root, 1.41);
        assert_eq!(selected, Some(c2));
    }

    #[test]
    fn select_child_prefers_higher_value() {
        let mut arena = NodeArena::with_capacity(16);
        let root = arena.alloc(Node::new(None, None, 1.0));
        arena.get_mut(root).visits = 100;

        let c1 = arena.alloc(Node::new(Some(root), None, 1.0));
        arena.get_mut(c1).visits = 10;
        arena.get_mut(c1).value_sum = 3.0;

        let c2 = arena.alloc(Node::new(Some(root), None, 1.0));
        arena.get_mut(c2).visits = 10;
        arena.get_mut(c2).value_sum = 9.0;

        arena.get_mut(root).children.push(c1);
        arena.get_mut(root).children.push(c2);

        let selected = select_child_ucb(&arena, root, 0.0);
        assert_eq!(selected, Some(c2));
    }
}
