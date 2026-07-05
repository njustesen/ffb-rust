// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Raisedeadmessage;

impl Raisedeadmessage {
    pub fn new() -> Self { Self }
}

impl Default for Raisedeadmessage {
    fn default() -> Self { Self::new() }
}
