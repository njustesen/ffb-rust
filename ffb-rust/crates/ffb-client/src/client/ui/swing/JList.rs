// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Jlist;

impl Jlist {
    pub fn new() -> Self { Self }
}

impl Default for Jlist {
    fn default() -> Self { Self::new() }
}
