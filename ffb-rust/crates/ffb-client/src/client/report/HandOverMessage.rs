// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Handovermessage;

impl Handovermessage {
    pub fn new() -> Self { Self }
}

impl Default for Handovermessage {
    fn default() -> Self { Self::new() }
}
