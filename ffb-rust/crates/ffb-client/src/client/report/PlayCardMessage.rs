// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Playcardmessage;

impl Playcardmessage {
    pub fn new() -> Self { Self }
}

impl Default for Playcardmessage {
    fn default() -> Self { Self::new() }
}
