// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Standuprollmessage;

impl Standuprollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Standuprollmessage {
    fn default() -> Self { Self::new() }
}
