// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogprogressbar;

impl Dialogprogressbar {
    pub fn new() -> Self { Self }
}

impl Default for Dialogprogressbar {
    fn default() -> Self { Self::new() }
}
