use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.INetCommandHandler`.
/// Callback interface for objects that handle incoming `NetCommand` payloads.
pub trait INetCommandHandler {
    fn handle_command(&mut self, command: &dyn NetCommand);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::NetCommandId;

    struct Dummy;
    impl NetCommand for Dummy {
        fn get_id(&self) -> NetCommandId {
            NetCommandId::ClientJoin
        }
    }

    struct Counter {
        count: usize,
    }

    impl INetCommandHandler for Counter {
        fn handle_command(&mut self, _command: &dyn NetCommand) {
            self.count += 1;
        }
    }

    #[test]
    fn handle_command_increments_counter() {
        let mut h = Counter { count: 0 };
        let cmd = Dummy;
        h.handle_command(&cmd);
        assert_eq!(h.count, 1);
    }

    #[test]
    fn handle_command_called_multiple_times() {
        let mut h = Counter { count: 0 };
        let cmd = Dummy;
        h.handle_command(&cmd);
        h.handle_command(&cmd);
        assert_eq!(h.count, 2);
    }
}
