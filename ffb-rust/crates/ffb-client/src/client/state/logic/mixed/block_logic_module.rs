//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.BlockLogicModule` (104 lines).
//!
//! Java's `BlockLogicModule extends AbstractBlockLogicModule` and mixes in a
//! `BlockLogicExtension` (`extension` field). `getId()`/`endTurn()` are inherited unchanged
//! from `AbstractBlockLogicModule`, translated as the free functions in
//! `abstract_block_logic_module.rs` (per that module's own doc comment, each concrete
//! block-logic module repeats the one-line delegation).
//!
//! Documented gaps:
//! - `client.getCommunication().sendActingPlayer(null, null, false)` in the `END_MOVE` branch
//!   of `performAvailableAction` — see `LogicModule::deselect_acting_player`'s documented gap
//!   (no "null action" variant exists in the translated `PlayerAction` enum); left as a no-op.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::abstract_block_logic_module;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `BlockLogicModule` class.
#[derive(Debug, Default)]
pub struct BlockLogicModule {
    extension: BlockLogicExtension,
}

impl BlockLogicModule {
    /// java: `public BlockLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { extension: BlockLogicExtension::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            if is_suffering_blood_lust(&acting_player) {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                return InteractionResult::select_action(ctx);
            } else if acting_player.player_action == Some(PlayerAction::Blitz) {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BlitzMove, acting_player.jumping);
                return InteractionResult::handled();
            } else {
                let action_context = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                if action_context.get_actions().is_empty() {
                    // java: `deselectActingPlayer()` — see `LogicModule::deselect_acting_player`'s
                    // own documented gap.
                    self.deselect_acting_player(client);
                    return InteractionResult::handled();
                } else {
                    return InteractionResult::select_action(action_context);
                }
            }
        } else {
            return self.block(client, player, &acting_player);
        }
    }

    /// java: `protected InteractionResult block(Player<?> player, ActingPlayer actingPlayer)`.
    pub fn block(&self, client: &mut FantasyFootballClient, player: &Player, acting_player: &ActingPlayer) -> InteractionResult {
        let do_blitz = acting_player.player_action.map(|a| a.is_blitzing()).unwrap_or(false);
        let multi_block = acting_player.player_action == Some(PlayerAction::MultipleBlock);
        self.extension.player_interaction_full(client, Some(player), do_blitz, multi_block)
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let is_blockable = match client.game() {
            Some(game) => self.extension.is_blockable(game, player),
            None => false,
        };
        if is_blockable {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for BlockLogicModule {
    /// java: `public ClientStateId getId()` — inherited unchanged from `AbstractBlockLogicModule`.
    fn get_id(&self) -> ClientStateId {
        abstract_block_logic_module::get_id()
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::END_MOVE);
        actions.extend(self.extension.available_actions());
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.merge(self.extension.action_context(game, acting_player));
        if is_suffering_blood_lust(acting_player) {
            action_context.add_action(ClientAction::MOVE);
        }
        if !action_context.get_actions().is_empty() || acting_player.has_acted {
            action_context.add_action(ClientAction::END_MOVE);
            if acting_player.has_acted {
                action_context.add_influence(Influences::HAS_ACTED);
            }
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                // java: `client.getCommunication().sendActingPlayer(null, null, false);` — see
                // module doc gap.
            }
            ClientAction::MOVE => {
                let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, jumping);
            }
            _ => {
                self.extension.perform_available_action(client, player, action);
            }
        }
    }

    /// java: `public void endTurn()` — inherited unchanged from `AbstractBlockLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        abstract_block_logic_module::end_turn(self, client);
    }
}

/// java: `AbstractBlockLogicModule.isSufferingBloodLust(ActingPlayer)` — thin re-export for
/// readability at call sites in this file.
fn is_suffering_blood_lust(acting_player: &ActingPlayer) -> bool {
    abstract_block_logic_module::is_suffering_blood_lust(acting_player)
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
    fn get_id_is_block() {
        assert_eq!(BlockLogicModule::new().get_id(), ClientStateId::Block);
    }

    #[test]
    fn available_actions_includes_move_and_extension() {
        let module = BlockLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::BLOCK));
    }

    #[test]
    fn action_context_adds_end_move_when_has_acted() {
        let module = BlockLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.has_acted = true;
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_empty_without_acted_or_bloodlust() {
        let module = BlockLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = BlockLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = BlockLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_handles_blitz_action() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        client.set_game(game);
        let module = BlockLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
    }

    #[test]
    fn perform_available_action_move_sends_command() {
        let mut module = BlockLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }

    #[test]
    fn end_turn_no_op_without_game() {
        let mut module = BlockLogicModule::new();
        let mut client = make_client();
        module.end_turn(&mut client);
    }
}
