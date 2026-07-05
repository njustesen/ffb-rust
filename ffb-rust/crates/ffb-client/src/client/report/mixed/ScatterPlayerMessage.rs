// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Scatterplayermessage;

impl Scatterplayermessage {
    pub fn new() -> Self { Self }
}

impl Default for Scatterplayermessage {
    fn default() -> Self { Self::new() }
}
