/// 1:1 translation of `com.fumbbl.ffb.net.IConnectionListener`.
/// Callback interface notified when a WebSocket connection attempt completes.
pub trait IConnectionListener {
    /// Called when the connection either succeeds (`successful = true`) or fails.
    fn connection_established(&mut self, successful: bool);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        result: bool,
    }

    impl IConnectionListener for MockListener {
        fn connection_established(&mut self, successful: bool) {
            self.called = true;
            self.result = successful;
        }
    }

    #[test]
    fn listener_called_on_success() {
        let mut l = MockListener { called: false, result: false };
        l.connection_established(true);
        assert!(l.called);
        assert!(l.result);
    }

    #[test]
    fn listener_called_on_failure() {
        let mut l = MockListener { called: false, result: true };
        l.connection_established(false);
        assert!(l.called);
        assert!(!l.result);
    }
}
