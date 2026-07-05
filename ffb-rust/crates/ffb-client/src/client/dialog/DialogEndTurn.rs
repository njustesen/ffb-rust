// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogendturn;

impl Dialogendturn {
    pub fn new() -> Self { Self }
}

impl Default for Dialogendturn {
    fn default() -> Self { Self::new() }
}
