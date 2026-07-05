// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Reportmessagebase;

impl Reportmessagebase {
    pub fn new() -> Self { Self }
}

impl Default for Reportmessagebase {
    fn default() -> Self { Self::new() }
}
