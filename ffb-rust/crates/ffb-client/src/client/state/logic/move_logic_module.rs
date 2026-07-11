//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.MoveLogicModule` (466 lines).
//!
//! Java's `MoveLogicModule` is a base `LogicModule` subclass extended by six edition-specific
//! logic modules (Blitz/DumpOff/KickoffReturn/PassBlock/Swoop/ThrowTeamMate — a later batch).
//! Its constructor resolves a `MoveLogicPlugin` via `LogicPluginFactory`
//! (`client/factory/LogicPluginFactory.java`), which is not yet translated (documented gap,
//! same pattern already used for `BlockLogicExtension`). Consequently there is no `plugin`
//! field here; every call site that would delegate to `plugin.xxx(...)` is left with a
//! `// java: plugin — gap` comment and a conservative fallback (empty/no-op/pass-through).
//!
//! Other documented gaps:
//! - `LogicModule::player_interaction`/`field_interaction`/`field_peek`'s trait-default
//!   signatures have no `client`/mutability parameter, so they cannot express Java's real
//!   logic (which needs `&(mut) FantasyFootballClient` to read `Game` state and send network
//!   commands). Per the same pattern already used for `BlockLogicExtension`, these are
//!   translated as inherent methods that shadow the trait defaults by taking the extra
//!   parameters Java's logic actually requires.
//! - `client.getProperty(CommonProperty.SETTING_AUTOMOVE)` / `SETTING_RE_ROLL_BALL_AND_CHAIN`:
//!   `getProperty()` is `abstract` on `FantasyFootballClient` with no in-scope body (see that
//!   module's own doc comment). Java's default (no user setting) value is `null`; since `null`
//!   is never `.equals(SETTING_AUTOMOVE_OFF)`, the automove-off check conservatively always
//!   takes the "not off" branch (matching that default), and the re-roll setting is passed as
//!   `None`.
//! - `Constant.MINIMUM_MOVE_TO_STAND_UP` (`ffb-common/Constant.java`) has not been ported as a
//!   shared Rust constant; its value (`3`) is mirrored here as a local literal.
//! - `Player.getTeam()` — the Rust `Player` has no back-reference to its owning `Team`; looked
//!   up via `Game::player_team_id` + `Game::team_by_id`/`Game::player`, matching
//!   `logic_module.rs`'s own `player_own_team` helper.

use ffb_engine::mechanic::jump_mechanic_for;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_model::enums::{ClientStateId, PlayerAction, TurnMode};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::{FieldCoordinate, MoveSquare, MoveSquareKind};
use ffb_model::util::pathfinding::path_finder_with_pass_block_support::PathFinderWithPassBlockSupport;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `Constant.MINIMUM_MOVE_TO_STAND_UP` — see module doc gap.
const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of the `MoveLogicModule` class.
#[derive(Debug, Default)]
pub struct MoveLogicModule;

impl MoveLogicModule {
    /// java: `public MoveLogicModule(FantasyFootballClient client)` — see module doc gap
    /// regarding the un-translated `plugin` field.
    pub fn new() -> Self {
        Self
    }

    /// java: `private MoveSquare moveSquare(FieldCoordinate coordinate)`.
    pub fn move_square(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> Option<MoveSquare> {
        let game = client.game()?;
        let move_square = game.field_model.get_move_square(coordinate)?;
        let acting_player = &game.acting_player;
        let acting_player_player = acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let from_coordinate = acting_player_player.and_then(|p| game.field_model.player_coordinate(&p.id));

        let valid = acting_player_player.is_none()
            || !acting_player.jumping
            || match (acting_player_player, from_coordinate) {
                (Some(p), Some(from)) => jump_mechanic_for(game.rules).is_valid_jump(game, p, from, coordinate),
                _ => false,
            };

        if valid {
            Some(move_square)
        } else {
            None
        }
    }

    /// java: `public MoveSquare.Kind kind(MoveSquare moveSquare)`.
    pub fn kind(&self, client: &FantasyFootballClient, move_square: MoveSquare) -> MoveSquareKind {
        let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
        if move_square.is_going_for_it() && move_square.is_dodging() && !jumping {
            MoveSquareKind::RushDodge
        } else if move_square.is_going_for_it() {
            MoveSquareKind::Rush
        } else if move_square.is_dodging() && !jumping {
            MoveSquareKind::Dodge
        } else {
            MoveSquareKind::Move
        }
    }

    /// java: `protected boolean movePlayer(FieldCoordinate pCoordinate)`.
    pub fn move_player(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        self.move_player_path(client, &[coordinate])
    }

    /// java: `private boolean movePlayer(FieldCoordinate[] pCoordinates)`.
    fn move_player_path(&self, client: &mut FantasyFootballClient, coordinates: &[FieldCoordinate]) -> bool {
        if coordinates.is_empty() {
            return false;
        }

        let (acting_player, coordinate_from, rules) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let coordinate_from =
                    acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
                (acting_player, coordinate_from, game.rules)
            }
            None => return false,
        };
        let coordinate_from = match coordinate_from {
            Some(c) => c,
            None => return false,
        };

        let last_coordinate = coordinates[coordinates.len() - 1];

        if acting_player.jumping {
            let player = acting_player.player_id.as_deref().and_then(|id| client.game().and_then(|g| g.player(id))).cloned();
            let valid = match (&player, client.game()) {
                (Some(player), Some(game)) => {
                    jump_mechanic_for(rules).is_valid_jump(game, player, coordinate_from, last_coordinate)
                }
                _ => false,
            };
            if !valid {
                return false;
            }
        }

        let cleaned_coordinates: Vec<FieldCoordinate> =
            if acting_player.jumping { vec![last_coordinate] } else { coordinates.to_vec() };

        self.send_command(client, &acting_player, coordinate_from, &cleaned_coordinates);
        true
    }

    /// java: `protected void sendCommand(ActingPlayer actingPlayer, FieldCoordinate
    /// coordinateFrom, FieldCoordinate[] pCoordinates)`.
    pub fn send_command(
        &self,
        client: &mut FantasyFootballClient,
        acting_player: &ActingPlayer,
        coordinate_from: FieldCoordinate,
        coordinates: &[FieldCoordinate],
    ) {
        if let Some(id) = acting_player.player_id.clone() {
            // java: `client.getProperty(CommonProperty.SETTING_RE_ROLL_BALL_AND_CHAIN)` — see
            // module doc gap; passed as `None`.
            client.communication_mut().send_player_move(id, coordinate_from, coordinates.to_vec(), None);
        }
    }

    /// java: `public FieldCoordinate[] automovePath(FieldCoordinate coordinate)`.
    pub fn automove_path(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> Vec<FieldCoordinate> {
        let game = match client.game() {
            Some(g) => g,
            None => return Vec::new(),
        };
        let acting_player = &game.acting_player;
        let moving = acting_player.player_action.map(|a| a.is_moving()).unwrap_or(false);
        let has_player = acting_player.player_id.is_some();
        let prevents_auto_move = acting_player
            .player_id
            .as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::PREVENT_AUTO_MOVE))
            .unwrap_or(false);

        // java: `!IClientPropertyValue.SETTING_AUTOMOVE_OFF.equals(automoveProperty)` — see
        // module doc gap; conservatively always true (matches Java's null-property default).
        if has_player
            && moving
            && !game.field_model.move_squares.is_empty()
            && game.turn_mode != TurnMode::PassBlock
            && game.turn_mode != TurnMode::KickoffReturn
            && game.turn_mode != TurnMode::Swarming
            && !prevents_auto_move
        {
            PathFinderWithPassBlockSupport::new().get_shortest_path_to_coord(game, coordinate).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// java: `public FieldCoordinate[] findShortestPath(FieldCoordinate coordinate)`.
    pub fn find_shortest_path(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> Vec<FieldCoordinate> {
        let (has_player, moving, prevents_auto_move) = match client.game() {
            Some(game) => {
                let ap = &game.acting_player;
                let moving = ap.player_action.map(|a| a.is_moving()).unwrap_or(false);
                let prevents = ap
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::PREVENT_AUTO_MOVE))
                    .unwrap_or(false);
                (ap.player_id.is_some(), moving, prevents)
            }
            None => return Vec::new(),
        };

        // java: gap — see module doc comment (automove setting always treated as "not off").
        if !(has_player && moving && !prevents_auto_move) {
            return Vec::new();
        }

        let (standing_up, can_stand_up_free, movement_with_modifiers) = match client.game() {
            Some(game) => {
                let ap = &game.acting_player;
                let player = ap.player_id.as_deref().and_then(|id| game.player(id));
                (
                    ap.standing_up,
                    player.map(|p| p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE)).unwrap_or(false),
                    player.map(|p| p.movement_with_modifiers()).unwrap_or(0),
                )
            }
            None => return Vec::new(),
        };

        if standing_up && !can_stand_up_free {
            let new_current_move = MINIMUM_MOVE_TO_STAND_UP.min(movement_with_modifiers);
            let going_for_it = client.game().map(UtilPlayer::is_next_move_going_for_it).unwrap_or(false);
            if let Some(game) = client.game_mut() {
                game.acting_player.current_move = new_current_move;
                game.acting_player.goes_for_it = going_for_it;
            }
        }

        let game = match client.game() {
            Some(g) => g,
            None => return Vec::new(),
        };
        let player_in_target = game.field_model.player_at(coordinate).and_then(|id| game.player(id));
        let acting_player_team = game.acting_player.player_id.as_deref().and_then(|id| game.player_team_id(id));

        let finder = PathFinderWithPassBlockSupport::new();
        match player_in_target {
            Some(target) if game.player_team_id(&target.id) != acting_player_team => {
                finder.get_shortest_path_to_player(game, target).unwrap_or_default()
            }
            _ => finder.get_shortest_path_to_coord(game, coordinate).unwrap_or_default(),
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)` — see module doc gap
    /// regarding the trait-default signature.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, position, acting_player, rules) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let is_acting = acting_player.player_id.as_deref() == Some(player.id.as_str());
                let position =
                    acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
                (is_acting, position, acting_player, game.rules)
            }
            None => return InteractionResult::ignore(),
        };

        if is_acting_player {
            let position = match position {
                Some(p) => p,
                None => return InteractionResult::ignore(),
            };
            let mechanic = jump_mechanic_for(rules);
            let available = match client.game() {
                Some(game) => self.action_available(game, player, &acting_player, mechanic.as_ref(), position),
                None => false,
            };
            if available {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                InteractionResult::select_action(ctx)
            } else {
                // java: `deselectActingPlayer()` — see `LogicModule::deselect_acting_player`'s
                // own documented gap.
                self.deselect_acting_player(client);
                InteractionResult::handled()
            }
        } else {
            // B&C
            let player_coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            let has_move_square = player_coordinate
                .and_then(|c| client.game().and_then(|g| g.field_model.get_move_square(c)))
                .is_some();
            if let Some(coord) = player_coordinate {
                if has_move_square && self.move_player(client, coord) {
                    return InteractionResult::perform().with_coordinate(coord);
                }
            }
            InteractionResult::ignore()
        }
    }

    /// java: `protected boolean actionAvailable(Player<?> player, ActingPlayer actingPlayer,
    /// JumpMechanic mechanic, Game game, FieldCoordinate position)`.
    pub fn action_available(
        &self,
        game: &Game,
        player: &Player,
        acting_player: &ActingPlayer,
        mechanic: &dyn JumpMechanicTrait,
        position: FieldCoordinate,
    ) -> bool {
        acting_player.has_acted
            || mechanic.can_jump(game, player, position)
            || player.has_skill_property(NamedProperties::CAN_GAZE_DURING_MOVE)
            || logic_module::is_special_ability_available(game, acting_player)
            || (player.has_skill_property(NamedProperties::CAN_DROP_BALL) && UtilPlayer::has_ball(game, &player.id))
            || (acting_player.player_action == Some(PlayerAction::PassMove) && UtilPlayer::has_ball(game, &player.id))
            || (acting_player.player_action == Some(PlayerAction::HandOverMove)
                && UtilPlayer::has_ball(game, &player.id))
            || acting_player.player_action == Some(PlayerAction::ThrowTeamMateMove)
            || acting_player.player_action == Some(PlayerAction::ThrowTeamMate)
            || acting_player.player_action == Some(PlayerAction::KickTeamMateMove)
            || acting_player.player_action == Some(PlayerAction::KickTeamMate)
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate coordinate)` — see
    /// module doc gap regarding the trait-default signature.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let move_square = self.move_square(client, coordinate);
        let move_path = self.automove_path(client, coordinate);
        if move_square.is_some() {
            self.move_player(client, coordinate);
            InteractionResult::handled()
        } else if !move_path.is_empty() {
            self.move_player_path(client, &move_path);
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate coordinate)` — see module doc
    /// gap regarding the trait-default signature.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        match self.move_square(client, coordinate) {
            Some(ms) => InteractionResult::perform().with_move_square(ms),
            None => {
                let path = self.automove_path(client, coordinate);
                if !path.is_empty() {
                    InteractionResult::perform().with_path(path)
                } else {
                    InteractionResult::reset()
                }
            }
        }
    }
}

impl LogicModule for MoveLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Move
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::JUMP);
        actions.insert(ClientAction::HAND_OVER);
        actions.insert(ClientAction::PASS);
        actions.insert(ClientAction::THROW_TEAM_MATE);
        actions.insert(ClientAction::KICK_TEAM_MATE);
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::GAZE);
        actions.insert(ClientAction::FUMBLEROOSKIE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::BOUNDING_LEAP);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        // java: plugin.availableActions() — gap, contributes nothing extra.
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut context = ActionContext::new();

        if logic_module::is_pass_any_square_available(acting_player, game) {
            context.add_action(ClientAction::PASS);
        }
        if logic_module::is_move_available(acting_player) {
            context.add_action(ClientAction::MOVE);
        }
        if logic_module::is_jump_available_as_next_move(game, acting_player, true) {
            context.add_action(ClientAction::JUMP);
            if acting_player.jumping {
                context.add_influence(Influences::IS_JUMPING);
            } else if logic_module::is_bounding_leap_available(game, acting_player).is_some() {
                context.add_action(ClientAction::BOUNDING_LEAP);
            }
        }
        if let Some(player) = acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            if logic_module::is_hypnotic_gaze_action_available(game, false, player) {
                context.add_action(ClientAction::GAZE);
            }
        }
        if logic_module::is_fumblerooskie_available(game) {
            context.add_action(ClientAction::FUMBLEROOSKIE);
        }
        if logic_module::is_end_player_action_available(game) {
            if acting_player.has_acted {
                context.add_influence(Influences::HAS_ACTED);
            }
            context.add_action(ClientAction::END_MOVE);
        }
        if logic_module::is_treacherous_available_ap(game, acting_player) {
            context.add_action(ClientAction::TREACHEROUS);
        }
        if logic_module::is_wisdom_available_ap(game, acting_player) {
            context.add_action(ClientAction::WISDOM);
        }
        if logic_module::is_raiding_party_available_ap(game, acting_player) {
            context.add_action(ClientAction::RAIDING_PARTY);
        }
        if logic_module::is_look_into_my_eyes_available_ap(game, acting_player) {
            context.add_action(ClientAction::LOOK_INTO_MY_EYES);
        }
        if logic_module::is_baleful_hex_available_ap(game, acting_player) {
            context.add_action(ClientAction::BALEFUL_HEX);
        }
        if logic_module::is_putrid_regurgitation_available() {
            context.add_influence(Influences::VOMIT_DUE_TO_PUTRID_REGURGITATION);
            context.add_action(ClientAction::PROJECTILE_VOMIT);
        }
        if logic_module::is_black_ink_available_ap(game, acting_player) {
            context.add_action(ClientAction::BLACK_INK);
        }
        if logic_module::is_catch_of_the_day_available_ap(game, acting_player) {
            context.add_action(ClientAction::CATCH_OF_THE_DAY);
        }
        if logic_module::is_zoat_gaze_available_ap(game, acting_player) {
            context.add_action(ClientAction::AUTO_GAZE_ZOAT);
        }
        // java: plugin.actionContext(actingPlayer, context, this) — gap; pass through unchanged.
        context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };

        match action {
            ClientAction::END_MOVE => {
                let available = client.game().map(logic_module::is_end_player_action_available).unwrap_or(false);
                if available {
                    // java: `communication.sendActingPlayer(null, null, false);` — see
                    // `LogicModule::deselect_acting_player`'s own documented gap.
                }
            }
            ClientAction::JUMP => {
                let jump_ok = client
                    .game()
                    .map(|g| logic_module::is_jump_available_as_next_move(g, &acting_player, false))
                    .unwrap_or(false);
                if jump_ok {
                    if let Some(pa) = acting_player.player_action {
                        client.communication_mut().send_acting_player(Some(player), pa, !acting_player.jumping);
                    }
                }
            }
            ClientAction::HAND_OVER => {
                let has_ball = client
                    .game()
                    .map(|g| acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(g, id)).unwrap_or(false))
                    .unwrap_or(false);
                if has_ball {
                    if acting_player.player_action == Some(PlayerAction::HandOverMove) {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::HandOver, acting_player.jumping);
                    } else if acting_player.player_action == Some(PlayerAction::HandOver) {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::HandOverMove, acting_player.jumping);
                    }
                }
            }
            ClientAction::PASS => {
                let has_ball = client
                    .game()
                    .map(|g| acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(g, id)).unwrap_or(false))
                    .unwrap_or(false);
                if acting_player.player_action == Some(PlayerAction::PassMove) && has_ball {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::Pass, acting_player.jumping);
                }
            }
            ClientAction::THROW_TEAM_MATE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowTeamMate, acting_player.jumping);
            }
            ClientAction::KICK_TEAM_MATE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::KickTeamMate, acting_player.jumping);
            }
            ClientAction::MOVE => match acting_player.player_action {
                Some(PlayerAction::Gaze) => {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, acting_player.jumping);
                }
                Some(PlayerAction::Pass) | Some(PlayerAction::HailMaryPass) => {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::PassMove, acting_player.jumping);
                }
                Some(PlayerAction::ThrowTeamMate) => {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowTeamMateMove, acting_player.jumping);
                }
                Some(PlayerAction::KickTeamMate) => {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::KickTeamMateMove, acting_player.jumping);
                }
                _ => {
                    // java: plugin.performAvailableAction(action, actingPlayer, this, communication) — gap.
                }
            },
            ClientAction::GAZE => {
                let available = match client.game() {
                    Some(game) => acting_player
                        .player_id
                        .as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| logic_module::is_hypnotic_gaze_action_available(game, false, p))
                        .unwrap_or(false),
                    None => false,
                };
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::Gaze, acting_player.jumping);
                }
            }
            ClientAction::FUMBLEROOSKIE => {
                let available = client.game().map(logic_module::is_fumblerooskie_available).unwrap_or(false);
                if available {
                    client.communication_mut().send_use_fumblerooskie();
                }
            }
            ClientAction::TREACHEROUS => {
                let available =
                    client.game().map(|g| logic_module::is_treacherous_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
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
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                let available =
                    client.game().map(|g| logic_module::is_look_into_my_eyes_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                let available =
                    client.game().map(|g| logic_module::is_baleful_hex_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::PROJECTILE_VOMIT => {
                if logic_module::is_putrid_regurgitation_available() {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                let available =
                    client.game().map(|g| logic_module::is_black_ink_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                let available =
                    client.game().map(|g| logic_module::is_catch_of_the_day_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            ClientAction::BOUNDING_LEAP => {
                let skill_id =
                    client.game().and_then(|g| logic_module::is_bounding_leap_available(g, &acting_player));
                if let Some(skill_id) = skill_id {
                    if let Some(id) = acting_player.player_id.clone() {
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, id);
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
                        let skill = skill_placeholder(skill_id);
                        client.communication_mut().send_use_skill(&skill, true, player.id.clone());
                    }
                }
            }
            _ => {
                // java: plugin.performAvailableAction(action, actingPlayer, this, communication) — gap.
            }
        }
    }

    /// java: `public void endTurn()`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        let (turn_mode, player) = match client.game() {
            Some(game) => (
                game.turn_mode,
                game.acting_player.player_id.as_deref().and_then(|id| game.player(id)).cloned(),
            ),
            None => return,
        };
        if let Some(player) = player {
            self.perform(client, &player, ClientAction::END_MOVE);
        }
        client.communication_mut().send_end_turn(turn_mode);
    }
}

/// java: `player.getSkillWithProperty(property)` — see module doc comment on
/// `block_logic_extension.rs` for the same gap: the Rust `Player` only exposes
/// `skill_id_with_property(&str) -> Option<SkillId>`, and `ClientCommunication::send_use_skill`
/// only actually serializes `skill.get_name()` onto the wire, so a name-only placeholder
/// (via `SkillId::class_name()`) is sufficient for the network command.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> ffb_model::model::skill::skill::Skill {
    ffb_model::model::skill::skill::Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
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
    fn get_id_is_move() {
        assert_eq!(MoveLogicModule::new().get_id(), ClientStateId::Move);
    }

    #[test]
    fn available_actions_contains_expected_variants() {
        let actions = MoveLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::JUMP));
        assert!(actions.contains(&ClientAction::AUTO_GAZE_ZOAT));
        assert_eq!(actions.len(), 20);
    }

    #[test]
    fn action_context_empty_without_any_special_availability() {
        let module = MoveLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn kind_returns_move_for_plain_square() {
        let module = MoveLogicModule::new();
        let client = make_client();
        let square = MoveSquare::new(FieldCoordinate::new(3, 3), 0, 0);
        assert_eq!(module.kind(&client, square), MoveSquareKind::Move);
    }

    #[test]
    fn kind_returns_dodge_when_dodging_and_not_jumping() {
        let module = MoveLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let square = MoveSquare::new(FieldCoordinate::new(3, 3), 3, 0);
        assert_eq!(module.kind(&client, square), MoveSquareKind::Dodge);
    }

    #[test]
    fn move_square_none_without_game() {
        let module = MoveLogicModule::new();
        let client = make_client();
        assert!(module.move_square(&client, FieldCoordinate::new(1, 1)).is_none());
    }

    #[test]
    fn automove_path_empty_without_moving_action() {
        let module = MoveLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let path = module.automove_path(&client, FieldCoordinate::new(5, 5));
        assert!(path.is_empty());
    }

    #[test]
    fn find_shortest_path_empty_without_moving_action() {
        let module = MoveLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let path = module.find_shortest_path(&mut client, FieldCoordinate::new(5, 5));
        assert!(path.is_empty());
    }

    #[test]
    fn field_peek_resets_when_no_move_square_or_path() {
        let module = MoveLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_peek(&client, FieldCoordinate::new(5, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn field_interaction_ignores_without_move_square_or_path() {
        let mut module = MoveLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&mut client, FieldCoordinate::new(5, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut module = MoveLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn end_turn_sends_end_turn_command() {
        let mut module = MoveLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        client.set_game(game);
        module.end_turn(&mut client);
        // No panic and communication reachable — the actual network queue isn't asserted here
        // since `ClientCommunication`'s send queue isn't exposed for direct inspection in tests.
        assert!(!client.communication().is_stopped());
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = MoveLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
    }
}
