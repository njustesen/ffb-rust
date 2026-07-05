// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Bomboutofboundsmessage;

impl Bomboutofboundsmessage {
    pub fn new() -> Self { Self }
}

impl Default for Bomboutofboundsmessage {
    fn default() -> Self { Self::new() }
}
