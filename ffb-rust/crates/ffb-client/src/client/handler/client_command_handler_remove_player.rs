//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerRemovePlayer`.

use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use ffb_protocol::commands::any_server_command::AnyServerCommand;
use ffb_protocol::commands::server_command_remove_player::ServerCommandRemovePlayer;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerRemovePlayer;

impl ClientCommandHandlerRemovePlayer {
    pub fn new() -> Self {
        Self
    }

    /// Java: the game-model-mutation portion of `handleNetCommand`, i.e. everything
    /// reachable via `getClient().getGame()`. `FantasyFootballClient`/`ClientData` are
    /// still GUI stubs with no working `getGame()`, so this is exposed as a free
    /// function taking `game: &mut Game` directly, testable independent of the GUI.
    ///
    /// ```java
    /// Game game = getClient().getGame();
    /// GameResult gameResult = game.getGameResult();
    /// Player<?> player = game.getPlayerById(removePlayerCommand.getPlayerId());
    /// game.getFieldModel().remove(player);
    /// game.getFieldModel().setPlayerState(player, null);
    /// if (game.getTeamHome().hasPlayer(player)) {
    ///     game.getTeamHome().removePlayer(player);
    ///     gameResult.getTeamResultHome().removePlayerResult(player);
    /// }
    /// if (game.getTeamAway().hasPlayer(player)) {
    ///     game.getTeamAway().removePlayer(player);
    ///     gameResult.getTeamResultAway().removePlayerResult(player);
    /// }
    /// ```
    pub fn apply_to_game(command: &ServerCommandRemovePlayer, game: &mut Game) {
        let player_id = command.get_player_id();

        // Java calls both `getFieldModel().remove(player)` and
        // `getFieldModel().setPlayerState(player, null)`; the Rust `FieldModel::remove_player`
        // already clears both the player's coordinate and state map entries, so a single
        // call covers both Java calls (no separate "set state to null" API exists).
        game.field_model.remove_player(player_id);

        if game.team_home.has_player(player_id) {
            game.team_home.players.retain(|p| p.id != player_id);
            game.game_result.team_result_mut(true).player_results.remove(player_id);
        }
        if game.team_away.has_player(player_id) {
            game.team_away.players.retain(|p| p.id != player_id);
            game.game_result.team_result_mut(false).player_results.remove(player_id);
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerRemovePlayer {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerRemovePlayer
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerRemovePlayer(_command) = net_command {
            // java: Self::apply_to_game(_command, getClient().getGame()) would run here once a
            // live Game is reachable from a working FantasyFootballClient; see `apply_to_game`.
            if mode == ClientCommandHandlerMode::PLAYING {
                // java: refreshGameMenuBar(); refreshFieldComponent(); refreshSideBars();
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
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

    fn make_game_with_player(player_id: &str) -> Game {
        let mut home = make_team("home");
        home.players.push(Player { id: player_id.into(), ..Player::default() });
        let away = make_team("away");
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.field_model.set_player_state(player_id, ffb_model::enums::PlayerState::new(0));
        game.game_result.team_result_mut(true).player_results.insert(player_id.to_string(), Default::default());
        game
    }

    #[test]
    fn get_id_is_server_remove_player() {
        assert_eq!(ClientCommandHandlerRemovePlayer::new().get_id(), NetCommandId::ServerRemovePlayer);
    }

    #[test]
    fn apply_to_game_removes_player_from_home_team() {
        let mut game = make_game_with_player("p1");
        assert!(game.team_home.has_player("p1"));
        ClientCommandHandlerRemovePlayer::apply_to_game(&ServerCommandRemovePlayer::new("p1"), &mut game);
        assert!(!game.team_home.has_player("p1"));
        assert!(game.game_result.team_result(true).player_result("p1").is_none());
    }

    #[test]
    fn apply_to_game_clears_field_model_state() {
        let mut game = make_game_with_player("p1");
        assert!(game.field_model.player_state("p1").is_some());
        ClientCommandHandlerRemovePlayer::apply_to_game(&ServerCommandRemovePlayer::new("p1"), &mut game);
        assert!(game.field_model.player_state("p1").is_none());
    }

    #[test]
    fn apply_to_game_is_a_no_op_for_unknown_player() {
        let mut game = make_game_with_player("p1");
        ClientCommandHandlerRemovePlayer::apply_to_game(&ServerCommandRemovePlayer::new("nobody"), &mut game);
        assert!(game.team_home.has_player("p1"));
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerRemovePlayer::new();
        let cmd = AnyServerCommand::ServerRemovePlayer(ServerCommandRemovePlayer::new("p1"));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerRemovePlayer::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(ffb_model::model::SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
