// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Resourceslot;

impl Resourceslot {
    pub fn new() -> Self { Self }
}

impl Default for Resourceslot {
    fn default() -> Self { Self::new() }
}
