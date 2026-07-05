// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Reportmessagetype;

impl Reportmessagetype {
    pub fn new() -> Self { Self }
}

impl Default for Reportmessagetype {
    fn default() -> Self { Self::new() }
}
