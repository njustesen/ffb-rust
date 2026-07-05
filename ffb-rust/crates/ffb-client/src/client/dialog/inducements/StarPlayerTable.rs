// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Starplayertable;

impl Starplayertable {
    pub fn new() -> Self { Self }
}

impl Default for Starplayertable {
    fn default() -> Self { Self::new() }
}
