// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Playereventmessage;

impl Playereventmessage {
    pub fn new() -> Self { Self }
}

impl Default for Playereventmessage {
    fn default() -> Self { Self::new() }
}
