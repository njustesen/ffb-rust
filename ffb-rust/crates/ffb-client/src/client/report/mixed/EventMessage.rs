// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Eventmessage;

impl Eventmessage {
    pub fn new() -> Self { Self }
}

impl Default for Eventmessage {
    fn default() -> Self { Self::new() }
}
