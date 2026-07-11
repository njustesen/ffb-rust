/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandFumbblGameChecked.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandFumbblGameChecked {
    pub game_id: i64,
}

impl InternalServerCommandFumbblGameChecked {
    pub fn new(game_id: i64) -> Self {
        Self { game_id }
    }
}

impl InternalServerCommand for InternalServerCommandFumbblGameChecked {
    fn get_id(&self) -> &'static str {
        "internalServerFumbblGameChecked"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandFumbblGameChecked::new(42);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandFumbblGameChecked::new(1);
        assert_eq!(c.get_id(), "internalServerFumbblGameChecked");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandFumbblGameChecked::new(7);
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandFumbblGameChecked::new(1);
        assert!(c.is_internal());
    }
}
