// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Weathermessage;

impl Weathermessage {
    pub fn new() -> Self { Self }
}

impl Default for Weathermessage {
    fn default() -> Self { Self::new() }
}
