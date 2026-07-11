//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.SpectateLogicModule`.
//!
//! Handles the `SPECTATE` client state, reached once a game has finished while the client was
//! still in `PLAYER` mode: `set_up` flips the client into `SPECTATOR` mode, and
//! `canSwitchToSpectate`/`startReplay` expose the same checks/transition Java's UI wires up to a
//! menu action.
//!
//! Documented gap: `startReplay()` reads `client.getReplayer()`/`getReplayControl()` (Java
//! `ClientReplayer`/`ReplayControl`) and calls `client.updateClientState()` (`FantasyFootballClient`
//! abstract UI hook) — none of these have an in-scope translated body (`ClientReplayer.rs` and
//! `ReplayControl.rs` remain trivial untranslated PascalCase stubs; `updateClientState()` is not
//! among the promoted methods on `FantasyFootballClient`, see that file's module doc). The method
//! is translated as a no-op with the gap documented at the call site, per the batch plan's
//! documented-gap convention.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::ClientMode;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;

/// Java: `SpectateLogicModule`.
#[derive(Debug, Default)]
pub struct SpectateLogicModule;

impl SpectateLogicModule {
    /// Java: `public SpectateLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public boolean canSwitchToSpectate()`
    pub fn can_switch_to_spectate(&self, client: &FantasyFootballClient) -> bool {
        client
            .game()
            .map(|game| game.is_finished() && client.mode() == Some(ClientMode::PLAYER))
            .unwrap_or(false)
    }

    /// java: `public void startReplay()` — see module doc for the documented gap
    /// (`ClientReplayer`/`ReplayControl`/`updateClientState()` are not in-scope).
    pub fn start_replay(&self, _client: &mut FantasyFootballClient) {
        // java: gap — see module doc comment.
    }
}

impl LogicModule for SpectateLogicModule {
    /// java: `public ClientStateId getId()`
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Spectate
    }

    /// java: `public void setUp()`
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        if self.can_switch_to_spectate(client) {
            client.set_mode(ClientMode::SPECTATOR);
        }
    }

    /// java: `public Set<ClientAction> availableActions()`
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in spectate context");
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// empty body in Java.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_parameters::ClientParameters;
    use ffb_model::enums::Rules;
    use ffb_model::model::game_status::GameStatus;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
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
        let mut client = FantasyFootballClient::new(params);
        client.set_game(Game::new(make_team("home"), make_team("away"), Rules::Bb2025));
        client
    }

    #[test]
    fn get_id_is_spectate() {
        let module = SpectateLogicModule::new();
        assert_eq!(module.get_id(), ClientStateId::Spectate);
    }

    #[test]
    fn available_actions_is_always_empty() {
        let module = SpectateLogicModule::new();
        assert!(module.available_actions().is_empty());
    }

    #[test]
    fn can_switch_to_spectate_requires_finished_game_and_player_mode() {
        let module = SpectateLogicModule::new();
        let mut client = make_client(ClientMode::PLAYER);
        assert!(!module.can_switch_to_spectate(&client));
        client.game_mut().unwrap().status = GameStatus::Finished;
        assert!(module.can_switch_to_spectate(&client));
    }

    #[test]
    fn can_switch_to_spectate_false_when_not_player_mode() {
        let module = SpectateLogicModule::new();
        let mut client = make_client(ClientMode::SPECTATOR);
        client.game_mut().unwrap().status = GameStatus::Finished;
        assert!(!module.can_switch_to_spectate(&client));
    }

    #[test]
    fn set_up_switches_mode_when_eligible() {
        let mut module = SpectateLogicModule::new();
        let mut client = make_client(ClientMode::PLAYER);
        client.game_mut().unwrap().status = GameStatus::Finished;
        module.set_up(&mut client);
        assert_eq!(client.mode(), Some(ClientMode::SPECTATOR));
    }

    #[test]
    fn set_up_does_not_switch_mode_when_not_eligible() {
        let mut module = SpectateLogicModule::new();
        let mut client = make_client(ClientMode::PLAYER);
        module.set_up(&mut client);
        assert_eq!(client.mode(), Some(ClientMode::PLAYER));
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in spectate context")]
    fn action_context_panics() {
        let module = SpectateLogicModule::new();
        let client = make_client(ClientMode::SPECTATOR);
        let game = client.game().unwrap();
        let ap = ActingPlayer::new();
        module.action_context(game, &ap);
    }
}
