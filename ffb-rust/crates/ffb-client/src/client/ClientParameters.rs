// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Clientparameters;

impl Clientparameters {
    pub fn new() -> Self { Self }
}

impl Default for Clientparameters {
    fn default() -> Self { Self::new() }
}
