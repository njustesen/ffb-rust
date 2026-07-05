// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Creditentry;

impl Creditentry {
    pub fn new() -> Self { Self }
}

impl Default for Creditentry {
    fn default() -> Self { Self::new() }
}
