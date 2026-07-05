// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Component;

impl Component {
    pub fn new() -> Self { Self }
}

impl Default for Component {
    fn default() -> Self { Self::new() }
}
