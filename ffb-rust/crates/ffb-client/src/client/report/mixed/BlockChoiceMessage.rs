// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Blockchoicemessage;

impl Blockchoicemessage {
    pub fn new() -> Self { Self }
}

impl Default for Blockchoicemessage {
    fn default() -> Self { Self::new() }
}
