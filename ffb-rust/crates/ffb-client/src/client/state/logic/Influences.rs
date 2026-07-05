// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Influences;

impl Influences {
    pub fn new() -> Self { Self }
}

impl Default for Influences {
    fn default() -> Self { Self::new() }
}
