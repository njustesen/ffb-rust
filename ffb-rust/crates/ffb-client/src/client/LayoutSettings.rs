// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Layoutsettings;

impl Layoutsettings {
    pub fn new() -> Self { Self }
}

impl Default for Layoutsettings {
    fn default() -> Self { Self::new() }
}
