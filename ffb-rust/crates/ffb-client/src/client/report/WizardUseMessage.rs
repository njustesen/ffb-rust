// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Wizardusemessage;

impl Wizardusemessage {
    pub fn new() -> Self { Self }
}

impl Default for Wizardusemessage {
    fn default() -> Self { Self::new() }
}
