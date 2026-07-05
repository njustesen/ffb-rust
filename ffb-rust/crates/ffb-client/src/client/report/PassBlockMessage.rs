// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Passblockmessage;

impl Passblockmessage {
    pub fn new() -> Self { Self }
}

impl Default for Passblockmessage {
    fn default() -> Self { Self::new() }
}
