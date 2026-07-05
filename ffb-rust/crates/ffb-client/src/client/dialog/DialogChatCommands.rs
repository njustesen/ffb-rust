// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogchatcommands;

impl Dialogchatcommands {
    pub fn new() -> Self { Self }
}

impl Default for Dialogchatcommands {
    fn default() -> Self { Self::new() }
}
