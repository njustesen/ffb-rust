// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Cloudburstermessage;

impl Cloudburstermessage {
    pub fn new() -> Self { Self }
}

impl Default for Cloudburstermessage {
    fn default() -> Self { Self::new() }
}
