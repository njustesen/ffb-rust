// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoguseinducement;

impl Dialoguseinducement {
    pub fn new() -> Self { Self }
}

impl Default for Dialoguseinducement {
    fn default() -> Self { Self::new() }
}
