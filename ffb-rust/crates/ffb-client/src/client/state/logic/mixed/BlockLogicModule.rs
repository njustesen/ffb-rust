// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Blocklogicmodule;

impl Blocklogicmodule {
    pub fn new() -> Self { Self }
}

impl Default for Blocklogicmodule {
    fn default() -> Self { Self::new() }
}
