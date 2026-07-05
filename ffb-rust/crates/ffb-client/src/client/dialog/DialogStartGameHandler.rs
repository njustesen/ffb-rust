// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogstartgamehandler;

impl Dialogstartgamehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogstartgamehandler {
    fn default() -> Self { Self::new() }
}
