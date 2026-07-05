// client-only: Java server command handler — superseded by crate::handlers::mod.
pub struct Clientcommandhandlerleave;

impl Clientcommandhandlerleave {
    pub fn new() -> Self { Self }
}

impl Default for Clientcommandhandlerleave {
    fn default() -> Self { Self::new() }
}
