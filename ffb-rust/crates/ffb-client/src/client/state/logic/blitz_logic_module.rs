//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.BlitzLogicModule` (145 lines).
//!
//! Java's `BlitzLogicModule extends MoveLogicModule` and mixes in a `BlockLogicExtension`
//! (`extension` field). Per the batch conventions established for `MoveLogicModule` and
//! `BlockLogicExtension`, this struct holds a `BlockLogicExtension` value and delegates to
//! `move_logic_module`/`logic_module` free functions for inherited (non-overridden) behavior.
//!
//! Documented gaps:
//! - `Player.hasActiveEnhancement(skill)` — no enhancement-tracking exists on the Rust
//!   `Player` (same gap as `logic_module.rs`'s `is_incorporeal_available_ap`); conservatively
//!   `false`, so `INCORPOREAL`'s toggle always requests activation (`!active` == `true`).
//! - `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap;
//!   `skill_placeholder(SkillId)` is reused for the network command.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// 1:1 translation of the `BlitzLogicModule` class.
#[derive(Debug, Default)]
pub struct BlitzLogicModule {
    move_logic: MoveLogicModule,
    extension: BlockLogicExtension,
}

impl BlitzLogicModule {
    /// java: `public BlitzLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new(), extension: BlockLogicExtension::new() }
    }

    /// java: `public boolean playerActivationUsed()`.
    pub fn player_activation_used(&self, client: &FantasyFootballClient) -> bool {
        let field_model = match client.game() {
            Some(game) => &game.field_model,
            None => return <Self as LogicModule>::player_activation_used(self, client),
        };
        match &field_model.target_selection_state {
            Some(state) => state.is_committed(),
            None => <Self as LogicModule>::player_activation_used(self, client),
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (acting_player_player_id, going_for_it_check, has_blocked, multi_block) = match client.game() {
            Some(game) => {
                let acting_player = &game.acting_player;
                (
                    acting_player.player_id.clone(),
                    UtilPlayer::is_next_move_going_for_it(game) && !acting_player.goes_for_it,
                    acting_player.has_blocked,
                    acting_player.player_action == Some(PlayerAction::MultipleBlock),
                )
            }
            None => return InteractionResult::ignore(),
        };

        if acting_player_player_id.as_deref() == Some(player.id.as_str()) {
            return self.move_logic.player_interaction(client, player);
        }

        if going_for_it_check {
            let (game_ref, acting_player) = match client.game() {
                Some(game) => (game, game.acting_player.clone()),
                None => return InteractionResult::ignore(),
            };
            let ctx = self.action_context(game_ref, &acting_player);
            return InteractionResult::select_action(ctx);
        }

        if !has_blocked {
            return self.extension.player_interaction_full(client, Some(player), true, multi_block);
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::reset(),
        };
        let acting_player = &game.acting_player;
        if !acting_player.has_blocked && self.extension.is_blockable(game, player) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `protected PlayerAction moveAction()`.
    pub fn move_action(&self) -> PlayerAction {
        PlayerAction::BlitzMove
    }

    /// java: `protected void sendCommand(ActingPlayer actingPlayer, FieldCoordinate
    /// coordinateFrom, FieldCoordinate[] pCoordinates)`.
    pub fn send_command(
        &self,
        client: &mut FantasyFootballClient,
        acting_player: &ActingPlayer,
        coordinate_from: ffb_model::types::FieldCoordinate,
        coordinates: &[ffb_model::types::FieldCoordinate],
    ) {
        if let Some(id) = acting_player.player_id.clone() {
            client.communication_mut().send_player_blitz_move(id, coordinate_from, coordinates.to_vec());
        }
    }

    /// java: `public boolean isGoredAvailable()`.
    pub fn is_gored_available(&self, client: &FantasyFootballClient) -> bool {
        self.extension.is_gored_available(client)
    }
}

impl LogicModule for BlitzLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Blitz
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::JUMP);
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::FUMBLEROOSKIE);
        actions.insert(ClientAction::BOUNDING_LEAP);
        actions.insert(ClientAction::GORED_BY_THE_BULL);
        actions.insert(ClientAction::INCORPOREAL);
        actions.extend(self.extension.available_actions());
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — inherited
    /// unchanged from `MoveLogicModule` (not overridden in Java).
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };
        match action {
            ClientAction::END_MOVE => {
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s documented gap.
            }
            ClientAction::JUMP => {
                if logic_module::is_jump_available_as_next_move(
                    client.game().unwrap(),
                    &acting_player,
                    false,
                ) {
                    if let Some(pa) = acting_player.player_action {
                        client.communication_mut().send_acting_player(Some(player), pa, !acting_player.jumping);
                    }
                }
            }
            ClientAction::MOVE => {
                if acting_player.suffering_blood_lust {
                    client.communication_mut().send_acting_player(Some(player), self.move_action(), acting_player.jumping);
                }
            }
            ClientAction::FUMBLEROOSKIE => {
                client.communication_mut().send_use_fumblerooskie();
            }
            ClientAction::BOUNDING_LEAP => {
                let skill_id = client.game().and_then(|g| logic_module::is_bounding_leap_available(g, &acting_player));
                if let Some(skill_id) = skill_id {
                    if let Some(id) = acting_player.player_id.clone() {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, id);
                    }
                }
            }
            ClientAction::GORED_BY_THE_BULL => {
                // TODO (java): "almost identical to block kind logic but is not sending the
                // block command probably because we handle frenzy blocks here?"
                if self.is_gored_available(client) {
                    if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                        if let Some(skill_id) =
                            UtilCards::get_unused_skill_with_property(&attacker, NamedProperties::CAN_ADD_BLOCK_DIE)
                        {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                        }
                    }
                }
            }
            ClientAction::INCORPOREAL => {
                if logic_module::is_incorporeal_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                        if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_AVOID_DODGING) {
                            // java: `player.hasActiveEnhancement(skill)` — see module doc gap;
                            // conservatively `false`, so `active` is always `false` and the
                            // command always requests activation (`!active` == `true`).
                            let active = false;
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), !active, attacker.id);
                        }
                    }
                }
            }
            _ => {
                self.extension.perform_available_action(client, player, action);
            }
        }
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
    fn get_id_is_blitz() {
        assert_eq!(BlitzLogicModule::new().get_id(), ClientStateId::Blitz);
    }

    #[test]
    fn available_actions_includes_extension_and_own_actions() {
        let module = BlitzLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::GORED_BY_THE_BULL));
        assert!(actions.contains(&ClientAction::INCORPOREAL));
        // From the BlockLogicExtension mixin:
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::TREACHEROUS));
    }

    #[test]
    fn move_action_is_blitz_move() {
        let module = BlitzLogicModule::new();
        assert_eq!(module.move_action(), PlayerAction::BlitzMove);
    }

    #[test]
    fn player_activation_used_falls_back_without_target_selection_state() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BlitzLogicModule::new();
        assert!(!module.player_activation_used(&client));
    }

    #[test]
    fn player_peek_resets_when_no_game() {
        let client = make_client();
        let module = BlitzLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_peek_resets_when_not_blockable() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10));
        client.set_game(game);
        let module = BlitzLogicModule::new();
        let player = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = BlitzLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn is_gored_available_false_without_target_selection_state() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BlitzLogicModule::new();
        assert!(!module.is_gored_available(&client));
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = BlitzLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::FUMBLEROOSKIE);
    }
}
