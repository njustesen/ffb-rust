//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.BombLogicModule` (247 lines).
//!
//! Java's `BombLogicModule extends LogicModule` directly (no shared move/base module),
//! overriding every abstract method plus the four interaction hooks, and adding
//! `showRangeRuler`/`setShowRangeRuler`/`isEndTurnActionAvailable`/`playerIsAboutToThrow`.
//!
//! Documented gaps:
//! - `UtilRangeRuler.createRangeRuler(...)` is only a placeholder struct in the Rust model; its
//!   real logic needs a live `PassMechanic` + `PassModifierFactory`, so it is reimplemented here
//!   as a private `create_range_ruler` helper mirroring `UtilRangeRuler.java` exactly, matching
//!   the identical gap/workaround already used in `dump_off_logic_module.rs`.
//! - `InteractionResult.with(RangeRuler)` in Java can carry a `null` range ruler (when
//!   `UtilRangeRuler.createRangeRuler` finds no valid passing distance); the Rust
//!   `with_range_ruler` takes an owned `RangeRuler`, not an `Option`, so the `None` case falls
//!   back to `preview_throw()` without an attached ruler (closest faithful equivalent).

use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::{FieldCoordinate, RangeRuler};
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

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
        // java: gap — see `dump_off_logic_module.rs`'s own doc comment; unreachable here since
        // `BombLogicModule` always calls with `throw_team_mate == false`.
        0
    } else {
        mechanic.minimum_roll_simple(thrower, distance, &pass_modifiers).unwrap_or(0)
    };

    Some(RangeRuler::new(thrower.id.clone(), Some(target_coordinate), minimum_roll, throw_team_mate))
}

/// java: `public boolean isEndTurnActionAvailable()`.
pub fn is_end_turn_action_available(game: &Game) -> bool {
    !game.turn_mode.is_bomb_turn() && !game.acting_player.is_must_complete_action()
}

/// java: `public boolean playerIsAboutToThrow()`.
pub fn player_is_about_to_throw(game: &Game) -> bool {
    let acting_player = &game.acting_player;
    acting_player.player_action == Some(PlayerAction::ThrowBomb)
        || acting_player.player_action == Some(PlayerAction::HailMaryBomb)
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

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, coordinate) = match client.game() {
            Some(game) => (
                game.acting_player.player_id.as_deref() == Some(player.id.as_str()),
                game.field_model.player_coordinate(&player.id),
            ),
            None => return InteractionResult::ignore(),
        };

        if is_acting_player {
            let ap = client.game().unwrap().acting_player.clone();
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &ap),
                None => ActionContext::new(),
            };
            InteractionResult::select_action(ctx)
        } else {
            match coordinate {
                Some(c) => InteractionResult::perform().with_coordinate(c),
                None => InteractionResult::perform(),
            }
        }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate coordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let (should_pass, acting_player_id) = match client.game() {
            Some(game) => {
                let acting_player = &game.acting_player;
                let thrower_coordinate = acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
                let mechanic = ffb_engine::mechanic::pass_mechanic_for(game.rules);
                let passing_distance = mechanic.find_passing_distance(game, thrower_coordinate, Some(coordinate), false);
                let should_pass =
                    acting_player.player_action == Some(PlayerAction::HailMaryBomb) || passing_distance.is_some();
                (should_pass, acting_player.player_id.clone())
            }
            None => return InteractionResult::ignore(),
        };

        if !should_pass {
            return InteractionResult::ignore();
        }
        if let Some(game) = client.game_mut() {
            game.pass_coordinate = Some(coordinate);
        }
        if let Some(id) = acting_player_id {
            client.communication_mut().send_pass(id, coordinate);
        }
        if let Some(game) = client.game_mut() {
            game.field_model.range_ruler = None;
        }
        InteractionResult::perform()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
        match coordinate {
            Some(c) => InteractionResult::perform().with_coordinate(c),
            None => InteractionResult::perform(),
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate coordinate)`.
    pub fn field_peek(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let hail_mary = match client.game() {
            Some(game) => game.acting_player.player_action == Some(PlayerAction::HailMaryBomb),
            None => return InteractionResult::ignore(),
        };

        if hail_mary {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::perform();
        }

        if !self.show_range_ruler(client) {
            return InteractionResult::ignore();
        }

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
        match range_ruler {
            Some(rr) => InteractionResult::preview_throw().with_range_ruler(rr),
            None => InteractionResult::preview_throw(),
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

        if is_end_turn_action_available(game) {
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
                let available = client.game().map(is_end_turn_action_available).unwrap_or(false);
                if available {
                    // java: `client.getCommunication().sendActingPlayer(null, null, false);` —
                    // see `LogicModule::deselect_acting_player`'s documented gap.
                }
            }
            ClientAction::HAIL_MARY_BOMB => {
                let available = client.game().map(logic_module::is_hail_mary_pass_action_available).unwrap_or(false);
                if available {
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
                let available =
                    client.game().map(|g| logic_module::is_treacherous_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                let available =
                    client.game().map(|g| logic_module::is_wisdom_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                let available =
                    client.game().map(|g| logic_module::is_raiding_party_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                let available =
                    client.game().map(|g| logic_module::is_look_into_my_eyes_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) =
                        UtilCards::get_unused_skill_with_property(player, NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT)
                    {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                let available =
                    client.game().map(|g| logic_module::is_baleful_hex_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                let available =
                    client.game().map(|g| logic_module::is_black_ink_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                let available =
                    client.game().map(|g| logic_module::is_catch_of_the_day_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                let available =
                    client.game().map(|g| logic_module::is_zoat_gaze_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) =
                        player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

/// java: `player.getSkillWithProperty(property)` — see `move_logic_module.rs`'s own doc gap.
fn send_skill(client: &mut FantasyFootballClient, skill_id: ffb_model::enums::SkillId, player_id: String) {
    let skill = ffb_model::model::skill::skill::Skill::new(skill_id.class_name(), ffb_model::enums::SkillCategory::General);
    client.communication_mut().send_use_skill(&skill, true, player_id);
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
    fn is_end_turn_action_available_true_when_not_bomb_turn() {
        let game = make_game();
        assert!(is_end_turn_action_available(&game));
    }

    #[test]
    fn is_end_turn_action_available_false_when_must_complete_action() {
        let mut game = make_game();
        game.acting_player.set_must_complete_action(true);
        assert!(!is_end_turn_action_available(&game));
    }

    #[test]
    fn player_is_about_to_throw_false_by_default() {
        let game = make_game();
        assert!(!player_is_about_to_throw(&game));
    }

    #[test]
    fn player_is_about_to_throw_true_for_hail_mary_bomb() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::HailMaryBomb);
        assert!(player_is_about_to_throw(&game));
    }

    #[test]
    fn show_range_ruler_true_by_default_without_pass_coordinate() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BombLogicModule::new();
        assert!(module.show_range_ruler(&client));
    }

    #[test]
    fn show_range_ruler_false_after_disabling() {
        let mut client = make_client();
        client.set_game(make_game());
        let mut module = BombLogicModule::new();
        module.set_show_range_ruler(false);
        assert!(!module.show_range_ruler(&client));
    }

    #[test]
    fn available_actions_contains_hail_mary_bomb() {
        let module = BombLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::HAIL_MARY_BOMB));
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_adds_end_move_by_default() {
        let module = BombLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = BombLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = BombLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_ignores_without_game() {
        let mut client = make_client();
        let module = BombLogicModule::new();
        let result = module.field_peek(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_peek_sets_selected_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let module = BombLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_peek(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
        assert_eq!(client.client_data().selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = BombLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::WISDOM);
    }
}
