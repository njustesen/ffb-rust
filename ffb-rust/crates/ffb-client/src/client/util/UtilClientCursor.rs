// client-only: Java AWT/Swing utility — no headless equivalent.
pub struct Utilclientcursor;

impl Utilclientcursor {
    pub fn new() -> Self { Self }
}

impl Default for Utilclientcursor {
    fn default() -> Self { Self::new() }
}
