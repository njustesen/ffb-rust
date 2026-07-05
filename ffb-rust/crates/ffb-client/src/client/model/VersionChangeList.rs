// client-only: Java client changelog/version model — no Rust UI equivalent.
pub struct Versionchangelist;

impl Versionchangelist {
    pub fn new() -> Self { Self }
}

impl Default for Versionchangelist {
    fn default() -> Self { Self::new() }
}
