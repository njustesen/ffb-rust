// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Clientstate;

impl Clientstate {
    pub fn new() -> Self { Self }
}

impl Default for Clientstate {
    fn default() -> Self { Self::new() }
}
