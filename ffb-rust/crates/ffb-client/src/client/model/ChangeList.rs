// client-only: Java client changelog/version model — no Rust UI equivalent.
pub struct Changelist;

impl Changelist {
    pub fn new() -> Self { Self }
}

impl Default for Changelist {
    fn default() -> Self { Self::new() }
}
