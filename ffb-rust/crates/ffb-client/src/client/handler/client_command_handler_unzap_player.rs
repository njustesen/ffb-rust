//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerUnzapPlayer`.
//!
//! See `client_command_handler_zap_player.rs` for the module-level note on why
//! this operates on `&mut Game` directly instead of through
//! `FantasyFootballClient`/`ClientData` (still empty stubs): Java's
//! `player instanceof ZappedPlayer` check becomes `Game::is_zapped_player`, and
//! restoring the original `RosterPlayer` mirrors `Game::unzap_all_players` but
//! for a single targeted player id (that method unzaps every zapped player at
//! once, e.g. at end of drive, so it isn't reused here).

use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerUnzapPlayer;

impl ClientCommandHandlerUnzapPlayer {
    pub fn new() -> Self {
        Self
    }

    /// Java: `getId()`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUnzapPlayer
    }

    /// Java:
    /// ```java
    /// ServerCommandUnzapPlayer command = (ServerCommandUnzapPlayer) pNetCommand;
    /// Team team = getClient().getGame().getTeamById(command.getTeamId());
    /// Player<?> player = team.getPlayerById(command.getPlayerId());
    /// if (player instanceof ZappedPlayer) {
    ///     ZappedPlayer zappedPlayer = (ZappedPlayer) player;
    ///     RosterPlayer rosterPlayer = zappedPlayer.getOriginalPlayer();
    ///     team.addPlayer(rosterPlayer);
    ///     getClient().getGame().getFieldModel().sendPosition(rosterPlayer);
    /// }
    /// return true;
    /// ```
    pub fn handle_net_command(
        &mut self,
        game: &mut Game,
        net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        if let AnyServerCommand::ServerUnzapPlayer(command) = net_command {
            let team_id = command.get_team_id();
            let player_id = command.get_player_id().to_string();
            let team_has_player = game
                .team_by_id(team_id)
                .map(|team| team.player(&player_id).is_some())
                .unwrap_or(false);
            // Java: `player instanceof ZappedPlayer` — true only when this player is
            // currently tracked as zapped.
            if team_has_player && game.is_zapped_player(&player_id) {
                if let Some(pos) = game
                    .zapped_players
                    .iter()
                    .position(|z| z.original_player.id == player_id)
                {
                    let zapped = game.zapped_players.remove(pos);
                    if let Some(p) = game.player_mut(&player_id) {
                        *p = zapped.original_player;
                    }
                    // java: getClient().getGame().getFieldModel().sendPosition(rosterPlayer);
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::model::zapped_player::ZappedPlayer;
    use ffb_model::model::zapped_position::ZappedPosition;
    use ffb_protocol::commands::server_command_unzap_player::ServerCommandUnzapPlayer;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: "Bob".into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, player: Player) -> Team {
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
            players: vec![player],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game_with_zapped_player() -> Game {
        let mut game = Game::new(
            make_team("home", make_player("p1")),
            make_team("away", make_player("away1")),
            Rules::Bb2020,
        );
        let original = game.player("p1").unwrap().clone();
        let position = ZappedPosition::new_bb2020(original.position_id.clone(), original.name.clone());
        game.add_zapped_player(ZappedPlayer::new(original, position));
        game
    }

    #[test]
    fn get_id_is_server_unzap_player() {
        assert_eq!(ClientCommandHandlerUnzapPlayer::new().get_id(), NetCommandId::ServerUnzapPlayer);
    }

    #[test]
    fn unzaps_a_currently_zapped_player() {
        let mut game = make_game_with_zapped_player();
        assert!(game.player("p1").unwrap().is_zapped());
        let mut handler = ClientCommandHandlerUnzapPlayer::new();
        let cmd = AnyServerCommand::ServerUnzapPlayer(ServerCommandUnzapPlayer::new("home", "p1"));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(!game.is_zapped_player("p1"));
        assert!(!game.player("p1").unwrap().is_zapped());
        assert_eq!(game.player("p1").unwrap().agility, 3);
    }

    #[test]
    fn does_nothing_for_a_player_that_is_not_zapped() {
        let mut game = Game::new(
            make_team("home", make_player("p1")),
            make_team("away", make_player("away1")),
            Rules::Bb2020,
        );
        let mut handler = ClientCommandHandlerUnzapPlayer::new();
        let cmd = AnyServerCommand::ServerUnzapPlayer(ServerCommandUnzapPlayer::new("home", "p1"));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(game.zapped_players.is_empty());
    }

    #[test]
    fn unknown_player_id_is_a_no_op_but_still_returns_true() {
        let mut game = make_game_with_zapped_player();
        let mut handler = ClientCommandHandlerUnzapPlayer::new();
        let cmd = AnyServerCommand::ServerUnzapPlayer(ServerCommandUnzapPlayer::new("home", "ghost"));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(game.zapped_players.len(), 1);
    }

    #[test]
    fn wrong_command_type_is_ignored_but_returns_true() {
        let mut game = make_game_with_zapped_player();
        let mut handler = ClientCommandHandlerUnzapPlayer::new();
        let cmd = AnyServerCommand::ServerClearSketches(
            ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches::new(),
        );
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(game.zapped_players.len(), 1);
    }
}
