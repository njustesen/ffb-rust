/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFindContext`.
///
/// Immutable configuration for a path-find run; constructed via `Builder`.
#[derive(Debug, Clone, Default)]
pub struct PathFindContext {
    /// Java: allowJump
    pub allow_jump: bool,
    /// Java: allowExitEndzoneWithBall
    pub allow_exit_endzone_with_ball: bool,
    /// Java: blockTacklezones
    pub block_tacklezones: bool,
    /// Java: blockTrapdoors
    pub block_trapdoors: bool,
    /// Java: blockBall
    pub block_ball: bool,
}

impl PathFindContext {
    pub fn is_allow_jump(&self) -> bool { self.allow_jump }
    pub fn is_allow_exit_endzone_with_ball(&self) -> bool { self.allow_exit_endzone_with_ball }
    pub fn is_block_tacklezones(&self) -> bool { self.block_tacklezones }
    pub fn is_block_trapdoors(&self) -> bool { self.block_trapdoors }
    pub fn is_block_ball(&self) -> bool { self.block_ball }
}

/// Java: `PathFindContext.Builder`.
#[derive(Debug, Default)]
pub struct PathFindContextBuilder {
    context: PathFindContext,
}

impl PathFindContextBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn allow_jump(mut self) -> Self { self.context.allow_jump = true; self }
    pub fn allow_exit_endzone_with_ball(mut self) -> Self { self.context.allow_exit_endzone_with_ball = true; self }
    pub fn block_tacklezones(mut self) -> Self { self.context.block_tacklezones = true; self }
    pub fn block_trapdoors(mut self) -> Self { self.context.block_trapdoors = true; self }
    pub fn block_ball(mut self) -> Self { self.context.block_ball = true; self }

    pub fn build(self) -> PathFindContext { self.context }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context_all_false() {
        let ctx = PathFindContext::default();
        assert!(!ctx.is_allow_jump());
        assert!(!ctx.is_block_tacklezones());
    }

    #[test]
    fn builder_sets_flags() {
        let ctx = PathFindContextBuilder::new().allow_jump().block_tacklezones().build();
        assert!(ctx.is_allow_jump());
        assert!(ctx.is_block_tacklezones());
        assert!(!ctx.is_block_ball());
    }
}
