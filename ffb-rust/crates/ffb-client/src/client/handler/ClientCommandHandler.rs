// client-only: Java server command handler — superseded by crate::handlers::mod.
pub struct Clientcommandhandler;

impl Clientcommandhandler {
    pub fn new() -> Self { Self }
}

impl Default for Clientcommandhandler {
    fn default() -> Self { Self::new() }
}
