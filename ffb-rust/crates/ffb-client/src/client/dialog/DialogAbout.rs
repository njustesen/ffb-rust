// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogabout;

impl Dialogabout {
    pub fn new() -> Self { Self }
}

impl Default for Dialogabout {
    fn default() -> Self { Self::new() }
}
