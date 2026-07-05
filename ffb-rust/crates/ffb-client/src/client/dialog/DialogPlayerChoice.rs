// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogplayerchoice;

impl Dialogplayerchoice {
    pub fn new() -> Self { Self }
}

impl Default for Dialogplayerchoice {
    fn default() -> Self { Self::new() }
}
