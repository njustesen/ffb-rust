// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Bitespectatormessage;

impl Bitespectatormessage {
    pub fn new() -> Self { Self }
}

impl Default for Bitespectatormessage {
    fn default() -> Self { Self::new() }
}
