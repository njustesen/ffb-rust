/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameTime`.
/// Sends current game clock and turn clock to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandGameTime {
    /// Java: `fGameTime` — total elapsed game time in ms.
    pub game_time: i64,
    /// Java: `fTurnTime` — elapsed time for the current turn in ms.
    pub turn_time: i64,
}

impl ServerCommandGameTime {
    pub fn new(game_time: i64, turn_time: i64) -> Self {
        Self { game_time, turn_time }
    }
    pub fn get_game_time(&self) -> i64 { self.game_time }
    pub fn get_turn_time(&self) -> i64 { self.turn_time }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandGameTime::new(60_000, 30_000);
        assert_eq!(cmd.get_game_time(), 60_000);
        assert_eq!(cmd.get_turn_time(), 30_000);
    }

    #[test]
    fn default_zeros() {
        let cmd = ServerCommandGameTime::default();
        assert_eq!(cmd.game_time, 0);
        assert_eq!(cmd.turn_time, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandGameTime::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandGameTime::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandGameTime::default());
        assert!(s.contains("ServerCommandGameTime"));
    }
}
