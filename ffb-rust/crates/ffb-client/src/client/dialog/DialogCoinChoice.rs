// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogcoinchoice;

impl Dialogcoinchoice {
    pub fn new() -> Self { Self }
}

impl Default for Dialogcoinchoice {
    fn default() -> Self { Self::new() }
}
