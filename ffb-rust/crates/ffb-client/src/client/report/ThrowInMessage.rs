// client-only: Java Swing StatusReport message renderer — no headless text output.
pub struct Throwinmessage;

impl Throwinmessage {
    pub fn new() -> Self { Self }
}

impl Default for Throwinmessage {
    fn default() -> Self { Self::new() }
}
