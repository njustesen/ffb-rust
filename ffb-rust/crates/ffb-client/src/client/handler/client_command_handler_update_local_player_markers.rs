//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerUpdateLocalPlayerMarkers`.
//!
//! Operates on `&mut Game` directly (a plain model type, not a GUI object) —
//! see `client_command_handler_zap_player.rs` for why: `FantasyFootballClient`/
//! `ClientData`/`UserInterface` are still empty stubs.

use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerUpdateLocalPlayerMarkers;

impl ClientCommandHandlerUpdateLocalPlayerMarkers {
    pub fn new() -> Self {
        Self
    }

    /// Java: `getId()`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUpdateLocalPlayerMarkers
    }

    /// Java:
    /// ```java
    /// ServerCommandUpdateLocalPlayerMarkers commandUpdateLocalPlayerMarkers = (ServerCommandUpdateLocalPlayerMarkers) pNetCommand;
    /// FieldModel fieldModel = getClient().getGame().getFieldModel();
    /// for (PlayerMarker marker : fieldModel.getPlayerMarkers()) {
    ///     fieldModel.remove(marker);
    ///     getClient().getGame().notifyObservers(ModelChangeId.FIELD_MODEL_REMOVE_PLAYER_MARKER, null, marker);
    /// }
    /// commandUpdateLocalPlayerMarkers.getMarkers().forEach(marker -> {
    ///     fieldModel.add(marker);
    ///     getClient().getGame().notifyObservers(ModelChangeId.FIELD_MODEL_ADD_PLAYER_MARKER, null, marker);
    /// });
    /// getClient().getUserInterface().refresh();
    /// return true;
    /// ```
    /// `notifyObservers` (the model-change/observer pattern) and
    /// `UserInterface.refresh()` are GUI side effects with no Rust equivalent
    /// yet — noted with `// java:` rather than invented.
    pub fn handle_net_command(
        &mut self,
        game: &mut Game,
        net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        if let AnyServerCommand::ServerUpdateLocalPlayerMarkers(command) = net_command {
            let existing_ids: Vec<String> = game
                .field_model
                .get_player_markers()
                .iter()
                .filter_map(|marker| marker.get_player_id().map(|s| s.to_string()))
                .collect();
            for player_id in existing_ids {
                game.field_model.remove_player_marker(&player_id);
                // java: getClient().getGame().notifyObservers(ModelChangeId.FIELD_MODEL_REMOVE_PLAYER_MARKER, null, marker);
            }
            for marker in command.get_markers() {
                game.field_model.add_player_marker(marker.clone());
                // java: getClient().getGame().notifyObservers(ModelChangeId.FIELD_MODEL_ADD_PLAYER_MARKER, null, marker);
            }
            // java: getClient().getUserInterface().refresh();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::marking::player_marker::PlayerMarker;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_protocol::commands::server_command_update_local_player_markers::ServerCommandUpdateLocalPlayerMarkers;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![Player::default()],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn get_id_is_server_update_local_player_markers() {
        assert_eq!(
            ClientCommandHandlerUpdateLocalPlayerMarkers::new().get_id(),
            NetCommandId::ServerUpdateLocalPlayerMarkers
        );
    }

    #[test]
    fn replaces_existing_markers_with_the_new_set() {
        let mut game = make_game();
        game.field_model.add_player_marker(PlayerMarker::with_player_id("old"));
        let mut handler = ClientCommandHandlerUpdateLocalPlayerMarkers::new();
        let cmd = AnyServerCommand::ServerUpdateLocalPlayerMarkers(
            ServerCommandUpdateLocalPlayerMarkers::new(vec![PlayerMarker::with_player_id("new")]),
        );
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        let markers = game.field_model.get_player_markers();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].get_player_id(), Some("new"));
    }

    #[test]
    fn empty_marker_list_clears_all_markers() {
        let mut game = make_game();
        game.field_model.add_player_marker(PlayerMarker::with_player_id("old"));
        let mut handler = ClientCommandHandlerUpdateLocalPlayerMarkers::new();
        let cmd = AnyServerCommand::ServerUpdateLocalPlayerMarkers(
            ServerCommandUpdateLocalPlayerMarkers::new(vec![]),
        );
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(game.field_model.get_player_markers().is_empty());
    }

    #[test]
    fn wrong_command_type_is_ignored_but_returns_true() {
        let mut game = make_game();
        game.field_model.add_player_marker(PlayerMarker::with_player_id("old"));
        let mut handler = ClientCommandHandlerUpdateLocalPlayerMarkers::new();
        let cmd = AnyServerCommand::ServerClearSketches(
            ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches::new(),
        );
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(game.field_model.get_player_markers().len(), 1);
    }

    #[test]
    fn multiple_new_markers_are_all_added() {
        let mut game = make_game();
        let mut handler = ClientCommandHandlerUpdateLocalPlayerMarkers::new();
        let cmd = AnyServerCommand::ServerUpdateLocalPlayerMarkers(ServerCommandUpdateLocalPlayerMarkers::new(
            vec![PlayerMarker::with_player_id("a"), PlayerMarker::with_player_id("b")],
        ));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(game.field_model.get_player_markers().len(), 2);
    }
}
