// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogselectweather;

impl Dialogselectweather {
    pub fn new() -> Self { Self }
}

impl Default for Dialogselectweather {
    fn default() -> Self { Self::new() }
}
