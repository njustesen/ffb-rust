/// 1:1 translation of `com.fumbbl.ffb.net.commands.ICommandWithActingPlayer`.
/// Marker trait for commands that carry an acting player reference.
pub trait ICommandWithActingPlayer {
    /// Returns the id of the player performing the action.
    fn acting_player_id(&self) -> Option<&str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCmd {
        player_id: Option<String>,
    }

    impl ICommandWithActingPlayer for TestCmd {
        fn acting_player_id(&self) -> Option<&str> {
            self.player_id.as_deref()
        }
    }

    #[test]
    fn returns_some_when_set() {
        let cmd = TestCmd { player_id: Some("p1".into()) };
        assert_eq!(cmd.acting_player_id(), Some("p1"));
    }

    #[test]
    fn returns_none_when_unset() {
        let cmd = TestCmd { player_id: None };
        assert!(cmd.acting_player_id().is_none());
    }
}
