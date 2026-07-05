// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogpiledriver;

impl Dialogpiledriver {
    pub fn new() -> Self { Self }
}

impl Default for Dialogpiledriver {
    fn default() -> Self { Self::new() }
}
