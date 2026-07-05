// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogstartgame;

impl Dialogstartgame {
    pub fn new() -> Self { Self }
}

impl Default for Dialogstartgame {
    fn default() -> Self { Self::new() }
}
