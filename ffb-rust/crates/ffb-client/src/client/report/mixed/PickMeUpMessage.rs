// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Pickmeupmessage;

impl Pickmeupmessage {
    pub fn new() -> Self { Self }
}

impl Default for Pickmeupmessage {
    fn default() -> Self { Self::new() }
}
