// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Injurymessage;

impl Injurymessage {
    pub fn new() -> Self { Self }
}

impl Default for Injurymessage {
    fn default() -> Self { Self::new() }
}
