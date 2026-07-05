// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Kickoffriotmessage;

impl Kickoffriotmessage {
    pub fn new() -> Self { Self }
}

impl Default for Kickoffriotmessage {
    fn default() -> Self { Self::new() }
}
