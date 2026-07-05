// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogkeybindings;

impl Dialogkeybindings {
    pub fn new() -> Self { Self }
}

impl Default for Dialogkeybindings {
    fn default() -> Self { Self::new() }
}
