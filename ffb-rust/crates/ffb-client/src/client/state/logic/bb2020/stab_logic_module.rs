//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.StabLogicModule` (59 lines).
//!
//! Java's `StabLogicModule extends com.fumbbl.ffb.client.state.logic.mixed.BlockLogicModule`
//! (which itself `extends AbstractBlockLogicModule extends LogicModule`), overriding `getId`,
//! `setUp`, `block`, `playerPeek`, and adding `findTargets`/`getTargets`.
//!
//! `mixed.BlockLogicModule` is not yet translated in this repository (it is owned by a separate
//! parallel batch covering `client/state/logic/mixed/`, per this batch's task scope, which
//! explicitly excludes touching that directory). Since `StabLogicModule` does not override most
//! of `BlockLogicModule`'s behavior (`playerInteraction`, `availableActions`, `actionContext`,
//! `performAvailableAction`), that inherited behavior is inlined directly here (mirroring
//! `BlockLogicModule.java` and `AbstractBlockLogicModule.java` exactly) rather than composed
//! over an untranslated struct — matching the same "compose or inline the base class" latitude
//! already used for `BlitzLogicModule`/`MoveLogicModule`. When `mixed::BlockLogicModule` is
//! translated, this file could be refactored to compose over it directly instead.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `StabLogicModule` class.
#[derive(Debug, Default)]
pub struct StabLogicModule {
    extension: BlockLogicExtension,
    targets: Vec<Player>,
}

impl StabLogicModule {
    /// java: `public StabLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { extension: BlockLogicExtension::new(), targets: Vec::new() }
    }

    /// java: `private Player<?>[] findTargets()`.
    fn find_targets(&self, client: &FantasyFootballClient) -> Vec<Player> {
        let game = match client.game() {
            Some(g) => g,
            None => return Vec::new(),
        };
        let acting_player = &game.acting_player;
        let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let opponent_team = UtilPlayer::find_other_team(game, &player.id);
        let coordinate = match game.field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        UtilPlayer::find_adjacent_blockable_players(game, opponent_team, coordinate)
            .into_iter()
            .filter_map(|id| game.player(id))
            .cloned()
            .collect()
    }

    /// java: `public Player<?>[] getTargets()`.
    pub fn get_targets(&self) -> &[Player] {
        &self.targets
    }

    /// java: `protected InteractionResult block(Player<?> player, ActingPlayer actingPlayer)`.
    fn block(&self, client: &mut FantasyFootballClient, player: &Player, acting_player: &ActingPlayer) -> InteractionResult {
        if let Some(acting_player_id) = acting_player.player_id.clone() {
            self.extension.block(client, &acting_player_id, Some(player), true, false, false, false, false);
        }
        InteractionResult::handled()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, player: &Player) -> InteractionResult {
        if self.targets.iter().any(|target| target.id == player.id) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)` — inherited
    /// unchanged from `BlockLogicModule` (not overridden in `StabLogicModule`); see module doc
    /// comment regarding the un-translated base class.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        let is_acting_player = acting_player.player_id.as_deref() == Some(player.id.as_str());
        if is_acting_player {
            if acting_player.suffering_blood_lust {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                return InteractionResult::select_action(ctx);
            } else if acting_player.player_action == Some(PlayerAction::Blitz) {
                client.communication_mut().send_acting_player(
                    Some(player),
                    PlayerAction::BlitzMove,
                    acting_player.jumping,
                );
                return InteractionResult::handled();
            } else {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                if ctx.get_actions().is_empty() {
                    // java: `deselectActingPlayer()` — see `LogicModule::deselect_acting_player`'s
                    // own documented gap.
                    return InteractionResult::handled();
                }
                return InteractionResult::select_action(ctx);
            }
        }
        self.block(client, player, &acting_player)
    }
}

impl LogicModule for StabLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Stab
    }

    /// java: `public void setUp()` — `super.setUp()` is `AbstractBlockLogicModule`'s inherited
    /// (empty) `LogicModule.setUp()`, so only `targets = findTargets()` has an observable
    /// effect.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.targets = self.find_targets(client);
    }

    /// java: `public Set<ClientAction> availableActions()` — inherited unchanged from
    /// `BlockLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::END_MOVE);
        actions.extend(self.extension.available_actions());
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — inherited
    /// unchanged from `BlockLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.merge(self.extension.action_context(game, acting_player));
        if acting_player.suffering_blood_lust {
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

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// inherited unchanged from `BlockLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s documented gap.
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
        crate::client::state::logic::abstract_block_logic_module::end_turn(self, client);
    }
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
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
    fn get_id_is_stab() {
        assert_eq!(StabLogicModule::new().get_id(), ClientStateId::Stab);
    }

    #[test]
    fn available_actions_includes_extension_and_own_actions() {
        let module = StabLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::STAB));
    }

    #[test]
    fn get_targets_empty_by_default() {
        let module = StabLogicModule::new();
        assert!(module.get_targets().is_empty());
    }

    #[test]
    fn set_up_populates_targets_from_adjacent_opponents() {
        let mut module = StabLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6));
        game.acting_player.player_id = Some("attacker".to_string());
        client.set_game(game);
        module.set_up(&mut client);
        assert_eq!(module.get_targets().len(), 1);
        assert_eq!(module.get_targets()[0].id, "defender");
    }

    #[test]
    fn player_peek_performs_for_target() {
        let mut module = StabLogicModule::new();
        let mut player = Player::default();
        player.id = "d1".to_string();
        module.targets = vec![player.clone()];
        let result = module.player_peek(&player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn player_peek_resets_for_non_target() {
        let module = StabLogicModule::new();
        let mut player = Player::default();
        player.id = "other".to_string();
        let result = module.player_peek(&player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn action_context_empty_without_special_ability_or_acted() {
        let module = StabLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = StabLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_without_game_for_move() {
        let mut module = StabLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
