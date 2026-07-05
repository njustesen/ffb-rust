// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Trapdoormessage;

impl Trapdoormessage {
    pub fn new() -> Self { Self }
}

impl Default for Trapdoormessage {
    fn default() -> Self { Self::new() }
}
