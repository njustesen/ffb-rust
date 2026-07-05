// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Textstyle;

impl Textstyle {
    pub fn new() -> Self { Self }
}

impl Default for Textstyle {
    fn default() -> Self { Self::new() }
}
