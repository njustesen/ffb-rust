// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Actionkeybindings;

impl Actionkeybindings {
    pub fn new() -> Self { Self }
}

impl Default for Actionkeybindings {
    fn default() -> Self { Self::new() }
}
