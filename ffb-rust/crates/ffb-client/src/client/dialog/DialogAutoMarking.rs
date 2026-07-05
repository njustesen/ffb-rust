// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogautomarking;

impl Dialogautomarking {
    pub fn new() -> Self { Self }
}

impl Default for Dialogautomarking {
    fn default() -> Self { Self::new() }
}
