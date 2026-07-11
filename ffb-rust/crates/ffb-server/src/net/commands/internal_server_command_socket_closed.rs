/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandSocketClosed.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandSocketClosed;

impl InternalServerCommandSocketClosed {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InternalServerCommandSocketClosed {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalServerCommand for InternalServerCommandSocketClosed {
    fn get_id(&self) -> &'static str {
        "internalServerSocketClosed"
    }

    fn get_game_id(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandSocketClosed::new();
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandSocketClosed::new();
        assert_eq!(c.get_id(), "internalServerSocketClosed");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandSocketClosed::new();
        assert_eq!(c.get_game_id(), 0);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandSocketClosed::new();
        assert!(c.is_internal());
    }

    #[test]
    fn default() {
        let _ = InternalServerCommandSocketClosed::default();
    }
}
