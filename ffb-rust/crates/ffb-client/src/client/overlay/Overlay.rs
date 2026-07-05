// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Overlay;

impl Overlay {
    pub fn new() -> Self { Self }
}

impl Default for Overlay {
    fn default() -> Self { Self::new() }
}
