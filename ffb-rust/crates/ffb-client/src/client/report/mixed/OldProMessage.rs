// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Oldpromessage;

impl Oldpromessage {
    pub fn new() -> Self { Self }
}

impl Default for Oldpromessage {
    fn default() -> Self { Self::new() }
}
