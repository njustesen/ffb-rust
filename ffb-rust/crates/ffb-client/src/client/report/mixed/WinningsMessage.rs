// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Winningsmessage;

impl Winningsmessage {
    pub fn new() -> Self { Self }
}

impl Default for Winningsmessage {
    fn default() -> Self { Self::new() }
}
