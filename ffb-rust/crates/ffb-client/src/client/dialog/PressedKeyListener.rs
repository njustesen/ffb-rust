// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Pressedkeylistener;

impl Pressedkeylistener {
    pub fn new() -> Self { Self }
}

impl Default for Pressedkeylistener {
    fn default() -> Self { Self::new() }
}
