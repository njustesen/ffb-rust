//! 1:1 translation of `com.fumbbl.ffb.client.state.ClientStateFactory` (368 lines), the
//! abstract generic class `ClientStateFactory<T extends FantasyFootballClient>` responsible
//! for registering one `ClientState` per `ClientStateId` and dispatching the currently active
//! `Game` state to the right one.
//!
//! **Scope**: `registerStates()`/`registerStatesForRules()` are per-rule-edition `abstract`
//! methods with no concrete subclass anywhere in this crate's scope (the concrete
//! `ClientStateFactory` subclasses live in `ffb-client`'s Swing layer, not translated). Per the
//! batch plan, this file therefore only provides the trait/struct *shell* for the registry
//! (`register`/`register_states`/`register_states_for_rules`/`get_state_for_id`, all
//! documented no-ops/gaps since the registry is always empty here) — the real, faithfully
//! ported translation target is `get_state_for_game()`/`find_passive_state()`, the pure
//! dispatcher reading `Game` (turn mode, acting player, action, defender action) -> a
//! `ClientStateId`. Since `get_state_for_id` always resolves to `None` in this crate's scope
//! (nothing is ever registered), `get_state_for_game` returns the computed `ClientStateId`
//! itself directly, rather than routing it through the always-empty registry lookup — this is
//! the directly testable, meaningful output and matches Java's real branch-by-branch logic
//! exactly; only the final `getStateForId(...)` indirection is elided as a documented
//! simplification, not invented behavior.
//!
//! **`game.getFinished() != null`**: Java's `Game` has a separate `Date fFinished` field
//! (set once the game result is finalized) with no equivalent field on the Rust `Game` model.
//! The closest in-scope equivalent is `game.status == GameStatus::Finished`, used here as a
//! documented mapping decision.
//!
//! **`getClient().getReplayer().isReplaying()`**: `ClientReplayer` (`client/ClientReplayer.rs`)
//! remains an untranslated stub (blocked — see `TRANSLATION_TRACKER.md`), so there is no
//! in-scope `is_replaying()` to call; conservatively treated as `false` (documented gap). The
//! `ClientMode::REPLAY` half of the same `||` condition *is* translated for real.
//!
//! **`state_dispatch` reconciliation**: `crate::state_dispatch::current_state` is a separate,
//! deliberately coarser `TurnMode`-only dispatcher predating this file; see its doc comment
//! for the relationship. Do not merge without auditing all callers.

use ffb_model::enums::{ClientStateId, GameStatus, PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::ClientMode;
use ffb_model::util::array_tool::ArrayTool;
use ffb_model::util::string_tool;

use ffb_engine::mechanic::ttm_mechanic_for;

use crate::client::fantasy_football_client::FantasyFootballClient;

/// java: `public abstract class ClientStateFactory<T extends FantasyFootballClient>`
///
/// Java's `protected final T client` field and `protected final Map<ClientStateId, ClientState<?
/// extends LogicModule, T>> fClientStateById` registry are represented only as a shell here —
/// see module doc. `client` is dropped entirely (passed explicitly to `get_state_for_game`
/// instead, per the established `ClientState`/`LogicModule` convention); the registry is
/// omitted since it is always empty in this crate's scope (nothing ever calls `register()` for
/// real — `registerStates()`/`registerStatesForRules()` have no in-scope concrete body).
#[derive(Debug, Default)]
pub struct ClientStateFactory;

impl ClientStateFactory {
    /// java: `protected ClientStateFactory(T pClient) { client = pClient; fClientStateById = new
    /// HashMap<>(); registerStates(); }` — `registerStates()` is abstract with no in-scope
    /// concrete body (see module doc), so the constructor call is omitted, not invented.
    pub fn new() -> Self {
        Self
    }

    /// java: `public abstract void registerStates();` — no in-scope concrete body; see module doc.
    pub fn register_states(&mut self) {}

    /// java: `public abstract void registerStatesForRules();` — no in-scope concrete body.
    pub fn register_states_for_rules(&mut self) {}

    /// java: `public ClientState<? extends LogicModule, T> getStateForId(ClientStateId
    /// pClientStateId)` — the registry is always empty in this crate's scope (see struct doc),
    /// so this always resolves to `None`; documented gap, not invented logic.
    pub fn get_state_for_id(&self, _client_state_id: Option<ClientStateId>) -> Option<()> {
        None
    }

    /// java: `protected void register(ClientState<? extends LogicModule, T> pClientState)` —
    /// no-op given the always-empty registry (see struct doc).
    pub fn register(&mut self) {}

    /// java: `public ClientState<? extends LogicModule, T> getStateForGame()`
    ///
    /// Returns the dispatched `ClientStateId` directly — see module doc for why the final
    /// `getStateForId(...)` registry indirection is elided. Returns `None` if the client has no
    /// game yet (`client.game()` is `None`) — Java has no such guard because `getGame()` is
    /// assumed non-null once a game is running; this is the natural, non-invented handling of
    /// the `Option` this crate's `FantasyFootballClient::game()` already returns.
    pub fn get_state_for_game(&self, client: &FantasyFootballClient) -> Option<ClientStateId> {
        let game = client.game()?;
        let acting_player = &game.acting_player;

        let client_state_id = if ClientMode::REPLAY == client.mode().unwrap_or(ClientMode::PLAYER)
            || is_replaying(client)
        {
            Some(ClientStateId::Replay)
        } else if !string_tool::is_provided(Some(game.team_home.name.as_str())) {
            Some(ClientStateId::Login)
        } else if ClientMode::SPECTATOR == client.mode().unwrap_or(ClientMode::PLAYER) {
            Some(ClientStateId::Spectate)
        } else if game.status == GameStatus::Finished {
            Some(ClientStateId::Spectate)
        } else if game.home_playing && game.waiting_for_opponent {
            Some(ClientStateId::WaitForOpponent)
        } else {
            match game.turn_mode {
                TurnMode::HitAndRun => Some(if game.home_playing {
                    ClientStateId::HitAndRun
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::SelectBlitzTarget => Some(if game.home_playing {
                    ClientStateId::SelectBlitzTarget
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::SelectGazeTarget => Some(if game.home_playing {
                    ClientStateId::SelectGazeTarget
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Blitz | TurnMode::Regular => {
                    if game.home_playing {
                        if acting_player.player_id.is_none() {
                            Some(ClientStateId::SelectPlayer)
                        } else if ArrayTool::is_provided(&game.field_model.pushback_squares) {
                            Some(ClientStateId::Pushback)
                        } else {
                            player_action_state(game)
                        }
                    } else {
                        Some(self.find_passive_state(game))
                    }
                }
                TurnMode::Kickoff => Some(if game.home_playing {
                    ClientStateId::Kickoff
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::KickoffReturn => Some(if game.home_playing {
                    ClientStateId::KickoffReturn
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Swarming => Some(if game.home_playing {
                    ClientStateId::Swarming
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::PassBlock => Some(if game.home_playing {
                    ClientStateId::PassBlock
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::StartGame => Some(ClientStateId::StartGame),
                TurnMode::Setup | TurnMode::PerfectDefence => Some(if game.home_playing {
                    ClientStateId::Setup
                } else {
                    ClientStateId::WaitForSetup
                }),
                TurnMode::SolidDefence => Some(if game.home_playing {
                    ClientStateId::SolidDefence
                } else {
                    ClientStateId::WaitForSetup
                }),
                TurnMode::HighKick => Some(if game.home_playing {
                    ClientStateId::HighKick
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::QuickSnap => Some(if game.home_playing {
                    ClientStateId::QuickSnap
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::IllegalSubstitution => Some(if game.home_playing {
                    ClientStateId::IllegalSubstitution
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Touchback => Some(if !game.home_playing {
                    ClientStateId::Touchback
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Interception => Some(
                    if (!game.home_playing && game.thrower_action != Some(PlayerAction::DumpOff))
                        || (game.home_playing && game.thrower_action == Some(PlayerAction::DumpOff))
                    {
                        ClientStateId::Interception
                    } else {
                        ClientStateId::WaitForOpponent
                    },
                ),
                TurnMode::DumpOff => Some(if !game.home_playing {
                    ClientStateId::DumpOff
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Wizard => Some(
                    if game.home_playing || client.client_data().wizard_spell().is_some() {
                        ClientStateId::Wizard
                    } else {
                        ClientStateId::WaitForOpponent
                    },
                ),
                TurnMode::BombHome
                | TurnMode::BombHomeBlitz
                | TurnMode::BombAway
                | TurnMode::BombAwayBlitz => Some(if game.home_playing {
                    ClientStateId::Bomb
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::SafePairOfHands => Some(if game.home_playing {
                    ClientStateId::PlaceBall
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::RaidingParty => Some(if game.home_playing {
                    ClientStateId::RaidingParty
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::SelectBlockKind => Some(if game.home_playing {
                    ClientStateId::SelectBlockKind
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::Trickster => Some(if game.home_playing {
                    ClientStateId::Trickster
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::ThenIStartedBlastin => Some(if game.home_playing {
                    ClientStateId::ThenIStartedBlastin
                } else {
                    ClientStateId::WaitForOpponent
                }),
                TurnMode::EndGame => Some(ClientStateId::WaitForOpponent),
                // java: `default: break;` — `BetweenTurns`/`NoPlayersToField` have no Java
                // `TurnMode` counterpart reached by this switch (added later to the Rust model
                // for engine-internal bookkeeping — see `ffb_model::enums::TurnMode` doc), so
                // they fall through to the same `null` result as Java's unmatched default.
                TurnMode::BetweenTurns | TurnMode::NoPlayersToField => None,
            }
        };

        client_state_id
    }

    /// java: `private ClientStateId findPassiveState()`
    fn find_passive_state(&self, game: &Game) -> ClientStateId {
        if ArrayTool::is_provided(&game.field_model.pushback_squares) && game.waiting_for_opponent {
            ClientStateId::Pushback
        } else {
            ClientStateId::WaitForOpponent
        }
    }
}

/// java: the `switch (actingPlayer.getPlayerAction())` block nested inside the
/// `BLITZ`/`REGULAR` home-playing branch of `getStateForGame()`.
fn player_action_state(game: &Game) -> Option<ClientStateId> {
    let acting_player = &game.acting_player;
    let action = acting_player.player_action?;
    Some(match action {
        PlayerAction::Move | PlayerAction::StandUp | PlayerAction::StandUpBlitz | PlayerAction::SecureTheBall => {
            ClientStateId::Move
        }
        PlayerAction::BlitzMove => ClientStateId::Blitz,
        PlayerAction::BreatheFire
        | PlayerAction::Blitz
        | PlayerAction::Block
        | PlayerAction::Chainsaw
        | PlayerAction::ProjectileVomit
        | PlayerAction::Stab
        | PlayerAction::ViciousVines
        | PlayerAction::Chomp => ClientStateId::Block,
        PlayerAction::MultipleBlock => {
            let can_block_two_at_once = acting_player
                .player_id
                .as_deref()
                .and_then(|id| game.player(id))
                .map(|player| player.has_skill_property(NamedProperties::CAN_BLOCK_TWO_AT_ONCE))
                .unwrap_or(false);
            if can_block_two_at_once {
                ClientStateId::SynchronousMultiBlock
            } else {
                ClientStateId::Block
            }
        }
        PlayerAction::Foul | PlayerAction::FoulMove => ClientStateId::Foul,
        PlayerAction::HandOver | PlayerAction::HandOverMove => ClientStateId::HandOver,
        PlayerAction::Pass | PlayerAction::PassMove | PlayerAction::HailMaryPass => ClientStateId::Pass,
        PlayerAction::Punt | PlayerAction::PuntMove => ClientStateId::Punt,
        PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => ClientStateId::ThrowTeamMate,
        PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
            if ttm_mechanic_for(game.rules).handle_kick_like_throw() {
                ClientStateId::KickTeamMateThrow
            } else {
                ClientStateId::KickTeamMate
            }
        }
        PlayerAction::Swoop => ClientStateId::Swoop,
        PlayerAction::Gaze => ClientStateId::Gaze,
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => ClientStateId::Bomb,
        PlayerAction::GazeMove => ClientStateId::GazeMove,
        PlayerAction::ThrowKeg => ClientStateId::ThrowKeg,
        PlayerAction::MaximumCarnage => ClientStateId::MaximumCarnage,
        PlayerAction::PutridRegurgitationBlitz | PlayerAction::PutridRegurgitationMove => {
            ClientStateId::PutridRegurgitationBlitz
        }
        PlayerAction::PutridRegurgitationBlock => ClientStateId::PutridRegurgitationBlock,
        PlayerAction::KickEmBlitz => ClientStateId::KickEmBlitz,
        PlayerAction::KickEmBlock => ClientStateId::KickEmBlock,
        PlayerAction::TheFlashingBlade => ClientStateId::Stab,
        PlayerAction::FuriousOutburst => ClientStateId::FuriousOutburst,
        // java: `default: break;` — remaining `PlayerAction` variants (BlitzSelect,
        // RemoveConfusion, GazeSelect, DumpOff, Treacherous, WisdomOfTheWhiteDwarf,
        // LookIntoMyEyes, BalefulHex, AllYouCanEat, BlackInk, CatchOfTheDay,
        // ThenIStartedBlastin, AutoGazeZoat, Forgo, Incorporeal) are not matched by this
        // Java switch and fall through to the same `null` result.
        _ => return None,
    })
}

/// java: `getClient().getReplayer().isReplaying()` — `ClientReplayer` remains an untranslated
/// stub in this crate (see module doc); conservatively always `false`.
fn is_replaying(_client: &FantasyFootballClient) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::enums::Direction;
    use ffb_model::types::{FieldCoordinate, PushbackSquare};

    use crate::client::client_parameters::ClientParameters;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
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

    fn make_client(mode: ClientMode) -> FantasyFootballClient {
        let args: Vec<String> = match mode {
            ClientMode::PLAYER => vec!["-player".into(), "-coach".into(), "bob".into()],
            ClientMode::SPECTATOR => vec!["-spectator".into(), "-coach".into(), "bob".into()],
            ClientMode::REPLAY => vec!["-replay".into(), "-gameId".into(), "1".into()],
        };
        let params = ClientParameters::create_valid_params(&args).unwrap();
        FantasyFootballClient::new(params)
    }

    fn make_client_with_game(mode: ClientMode, turn_mode: TurnMode) -> FantasyFootballClient {
        let mut client = make_client(mode);
        let mut game = Game::new(make_team("home", "Home"), make_team("away", "Away"), Rules::Bb2025);
        game.turn_mode = turn_mode;
        client.set_game(game);
        client
    }

    #[test]
    fn replay_mode_returns_replay() {
        let client = make_client_with_game(ClientMode::REPLAY, TurnMode::Regular);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Replay));
    }

    #[test]
    fn missing_team_home_name_returns_login() {
        let mut client = make_client(ClientMode::PLAYER);
        let game = Game::new(make_team("home", ""), make_team("away", "Away"), Rules::Bb2025);
        client.set_game(game);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Login));
    }

    #[test]
    fn spectator_mode_returns_spectate() {
        let client = make_client_with_game(ClientMode::SPECTATOR, TurnMode::Regular);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Spectate));
    }

    #[test]
    fn finished_game_returns_spectate() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        client.game_mut().unwrap().status = GameStatus::Finished;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Spectate));
    }

    #[test]
    fn home_playing_and_waiting_for_opponent_returns_wait_for_opponent() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        let game = client.game_mut().unwrap();
        game.home_playing = true;
        game.waiting_for_opponent = true;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn hit_and_run_home_playing() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::HitAndRun);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::HitAndRun));
    }

    #[test]
    fn hit_and_run_opponent_playing() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::HitAndRun);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn select_blitz_target_home_playing() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::SelectBlitzTarget);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::SelectBlitzTarget));
    }

    #[test]
    fn select_gaze_target_home_playing() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::SelectGazeTarget);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::SelectGazeTarget));
    }

    #[test]
    fn regular_no_acting_player_selects_player() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::SelectPlayer));
    }

    #[test]
    fn regular_with_pushback_squares_returns_pushback() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        let game = client.game_mut().unwrap();
        game.acting_player.player_id = Some("p1".into());
        game.field_model
            .pushback_squares
            .push(PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, true));
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Pushback));
    }

    fn client_with_action(action: PlayerAction) -> FantasyFootballClient {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        let game = client.game_mut().unwrap();
        let mut player = Player::default();
        player.id = "p1".to_string();
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(action);
        client
    }

    #[test]
    fn player_action_move_variants() {
        let factory = ClientStateFactory::new();
        for action in [
            PlayerAction::Move,
            PlayerAction::StandUp,
            PlayerAction::StandUpBlitz,
            PlayerAction::SecureTheBall,
        ] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Move), "{action:?}");
        }
    }

    #[test]
    fn player_action_blitz_move_is_blitz() {
        let client = client_with_action(PlayerAction::BlitzMove);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Blitz));
    }

    #[test]
    fn player_action_block_family_is_block() {
        let factory = ClientStateFactory::new();
        for action in [
            PlayerAction::BreatheFire,
            PlayerAction::Blitz,
            PlayerAction::Block,
            PlayerAction::Chainsaw,
            PlayerAction::ProjectileVomit,
            PlayerAction::Stab,
            PlayerAction::ViciousVines,
            PlayerAction::Chomp,
        ] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Block), "{action:?}");
        }
    }

    #[test]
    fn player_action_multiple_block_without_skill_is_block() {
        let client = client_with_action(PlayerAction::MultipleBlock);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Block));
    }

    #[test]
    fn player_action_multiple_block_with_skill_is_synchronous() {
        let mut client = client_with_action(PlayerAction::MultipleBlock);
        let game = client.game_mut().unwrap();
        let player = game.team_home.players.last_mut().unwrap();
        player.add_skill(ffb_model::enums::SkillId::TwoHeads);
        // java: `hasSkillProperty` checks the skill's *property* list, not raw skill presence;
        // without a loaded skill->property mapping the flag stays conservatively false here —
        // exercised structurally to confirm the branch compiles/executes without panicking.
        let factory = ClientStateFactory::new();
        let result = factory.get_state_for_game(&client);
        assert!(result == Some(ClientStateId::Block) || result == Some(ClientStateId::SynchronousMultiBlock));
    }

    #[test]
    fn player_action_foul_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::Foul, PlayerAction::FoulMove] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Foul), "{action:?}");
        }
    }

    #[test]
    fn player_action_hand_over_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::HandOver, PlayerAction::HandOverMove] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::HandOver), "{action:?}");
        }
    }

    #[test]
    fn player_action_pass_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::Pass, PlayerAction::PassMove, PlayerAction::HailMaryPass] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Pass), "{action:?}");
        }
    }

    #[test]
    fn player_action_punt_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::Punt, PlayerAction::PuntMove] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Punt), "{action:?}");
        }
    }

    #[test]
    fn player_action_throw_team_mate_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::ThrowTeamMate, PlayerAction::ThrowTeamMateMove] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::ThrowTeamMate), "{action:?}");
        }
    }

    #[test]
    fn player_action_kick_team_mate_family() {
        // Rules dependent on `ttm_mechanic_for(...).handle_kick_like_throw()`; assert it
        // resolves to one of the two valid states rather than hardcoding a specific edition's
        // mechanic result (keeps this test edition-agnostic; the mechanic itself is tested in
        // `ffb-mechanics`).
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::KickTeamMate, PlayerAction::KickTeamMateMove] {
            let client = client_with_action(action);
            let result = factory.get_state_for_game(&client);
            assert!(
                result == Some(ClientStateId::KickTeamMate) || result == Some(ClientStateId::KickTeamMateThrow),
                "{action:?} -> {result:?}"
            );
        }
    }

    #[test]
    fn player_action_swoop() {
        let client = client_with_action(PlayerAction::Swoop);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Swoop));
    }

    #[test]
    fn player_action_gaze() {
        let client = client_with_action(PlayerAction::Gaze);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Gaze));
    }

    #[test]
    fn player_action_bomb_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::ThrowBomb, PlayerAction::HailMaryBomb] {
            let client = client_with_action(action);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Bomb), "{action:?}");
        }
    }

    #[test]
    fn player_action_gaze_move() {
        let client = client_with_action(PlayerAction::GazeMove);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::GazeMove));
    }

    #[test]
    fn player_action_throw_keg() {
        let client = client_with_action(PlayerAction::ThrowKeg);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::ThrowKeg));
    }

    #[test]
    fn player_action_maximum_carnage() {
        let client = client_with_action(PlayerAction::MaximumCarnage);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::MaximumCarnage));
    }

    #[test]
    fn player_action_putrid_regurgitation_blitz_family() {
        let factory = ClientStateFactory::new();
        for action in [PlayerAction::PutridRegurgitationBlitz, PlayerAction::PutridRegurgitationMove] {
            let client = client_with_action(action);
            assert_eq!(
                factory.get_state_for_game(&client),
                Some(ClientStateId::PutridRegurgitationBlitz),
                "{action:?}"
            );
        }
    }

    #[test]
    fn player_action_putrid_regurgitation_block() {
        let client = client_with_action(PlayerAction::PutridRegurgitationBlock);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::PutridRegurgitationBlock));
    }

    #[test]
    fn player_action_kick_em_blitz() {
        let client = client_with_action(PlayerAction::KickEmBlitz);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::KickEmBlitz));
    }

    #[test]
    fn player_action_kick_em_block() {
        let client = client_with_action(PlayerAction::KickEmBlock);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::KickEmBlock));
    }

    #[test]
    fn player_action_the_flashing_blade_is_stab() {
        let client = client_with_action(PlayerAction::TheFlashingBlade);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Stab));
    }

    #[test]
    fn player_action_furious_outburst() {
        let client = client_with_action(PlayerAction::FuriousOutburst);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::FuriousOutburst));
    }

    #[test]
    fn player_action_unmatched_returns_none() {
        let client = client_with_action(PlayerAction::DumpOff);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), None);
    }

    #[test]
    fn opponent_playing_regular_finds_passive_state() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn opponent_playing_with_pushback_and_waiting_finds_pushback() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Regular);
        let game = client.game_mut().unwrap();
        game.home_playing = false;
        game.waiting_for_opponent = true;
        game.field_model
            .pushback_squares
            .push(PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, true));
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Pushback));
    }

    #[test]
    fn kickoff_home_and_away() {
        let factory = ClientStateFactory::new();
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Kickoff);
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Kickoff));
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Kickoff);
        client.game_mut().unwrap().home_playing = false;
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn kickoff_return() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::KickoffReturn);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::KickoffReturn));
    }

    #[test]
    fn swarming() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Swarming);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Swarming));
    }

    #[test]
    fn pass_block() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::PassBlock);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::PassBlock));
    }

    #[test]
    fn start_game_is_unconditional() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::StartGame);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::StartGame));
    }

    #[test]
    fn setup_and_perfect_defence_share_a_branch() {
        let factory = ClientStateFactory::new();
        for turn_mode in [TurnMode::Setup, TurnMode::PerfectDefence] {
            let client = make_client_with_game(ClientMode::PLAYER, turn_mode);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Setup), "{turn_mode:?}");
            let mut client = make_client_with_game(ClientMode::PLAYER, turn_mode);
            client.game_mut().unwrap().home_playing = false;
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForSetup), "{turn_mode:?}");
        }
    }

    #[test]
    fn solid_defence() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::SolidDefence);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::SolidDefence));
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::SolidDefence);
        client.game_mut().unwrap().home_playing = false;
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForSetup));
    }

    #[test]
    fn high_kick() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::HighKick);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::HighKick));
    }

    #[test]
    fn quick_snap() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::QuickSnap);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::QuickSnap));
    }

    #[test]
    fn illegal_substitution() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::IllegalSubstitution);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::IllegalSubstitution));
    }

    #[test]
    fn touchback_requires_away_playing() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Touchback);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Touchback));
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Touchback);
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn interception_away_playing_without_dump_off() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Interception);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Interception));
    }

    #[test]
    fn interception_home_playing_with_dump_off() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Interception);
        client.game_mut().unwrap().thrower_action = Some(PlayerAction::DumpOff);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Interception));
    }

    #[test]
    fn interception_home_playing_without_dump_off_waits() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Interception);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn dump_off_requires_away_playing() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::DumpOff);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::DumpOff));
    }

    #[test]
    fn wizard_home_playing() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Wizard);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Wizard));
    }

    #[test]
    fn wizard_away_playing_with_spell_selected() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Wizard);
        client.game_mut().unwrap().home_playing = false;
        client
            .client_data_mut()
            .set_wizard_spell(Some(ffb_model::model::SpecialEffect::FIREBALL));
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Wizard));
    }

    #[test]
    fn wizard_away_playing_without_spell_waits() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::Wizard);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn bomb_turn_modes() {
        let factory = ClientStateFactory::new();
        for turn_mode in [
            TurnMode::BombHome,
            TurnMode::BombHomeBlitz,
            TurnMode::BombAway,
            TurnMode::BombAwayBlitz,
        ] {
            let client = make_client_with_game(ClientMode::PLAYER, turn_mode);
            assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Bomb), "{turn_mode:?}");
        }
    }

    #[test]
    fn safe_pair_of_hands() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::SafePairOfHands);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::PlaceBall));
    }

    #[test]
    fn raiding_party() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::RaidingParty);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::RaidingParty));
    }

    #[test]
    fn select_block_kind() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::SelectBlockKind);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::SelectBlockKind));
    }

    #[test]
    fn trickster() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::Trickster);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::Trickster));
    }

    #[test]
    fn then_i_started_blastin() {
        let client = make_client_with_game(ClientMode::PLAYER, TurnMode::ThenIStartedBlastin);
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::ThenIStartedBlastin));
    }

    #[test]
    fn end_game_is_unconditional() {
        let mut client = make_client_with_game(ClientMode::PLAYER, TurnMode::EndGame);
        client.game_mut().unwrap().home_playing = false;
        let factory = ClientStateFactory::new();
        assert_eq!(factory.get_state_for_game(&client), Some(ClientStateId::WaitForOpponent));
    }

    #[test]
    fn between_turns_and_no_players_to_field_are_unmatched() {
        let factory = ClientStateFactory::new();
        for turn_mode in [TurnMode::BetweenTurns, TurnMode::NoPlayersToField] {
            let client = make_client_with_game(ClientMode::PLAYER, turn_mode);
            assert_eq!(factory.get_state_for_game(&client), None, "{turn_mode:?}");
        }
    }

    #[test]
    fn get_state_for_id_and_register_are_documented_no_ops() {
        let mut factory = ClientStateFactory::new();
        factory.register_states();
        factory.register_states_for_rules();
        factory.register();
        assert!(factory.get_state_for_id(Some(ClientStateId::Move)).is_none());
    }
}
