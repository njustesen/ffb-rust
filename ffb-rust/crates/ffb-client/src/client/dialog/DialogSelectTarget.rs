// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogselecttarget;

impl Dialogselecttarget {
    pub fn new() -> Self { Self }
}

impl Default for Dialogselecttarget {
    fn default() -> Self { Self::new() }
}
