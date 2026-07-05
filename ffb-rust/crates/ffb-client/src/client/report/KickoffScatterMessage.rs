// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Kickoffscattermessage;

impl Kickoffscattermessage {
    pub fn new() -> Self { Self }
}

impl Default for Kickoffscattermessage {
    fn default() -> Self { Self::new() }
}
