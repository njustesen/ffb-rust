// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Emojipicker;

impl Emojipicker {
    pub fn new() -> Self { Self }
}

impl Default for Emojipicker {
    fn default() -> Self { Self::new() }
}
