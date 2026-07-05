// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Twoforonemessage;

impl Twoforonemessage {
    pub fn new() -> Self { Self }
}

impl Default for Twoforonemessage {
    fn default() -> Self { Self::new() }
}
