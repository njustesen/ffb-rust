// client-only: Java client changelog/version model — no Rust UI equivalent.
pub struct Onlineaware;

impl Onlineaware {
    pub fn new() -> Self { Self }
}

impl Default for Onlineaware {
    fn default() -> Self { Self::new() }
}
