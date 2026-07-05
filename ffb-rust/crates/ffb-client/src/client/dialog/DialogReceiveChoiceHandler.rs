// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogreceivechoicehandler;

impl Dialogreceivechoicehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogreceivechoicehandler {
    fn default() -> Self { Self::new() }
}
