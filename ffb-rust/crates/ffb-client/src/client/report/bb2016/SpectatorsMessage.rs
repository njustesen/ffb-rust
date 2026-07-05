// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Spectatorsmessage;

impl Spectatorsmessage {
    pub fn new() -> Self { Self }
}

impl Default for Spectatorsmessage {
    fn default() -> Self { Self::new() }
}
