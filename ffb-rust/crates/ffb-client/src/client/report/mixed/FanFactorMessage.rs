// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Fanfactormessage;

impl Fanfactormessage {
    pub fn new() -> Self { Self }
}

impl Default for Fanfactormessage {
    fn default() -> Self { Self::new() }
}
