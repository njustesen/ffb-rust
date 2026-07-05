// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Pettycashmessage;

impl Pettycashmessage {
    pub fn new() -> Self { Self }
}

impl Default for Pettycashmessage {
    fn default() -> Self { Self::new() }
}
