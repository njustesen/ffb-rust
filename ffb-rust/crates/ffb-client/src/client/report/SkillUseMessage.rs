// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Skillusemessage;

impl Skillusemessage {
    pub fn new() -> Self { Self }
}

impl Default for Skillusemessage {
    fn default() -> Self { Self::new() }
}
