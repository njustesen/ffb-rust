// client-only: Java AWT/Swing utility — no headless equivalent.
pub struct Utilclienttimeout;

impl Utilclienttimeout {
    pub fn new() -> Self { Self }
}

impl Default for Utilclienttimeout {
    fn default() -> Self { Self::new() }
}
