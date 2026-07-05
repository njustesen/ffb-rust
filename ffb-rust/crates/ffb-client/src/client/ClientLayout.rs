// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Clientlayout;

impl Clientlayout {
    pub fn new() -> Self { Self }
}

impl Default for Clientlayout {
    fn default() -> Self { Self::new() }
}
