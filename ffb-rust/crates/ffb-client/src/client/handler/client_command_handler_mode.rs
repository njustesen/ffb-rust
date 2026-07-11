/// 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerMode`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientCommandHandlerMode {
    PLAYING,
    REPLAYING,
    INITIALIZING,
    QUEUING,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(ClientCommandHandlerMode::PLAYING, ClientCommandHandlerMode::REPLAYING);
        assert_ne!(ClientCommandHandlerMode::INITIALIZING, ClientCommandHandlerMode::QUEUING);
    }

    #[test]
    fn copy_and_clone_work() {
        let mode = ClientCommandHandlerMode::PLAYING;
        let copied = mode;
        let cloned = mode.clone();
        assert_eq!(copied, cloned);
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandHandlerMode::QUEUING).is_empty());
    }

    #[test]
    fn all_four_variants_exist() {
        let all = [
            ClientCommandHandlerMode::PLAYING,
            ClientCommandHandlerMode::REPLAYING,
            ClientCommandHandlerMode::INITIALIZING,
            ClientCommandHandlerMode::QUEUING,
        ];
        assert_eq!(all.len(), 4);
    }
}
