// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Turnendmessage;

impl Turnendmessage {
    pub fn new() -> Self { Self }
}

impl Default for Turnendmessage {
    fn default() -> Self { Self::new() }
}
