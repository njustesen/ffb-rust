// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Swoopplayermessage;

impl Swoopplayermessage {
    pub fn new() -> Self { Self }
}

impl Default for Swoopplayermessage {
    fn default() -> Self { Self::new() }
}
