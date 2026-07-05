// client-only: Java WebSocket client utility — superseded by crate::connection::mod.
pub struct Commandendpoint;

impl Commandendpoint {
    pub fn new() -> Self { Self }
}

impl Default for Commandendpoint {
    fn default() -> Self { Self::new() }
}
