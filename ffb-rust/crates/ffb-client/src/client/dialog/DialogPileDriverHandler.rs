// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogpiledriverhandler;

impl Dialogpiledriverhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogpiledriverhandler {
    fn default() -> Self { Self::new() }
}
