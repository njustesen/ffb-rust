// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Playeractionmessage;

impl Playeractionmessage {
    pub fn new() -> Self { Self }
}

impl Default for Playeractionmessage {
    fn default() -> Self { Self::new() }
}
