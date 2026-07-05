// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogbribes;

impl Dialogbribes {
    pub fn new() -> Self { Self }
}

impl Default for Dialogbribes {
    fn default() -> Self { Self::new() }
}
