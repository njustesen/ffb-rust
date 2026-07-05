// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Positionchecklist;

impl Positionchecklist {
    pub fn new() -> Self { Self }
}

impl Default for Positionchecklist {
    fn default() -> Self { Self::new() }
}
