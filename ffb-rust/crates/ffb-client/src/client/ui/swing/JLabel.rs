// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Jlabel;

impl Jlabel {
    pub fn new() -> Self { Self }
}

impl Default for Jlabel {
    fn default() -> Self { Self::new() }
}
