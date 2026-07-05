// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Idialog;

impl Idialog {
    pub fn new() -> Self { Self }
}

impl Default for Idialog {
    fn default() -> Self { Self::new() }
}
