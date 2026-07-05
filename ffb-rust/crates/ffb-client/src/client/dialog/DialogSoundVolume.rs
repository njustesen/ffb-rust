// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogsoundvolume;

impl Dialogsoundvolume {
    pub fn new() -> Self { Self }
}

impl Default for Dialogsoundvolume {
    fn default() -> Self { Self::new() }
}
