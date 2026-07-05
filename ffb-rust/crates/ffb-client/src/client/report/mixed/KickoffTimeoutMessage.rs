// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Kickofftimeoutmessage;

impl Kickofftimeoutmessage {
    pub fn new() -> Self { Self }
}

impl Default for Kickofftimeoutmessage {
    fn default() -> Self { Self::new() }
}
