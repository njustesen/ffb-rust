// client-only: Java AWT/Swing utility — no headless equivalent.
pub struct Markerservice;

impl Markerservice {
    pub fn new() -> Self { Self }
}

impl Default for Markerservice {
    fn default() -> Self { Self::new() }
}
