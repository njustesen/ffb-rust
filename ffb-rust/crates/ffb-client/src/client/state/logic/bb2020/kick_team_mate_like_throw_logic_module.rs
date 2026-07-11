//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.KickTeamMateLikeThrowLogicModule`
//! (126 lines).
//!
//! Java's `KickTeamMateLikeThrowLogicModule extends MoveLogicModule`, overriding `getId`,
//! `showGridForKTM`, `playerInteraction`, `fieldInteraction`, `fieldPeek`, `playerPeek`, and
//! adding `canBeKicked`/`findKickablePlayers` helpers. Non-overridden methods
//! (`availableActions`, `actionContext`, `performAvailableAction`, `endTurn`) are inherited
//! from a held `MoveLogicModule` instance, per the established batch convention (see
//! `throw_team_mate_logic_module.rs`).
//!
//! Documented gap: `UtilPlayer.canKickTeamMate(Game, Player<?>, boolean)` is not translated in
//! `ffb-model`; reimplemented locally as `can_kick_team_mate` mirroring the Java body exactly.

use ffb_engine::mechanic::ttm_mechanic_for;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// java: `FieldModel.findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds.FIELD, 1,
/// false)` — no shared public helper exists on `FieldModel`; mirrors the private duplicate
/// already used in `logic_module.rs`'s own `adjacent_field_coordinates` (distance == 1 case).
fn adjacent_field_coordinates(game: &Game, coord: FieldCoordinate) -> Vec<FieldCoordinate> {
    game.field_model.adjacent_on_pitch(coord)
}

/// java: `UtilPlayer.canKickTeamMate(Game, Player<?>, boolean)` — not translated in
/// `ffb-model` (documented gap); reimplemented here directly, matching the Java body exactly.
fn can_kick_team_mate(game: &Game, kicker: &Player, check_blitz_used: bool) -> bool {
    let mechanic = ttm_mechanic_for(game.rules);
    (!check_blitz_used || !game.turn_data().blitz_used)
        && kicker.has_skill_property(NamedProperties::CAN_KICK_TEAM_MATES)
        && !mechanic.find_kickable_team_mates(game, kicker).is_empty()
}

/// 1:1 translation of the `KickTeamMateLikeThrowLogicModule` class.
#[derive(Debug, Default)]
pub struct KickTeamMateLikeThrowLogicModule {
    move_logic: MoveLogicModule,
}

impl KickTeamMateLikeThrowLogicModule {
    /// java: `public KickTeamMateLikeThrowLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `protected boolean showGridForKTM(Game game, ActingPlayer actingPlayer)`.
    pub fn show_grid_for_ktm(&self, game: &Game, acting_player: &ActingPlayer) -> bool {
        match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(player) => can_kick_team_mate(game, player, false),
            None => false,
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, has_defender, acting_player_id) = match client.game() {
            Some(game) => (
                game.acting_player.player_id.as_deref() == Some(player.id.as_str()),
                game.defender_id.is_some(),
                game.acting_player.player_id.clone(),
            ),
            None => return InteractionResult::ignore(),
        };

        if is_acting_player {
            return self.move_logic.player_interaction(client, player);
        }

        if !has_defender && self.can_be_kicked(client, player) {
            if let Some(id) = acting_player_id.clone() {
                client.communication_mut().send_throw_team_mate_by_id(id, player.id.clone());
            }
            return InteractionResult::perform();
        }
        if has_defender {
            let coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            if let (Some(id), Some(coordinate)) = (acting_player_id, coordinate) {
                client.communication_mut().send_throw_team_mate(id, coordinate);
            }
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let (is_ktm_move, acting_player_id) = match client.game() {
            Some(game) => (
                game.acting_player.player_action == Some(PlayerAction::KickTeamMateMove),
                game.acting_player.player_id.clone(),
            ),
            None => return InteractionResult::ignore(),
        };
        if is_ktm_move {
            return InteractionResult::delegate(self.move_logic.get_id());
        }
        if let Some(game) = client.game_mut() {
            game.field_model.range_ruler = None;
        }
        if let Some(id) = acting_player_id {
            client.communication_mut().send_throw_team_mate(id, coordinate);
        }
        InteractionResult::handled()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let show_preview = match client.game() {
            Some(game) => game.defender_id.is_some() && game.pass_coordinate.is_none(),
            None => false,
        };
        if show_preview {
            return InteractionResult::preview_throw();
        }
        self.move_logic.field_peek(client, coordinate)
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let (has_defender, has_pass_coordinate) = match client.game() {
            Some(game) => (game.defender_id.is_some(), game.pass_coordinate.is_some()),
            None => return InteractionResult::ignore(),
        };
        if !has_defender && !has_pass_coordinate {
            return if self.can_be_kicked(client, player) {
                InteractionResult::perform()
            } else {
                InteractionResult::reset()
            };
        }
        if has_defender && !has_pass_coordinate {
            return InteractionResult::preview_throw();
        }
        InteractionResult::ignore()
    }

    /// java: `private boolean canBeKicked(Player<?> pPlayer)`.
    fn can_be_kicked(&self, client: &FantasyFootballClient, player: &Player) -> bool {
        let game = match client.game() {
            Some(g) => g,
            None => return false,
        };
        let mechanic = ttm_mechanic_for(game.rules);
        let acting_player = &game.acting_player;
        let attacker = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return false,
        };
        let thrower_coordinate = game.field_model.player_coordinate(&attacker.id);
        let catcher_coordinate = game.field_model.player_coordinate(&player.id);
        // java: added a check, so you could not throw the opponents players, maybe this should
        // be in the server-check?
        attacker.has_skill_property(NamedProperties::CAN_KICK_TEAM_MATES)
            && mechanic.can_be_kicked(game, player)
            && match (catcher_coordinate, thrower_coordinate) {
                (Some(c), Some(t)) => c.is_adjacent(t),
                _ => false,
            }
    }

    /// java: `public Player<?>[] findKickablePlayers(Game game, Player<?> pThrower)`.
    pub fn find_kickable_players<'a>(&self, game: &'a Game, thrower: &Player) -> Option<Vec<&'a Player>> {
        if game.defender_id.is_some() {
            return None;
        }
        let mechanic = ttm_mechanic_for(game.rules);
        let thrower_coordinate = game.field_model.player_coordinate(&thrower.id)?;
        Some(
            adjacent_field_coordinates(game, thrower_coordinate)
                .into_iter()
                .filter_map(|coord| game.field_model.player_at(coord).and_then(|id| game.player(id)))
                .filter(|player| mechanic.can_be_kicked(game, player))
                .collect(),
        )
    }
}

impl LogicModule for KickTeamMateLikeThrowLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::KickTeamMateThrow
    }

    /// java: `public Set<ClientAction> availableActions()` — not overridden; inherited from
    /// `MoveLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden; inherited from `MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden; inherited from `MoveLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.move_logic.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — not overridden; inherited from `MoveLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.move_logic.end_turn(client);
    }
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
    fn get_id_is_kick_team_mate_throw() {
        assert_eq!(KickTeamMateLikeThrowLogicModule::new().get_id(), ClientStateId::KickTeamMateThrow);
    }

    #[test]
    fn available_actions_matches_move_logic_module() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        assert_eq!(module.available_actions().len(), MoveLogicModule::new().available_actions().len());
    }

    #[test]
    fn show_grid_for_ktm_false_without_player() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        assert!(!module.show_grid_for_ktm(&game, &ap));
    }

    #[test]
    fn can_be_kicked_false_without_game() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!module.can_be_kicked(&client, &player));
    }

    #[test]
    fn find_kickable_players_none_with_defender() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(2, 2));
        game.defender_id = Some("d1".to_string());
        let thrower = game.player("thrower").unwrap().clone();
        assert!(module.find_kickable_players(&game, &thrower).is_none());
    }

    #[test]
    fn find_kickable_players_empty_without_kickable_teammates() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(2, 2));
        let thrower = game.player("thrower").unwrap().clone();
        let result = module.find_kickable_players(&game, &thrower).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn field_peek_preview_throw_when_defender_present() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        let result = module.field_peek(&client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::PreviewThrow
        );
    }

    #[test]
    fn field_interaction_delegates_when_ktm_move() {
        let module = KickTeamMateLikeThrowLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMateMove);
        client.set_game(game);
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Delegate
        );
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = KickTeamMateLikeThrowLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
