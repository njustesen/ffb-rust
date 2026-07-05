// client-only: Java WebSocket client utility — superseded by crate::connection::mod.
pub struct Clientcommunication;

impl Clientcommunication {
    pub fn new() -> Self { Self }
}

impl Default for Clientcommunication {
    fn default() -> Self { Self::new() }
}
