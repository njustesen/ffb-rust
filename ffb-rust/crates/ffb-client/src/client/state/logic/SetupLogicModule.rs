// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Setuplogicmodule;

impl Setuplogicmodule {
    pub fn new() -> Self { Self }
}

impl Default for Setuplogicmodule {
    fn default() -> Self { Self::new() }
}
