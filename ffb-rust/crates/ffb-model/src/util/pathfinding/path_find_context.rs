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
