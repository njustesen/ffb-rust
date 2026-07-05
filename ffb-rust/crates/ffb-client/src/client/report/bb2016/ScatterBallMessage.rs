// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Scatterballmessage;

impl Scatterballmessage {
    pub fn new() -> Self { Self }
}

impl Default for Scatterballmessage {
    fn default() -> Self { Self::new() }
}
