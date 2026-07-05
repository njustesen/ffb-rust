// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Passdeviatemessage;

impl Passdeviatemessage {
    pub fn new() -> Self { Self }
}

impl Default for Passdeviatemessage {
    fn default() -> Self { Self::new() }
}
