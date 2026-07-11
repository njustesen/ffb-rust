//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.WaitForSetupLogicModule` (41 lines).
//!
//! Extends `LogicModule` directly (no plugin dependency). No actions are available and
//! `actionContext` always throws (there is no acting player while waiting).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `WaitForSetupLogicModule` class.
///
/// @author Kalimar
#[derive(Debug, Default)]
pub struct WaitForSetupLogicModule;

impl WaitForSetupLogicModule {
    /// java: `public WaitForSetupLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }
}

impl LogicModule for WaitForSetupLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::WaitForSetup
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in waiting context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn get_id_is_wait_for_setup() {
        assert_eq!(WaitForSetupLogicModule::new().get_id(), ClientStateId::WaitForSetup);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(WaitForSetupLogicModule::new().available_actions().is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in waiting context")]
    fn action_context_panics() {
        let module = WaitForSetupLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
