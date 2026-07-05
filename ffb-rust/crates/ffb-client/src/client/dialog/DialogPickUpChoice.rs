// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogpickupchoice;

impl Dialogpickupchoice {
    pub fn new() -> Self { Self }
}

impl Default for Dialogpickupchoice {
    fn default() -> Self { Self::new() }
}
