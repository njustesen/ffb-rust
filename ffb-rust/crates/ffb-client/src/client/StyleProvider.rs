// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Styleprovider;

impl Styleprovider {
    pub fn new() -> Self { Self }
}

impl Default for Styleprovider {
    fn default() -> Self { Self::new() }
}
