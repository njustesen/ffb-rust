// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogchangelist;

impl Dialogchangelist {
    pub fn new() -> Self { Self }
}

impl Default for Dialogchangelist {
    fn default() -> Self { Self::new() }
}
