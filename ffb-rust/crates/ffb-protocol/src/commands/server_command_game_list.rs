use ffb_model::model::game_list::GameList;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameList`.
/// Sends the lobby game list to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandGameList {
    /// Java: `fGameList` — the list of available games.
    pub game_list: GameList,
}

impl ServerCommandGameList {
    pub fn new(game_list: GameList) -> Self { Self { game_list } }
    pub fn get_game_list(&self) -> &GameList { &self.game_list }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandGameList::new(GameList::default());
        let _ = cmd.get_game_list();
    }

    #[test]
    fn default_works() {
        let _ = ServerCommandGameList::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandGameList::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandGameList::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandGameList::default());
        assert!(s.contains("ServerCommandGameList"));
    }
}
