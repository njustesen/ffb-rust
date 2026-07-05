// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Receivechoicemessage;

impl Receivechoicemessage {
    pub fn new() -> Self { Self }
}

impl Default for Receivechoicemessage {
    fn default() -> Self { Self::new() }
}
