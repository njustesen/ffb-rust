//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.MaximumCarnageLogicModule` (65 lines).
//!
//! Java's `MaximumCarnageLogicModule extends BlockLogicModule` (the `mixed` package's own
//! `BlockLogicModule`, owned by a sibling translation batch — see the cross-worktree dependency
//! note in `kick_em_block_logic_module.rs`, which applies identically here). Composition mirrors
//! that file: a `BlockLogicModule` value for the (mostly unused, since every accessor is
//! overridden here) inherited behavior, plus its own `BlockLogicExtension` instance for
//! `extension.block(...)`.
//!
//! This class overrides every `LogicModule` accessor with fully custom logic (no `super.xxx()`
//! calls anywhere in the Java source), so the composed `BlockLogicModule` field only exists for
//! parity with the "hold what you extend" convention; it is otherwise unused.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::mixed::block_logic_module::BlockLogicModule;

/// 1:1 translation of the `MaximumCarnageLogicModule` class.
#[derive(Debug, Default)]
pub struct MaximumCarnageLogicModule {
    #[allow(dead_code)]
    block_logic: BlockLogicModule,
    extension: BlockLogicExtension,
}

impl MaximumCarnageLogicModule {
    /// java: `public MaximumCarnageLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { block_logic: BlockLogicModule::new(), extension: BlockLogicExtension::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
        }

        let should_block = match client.game() {
            Some(game) => {
                !player.id.eq_ignore_ascii_case(game.last_defender_id.as_deref().unwrap_or(""))
                    && !game.active_team().has_player(&player.id)
            }
            None => false,
        };

        if should_block {
            if let Some(id) = acting_player.player_id.clone() {
                self.extension.block(client, &id, Some(player), false, true, false, false, false);
            }
            return InteractionResult::handled();
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::reset(),
        };
        let acting_player = &game.acting_player;
        let is_acting_player = acting_player.player_id.as_deref() == Some(player.id.as_str());
        let is_last_defender = player.id.eq_ignore_ascii_case(game.last_defender_id.as_deref().unwrap_or(""));
        let on_acting_team = game.active_team().has_player(&player.id);

        if is_acting_player || is_last_defender || on_acting_team {
            InteractionResult::reset()
        } else {
            InteractionResult::perform()
        }
    }
}

impl LogicModule for MaximumCarnageLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::MaximumCarnage
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — Java does not
    /// override this method; `MaximumCarnageLogicModule` inherits `BlockLogicModule.actionContext`
    /// unchanged, which is not itself overridden either, so it ultimately resolves to
    /// `AbstractBlockLogicModule`'s parent (`LogicModule.actionContext` is abstract; the concrete
    /// implementation lives on `BlockLogicModule`). Delegated to the composed `BlockLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.block_logic, game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                // java: `super.performAvailableAction(player, action);` — `BlockLogicModule`'s
                // END_MOVE case: `client.getCommunication().sendActingPlayer(null, null, false);`
                // — see `LogicModule::deselect_acting_player`'s documented gap (no "null action"
                // variant exists for `send_acting_player`), matching `abstract_block_logic_module.rs`.
                let _ = (client, player);
            }
            _ => {
                // java: default: break; (no-op — `MaximumCarnageLogicModule` only ever performs
                // `END_MOVE`, matching its `availableActions()` singleton set).
            }
        }
    }
}

/// java: `AbstractBlockLogicModule.getId()`/`endTurn()` helpers — reused here since
/// `MaximumCarnageLogicModule` does not override `endTurn()` and inherits the
/// `AbstractBlockLogicModule` behavior via `BlockLogicModule`.
#[allow(dead_code)]
fn end_turn_via_abstract_block(module: &mut MaximumCarnageLogicModule, client: &mut FantasyFootballClient) {
    crate::client::state::logic::abstract_block_logic_module::end_turn(module, client);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player_state::PlayerState;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

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

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        let mut player = Player::default();
        player.id = id.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
    }

    fn make_client() -> FantasyFootballClient {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        FantasyFootballClient::new(params)
    }

    #[test]
    fn get_id_is_maximum_carnage() {
        assert_eq!(MaximumCarnageLogicModule::new().get_id(), ClientStateId::MaximumCarnage);
    }

    #[test]
    fn available_actions_is_end_move_only() {
        let actions = MaximumCarnageLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = MaximumCarnageLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_peek_perform_for_opponent_not_last_defender() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10));
        game.acting_player.player_id = Some("h1".to_string());
        client.set_game(game);
        let module = MaximumCarnageLogicModule::new();
        let target = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_peek(&client, &target);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn player_peek_resets_for_own_team_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "h2", FieldCoordinate::new(2, 2));
        game.acting_player.player_id = Some("h1".to_string());
        client.set_game(game);
        let module = MaximumCarnageLogicModule::new();
        let target = client.game().unwrap().player("h2").unwrap().clone();
        let result = module.player_peek(&client, &target);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = MaximumCarnageLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_for_unknown_action() {
        let mut module = MaximumCarnageLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
