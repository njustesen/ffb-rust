// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Iconcache;

impl Iconcache {
    pub fn new() -> Self { Self }
}

impl Default for Iconcache {
    fn default() -> Self { Self::new() }
}
