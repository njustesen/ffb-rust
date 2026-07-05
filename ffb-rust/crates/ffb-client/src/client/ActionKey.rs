// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Actionkey;

impl Actionkey {
    pub fn new() -> Self { Self }
}

impl Default for Actionkey {
    fn default() -> Self { Self::new() }
}
