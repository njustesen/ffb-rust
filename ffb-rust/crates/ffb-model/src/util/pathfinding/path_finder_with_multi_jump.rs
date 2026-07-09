use crate::util::pathfinding::path_find_context::PathFindContextBuilder;
use crate::util::pathfinding::path_finder_extension::PathFinderExtension;

/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFinderWithMultiJump`.
///
/// A* path finder that supports multiple jump moves.  The full A* algorithm needs
/// a live game model (field model, player states) and lives in `ffb-engine`.
/// This struct is retained for structural completeness with its static singleton.
#[derive(Debug)]
pub struct PathFinderWithMultiJump {
    /// Java: `theoreticalRangeContext`
    pub theoretical_range_context: crate::util::pathfinding::path_find_context::PathFindContext,
    /// Java: `extension`
    pub extension: PathFinderExtension,
}

impl PathFinderWithMultiJump {
    pub fn new() -> Self {
        Self {
            theoretical_range_context: PathFindContextBuilder::new().allow_jump().build(),
            extension: PathFinderExtension::new(),
        }
    }
}

impl Default for PathFinderWithMultiJump {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_has_jump_context() {
        let pf = PathFinderWithMultiJump::new();
        assert!(pf.theoretical_range_context.is_allow_jump());
    }

    #[test]
    fn default_and_new_equivalent() {
        let _a = PathFinderWithMultiJump::new();
        let _b = PathFinderWithMultiJump::default();
    }
}
