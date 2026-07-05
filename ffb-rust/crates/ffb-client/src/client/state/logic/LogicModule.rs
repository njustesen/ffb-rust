// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Logicmodule;

impl Logicmodule {
    pub fn new() -> Self { Self }
}

impl Default for Logicmodule {
    fn default() -> Self { Self::new() }
}
