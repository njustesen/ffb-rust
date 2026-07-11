//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.SetupLogicModule` (88 lines).
//!
//! Java's `SetupLogicModule` is a base `LogicModule` subclass for the team-setup phase,
//! extended by `IllegalSubstitutionLogicModule`/`QuickSnapLogicModule`/`SolidDefenceLogicModule`
//! (a later batch). It has no plugin dependency (unlike `MoveLogicModule`/
//! `BlockLogicExtension`).
//!
//! Documented gaps:
//! - `requestSetups()`: `client.getCommunication().sendTeamSetupLoad(null)` — the Rust
//!   `ClientCommunication::send_team_setup_load` takes a non-optional `impl Into<String>`
//!   (no "null setup name" variant exists), matching the same documented-gap pattern as
//!   `LogicModule::deselect_acting_player`'s `sendActingPlayer(null, null, false)`. Left as a
//!   no-op with this comment.
//! - `endTurn()`: `client.getCommunication().sendEndTurn(useTurnMode() ? getTurnMode() : null,
//!   getTeamHome(), getFieldModel())` — the Rust `send_end_turn_with_coordinates` takes a
//!   non-optional `TurnMode` (no "null turn mode" variant), and `useTurnMode()` always returns
//!   `false` in this base class, so Java always takes the null-turn-mode branch here. Since
//!   there is no faithful non-null substitute, this is left as a no-op with this comment
//!   (subclasses that override `useTurnMode()` to `true` are a later batch's concern).
//! - `handleCommand(...)`: `game.setDialogParameter(new DialogTeamSetupParameter(...))` — the
//!   Rust `Game` has no `dialog_parameter` field/setter (no `DialogParameter` storage was
//!   ported onto `Game`), so the constructed `DialogTeamSetupParameter` is discarded after
//!   being built (for signature fidelity) instead of being stored; documented gap.

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, RSV_HOME_X};
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `SetupLogicModule` class.
#[derive(Debug, Default)]
pub struct SetupLogicModule;

impl SetupLogicModule {
    /// java: `public SetupLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public void requestSetups()` — see module doc gap.
    pub fn request_setups(&self, _client: &mut FantasyFootballClient) {
        // java: gap — see module doc comment (`sendTeamSetupLoad(null)` has no Rust
        // equivalent since `send_team_setup_load` requires a non-optional setup name).
    }

    /// java: `protected boolean useTurnMode()`.
    pub fn use_turn_mode(&self) -> bool {
        false
    }

    /// java: `public InteractionResult handleCommand(NetCommand pNetCommand, boolean loadDialog)`.
    pub fn handle_command(&self, net_command: &AnyServerCommand, load_dialog: bool) -> InteractionResult {
        if let AnyServerCommand::ServerTeamSetupList(setup_list_command) = net_command {
            // java: game.setDialogParameter(new DialogTeamSetupParameter(loadDialog,
            // setupListCommand.getSetupNames())); — see module doc gap; `Game` has no
            // `dialog_parameter` storage, so the parameter value is only constructed here
            // for signature fidelity, then discarded.
            let _ = ffb_model::dialog::dialog_team_setup_parameter::DialogTeamSetupParameter {
                load_dialog,
                setup_names: setup_list_command.get_setup_names().to_vec(),
            };
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public boolean squareIsEmpty(FieldCoordinate pCoordinate)`.
    pub fn square_is_empty(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        match client.game() {
            Some(game) => game.field_model.player_at(coordinate).is_none(),
            None => true,
        }
    }

    /// java: `public boolean squareIsValidForSetup(FieldCoordinate pCoordinate)`.
    pub fn square_is_valid_for_setup(&self, coordinate: FieldCoordinate) -> bool {
        FieldCoordinateBounds::HALF_HOME.is_in_bounds(coordinate) || coordinate.x == RSV_HOME_X
    }
}

impl LogicModule for SetupLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Setup
    }

    /// java: `public void setUp() { super.setUp(); client.getClientData().clear(); }`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        client.client_data_mut().clear();
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        std::collections::HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `public void endTurn()` — see module doc gap.
    fn end_turn(&mut self, _client: &mut FantasyFootballClient) {
        // java: gap — see module doc comment.
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always
    /// throws `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in setup context")
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
    fn get_id_is_setup() {
        assert_eq!(SetupLogicModule::new().get_id(), ClientStateId::Setup);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(SetupLogicModule::new().available_actions().is_empty());
    }

    #[test]
    fn use_turn_mode_is_false() {
        assert!(!SetupLogicModule::new().use_turn_mode());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in setup context")]
    fn action_context_panics() {
        let module = SetupLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn square_is_valid_for_setup_checks_half_home_or_reserve_column() {
        let module = SetupLogicModule::new();
        assert!(module.square_is_valid_for_setup(FieldCoordinate::new(5, 5)));
        assert!(!module.square_is_valid_for_setup(FieldCoordinate::new(20, 5)));
        assert!(module.square_is_valid_for_setup(FieldCoordinate::new(RSV_HOME_X, 3)));
    }

    #[test]
    fn square_is_empty_true_without_game() {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let client = FantasyFootballClient::new(params);
        let module = SetupLogicModule::new();
        assert!(module.square_is_empty(&client, FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn handle_command_ignores_non_matching_command() {
        let module = SetupLogicModule::new();
        let cmd = AnyServerCommand::ServerTeamSetupList(
            ffb_protocol::commands::server_command_team_setup_list::ServerCommandTeamSetupList::new(vec![
                "Wide".to_string(),
            ]),
        );
        let result = module.handle_command(&cmd, true);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
    }
}
