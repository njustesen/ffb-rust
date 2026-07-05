// client-only: Java Swing/AWT client component — no Rust UI equivalent.
pub struct Rendercontext;

impl Rendercontext {
    pub fn new() -> Self { Self }
}

impl Default for Rendercontext {
    fn default() -> Self { Self::new() }
}
