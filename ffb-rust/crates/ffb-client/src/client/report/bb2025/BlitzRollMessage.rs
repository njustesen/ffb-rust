// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Blitzrollmessage;

impl Blitzrollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Blitzrollmessage {
    fn default() -> Self { Self::new() }
}
