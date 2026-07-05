// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Clientreplayer;

impl Clientreplayer {
    pub fn new() -> Self { Self }
}

impl Default for Clientreplayer {
    fn default() -> Self { Self::new() }
}
