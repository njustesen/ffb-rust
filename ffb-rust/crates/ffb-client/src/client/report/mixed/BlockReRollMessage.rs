// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Blockrerollmessage;

impl Blockrerollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Blockrerollmessage {
    fn default() -> Self { Self::new() }
}
