// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Jumprollmessage;

impl Jumprollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Jumprollmessage {
    fn default() -> Self { Self::new() }
}
