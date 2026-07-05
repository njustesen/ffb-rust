// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogskillusehandler;

impl Dialogskillusehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogskillusehandler {
    fn default() -> Self { Self::new() }
}
