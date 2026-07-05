// client-only: Java WebSocket client utility — superseded by crate::connection::mod.
pub struct Clientpingtask;

impl Clientpingtask {
    pub fn new() -> Self { Self }
}

impl Default for Clientpingtask {
    fn default() -> Self { Self::new() }
}
