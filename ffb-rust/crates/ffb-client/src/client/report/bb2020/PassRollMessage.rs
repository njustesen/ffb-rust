// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Passrollmessage;

impl Passrollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Passrollmessage {
    fn default() -> Self { Self::new() }
}
