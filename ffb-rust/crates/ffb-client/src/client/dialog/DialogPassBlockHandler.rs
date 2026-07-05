// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogpassblockhandler;

impl Dialogpassblockhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogpassblockhandler {
    fn default() -> Self { Self::new() }
}
