// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Chomprollmessage;

impl Chomprollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Chomprollmessage {
    fn default() -> Self { Self::new() }
}
