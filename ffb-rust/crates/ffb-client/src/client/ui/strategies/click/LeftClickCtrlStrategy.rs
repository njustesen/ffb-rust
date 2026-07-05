// client-only: Java Swing/AWT rendering component — no Rust UI equivalent.
pub struct Leftclickctrlstrategy;

impl Leftclickctrlstrategy {
    pub fn new() -> Self { Self }
}

impl Default for Leftclickctrlstrategy {
    fn default() -> Self { Self::new() }
}
