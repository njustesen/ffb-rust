//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.SolidDefenceLogicModule` (14 lines).
//!
//! Extends `SetupLogicModule` (delegated to via composition — see `setup_logic_module.rs`)
//! with no overrides beyond `getId()`.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::setup_logic_module::SetupLogicModule;

/// 1:1 translation of the `SolidDefenceLogicModule` class.
#[derive(Debug, Default)]
pub struct SolidDefenceLogicModule {
    setup: SetupLogicModule,
}

impl SolidDefenceLogicModule {
    /// java: `public SolidDefenceLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            setup: SetupLogicModule::new(),
        }
    }
}

impl LogicModule for SolidDefenceLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SolidDefence
    }

    /// java: `public void setUp()` — delegates to `SetupLogicModule`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.setup.set_up(client);
    }

    /// java: `public Set<ClientAction> availableActions()` — delegates to `SetupLogicModule`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        self.setup.available_actions()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}` —
    /// delegates to `SetupLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.setup.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — delegates to `SetupLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.setup.end_turn(client);
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — delegates to
    /// `SetupLogicModule`, which always panics.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.setup.action_context(game, acting_player)
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
    fn get_id_is_solid_defence() {
        assert_eq!(SolidDefenceLogicModule::new().get_id(), ClientStateId::SolidDefence);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(SolidDefenceLogicModule::new().available_actions().is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in setup context")]
    fn action_context_panics_via_delegation() {
        let module = SolidDefenceLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
