// client-only: Java server command handler — superseded by crate::handlers::mod.
pub struct Clientcommandhandlerjoin;

impl Clientcommandhandlerjoin {
    pub fn new() -> Self { Self }
}

impl Default for Clientcommandhandlerjoin {
    fn default() -> Self { Self::new() }
}
