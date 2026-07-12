//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.BombLogicModule` (257 lines).
//!
//! Java's `BombLogicModule extends LogicModule` directly (no `MoveLogicModule`/extension
//! mixin), holding one field (`showRangeRuler`, default `true`).
//!
//! Documented gaps:
//! - `UtilRangeRuler.createRangeRuler(Game, Player<?>, FieldCoordinate, boolean)` is only a
//!   placeholder struct in `ffb_model::util::util_range_ruler`; reimplemented locally as
//!   `create_range_ruler`, mirroring the exact same helper already established in
//!   `dump_off_logic_module.rs` (not touched by this batch).
//! - `player.getSkillWithProperty(property)` — see `move_logic_module.rs`'s own doc gap;
//!   `skill_placeholder(SkillId)` is reused for the network command.

use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::types::{FieldCoordinate, RangeRuler};
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `UtilRangeRuler.createRangeRuler(Game, Player<?>, FieldCoordinate, boolean)` — see
/// module doc gap.
fn create_range_ruler(
    game: &Game,
    thrower: &Player,
    target_coordinate: FieldCoordinate,
    throw_team_mate: bool,
) -> Option<RangeRuler> {
    let mechanic = ffb_engine::mechanic::pass_mechanic_for(game.rules);
    let thrower_coordinate = game.field_model.player_coordinate(&thrower.id);
    let distance =
        mechanic.find_passing_distance(game, thrower_coordinate, Some(target_coordinate), throw_team_mate)?;

    let factory = PassModifierFactory::for_rules(game.rules);
    let ctx = PassContext::new(game, thrower, distance, throw_team_mate);
    let collection_total: i32 = factory.find_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
    let skill_total: i32 = factory.find_skill_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
    let total = collection_total + skill_total;
    let pass_modifiers: Vec<PassModifier> =
        if total != 0 { vec![PassModifier::new("pass_mods", total, ModifierType::REGULAR)] } else { Vec::new() };

    let minimum_roll = if throw_team_mate {
        // java: gap — `TtmMechanic::minimum_roll` needs a `HashSet<PassModifier>`, unavailable
        // here; this branch is unreachable from `BombLogicModule`, which always calls with
        // `false`.
        0
    } else {
        mechanic.minimum_roll_simple(thrower, distance, &pass_modifiers).unwrap_or(0)
    };

    Some(RangeRuler::new(thrower.id.clone(), Some(target_coordinate), minimum_roll, throw_team_mate))
}

/// 1:1 translation of the `BombLogicModule` class.
#[derive(Debug)]
pub struct BombLogicModule {
    show_range_ruler: bool,
}

impl Default for BombLogicModule {
    fn default() -> Self {
        Self::new()
    }
}

impl BombLogicModule {
    /// java: `public BombLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { show_range_ruler: true }
    }

    /// java: `public boolean showRangeRuler()`.
    pub fn show_range_ruler(&self, client: &FantasyFootballClient) -> bool {
        self.show_range_ruler && client.game().map(|g| g.pass_coordinate.is_none()).unwrap_or(false)
    }

    /// java: `public void setShowRangeRuler(boolean showRangeRuler)`.
    pub fn set_show_range_ruler(&mut self, show_range_ruler: bool) {
        self.show_range_ruler = show_range_ruler;
    }

    /// java: `public boolean isEndTurnActionAvailable()`.
    pub fn is_end_turn_action_available(&self, client: &FantasyFootballClient) -> bool {
        match client.game() {
            Some(game) => {
                // java: `!game.getActingPlayer().isMustCompleteAction()`.
                !game.turn_mode.is_bomb_turn() && !game.acting_player.is_must_complete_action()
            }
            None => false,
        }
    }

    /// java: `public boolean playerIsAboutToThrow()`.
    pub fn player_is_about_to_throw(&self, client: &FantasyFootballClient) -> bool {
        match client.game() {
            Some(game) => {
                game.acting_player.player_action == Some(PlayerAction::ThrowBomb)
                    || game.acting_player.player_action == Some(PlayerAction::HailMaryBomb)
            }
            None => false,
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::ignore(),
        };
        let acting_player = &game.acting_player;
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            InteractionResult::select_action(self.action_context(game, acting_player))
        } else {
            match game.field_model.player_coordinate(&player.id) {
                Some(coord) => InteractionResult::perform().with_coordinate(coord),
                None => InteractionResult::perform(),
            }
        }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate coordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let (thrower_coordinate, player_action, rules) = match client.game() {
            Some(game) => (
                game.field_model.player_coordinate(
                    game.acting_player.player_id.as_deref().unwrap_or_default(),
                ),
                game.acting_player.player_action,
                game.rules,
            ),
            None => return InteractionResult::ignore(),
        };
        let mechanic = ffb_engine::mechanic::pass_mechanic_for(rules);
        let passing_distance = match client.game() {
            Some(game) => mechanic.find_passing_distance(game, thrower_coordinate, Some(coordinate), false),
            None => None,
        };
        if player_action == Some(PlayerAction::HailMaryBomb) || passing_distance.is_some() {
            let acting_player_id = match client.game_mut() {
                Some(game) => {
                    game.pass_coordinate = Some(coordinate);
                    game.acting_player.player_id.clone()
                }
                None => return InteractionResult::ignore(),
            };
            if let Some(id) = acting_player_id {
                client.communication_mut().send_pass(id, coordinate);
            }
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::perform();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let coord = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
        match coord {
            Some(coord) => InteractionResult::perform().with_coordinate(coord),
            None => InteractionResult::perform(),
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate coordinate)`.
    pub fn field_peek(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let is_hail_mary = client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::HailMaryBomb)).unwrap_or(false);
        if is_hail_mary {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::perform();
        }
        if self.show_range_ruler(client) {
            let range_ruler = match client.game() {
                Some(game) => {
                    let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
                    thrower.and_then(|thrower| create_range_ruler(game, thrower, coordinate, false))
                }
                None => None,
            };
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = range_ruler.clone();
            }
            let mut result = InteractionResult::preview_throw();
            if let Some(rr) = range_ruler {
                result = result.with_range_ruler(rr);
            }
            result
        } else {
            InteractionResult::ignore()
        }
    }
}

impl LogicModule for BombLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Bomb
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::HAIL_MARY_BOMB);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::THEN_I_STARTED_BLASTIN);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        if logic_module::is_hail_mary_pass_action_available(game) {
            action_context.add_action(ClientAction::HAIL_MARY_BOMB);
            if acting_player.player_action == Some(PlayerAction::HailMaryBomb) {
                action_context.add_influence(Influences::IS_THROWING_HAIL_MARY);
            }
        }

        // java: `isEndTurnActionAvailable()`.
        let end_turn_available = !game.turn_mode.is_bomb_turn() && !acting_player.is_must_complete_action();
        if end_turn_available {
            action_context.add_action(ClientAction::END_MOVE);
        }

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
        if logic_module::is_zoat_gaze_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
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
                if self.is_end_turn_action_available(client) {
                    // java: `client.getCommunication().sendActingPlayer(null, null, false);` —
                    // see `LogicModule::deselect_acting_player`'s documented gap.
                }
            }
            ClientAction::HAIL_MARY_BOMB => {
                if logic_module::is_hail_mary_pass_action_available(client.game().unwrap()) {
                    if acting_player.player_action == Some(PlayerAction::HailMaryBomb) {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowBomb, acting_player.jumping);
                        self.set_show_range_ruler(true);
                    } else {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::HailMaryBomb, acting_player.jumping);
                        self.set_show_range_ruler(false);
                    }
                    let should_clear = !self.show_range_ruler(client)
                        && client.game().map(|g| g.field_model.range_ruler.is_some()).unwrap_or(false);
                    if should_clear {
                        if let Some(game) = client.game_mut() {
                            game.field_model.range_ruler = None;
                        }
                    }
                }
            }
            ClientAction::TREACHEROUS => {
                if logic_module::is_treacherous_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                if logic_module::is_wisdom_available_ap(client.game().unwrap(), &acting_player) {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                if logic_module::is_raiding_party_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if logic_module::is_look_into_my_eyes_available(client.game().unwrap(), player) {
                    if let Some(skill_id) =
                        UtilCards::get_unused_skill_with_property(player, NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if logic_module::is_baleful_hex_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if logic_module::is_black_ink_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                if logic_module::is_catch_of_the_day_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::THEN_I_STARTED_BLASTIN => {
                if logic_module::is_then_i_started_blastin_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                if logic_module::is_zoat_gaze_available_ap(client.game().unwrap(), &acting_player) {
                    if let Some(skill_id) =
                        player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
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
    fn get_id_is_bomb() {
        assert_eq!(BombLogicModule::new().get_id(), ClientStateId::Bomb);
    }

    #[test]
    fn available_actions_has_expected_len() {
        let actions = BombLogicModule::new().available_actions();
        assert_eq!(actions.len(), 11);
        assert!(actions.contains(&ClientAction::HAIL_MARY_BOMB));
    }

    #[test]
    fn show_range_ruler_defaults_true_without_pass_coordinate() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BombLogicModule::new();
        assert!(module.show_range_ruler(&client));
    }

    #[test]
    fn show_range_ruler_false_after_setting_false() {
        let mut client = make_client();
        client.set_game(make_game());
        let mut module = BombLogicModule::new();
        module.set_show_range_ruler(false);
        assert!(!module.show_range_ruler(&client));
    }

    #[test]
    fn is_end_turn_action_available_false_during_bomb_turn() {
        let mut client = make_client();
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::BombHome;
        client.set_game(game);
        let module = BombLogicModule::new();
        assert!(!module.is_end_turn_action_available(&client));
    }

    #[test]
    fn is_end_turn_action_available_true_during_regular_turn() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BombLogicModule::new();
        assert!(module.is_end_turn_action_available(&client));
    }

    #[test]
    fn is_end_turn_action_available_false_when_must_complete_action() {
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.set_must_complete_action(true);
        client.set_game(game);
        let module = BombLogicModule::new();
        assert!(!module.is_end_turn_action_available(&client));
    }

    #[test]
    fn action_context_omits_end_move_when_must_complete_action() {
        let module = BombLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.set_must_complete_action(true);
        let ctx = module.action_context(&game, &ap);
        assert!(!ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn player_is_about_to_throw_true_for_throw_bomb_action() {
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::ThrowBomb);
        client.set_game(game);
        let module = BombLogicModule::new();
        assert!(module.player_is_about_to_throw(&client));
    }

    #[test]
    fn action_context_empty_without_special_availability() {
        let module = BombLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let client = make_client();
        let module = BombLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = BombLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_peek_sets_selected_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let module = BombLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let _ = module.player_peek(&mut client, &player);
        assert_eq!(client.client_data().selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn field_peek_ignores_without_range_ruler_enabled() {
        let mut client = make_client();
        let mut game = make_game();
        client.set_game(game.clone());
        let mut module = BombLogicModule::new();
        module.set_show_range_ruler(false);
        game.acting_player.player_action = Some(PlayerAction::Move);
        client.set_game(game);
        let result = module.field_peek(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_for_unavailable_treacherous() {
        let mut module = BombLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::TREACHEROUS);
    }
}
