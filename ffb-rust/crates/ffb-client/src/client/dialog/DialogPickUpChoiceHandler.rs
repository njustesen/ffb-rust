// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogpickupchoicehandler;

impl Dialogpickupchoicehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogpickupchoicehandler {
    fn default() -> Self { Self::new() }
}
