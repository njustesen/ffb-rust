//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerZapPlayer`.
//!
//! Java's model has `Player<?>` as an abstract base with `RosterPlayer` and
//! `ZappedPlayer` as sibling subclasses stored interchangeably in
//! `Team.players` — the handler tells them apart with `instanceof`. In the
//! Rust model (`ffb-model`), `Team::players` is a homogeneous `Vec<Player>`
//! and "is this player currently zapped" is tracked with `Player::zapped`
//! (`Game::zapped_players` holds the saved originals) — see
//! `ffb-engine/src/step/bb2020/step_special_effect.rs` for the existing
//! zap-application code this mirrors via `Game::add_zapped_player`.
//!
//! This handler therefore operates directly on `&mut Game` (a plain model
//! type, not a GUI object) rather than through `FantasyFootballClient`/
//! `ClientData`, which are still empty stubs.

use ffb_model::enums::{NetCommandId, Rules};
use ffb_model::model::game::Game;
use ffb_model::model::zapped_player::ZappedPlayer;
use ffb_model::model::zapped_position::ZappedPosition;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerZapPlayer;

impl ClientCommandHandlerZapPlayer {
    pub fn new() -> Self {
        Self
    }

    /// Java: `getId()`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerZapPlayer
    }

    /// Java:
    /// ```java
    /// ServerCommandZapPlayer command = (ServerCommandZapPlayer) pNetCommand;
    /// Team team = getClient().getGame().getTeamById(command.getTeamId());
    /// Player<?> player = team.getPlayerById(command.getPlayerId());
    /// if (player instanceof RosterPlayer) {
    ///     RosterPlayer rosterPlayer = (RosterPlayer) player;
    ///     ZappedPlayer zappedPlayer = new ZappedPlayer();
    ///     zappedPlayer.init(rosterPlayer, getClient().getGame().getRules());
    ///     team.addPlayer(zappedPlayer);
    ///     getClient().getGame().getFieldModel().sendPosition(player);
    /// }
    /// return true;
    /// ```
    pub fn handle_net_command(
        &mut self,
        game: &mut Game,
        net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        if let AnyServerCommand::ServerZapPlayer(command) = net_command {
            let team_id = command.get_team_id();
            let player_id = command.get_player_id();
            let player = game
                .team_by_id(team_id)
                .and_then(|team| team.player(player_id))
                .cloned();
            if let Some(player) = player {
                // Java: `player instanceof RosterPlayer` — true when the player has not
                // already been zapped this drive.
                if !player.is_zapped() {
                    let position = match game.rules {
                        Rules::Bb2016 => {
                            ZappedPosition::new_bb2016(player.position_id.clone(), player.name.clone())
                        }
                        _ => ZappedPosition::new_bb2020(player.position_id.clone(), player.name.clone()),
                    };
                    game.add_zapped_player(ZappedPlayer::new(player, position));
                    // java: getClient().getGame().getFieldModel().sendPosition(player);
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_protocol::commands::server_command_zap_player::ServerCommandZapPlayer;

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

    fn make_game(home_player: Player) -> Game {
        Game::new(
            make_team("home", home_player),
            make_team("away", make_player("away1")),
            Rules::Bb2020,
        )
    }

    #[test]
    fn get_id_is_server_zap_player() {
        assert_eq!(ClientCommandHandlerZapPlayer::new().get_id(), NetCommandId::ServerZapPlayer);
    }

    #[test]
    fn zaps_a_not_yet_zapped_player() {
        let mut game = make_game(make_player("p1"));
        let mut handler = ClientCommandHandlerZapPlayer::new();
        let cmd = AnyServerCommand::ServerZapPlayer(ServerCommandZapPlayer::new("home", "p1"));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(game.is_zapped_player("p1"));
        assert!(game.player("p1").unwrap().is_zapped());
    }

    #[test]
    fn does_not_zap_an_already_zapped_player_again() {
        let mut game = make_game(make_player("p1"));
        let mut handler = ClientCommandHandlerZapPlayer::new();
        let cmd = AnyServerCommand::ServerZapPlayer(ServerCommandZapPlayer::new("home", "p1"));
        handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING);
        assert_eq!(game.zapped_players.len(), 1);
        handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING);
        assert_eq!(game.zapped_players.len(), 1);
    }

    #[test]
    fn unknown_player_id_is_a_no_op_but_still_returns_true() {
        let mut game = make_game(make_player("p1"));
        let mut handler = ClientCommandHandlerZapPlayer::new();
        let cmd = AnyServerCommand::ServerZapPlayer(ServerCommandZapPlayer::new("home", "ghost"));
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(game.zapped_players.is_empty());
    }

    #[test]
    fn wrong_command_type_is_ignored_but_returns_true() {
        let mut game = make_game(make_player("p1"));
        let mut handler = ClientCommandHandlerZapPlayer::new();
        let cmd = AnyServerCommand::ServerClearSketches(
            ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches::new(),
        );
        assert!(handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING));
        assert!(game.zapped_players.is_empty());
    }

    #[test]
    fn bb2016_zap_uses_bb2016_stats() {
        let mut game = Game::new(
            make_team("home", make_player("p1")),
            make_team("away", make_player("away1")),
            Rules::Bb2016,
        );
        let mut handler = ClientCommandHandlerZapPlayer::new();
        let cmd = AnyServerCommand::ServerZapPlayer(ServerCommandZapPlayer::new("home", "p1"));
        handler.handle_net_command(&mut game, &cmd, ClientCommandHandlerMode::PLAYING);
        // BB2016 zap agility is 4 (vs BB2020's 2).
        assert_eq!(game.player("p1").unwrap().agility, 4);
    }
}
