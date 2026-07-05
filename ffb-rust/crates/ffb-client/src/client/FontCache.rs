// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Fontcache;

impl Fontcache {
    pub fn new() -> Self { Self }
}

impl Default for Fontcache {
    fn default() -> Self { Self::new() }
}
