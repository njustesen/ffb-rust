// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Blockmessage;

impl Blockmessage {
    pub fn new() -> Self { Self }
}

impl Default for Blockmessage {
    fn default() -> Self { Self::new() }
}
