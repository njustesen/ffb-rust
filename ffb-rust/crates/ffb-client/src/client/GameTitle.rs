// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Gametitle;

impl Gametitle {
    pub fn new() -> Self { Self }
}

impl Default for Gametitle {
    fn default() -> Self { Self::new() }
}
