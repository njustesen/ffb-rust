// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Hitandrunmessage;

impl Hitandrunmessage {
    pub fn new() -> Self { Self }
}

impl Default for Hitandrunmessage {
    fn default() -> Self { Self::new() }
}
