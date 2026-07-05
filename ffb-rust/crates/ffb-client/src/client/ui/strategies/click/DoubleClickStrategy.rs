// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Doubleclickstrategy;

impl Doubleclickstrategy {
    pub fn new() -> Self { Self }
}

impl Default for Doubleclickstrategy {
    fn default() -> Self { Self::new() }
}
