// client-only: Java logic plugin factory — superseded by crate::state_dispatch::mod.
pub struct Logicpluginfactory;

impl Logicpluginfactory {
    pub fn new() -> Self { Self }
}

impl Default for Logicpluginfactory {
    fn default() -> Self { Self::new() }
}
