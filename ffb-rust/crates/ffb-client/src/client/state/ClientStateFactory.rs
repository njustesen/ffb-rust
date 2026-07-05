// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Clientstatefactory;

impl Clientstatefactory {
    pub fn new() -> Self { Self }
}

impl Default for Clientstatefactory {
    fn default() -> Self { Self::new() }
}
