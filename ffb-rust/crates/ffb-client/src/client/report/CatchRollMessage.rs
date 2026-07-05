// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Catchrollmessage;

impl Catchrollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Catchrollmessage {
    fn default() -> Self { Self::new() }
}
