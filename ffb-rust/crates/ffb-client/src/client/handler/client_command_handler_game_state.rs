//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerGameState`.
//!
//! Java's `handleNetCommand` is mostly icon preloading (`IconCache`,
//! `PlayerIconFactory`, a `ForkJoinPool` of `LoadTask`s) and a Swing UI refresh —
//! all GUI/network-coupled with no Rust equivalent. The one piece of real game
//! logic (`subHandler.handleNetCommand(...)`, i.e. `SubHandlerGameStateMarking`)
//! is delegated to and exposed via `apply_game_state` for direct testing.

use ffb_model::enums::NetCommandId;
use ffb_model::model::client_mode::ClientMode;
use ffb_model::model::game::Game;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;
use crate::client::handler::sub_handler_game_state_marking::SubHandlerGameStateMarking;

#[derive(Default)]
pub struct ClientCommandHandlerGameState {
    sub_handler: SubHandlerGameStateMarking,
}

impl ClientCommandHandlerGameState {
    pub fn new() -> Self {
        Self { sub_handler: SubHandlerGameStateMarking::new() }
    }

    /// Java: `Game game = subHandler.handleNetCommand(gameStateCommand);` plus
    /// `getClient().initRulesDependentMembers();`. Icon downloading/preloading and
    /// the `SwingUtilities.invokeAndWait` UI refresh block (only run when
    /// `pMode == PLAYING`) have no Rust equivalent and are left as `// java:` notes.
    pub fn apply_game_state(
        &self,
        existing_game: &Game,
        incoming_game: Game,
        client_mode: ClientMode,
        is_manual_marking: bool,
    ) -> Game {
        let game = self.sub_handler.handle_net_command(existing_game, incoming_game, client_mode, is_manual_marking);

        // java: IconCache iconCache = getClient().getUserInterface().getIconCache();
        // java: Set<String> iconUrls = new HashSet<>(); addIconUrl(...) for team logos, roster
        // java: portraits/iconsets, players, zapped-player icon, and pitch urls; download any
        // java: not already cached via a ForkJoinPool of LoadTask callables.
        // java: UtilClientThrowTeamMate.updateThrownPlayer(getClient());
        // java: if (pMode == PLAYING) { SwingUtilities.invokeAndWait(() -> {
        // java:     userInterface.init(game.getOptions()); getClient().updateClientState();
        // java:     userInterface.getDialogManager().updateDialog();
        // java:     userInterface.getGameMenuBar().updateMissingPlayers(); ...updateInducements();
        // java:     userInterface.getChat().requestChatInputFocus();
        // java: }); }
        // java: getClient().initRulesDependentMembers();

        game
    }

    /// Java: `dialogClosed(IDialog pDialog)`.
    pub fn dialog_closed(&self) {
        // java: getClient().exitClient();
    }
}

impl ClientCommandHandler for ClientCommandHandlerGameState {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerGameState
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, _mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerGameState(_command) = net_command {
            // java: Self::apply_game_state(...) would run here once a live existing Game and
            // client mode are reachable from a working FantasyFootballClient; see
            // `apply_game_state`.
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::marking::player_marker::PlayerMarker;
    use ffb_model::model::SoundId;
    use ffb_model::model::team::Team;
    use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn get_id_is_server_game_state() {
        assert_eq!(ClientCommandHandlerGameState::new().get_id(), NetCommandId::ServerGameState);
    }

    #[test]
    fn apply_game_state_keeps_existing_player_markers_on_initial_state() {
        let mut existing = make_game();
        existing.id = 0;
        existing.field_model.player_markers.push(PlayerMarker::with_player_id("p1"));
        let incoming = make_game();

        let handler = ClientCommandHandlerGameState::new();
        let result = handler.apply_game_state(&existing, incoming, ClientMode::PLAYER, false);
        assert_eq!(result.field_model.player_markers.len(), 1);
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerGameState::new();
        let cmd = AnyServerCommand::ServerGameState(ServerCommandGameState::new(Some(make_game())));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerGameState::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn dialog_closed_does_not_panic() {
        ClientCommandHandlerGameState::new().dialog_closed();
    }
}
