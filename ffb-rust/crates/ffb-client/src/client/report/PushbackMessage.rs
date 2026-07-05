// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Pushbackmessage;

impl Pushbackmessage {
    pub fn new() -> Self { Self }
}

impl Default for Pushbackmessage {
    fn default() -> Self { Self::new() }
}
