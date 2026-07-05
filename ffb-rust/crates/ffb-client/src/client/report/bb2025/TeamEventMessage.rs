// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Teameventmessage;

impl Teameventmessage {
    pub fn new() -> Self { Self }
}

impl Default for Teameventmessage {
    fn default() -> Self { Self::new() }
}
