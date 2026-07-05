// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Baselogicplugin;

impl Baselogicplugin {
    pub fn new() -> Self { Self }
}

impl Default for Baselogicplugin {
    fn default() -> Self { Self::new() }
}
