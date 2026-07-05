// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Clientdata;

impl Clientdata {
    pub fn new() -> Self { Self }
}

impl Default for Clientdata {
    fn default() -> Self { Self::new() }
}
