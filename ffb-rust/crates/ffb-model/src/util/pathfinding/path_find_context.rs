/// Configuration object controlling pathfinding behaviour.
/// Java: PathFindContext (package-private) + PathFindContext.Builder (inner static class)
pub struct PathFindContext {
    allow_jump: bool,
    allow_exit_endzone_with_ball: bool,
    block_tacklezones: bool,
    block_trapdoors: bool,
    block_ball: bool,
}

impl PathFindContext {
    fn new() -> Self {
        PathFindContext {
            allow_jump: false,
            allow_exit_endzone_with_ball: false,
            block_tacklezones: false,
            block_trapdoors: false,
            block_ball: false,
        }
    }

    pub fn is_allow_exit_endzone_with_ball(&self) -> bool {
        self.allow_exit_endzone_with_ball
    }

    pub fn is_block_tacklezones(&self) -> bool {
        self.block_tacklezones
    }

    pub fn is_allow_jump(&self) -> bool {
        self.allow_jump
    }

    pub fn is_block_trapdoors(&self) -> bool {
        self.block_trapdoors
    }

    pub fn is_block_ball(&self) -> bool {
        self.block_ball
    }
}

pub struct Builder {
    context: PathFindContext,
}

impl Builder {
    pub fn new() -> Self {
        Builder { context: PathFindContext::new() }
    }

    pub fn allow_jump(mut self) -> Self {
        self.context.allow_jump = true;
        self
    }

    pub fn allow_exit_endzone_with_ball(mut self) -> Self {
        self.context.allow_exit_endzone_with_ball = true;
        self
    }

    pub fn block_tacklezones(mut self) -> Self {
        self.context.block_tacklezones = true;
        self
    }

    pub fn block_trapdoors(mut self) -> Self {
        self.context.block_trapdoors = true;
        self
    }

    pub fn block_ball(mut self) -> Self {
        self.context.block_ball = true;
        self
    }

    pub fn build(self) -> PathFindContext {
        self.context
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context_has_all_flags_false() {
        let ctx = Builder::new().build();
        assert!(!ctx.is_allow_jump());
        assert!(!ctx.is_allow_exit_endzone_with_ball());
        assert!(!ctx.is_block_tacklezones());
        assert!(!ctx.is_block_trapdoors());
        assert!(!ctx.is_block_ball());
    }

    #[test]
    fn allow_jump_sets_flag() {
        let ctx = Builder::new().allow_jump().build();
        assert!(ctx.is_allow_jump());
        assert!(!ctx.is_block_tacklezones());
    }

    #[test]
    fn block_tacklezones_sets_flag() {
        let ctx = Builder::new().block_tacklezones().build();
        assert!(ctx.is_block_tacklezones());
        assert!(!ctx.is_allow_jump());
    }

    #[test]
    fn multiple_flags_can_be_combined() {
        let ctx = Builder::new().allow_jump().allow_exit_endzone_with_ball().build();
        assert!(ctx.is_allow_jump());
        assert!(ctx.is_allow_exit_endzone_with_ball());
        assert!(!ctx.is_block_tacklezones());
    }

    #[test]
    fn block_trapdoors_and_block_ball_work() {
        let ctx = Builder::new().block_trapdoors().block_ball().build();
        assert!(ctx.is_block_trapdoors());
        assert!(ctx.is_block_ball());
        assert!(!ctx.is_allow_jump());
    }

    #[test]
    fn builder_default_same_as_new() {
        let ctx = Builder::default().build();
        assert!(!ctx.is_allow_jump());
    }
}
