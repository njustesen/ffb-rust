// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoginformation;

impl Dialoginformation {
    pub fn new() -> Self { Self }
}

impl Default for Dialoginformation {
    fn default() -> Self { Self::new() }
}
