//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.BlockLogicModule` (152 lines).
//!
//! Java's `BlockLogicModule extends AbstractBlockLogicModule` and mixes in a
//! `BlockLogicExtension` (`extension` field), matching the `BlitzLogicModule` convention (see
//! that module's own doc comment). `AbstractBlockLogicModule`'s `getId()`/`endTurn()`/
//! `isSufferingBloodLust()` are translated as free functions in `abstract_block_logic_module.rs`
//! and delegated to here.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::abstract_block_logic_module;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

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
            if abstract_block_logic_module::is_suffering_blood_lust(&acting_player) {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                InteractionResult::select_action(ctx)
            } else if acting_player.player_action == Some(PlayerAction::Blitz) {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BlitzMove, acting_player.jumping);
                InteractionResult::handled()
            } else {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                if ctx.get_actions().is_empty() {
                    // java: `deselectActingPlayer()` — see `LogicModule::deselect_acting_player`'s
                    // own documented gap.
                    InteractionResult::handled()
                } else {
                    InteractionResult::select_action(ctx)
                }
            }
        } else {
            self.block(client, player, &acting_player)
        }
    }

    /// java: `protected InteractionResult block(Player<?> player, ActingPlayer actingPlayer)`.
    fn block(&self, client: &mut FantasyFootballClient, player: &Player, acting_player: &ActingPlayer) -> InteractionResult {
        let is_blockable = match client.game() {
            Some(game) => self.extension.is_blockable(game, player),
            None => false,
        };
        if is_blockable {
            let action = acting_player.player_action;
            if let Some(acting_player_id) = &acting_player.player_id {
                client.communication_mut().send_block(
                    acting_player_id.clone(),
                    Some(player),
                    action == Some(PlayerAction::Stab),
                    action == Some(PlayerAction::Chainsaw),
                    action == Some(PlayerAction::ProjectileVomit),
                    action == Some(PlayerAction::BreatheFire),
                    action == Some(PlayerAction::Chomp),
                );
            }
        }
        InteractionResult::ignore()
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
    /// java: `public ClientStateId getId()` (inherited from `AbstractBlockLogicModule`).
    fn get_id(&self) -> ClientStateId {
        abstract_block_logic_module::get_id()
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.merge(self.extension.action_context(game, acting_player));
        if abstract_block_logic_module::is_suffering_blood_lust(acting_player) {
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
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };

        match action {
            ClientAction::END_MOVE => {
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s documented gap.
            }
            ClientAction::MOVE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, acting_player.jumping);
            }
            ClientAction::BLOCK => {
                self.block(client, player, &acting_player);
            }
            ClientAction::TREACHEROUS => {
                if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::WISDOM => {
                client.communication_mut().send_use_wisdom();
            }
            ClientAction::RAIDING_PARTY => {
                if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                    if let Some(skill_id) =
                        UtilCards::get_unused_skill_with_property(&attacker, NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if let Some(attacker) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                if let Some(skill_id) =
                    player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                {
                    client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                }
            }
            _ => {}
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
    fn get_id_is_block() {
        assert_eq!(BlockLogicModule::new().get_id(), ClientStateId::Block);
    }

    #[test]
    fn available_actions_contains_block_and_move() {
        let module = BlockLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::AUTO_GAZE_ZOAT));
    }

    #[test]
    fn action_context_empty_without_any_availability() {
        let module = BlockLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn action_context_adds_end_move_when_has_acted() {
        let module = BlockLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.has_acted = true;
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
        assert!(ctx.get_influences().contains(&Influences::HAS_ACTED));
    }

    #[test]
    fn player_peek_resets_when_no_game() {
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
    fn perform_available_action_no_op_without_game() {
        let mut module = BlockLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::WISDOM);
    }
}
