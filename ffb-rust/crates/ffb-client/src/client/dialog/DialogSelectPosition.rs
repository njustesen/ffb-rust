// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogselectposition;

impl Dialogselectposition {
    pub fn new() -> Self { Self }
}

impl Default for Dialogselectposition {
    fn default() -> Self { Self::new() }
}
