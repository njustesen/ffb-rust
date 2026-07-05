// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Actioncontext;

impl Actioncontext {
    pub fn new() -> Self { Self }
}

impl Default for Actioncontext {
    fn default() -> Self { Self::new() }
}
