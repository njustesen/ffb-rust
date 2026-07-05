// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Soundengine;

impl Soundengine {
    pub fn new() -> Self { Self }
}

impl Default for Soundengine {
    fn default() -> Self { Self::new() }
}
