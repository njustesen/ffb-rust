use crate::util::pathfinding::path_find_context::{PathFindContext, PathFindContextBuilder};

/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFinderWithPassBlockSupport`.
///
/// A* path finder that supports pass block moves.  The full A* algorithm needs
/// a live game model and lives in `ffb-engine`.  This struct holds the two
/// pre-built context objects that the Java static singleton stores.
#[derive(Debug)]
pub struct PathFinderWithPassBlockSupport {
    /// Java: `normalMoveContext`
    pub normal_move_context: PathFindContext,
    /// Java: `passBlockContext`
    pub pass_block_context: PathFindContext,
}

impl PathFinderWithPassBlockSupport {
    pub fn new() -> Self {
        Self {
            normal_move_context: PathFindContextBuilder::new().block_tacklezones().build(),
            pass_block_context: PathFindContextBuilder::new()
                .allow_exit_endzone_with_ball()
                .allow_jump()
                .build(),
        }
    }
}

impl Default for PathFinderWithPassBlockSupport {
    fn default() -> Self { Self::new() }
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
