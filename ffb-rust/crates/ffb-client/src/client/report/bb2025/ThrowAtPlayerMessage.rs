// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Throwatplayermessage;

impl Throwatplayermessage {
    pub fn new() -> Self { Self }
}

impl Default for Throwatplayermessage {
    fn default() -> Self { Self::new() }
}
