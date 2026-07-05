// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Clickstrategy;

impl Clickstrategy {
    pub fn new() -> Self { Self }
}

impl Default for Clickstrategy {
    fn default() -> Self { Self::new() }
}
