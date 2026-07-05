// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Blockrollmessage;

impl Blockrollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Blockrollmessage {
    fn default() -> Self { Self::new() }
}
