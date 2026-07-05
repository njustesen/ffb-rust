// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Leadermessage;

impl Leadermessage {
    pub fn new() -> Self { Self }
}

impl Default for Leadermessage {
    fn default() -> Self { Self::new() }
}
