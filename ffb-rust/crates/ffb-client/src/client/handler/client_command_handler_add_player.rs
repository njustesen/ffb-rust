//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerAddPlayer`.

use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use ffb_protocol::commands::any_server_command::AnyServerCommand;
use ffb_protocol::commands::server_command_add_player::ServerCommandAddPlayer;

/// Java: both sides of this call use the same `SendToBoxReason` enum
/// (`com.fumbbl.ffb.SendToBoxReason`). The Rust translation ended up with two
/// independent enums of the same name — `ffb_model::model::send_to_box_reason::SendToBoxReason`
/// (used by `ServerCommandAddPlayer`, the wire command) and `ffb_model::enums::SendToBoxReason`
/// (used by `PlayerResult`, the game-result model) — with no existing conversion between
/// them. This is a pre-existing modeling split, not introduced here; variants correspond
/// 1:1 in declaration order, so this maps by name rather than inventing new behavior.
fn convert_send_to_box_reason(
    reason: ffb_model::model::send_to_box_reason::SendToBoxReason,
) -> ffb_model::enums::SendToBoxReason {
    use ffb_model::enums::SendToBoxReason as Model;
    use ffb_model::model::send_to_box_reason::SendToBoxReason as Wire;
    match reason {
        Wire::MNG => Model::Mng,
        Wire::FOUL_BAN => Model::FoulBan,
        Wire::SECRET_WEAPON_BAN => Model::SecretWeaponBan,
        Wire::FOULED => Model::Fouled,
        Wire::BLOCKED => Model::Blocked,
        Wire::CROWD_PUSHED => Model::CrowdPushed,
        Wire::CROWD_KICKED => Model::CrowdKicked,
        Wire::DODGE_FAIL => Model::DodgeFail,
        Wire::GFI_FAIL => Model::GfiFail,
        Wire::KICKED => Model::Kicked,
        Wire::JUMP_FAIL => Model::JumpFail,
        Wire::STABBED => Model::Stabbed,
        Wire::HIT_BY_ROCK => Model::HitByRock,
        Wire::EATEN => Model::Eaten,
        Wire::HIT_BY_THROWN_PLAYER => Model::HitByThrownPlayer,
        Wire::LANDING_FAIL => Model::LandingFail,
        Wire::PILED_ON => Model::PiledOn,
        Wire::CHAINSAW => Model::Chainsaw,
        Wire::BITTEN => Model::Bitten,
        Wire::NURGLES_ROT => Model::NurglesRot,
        Wire::RAISED => Model::Raised,
        Wire::LIGHTNING => Model::Lightning,
        Wire::FIREBALL => Model::Fireball,
        Wire::KO_ON_PILING_ON => Model::KoOnPilingOn,
        Wire::BOMB => Model::Bomb,
        Wire::BALL_AND_CHAIN => Model::BallAndChain,
        Wire::PLAGUE_RIDDEN => Model::PlagueRidden,
        Wire::PROJECTILE_VOMIT => Model::ProjectileVomit,
        Wire::TRAP_DOOR_FALL => Model::TrapDoorFall,
        Wire::OFFICIOUS_REF => Model::OficiousRef,
        Wire::THROWN_KEG => Model::ThrownKeg,
        Wire::THREW_TWO_BOMBS => Model::ThrewTwoBombs,
        Wire::BREATHE_FIRE => Model::BreatheFire,
        Wire::THEN_I_STARTED_BLASTIN => Model::ThenIStartedBlastin,
        Wire::QUICK_BITE => Model::QuickBite,
        Wire::SABOTEUR => Model::Saboteur,
        Wire::SABOTAGED => Model::Sabotaged,
    }
}

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerAddPlayer;

impl ClientCommandHandlerAddPlayer {
    pub fn new() -> Self {
        Self
    }

    /// Java: the game-model-mutation portion of `handleNetCommand`, i.e. everything
    /// reachable via `getClient().getGame()`. `FantasyFootballClient` is still a GUI
    /// stub with no working `getGame()`, so this is exposed as a free function taking
    /// `game: &mut Game` directly, testable independent of the GUI. Returns `false`
    /// exactly where Java's `else { return false; }` branch would fire.
    ///
    /// ```java
    /// Team team = game.getTeamHome().getId().equals(addPlayerCommand.getTeamId()) ? game.getTeamHome() : game.getTeamAway();
    /// Player<?> oldPlayer = team.getPlayerById(addPlayerCommand.getPlayer().getId());
    /// if (oldPlayer == null) {
    ///     team.addPlayer(addPlayerCommand.getPlayer());
    ///     RosterPosition rosterPosition = team.getRoster().getPositionById(addPlayerCommand.getPlayer().getPositionId());
    ///     addPlayerCommand.getPlayer().updatePosition(rosterPosition, game.getRules(), game.getId());
    /// } else if (oldPlayer instanceof RosterPlayer) {
    ///     oldPlayer.init(addPlayerCommand.getPlayer(), game.getRules());
    /// } else {
    ///     return false;
    /// }
    /// game.getFieldModel().setPlayerState(addPlayerCommand.getPlayer(), addPlayerCommand.getPlayerState());
    /// UtilBox.putPlayerIntoBox(game, addPlayerCommand.getPlayer());
    /// PlayerResult playerResult = game.getGameResult().getPlayerResult(addPlayerCommand.getPlayer());
    /// playerResult.setSendToBoxReason(...); playerResult.setSendToBoxTurn(...); playerResult.setSendToBoxHalf(...);
    /// ```
    /// Deviations (documented, not invented): `RosterPlayer` is a type alias for
    /// `Player` in Rust (see `roster_player.rs`), so Java's `instanceof RosterPlayer`
    /// check is always true here — every existing player is replaced in place, matching
    /// `oldPlayer.init(...)`'s effect, and the Java `else { return false; }` branch
    /// (which only exists for non-`RosterPlayer` `Player` subclasses) is unreachable.
    /// `Team.getRoster()`/`RosterPosition.getPositionById`/`Player.updatePosition` and
    /// `UtilBox.putPlayerIntoBox` have no Rust translation yet (no `Roster` reference is
    /// stored on `Team`, only `roster_id: String`) and are left as `// java:` notes.
    pub fn apply_to_game(command: &ServerCommandAddPlayer, game: &mut Game) -> bool {
        let team = if game.team_home.id == command.get_team_id() {
            &mut game.team_home
        } else {
            &mut game.team_away
        };

        let incoming = command.get_player().clone();
        if let Some(old_player) = team.player_mut(&incoming.id) {
            // java: oldPlayer.init(addPlayerCommand.getPlayer(), game.getRules());
            *old_player = incoming.clone();
        } else {
            // java: team.addPlayer(addPlayerCommand.getPlayer());
            team.players.push(incoming.clone());
            // java: RosterPosition rosterPosition = team.getRoster().getPositionById(...);
            // java: addPlayerCommand.getPlayer().updatePosition(rosterPosition, game.getRules(), game.getId());
        }

        game.field_model.set_player_state(&incoming.id, *command.get_player_state());

        // java: UtilBox.putPlayerIntoBox(game, addPlayerCommand.getPlayer());

        let is_home = game.team_home.id == command.get_team_id();
        let player_result = game.game_result.team_result_mut(is_home).player_result_mut(&incoming.id);
        player_result.send_to_box_reason = command.get_send_to_box_reason().map(convert_send_to_box_reason);
        player_result.send_to_box_turn = command.get_send_to_box_turn();
        player_result.send_to_box_half = command.get_send_to_box_half();

        true
    }
}

impl ClientCommandHandler for ClientCommandHandlerAddPlayer {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAddPlayer
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerAddPlayer(_command) = net_command {
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
    use ffb_model::enums::{PlayerState, Rules};
    use ffb_model::enums::SendToBoxReason as ModelSendToBoxReason;
    use ffb_model::model::send_to_box_reason::SendToBoxReason;
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

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn get_id_is_server_add_player() {
        assert_eq!(ClientCommandHandlerAddPlayer::new().get_id(), NetCommandId::ServerAddPlayer);
    }

    #[test]
    fn apply_to_game_adds_new_player_to_correct_team() {
        let mut game = make_game();
        let player = Player { id: "p1".into(), ..Player::default() };
        let cmd = ServerCommandAddPlayer::new("home", player, PlayerState::new(0), None, 0);
        assert!(ClientCommandHandlerAddPlayer::apply_to_game(&cmd, &mut game));
        assert!(game.team_home.has_player("p1"));
        assert!(!game.team_away.has_player("p1"));
    }

    #[test]
    fn apply_to_game_replaces_existing_player_in_place() {
        let mut game = make_game();
        game.team_home.players.push(Player { id: "p1".into(), nr: 1, ..Player::default() });
        let updated = Player { id: "p1".into(), nr: 9, ..Player::default() };
        let cmd = ServerCommandAddPlayer::new("home", updated, PlayerState::new(0), None, 0);
        ClientCommandHandlerAddPlayer::apply_to_game(&cmd, &mut game);
        assert_eq!(game.team_home.player("p1").unwrap().nr, 9);
        assert_eq!(game.team_home.players.len(), 1);
    }

    #[test]
    fn apply_to_game_sets_field_model_player_state() {
        let mut game = make_game();
        let player = Player { id: "p1".into(), ..Player::default() };
        let state = PlayerState::new(5);
        let cmd = ServerCommandAddPlayer::new("home", player, state, None, 0);
        ClientCommandHandlerAddPlayer::apply_to_game(&cmd, &mut game);
        assert_eq!(game.field_model.player_state("p1"), Some(state));
    }

    #[test]
    fn apply_to_game_sets_send_to_box_fields_on_player_result() {
        let mut game = make_game();
        let player = Player { id: "p1".into(), ..Player::default() };
        let cmd = ServerCommandAddPlayer::new("home", player, PlayerState::new(0), Some(SendToBoxReason::MNG), 3);
        ClientCommandHandlerAddPlayer::apply_to_game(&cmd, &mut game);
        let result = game.game_result.team_result(true).player_result("p1").unwrap();
        assert_eq!(result.send_to_box_reason, Some(ModelSendToBoxReason::Mng));
        assert_eq!(result.send_to_box_turn, 3);
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerAddPlayer::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(ffb_model::model::SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
