// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Interactionresult;

impl Interactionresult {
    pub fn new() -> Self { Self }
}

impl Default for Interactionresult {
    fn default() -> Self { Self::new() }
}
