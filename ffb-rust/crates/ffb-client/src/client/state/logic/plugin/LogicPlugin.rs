// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Logicplugin;

impl Logicplugin {
    pub fn new() -> Self { Self }
}

impl Default for Logicplugin {
    fn default() -> Self { Self::new() }
}
