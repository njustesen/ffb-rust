// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogmanager;

impl Dialogmanager {
    pub fn new() -> Self { Self }
}

impl Default for Dialogmanager {
    fn default() -> Self { Self::new() }
}
