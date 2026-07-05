// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogjourneymenhandler;

impl Dialogjourneymenhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogjourneymenhandler {
    fn default() -> Self { Self::new() }
}
