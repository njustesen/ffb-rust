// client-only: Java client state machine — superseded by crate::state_dispatch::mod.
pub struct Movelogicplugin;

impl Movelogicplugin {
    pub fn new() -> Self { Self }
}

impl Default for Movelogicplugin {
    fn default() -> Self { Self::new() }
}
