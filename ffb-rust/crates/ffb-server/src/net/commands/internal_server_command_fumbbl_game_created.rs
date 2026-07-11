/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandFumbblGameCreated.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandFumbblGameCreated {
    pub game_id: i64,
}

impl InternalServerCommandFumbblGameCreated {
    pub fn new(game_id: i64) -> Self {
        Self { game_id }
    }
}

impl InternalServerCommand for InternalServerCommandFumbblGameCreated {
    fn get_id(&self) -> &'static str {
        "internalServerFumbblGameCreated"
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
        let _ = InternalServerCommandFumbblGameCreated::new(42);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandFumbblGameCreated::new(1);
        assert_eq!(c.get_id(), "internalServerFumbblGameCreated");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandFumbblGameCreated::new(7);
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandFumbblGameCreated::new(1);
        assert!(c.is_internal());
    }
}
