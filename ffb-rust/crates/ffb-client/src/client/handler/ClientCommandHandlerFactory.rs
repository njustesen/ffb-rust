// client-only: Java server command handler — superseded by crate::handlers::mod.
pub struct Clientcommandhandlerfactory;

impl Clientcommandhandlerfactory {
    pub fn new() -> Self { Self }
}

impl Default for Clientcommandhandlerfactory {
    fn default() -> Self { Self::new() }
}
