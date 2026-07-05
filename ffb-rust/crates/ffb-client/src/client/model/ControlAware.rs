// client-only: Java client changelog/version model — no Rust UI equivalent.
pub struct Controlaware;

impl Controlaware {
    pub fn new() -> Self { Self }
}

impl Default for Controlaware {
    fn default() -> Self { Self::new() }
}
