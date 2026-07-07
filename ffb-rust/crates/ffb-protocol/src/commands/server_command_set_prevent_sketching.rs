/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSetPreventSketching`.
/// Instructs the client whether to allow or block sketching.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSetPreventSketching {
    /// Java: `preventSketching` — true = sketching disabled.
    pub prevent_sketching: bool,
}

impl ServerCommandSetPreventSketching {
    pub fn new(prevent_sketching: bool) -> Self { Self { prevent_sketching } }
    pub fn is_prevent_sketching(&self) -> bool { self.prevent_sketching }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_stored() {
        let cmd = ServerCommandSetPreventSketching::new(true);
        assert!(cmd.is_prevent_sketching());
    }

    #[test]
    fn default_allow() {
        let cmd = ServerCommandSetPreventSketching::default();
        assert!(!cmd.prevent_sketching);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSetPreventSketching::default()).is_empty());
    }

}
