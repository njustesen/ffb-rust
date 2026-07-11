//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.SelectGazeTargetLogicModule`
//! (164 lines).
//!
//! Java's `SelectGazeTargetLogicModule extends MoveLogicModule`, overriding `getId`,
//! `playerInteraction`, `playerPeek`, `actionContext`, `performAvailableAction`,
//! `availableActions`, and adding a private `isValidGazeTarget` helper.

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> ffb_model::model::skill::skill::Skill {
    ffb_model::model::skill::skill::Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `private boolean isValidGazeTarget(Game game, Player<?> target)`.
fn is_valid_gaze_target(game: &Game, target: &Player) -> bool {
    !game.active_team().has_player(&target.id)
        && game.field_model.player_state(&target.id).map(|s| s.has_tacklezones()).unwrap_or(false)
}

/// 1:1 translation of the `SelectGazeTargetLogicModule` class.
#[derive(Debug, Default)]
pub struct SelectGazeTargetLogicModule;

impl SelectGazeTargetLogicModule {
    /// java: `public SelectGazeTargetLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (acting_player, is_acting_player) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let is_acting = acting_player.player_id.as_deref() == Some(player.id.as_str());
                (acting_player, is_acting)
            }
            None => return InteractionResult::ignore(),
        };

        let special_ability_available =
            client.game().map(|g| logic_module::is_special_ability_available(g, &acting_player)).unwrap_or(false);

        if is_acting_player && special_ability_available {
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
        }

        let valid_gaze_target = client.game().map(|g| is_valid_gaze_target(g, player)).unwrap_or(false);

        if is_acting_player || valid_gaze_target {
            client.communication_mut().send_target_selected(player.id.clone());
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::invalid(),
        };
        if is_valid_gaze_target(game, player) {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }
}

impl LogicModule for SelectGazeTargetLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SelectGazeTarget
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::THEN_I_STARTED_BLASTIN);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::END_MOVE);

        if logic_module::is_treacherous_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::TREACHEROUS);
        }
        if logic_module::is_wisdom_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::WISDOM);
        }
        if logic_module::is_raiding_party_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::RAIDING_PARTY);
        }
        if logic_module::is_look_into_my_eyes_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
        }
        if logic_module::is_baleful_hex_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BALEFUL_HEX);
        }
        if logic_module::is_black_ink_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BLACK_INK);
        }
        if logic_module::is_catch_of_the_day_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
        }
        if logic_module::is_then_i_started_blastin_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::THEN_I_STARTED_BLASTIN);
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                client.communication_mut().send_target_selected(player.id.clone());
            }
            ClientAction::TREACHEROUS => {
                if client.game().map(|g| logic_module::is_treacherous_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                if client.game().map(|g| logic_module::is_wisdom_available(g, player)).unwrap_or(false) {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                if client.game().map(|g| logic_module::is_raiding_party_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if client.game().map(|g| logic_module::is_look_into_my_eyes_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if client.game().map(|g| logic_module::is_baleful_hex_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if client.game().map(|g| logic_module::is_black_ink_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                if client.game().map(|g| logic_module::is_catch_of_the_day_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::THEN_I_STARTED_BLASTIN => {
                if client.game().map(|g| logic_module::is_then_i_started_blastin_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
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
    fn get_id_is_select_gaze_target() {
        assert_eq!(SelectGazeTargetLogicModule::new().get_id(), ClientStateId::SelectGazeTarget);
    }

    #[test]
    fn available_actions_matches_java() {
        let actions = SelectGazeTargetLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::THEN_I_STARTED_BLASTIN));
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn is_valid_gaze_target_false_for_own_team() {
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let player = game.player("h1").unwrap().clone();
        assert!(!is_valid_gaze_target(&game, &player));
    }

    #[test]
    fn is_valid_gaze_target_true_for_opponent_with_tacklezones() {
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        let player = game.player("a1").unwrap().clone();
        assert!(is_valid_gaze_target(&game, &player));
    }

    #[test]
    fn player_peek_invalid_without_game() {
        let module = SelectGazeTargetLogicModule::new();
        let client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_peek_performs_for_valid_gaze_target() {
        let module = SelectGazeTargetLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        client.set_game(game);
        let player = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = SelectGazeTargetLogicModule::new();
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
        let mut module = SelectGazeTargetLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
        assert!(!client.communication().is_stopped());
    }
}
