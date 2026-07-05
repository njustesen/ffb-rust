// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Bomblogicmodule;

impl Bomblogicmodule {
    pub fn new() -> Self { Self }
}

impl Default for Bomblogicmodule {
    fn default() -> Self { Self::new() }
}
