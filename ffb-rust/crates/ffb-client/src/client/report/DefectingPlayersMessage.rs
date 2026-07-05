// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Defectingplayersmessage;

impl Defectingplayersmessage {
    pub fn new() -> Self { Self }
}

impl Default for Defectingplayersmessage {
    fn default() -> Self { Self::new() }
}
