// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Puntdistancemessage;

impl Puntdistancemessage {
    pub fn new() -> Self { Self }
}

impl Default for Puntdistancemessage {
    fn default() -> Self { Self::new() }
}
