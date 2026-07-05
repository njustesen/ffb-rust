// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Carddeactivatedmessage;

impl Carddeactivatedmessage {
    pub fn new() -> Self { Self }
}

impl Default for Carddeactivatedmessage {
    fn default() -> Self { Self::new() }
}
