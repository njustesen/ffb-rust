// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Starthalfmessage;

impl Starthalfmessage {
    pub fn new() -> Self { Self }
}

impl Default for Starthalfmessage {
    fn default() -> Self { Self::new() }
}
