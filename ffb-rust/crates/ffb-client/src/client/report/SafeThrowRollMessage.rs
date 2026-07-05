// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Safethrowrollmessage;

impl Safethrowrollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Safethrowrollmessage {
    fn default() -> Self { Self::new() }
}
