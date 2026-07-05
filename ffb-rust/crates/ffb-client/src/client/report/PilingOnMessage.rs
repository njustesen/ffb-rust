// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Pilingonmessage;

impl Pilingonmessage {
    pub fn new() -> Self { Self }
}

impl Default for Pilingonmessage {
    fn default() -> Self { Self::new() }
}
