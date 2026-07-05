// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Breathefiremessage;

impl Breathefiremessage {
    pub fn new() -> Self { Self }
}

impl Default for Breathefiremessage {
    fn default() -> Self { Self::new() }
}
