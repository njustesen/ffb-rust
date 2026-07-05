// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Playerchecklist;

impl Playerchecklist {
    pub fn new() -> Self { Self }
}

impl Default for Playerchecklist {
    fn default() -> Self { Self::new() }
}
