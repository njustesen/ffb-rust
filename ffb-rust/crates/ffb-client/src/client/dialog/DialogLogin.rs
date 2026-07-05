// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoglogin;

impl Dialoglogin {
    pub fn new() -> Self { Self }
}

impl Default for Dialoglogin {
    fn default() -> Self { Self::new() }
}
