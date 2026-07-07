/// 1:1 translation of ClientCommandUseChainsaw (Java field: usingChainsaw).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseChainsaw {
    pub using_chainsaw: bool,
}

impl ClientCommandUseChainsaw {
    pub fn new(using_chainsaw: bool) -> Self {
        Self { using_chainsaw }
    }

    pub fn is_using_chainsaw(&self) -> bool {
        self.using_chainsaw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_true_stores_true() {
        let cmd = ClientCommandUseChainsaw::new(true);
        assert!(cmd.is_using_chainsaw());
    }

    #[test]
    fn new_false_stores_false() {
        let cmd = ClientCommandUseChainsaw::new(false);
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandUseChainsaw::default();
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandUseChainsaw::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseChainsaw::default().clone();
    }
}
