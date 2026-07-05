// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Rerollmessage;

impl Rerollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Rerollmessage {
    fn default() -> Self { Self::new() }
}
