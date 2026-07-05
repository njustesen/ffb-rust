// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Stallerdetectedmessage;

impl Stallerdetectedmessage {
    pub fn new() -> Self { Self }
}

impl Default for Stallerdetectedmessage {
    fn default() -> Self { Self::new() }
}
