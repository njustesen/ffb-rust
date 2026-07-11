//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.DumpOffLogicModule` (82 lines).
//!
//! Java's `DumpOffLogicModule extends MoveLogicModule`, overriding every abstract method and
//! most interaction hooks with dump-off-specific behavior (no delegation to the base class).
//!
//! Documented gaps:
//! - `availableActions()` returns `null` in Java (never consulted, since
//!   `performAvailableAction` is a no-op and nothing else calls it here); translated as an
//!   empty `HashSet` (the closest faithful Rust equivalent of "no actions").
//! - `UtilRangeRuler.createRangeRuler(...)` is only a placeholder struct in the Rust model
//!   (`ffb_model::util::util_range_ruler::UtilRangeRuler`); its real logic needs a live
//!   `PassMechanic` + `PassModifierFactory`, so it is reimplemented here as a private
//!   `create_range_ruler` helper mirroring `UtilRangeRuler.java` exactly, using the same
//!   "sum modifiers into one synthetic `PassModifier`" idiom already established in
//!   `ffb-engine/src/step/bb2025/pass/step_pass.rs` (Rust's `PassModifier` isn't `Clone`, so
//!   the `Vec<&PassModifier>` returned by `find_modifiers`/`find_skill_modifiers` can't be
//!   passed on directly). The `throw_team_mate == true` branch additionally needs a
//!   `HashSet<PassModifier>` for `TtmMechanic::minimum_roll`, but `PassModifier` has no
//!   `Hash`/`Eq` impl in the Rust model; conservatively treated as "no minimum roll" (`0`).
//!   This branch is unreachable from `DumpOffLogicModule`, which always calls with `false`.

use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_model::enums::ClientStateId;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::{FieldCoordinate, RangeRuler};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

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
        // java: gap — see module doc comment; TtmMechanic::minimum_roll needs a
        // `HashSet<PassModifier>`, unavailable here.
        0
    } else {
        mechanic.minimum_roll_simple(thrower, distance, &pass_modifiers).unwrap_or(0)
    };

    Some(RangeRuler::new(thrower.id.clone(), Some(target_coordinate), minimum_roll, throw_team_mate))
}

/// java: `private boolean testCoordinateInRange(FieldCoordinate coordinate)`.
fn test_coordinate_in_range(game: &Game, coordinate: FieldCoordinate) -> bool {
    let thrower = match game.thrower() {
        Some(t) => t,
        None => return false,
    };
    let thrower_coordinate = game.field_model.player_coordinate(&thrower.id);
    let mechanic = ffb_engine::mechanic::pass_mechanic_for(game.rules);
    let passing_distance = mechanic.find_passing_distance(game, thrower_coordinate, Some(coordinate), false);
    passing_distance == Some(ffb_model::enums::PassingDistance::QuickPass)
}

/// 1:1 translation of the `DumpOffLogicModule` class.
#[derive(Debug, Default)]
pub struct DumpOffLogicModule;

impl DumpOffLogicModule {
    /// java: `public DumpOffLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let in_range = match client.game() {
            Some(game) => test_coordinate_in_range(game, coordinate),
            None => false,
        };
        if !in_range {
            return InteractionResult::ignore();
        }
        let acting_player_id = match client.game_mut() {
            Some(game) => {
                game.pass_coordinate = Some(coordinate);
                game.field_model.range_ruler = None;
                game.acting_player.player_id.clone()
            }
            None => return InteractionResult::ignore(),
        };
        if let Some(id) = acting_player_id {
            client.communication_mut().send_pass(id, coordinate);
        }
        InteractionResult::perform()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let should_show_ruler = match client.game() {
            Some(game) => test_coordinate_in_range(game, coordinate) && game.pass_coordinate.is_none(),
            None => false,
        };
        if !should_show_ruler {
            return InteractionResult::reset();
        }
        let range_ruler = match client.game() {
            Some(game) => game.thrower().and_then(|thrower| create_range_ruler(game, thrower, coordinate, false)),
            None => None,
        };
        if let Some(game) = client.game_mut() {
            game.field_model.range_ruler = range_ruler;
        }
        InteractionResult::perform()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        InteractionResult::ignore()
    }
}

impl LogicModule for DumpOffLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::DumpOff
    }

    /// java: `public void setUp()`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        if let Some(game) = client.game_mut() {
            game.pass_coordinate = None;
        }
    }

    /// java: `public Set<ClientAction> availableActions()` — returns `null` in Java; see
    /// module doc gap.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        std::collections::HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden in Java; `DumpOffLogicModule` never calls it (its `availableActions()` is
    /// empty), so this is unreachable in practice. Returns an empty context.
    fn action_context(
        &self,
        _game: &Game,
        _acting_player: &ffb_model::model::acting_player::ActingPlayer,
    ) -> ActionContext {
        ActionContext::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, _client: &mut FantasyFootballClient, _player: &Player, _action: ClientAction) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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
        game.field_model
            .set_player_state(id, PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
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
    fn get_id_is_dump_off() {
        assert_eq!(DumpOffLogicModule::new().get_id(), ClientStateId::DumpOff);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(DumpOffLogicModule::new().available_actions().is_empty());
    }

    #[test]
    fn set_up_clears_pass_coordinate() {
        let mut module = DumpOffLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(3, 3));
        client.set_game(game);
        module.set_up(&mut client);
        assert!(client.game().unwrap().pass_coordinate.is_none());
    }

    #[test]
    fn field_interaction_ignores_out_of_range_coordinate() {
        let module = DumpOffLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(1, 1));
        game.thrower_id = Some("thrower".to_string());
        client.set_game(game);
        let result = module.field_interaction(&mut client, FieldCoordinate::new(20, 20));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_ignores_without_thrower() {
        let module = DumpOffLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&mut client, FieldCoordinate::new(2, 2));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_resets_without_thrower() {
        let module = DumpOffLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_peek(&mut client, FieldCoordinate::new(2, 2));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_peek_sets_selected_player_and_ignores() {
        let module = DumpOffLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
        assert_eq!(client.client_data().selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn perform_available_action_is_no_op() {
        let mut module = DumpOffLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
