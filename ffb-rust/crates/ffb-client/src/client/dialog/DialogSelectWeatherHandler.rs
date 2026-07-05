// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogselectweatherhandler;

impl Dialogselectweatherhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogselectweatherhandler {
    fn default() -> Self { Self::new() }
}
