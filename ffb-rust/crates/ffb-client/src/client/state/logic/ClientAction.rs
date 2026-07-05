// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Clientaction;

impl Clientaction {
    pub fn new() -> Self { Self }
}

impl Default for Clientaction {
    fn default() -> Self { Self::new() }
}
