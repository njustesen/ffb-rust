// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Skillchecklist;

impl Skillchecklist {
    pub fn new() -> Self { Self }
}

impl Default for Skillchecklist {
    fn default() -> Self { Self::new() }
}
