// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Foulmessage;

impl Foulmessage {
    pub fn new() -> Self { Self }
}

impl Default for Foulmessage {
    fn default() -> Self { Self::new() }
}
