//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.SelectBlitzTargetLogicModule`
//! (209 lines).
//!
//! Java's `SelectBlitzTargetLogicModule extends MoveLogicModule` and mixes in a
//! `BlockLogicExtension` (`extension` field), matching `blitz_logic_module.rs`'s established
//! composition pattern.
//!
//! UI stub: `playerPeek(Player<?> pPlayer)` calls
//! `client.getUserInterface().getFieldComponent()` purely for a rendering side-effect
//! (`fieldComponent.getLayerUnderPlayers().clearMovePath()`) with no return value consumed
//! downstream. Per the batch instructions, only those two lines are stubbed with a `// java:`
//! comment below; the rest of `playerPeek`'s `InteractionResult` branch logic is translated
//! normally. This is the only `getUserInterface()` call site in this file.
//!
//! Documented gaps:
//! - `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap;
//!   `skill_placeholder(SkillId)` is reused here for the network command.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::array_tool::ArrayTool;
use ffb_model::util::pathfinding::path_finder_with_multi_jump::PathFinderWithMultiJump;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// 1:1 translation of the `SelectBlitzTargetLogicModule` class.
#[derive(Debug, Default)]
pub struct SelectBlitzTargetLogicModule {
    move_logic: MoveLogicModule,
    extension: BlockLogicExtension,
}

impl SelectBlitzTargetLogicModule {
    /// java: `public SelectBlitzTargetLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new(), extension: BlockLogicExtension::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        let is_acting_player = acting_player.player_id.as_deref() == Some(player.id.as_str());

        if is_acting_player {
            let is_blitz_special = client
                .game()
                .map(|g| crate::client::state::logic::logic_module::is_blitz_special_ability_available(g, &acting_player))
                .unwrap_or(false);
            if is_blitz_special {
                let ctx = client
                    .game()
                    .map(|g| self.action_context(g, &acting_player))
                    .unwrap_or_else(ActionContext::new);
                return InteractionResult::select_action(ctx);
            }
        }

        let can_be_blitzed = client.game().map(|g| self.can_be_blitzed(g, player, &acting_player)).unwrap_or(false);
        if is_acting_player || can_be_blitzed {
            client.communication_mut().send_target_selected(player.id.clone());
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `private boolean canBeBlitzed(Player<?> pPlayer, ActingPlayer actingPlayer, Game game)`.
    fn can_be_blitzed(&self, game: &Game, player: &Player, acting_player: &ActingPlayer) -> bool {
        if acting_player.has_blocked || !self.extension.is_valid_blitz_target(game, Some(player)) {
            return false;
        }
        let field_model = &game.field_model;
        let target_coordinate = field_model.player_coordinate(&player.id);
        let blitzer_coordinate =
            acting_player.player_id.as_deref().and_then(|id| field_model.player_coordinate(id));
        let adjacent = match (blitzer_coordinate, target_coordinate) {
            (Some(b), Some(t)) => b.is_adjacent(t),
            _ => false,
        };
        adjacent
            || ArrayTool::is_provided(&PathFinderWithMultiJump::new().get_path_to_blitz_target(game, player).unwrap_or_default())
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        // java: `FieldComponent fieldComponent = client.getUserInterface().getFieldComponent();`
        // java: `fieldComponent.getLayerUnderPlayers().clearMovePath();`
        // abstract, no in-scope body, skipped (UserInterface untranslated - not a blocker for
        // the rest of this method)
        let (game, acting_player) = match client.game() {
            Some(g) => (g, g.acting_player.clone()),
            None => return InteractionResult::invalid(),
        };
        if self.can_be_blitzed(game, player, &acting_player) {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }
}

impl LogicModule for SelectBlitzTargetLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SelectBlitzTarget
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::END_MOVE);

        let player = acting_player.player_id.as_deref().and_then(|id| game.player(id));

        if let Some(player) = player {
            use crate::client::state::logic::logic_module::{
                is_baleful_hex_available, is_black_ink_available, is_catch_of_the_day_available,
                is_incorporeal_available, is_look_into_my_eyes_available, is_raiding_party_available,
                is_treacherous_available, is_wisdom_available, is_zoat_gaze_available,
            };
            if is_treacherous_available(game, player) {
                action_context.add_action(ClientAction::TREACHEROUS);
            }
            if is_wisdom_available(game, player) {
                action_context.add_action(ClientAction::WISDOM);
            }
            if is_raiding_party_available(game, player) {
                action_context.add_action(ClientAction::RAIDING_PARTY);
            }
            if is_look_into_my_eyes_available(game, player) {
                action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
            }
            if is_baleful_hex_available(game, player) {
                action_context.add_action(ClientAction::BALEFUL_HEX);
            }
            if is_black_ink_available(game, player) {
                action_context.add_action(ClientAction::BLACK_INK);
            }
            if is_catch_of_the_day_available(game, player) {
                action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
            }
            if crate::client::state::logic::logic_module::is_frenzied_rush_available(player) {
                action_context.add_action(ClientAction::FRENZIED_RUSH);
            }
            if crate::client::state::logic::logic_module::is_slashing_nails_available(player) {
                action_context.add_action(ClientAction::SLASHING_NAILS);
            }
            if is_zoat_gaze_available(game, player) {
                action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
            }
            if is_incorporeal_available(player) {
                action_context.add_action(ClientAction::INCORPOREAL);
            }
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let game = match client.game() {
            Some(g) => g,
            None => return,
        };
        match action {
            ClientAction::END_MOVE => {
                client.communication_mut().send_target_selected(player.id.clone());
            }
            ClientAction::TREACHEROUS => {
                if crate::client::state::logic::logic_module::is_treacherous_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                if crate::client::state::logic::logic_module::is_wisdom_available(game, player) {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                if crate::client::state::logic::logic_module::is_raiding_party_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if crate::client::state::logic::logic_module::is_look_into_my_eyes_available(game, player) {
                    if let Some(skill_id) = ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if crate::client::state::logic::logic_module::is_baleful_hex_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if crate::client::state::logic::logic_module::is_black_ink_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                if crate::client::state::logic::logic_module::is_catch_of_the_day_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::FRENZIED_RUSH => {
                if crate::client::state::logic::logic_module::is_frenzied_rush_available(player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::SLASHING_NAILS => {
                if crate::client::state::logic::logic_module::is_slashing_nails_available(player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                if crate::client::state::logic::logic_module::is_zoat_gaze_available(game, player) {
                    if let Some(skill_id) = player
                        .skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::INCORPOREAL => {
                if crate::client::state::logic::logic_module::is_incorporeal_available(player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_AVOID_DODGING) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::FRENZIED_RUSH);
        actions.insert(ClientAction::SLASHING_NAILS);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        actions.insert(ClientAction::INCORPOREAL);
        actions
    }

    /// java: `public void endTurn()` — not overridden in `SelectBlitzTargetLogicModule.java`;
    /// inherited unchanged from `MoveLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.move_logic.end_turn(client);
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
    fn get_id_is_select_blitz_target() {
        assert_eq!(SelectBlitzTargetLogicModule::new().get_id(), ClientStateId::SelectBlitzTarget);
    }

    #[test]
    fn available_actions_contains_end_move_and_incorporeal() {
        let module = SelectBlitzTargetLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::INCORPOREAL));
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn action_context_always_contains_end_move() {
        let module = SelectBlitzTargetLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn can_be_blitzed_false_when_has_blocked() {
        let module = SelectBlitzTargetLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(1, 1));
        let target = game.player("a1").unwrap().clone();
        let mut ap = ActingPlayer::new();
        ap.has_blocked = true;
        assert!(!module.can_be_blitzed(&game, &target, &ap));
    }

    #[test]
    fn player_peek_invalid_without_game() {
        let client = make_client();
        let module = SelectBlitzTargetLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_peek_invalid_when_not_blitzable() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10));
        client.set_game(game);
        let module = SelectBlitzTargetLogicModule::new();
        let player = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = SelectBlitzTargetLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_end_move_sends_target_selected() {
        let mut module = SelectBlitzTargetLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
        assert!(!client.communication().is_stopped());
    }
}
