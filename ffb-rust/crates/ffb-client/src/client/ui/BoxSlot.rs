// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Boxslot;

impl Boxslot {
    pub fn new() -> Self { Self }
}

impl Default for Boxslot {
    fn default() -> Self { Self::new() }
}
