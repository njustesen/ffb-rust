// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Quicksnaprollmessage;

impl Quicksnaprollmessage {
    pub fn new() -> Self { Self }
}

impl Default for Quicksnaprollmessage {
    fn default() -> Self { Self::new() }
}
