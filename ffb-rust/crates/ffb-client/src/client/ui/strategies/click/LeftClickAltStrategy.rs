// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Leftclickaltstrategy;

impl Leftclickaltstrategy {
    pub fn new() -> Self { Self }
}

impl Default for Leftclickaltstrategy {
    fn default() -> Self { Self::new() }
}
