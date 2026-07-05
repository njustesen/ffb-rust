// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Leftclicknomodifierstrategy;

impl Leftclicknomodifierstrategy {
    pub fn new() -> Self { Self }
}

impl Default for Leftclicknomodifierstrategy {
    fn default() -> Self { Self::new() }
}
