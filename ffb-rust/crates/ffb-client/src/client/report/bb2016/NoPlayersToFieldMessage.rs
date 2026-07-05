// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Noplayerstofieldmessage;

impl Noplayerstofieldmessage {
    pub fn new() -> Self { Self }
}

impl Default for Noplayerstofieldmessage {
    fn default() -> Self { Self::new() }
}
