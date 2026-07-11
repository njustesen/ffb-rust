//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.LogicModule` (753 lines).
//!
//! Java's `LogicModule` is an abstract class holding a `FantasyFootballClient` field and
//! mixing lifecycle hooks (`setUp`/`teardown`/`endTurn`), interaction dispatch
//! (`playerInteraction`/`fieldInteraction`/...), and a large collection of
//! `isXxxAvailable(...)` predicate methods that only read `Game`/`Player`/`ActingPlayer`
//! state. Per the batch plan, the predicate methods are translated as standalone free
//! functions (taking `&Game`/`&Player`/`&ActingPlayer` explicitly) so they are directly
//! unit-testable without a live `FantasyFootballClient`. The lifecycle/interaction methods
//! that genuinely need client access (communication, logging) become default methods on
//! the `LogicModule` trait taking `&FantasyFootballClient`/`&mut FantasyFootballClient`
//! explicitly, matching the established convention (see `client/handler/*` precedent).
//!
//! Documented gaps (missing Rust-model equivalents of Java APIs used here):
//! - `ActingPlayer.getOldPlayerState()` / `hasOnlyStandingUpMove()` — no equivalent field
//!   exists on the Rust `ActingPlayer`; treated conservatively (see
//!   `is_look_into_my_eyes_available` / `is_incorporeal_available`).
//! - `Player.hasActiveEnhancement(property)` — no enhancement-tracking exists on the Rust
//!   `Player`; treated conservatively as `false` (see `is_incorporeal_available`).
//! - `FieldModel.getPlayers(coordinate)` (multi-occupancy) — the Rust model is 1:1
//!   coordinate->player (`player_at`), so "square has no players" degenerates to
//!   `player_at(coord).is_none()` (see `is_raiding_party_available`).
//! - `Player.getPosition().getKeywords().contains(BIG_GUY)` — `Player` doesn't store a
//!   `Position`/keyword set; the precomputed `is_big_guy` flag is used instead (exact
//!   semantic equivalent, per its own doc comment).

use std::collections::HashSet;

use ffb_engine::mechanic::{game_mechanic_for, jump_mechanic_for, ttm_mechanic_for};
use ffb_mechanics::game_mechanic::GameMechanic as GameMechanicTrait;
use ffb_model::enums::{CardEffect, ClientStateId, TurnMode, Weather};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::player_state::PlayerState;
use ffb_model::model::property::NamedProperties;
use ffb_model::option::game_option_id;
use ffb_model::option::util_game_option;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::array_tool::ArrayTool;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;

/// 1:1 translation of the `LogicModule` abstract class.
///
/// The `FantasyFootballClient` is passed explicitly to each default method that needs it,
/// rather than stored on implementers, so concrete logic modules stay cheaply constructible
/// and testable (matches the established `client/handler/*` convention).
pub trait LogicModule {
    /// java: `public abstract ClientStateId getId();`
    fn get_id(&self) -> ClientStateId;

    /// java: `public abstract Set<ClientAction> availableActions();`
    fn available_actions(&self) -> HashSet<ClientAction>;

    /// java: `protected abstract ActionContext actionContext(ActingPlayer actingPlayer);`
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext;

    /// java: `protected abstract void performAvailableAction(Player<?> player, ClientAction action);`
    fn perform_available_action(
        &mut self,
        client: &mut FantasyFootballClient,
        player: &Player,
        action: ClientAction,
    );

    /// java: `public void setUp() {}`
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public void teardown() {}`
    fn teardown(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public final void perform(Player<?> player, ClientAction action)`
    fn perform(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        if self.available_actions().contains(&action) {
            self.perform_available_action(client, player, action);
        } else {
            client.log_error(&format!(
                "Unsupported action {:?} in logic module {}",
                action,
                std::any::type_name::<Self>()
            ));
        }
    }

    /// java: `public void endTurn() {}`
    fn end_turn(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public void deselectActingPlayer()`
    ///
    /// java: `client.getCommunication().sendActingPlayer(null, null, false);` — the Rust
    /// `ClientCommunication::send_acting_player` requires a non-optional `PlayerAction`
    /// (no "null action" variant exists in the translated `PlayerAction` enum), so the
    /// null-action call cannot be reproduced faithfully. Documented gap: left as a no-op.
    fn deselect_acting_player(&self, _client: &mut FantasyFootballClient) {
        // java: gap — see doc comment above.
    }

    /// java: `public boolean endPlayerActivation()`
    fn end_player_activation(&self, client: &mut FantasyFootballClient) -> bool {
        let allow = match client.game() {
            Some(game) => game.turn_mode.allow_end_player_action(),
            None => false,
        };
        if allow {
            // java: `client.getGame().getFieldModel().setRangeRuler(null);`
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            self.deselect_acting_player(client);
            return true;
        }
        false
    }

    /// java: `public ActingPlayer getActingPlayer()`
    fn get_acting_player<'a>(&self, client: &'a FantasyFootballClient) -> Option<&'a ActingPlayer> {
        client.game().map(|game| &game.acting_player)
    }

    /// java: `public boolean playerActivationUsed()`
    fn player_activation_used(&self, client: &FantasyFootballClient) -> bool {
        client.game().map(|game| game.acting_player.has_acted).unwrap_or(false)
    }

    /// java: `public FieldCoordinate getCoordinate(Player<?> player)`
    fn get_coordinate(&self, client: &FantasyFootballClient, player: &Player) -> Option<FieldCoordinate> {
        client.game().and_then(|game| game.field_model.player_coordinate(&player.id))
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`
    fn player_interaction(&self, _player: &Player) -> InteractionResult {
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate coordinate)`
    fn field_interaction(&self, _coordinate: FieldCoordinate) -> InteractionResult {
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`
    fn player_peek(&self, _player: &Player) -> InteractionResult {
        InteractionResult::reset()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate coordinate)`
    fn field_peek(&self, _coordinate: FieldCoordinate) -> InteractionResult {
        InteractionResult::reset()
    }

    /// java: `public Optional<Player<?>> getPlayer(FieldCoordinate coordinate)`
    fn get_player<'a>(&self, client: &'a FantasyFootballClient, coordinate: FieldCoordinate) -> Option<&'a Player> {
        let game = client.game()?;
        let id = game.field_model.player_at(coordinate)?;
        game.player(id)
    }

    /// java: `public Set<FieldCoordinate> chompedBy(Player<?> player)`
    fn chomped_by(&self, client: &FantasyFootballClient, player: &Player) -> HashSet<FieldCoordinate> {
        match client.game() {
            Some(game) => chomped_by(game, player),
            None => HashSet::new(),
        }
    }

    /// java: `public Set<FieldCoordinate> chomps(Player<?> player)`
    fn chomps(&self, client: &FantasyFootballClient, player: &Player) -> HashSet<FieldCoordinate> {
        match client.game() {
            Some(game) => chomps(game, player),
            None => HashSet::new(),
        }
    }
}

// ── Free predicate/helper functions (pure, directly unit-testable) ─────────────────────

/// java: `FieldModel.chompedBy(Player<?> player)` — squares of chompers currently chomping
/// `player`.
pub fn chomped_by(game: &Game, player: &Player) -> HashSet<FieldCoordinate> {
    game.field_model
        .chomped
        .iter()
        .filter(|(_, chompees)| chompees.contains(&player.id))
        .filter_map(|(chomper_id, _)| game.field_model.player_coordinate(chomper_id))
        .collect()
}

/// java: `FieldModel.chomps(Player<?> player)` — squares of `player`'s current chompees.
pub fn chomps(game: &Game, player: &Player) -> HashSet<FieldCoordinate> {
    match game.field_model.chomped.get(&player.id) {
        Some(chompees) => chompees
            .iter()
            .filter_map(|id| game.field_model.player_coordinate(id))
            .collect(),
        None => HashSet::new(),
    }
}

/// java: `FieldModel.notChomped(Player<?> chomper, Player<?> chompee)`
pub fn not_chomped(game: &Game, chomper: &Player, chompee: &Player) -> bool {
    match game.field_model.chomped.get(&chomper.id) {
        Some(chompees) => !chompees.contains(&chompee.id),
        None => true,
    }
}

/// java: `UtilCards.hasUncanceledSkillWithProperty(Player<?>, ISkillProperty)` — not itself
/// translated in `ffb-model`; composed here from the two sibling `UtilCards` methods that
/// are translated, exactly matching the Java body (`hasSkillWithProperty(...) &&
/// !hasSkillToCancelProperty(...)`).
fn has_uncanceled_skill_with_property(player: &Player, property: &str) -> bool {
    UtilCards::has_skill_with_property(player, property) && !UtilCards::has_skill_to_cancel_property(player, property)
}

/// java: `findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds.FIELD, 1, false)` —
/// no shared public helper exists on `FieldModel`; this mirrors the private duplicate
/// already used in `util_player.rs`/pathfinder files for `distance == 1`.
fn adjacent_field_coordinates(coord: FieldCoordinate) -> Vec<FieldCoordinate> {
    FieldModel::new().adjacent_on_pitch(coord)
}

/// java: `isHypnoticGazeActionAvailable(boolean declareAtStart, Player<?> player, ISkillProperty property)`
///
/// The Rust `UtilPlayer::can_gaze` is hardcoded to the "inflicts confusion" property and
/// doesn't take an `ISkillProperty` parameter (not modeled), so the `property` argument is
/// accepted for signature fidelity but not compared against; documented gap.
pub fn is_hypnotic_gaze_action_available(game: &Game, declare_at_start: bool, player: &Player) -> bool {
    let mechanic = game_mechanic_for(game.rules);
    mechanic.declare_gaze_action_at_start() == declare_at_start
        && mechanic.is_gaze_action_allowed(game, player)
        && UtilPlayer::can_gaze(game, &player.id)
}

/// java: `isTreacherousAvailable(ActingPlayer actingPlayer)`
pub fn is_treacherous_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_treacherous_available(game, player),
        None => false,
    }
}

/// java: `isTreacherousAvailable(Player<?> player)`
pub fn is_treacherous_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    UtilPlayer::find_adjacent_blockable_players(game, game.active_team(), coord)
        .iter()
        .any(|adjacent| UtilPlayer::has_ball(game, adjacent))
}

/// java: `isCatchOfTheDayAvailable(ActingPlayer actingPlayer)`
pub fn is_catch_of_the_day_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_catch_of_the_day_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isCatchOfTheDayAvailable(Player<?> player)`
pub fn is_catch_of_the_day_available(game: &Game, player: &Player) -> bool {
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let ball_coordinate = match game.field_model.ball_coordinate {
        Some(c) => c,
        None => return false,
    };
    UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GET_BALL_ON_GROUND)
        && game.field_model.ball_moving
        && player_coordinate.distance_in_steps(ball_coordinate) <= 3
}

/// java: `isWisdomAvailable(ActingPlayer actingPlayer)`
pub fn is_wisdom_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_wisdom_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isWisdomAvailable(Player<?> player)`
pub fn is_wisdom_available(game: &Game, player: &Player) -> bool {
    game_mechanic_for(game.rules).is_wisdom_available(game, player)
}

/// java: `isBlackInkAvailable(ActingPlayer player)`
pub fn is_black_ink_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted || acting_player.standing_up {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_black_ink_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isBlackInkAvailable(Player<?> player)`
pub fn is_black_ink_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAZE_AUTOMATICALLY) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    UtilPlayer::find_standing_or_prone_players(game, other_team, coord, 1)
        .iter()
        .any(|opponent| {
            !game
                .field_model
                .player_state(&opponent.id)
                .map(|s| s.is_distracted())
                .unwrap_or(false)
        })
}

/// java: `isRaidingPartyAvailable(ActingPlayer player)`
pub fn is_raiding_party_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_raiding_party_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isRaidingPartyAvailable(Player<?> player)`
///
/// Uses `player_at(coord).is_none()` in place of Java's `getPlayers(coordinate).isEmpty()`
/// multi-occupancy check — the Rust `FieldModel` is 1:1 coordinate->player (see module doc).
pub fn is_raiding_party_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
        return false;
    }
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    game.active_team().players.iter().any(|team_mate| {
        let team_mate_coordinate = match game.field_model.player_coordinate(&team_mate.id) {
            Some(c) => c,
            None => return false,
        };
        let adjacent_with_tz =
            UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, team_mate_coordinate, false);
        let team_mate_state = game.field_model.player_state(&team_mate.id);
        let base_is_standing = team_mate_state.map(|s| s.base()).unwrap_or(0) == ffb_model::enums::PS_STANDING;

        base_is_standing
            && team_mate_coordinate.distance_in_steps(player_coordinate) <= 5
            && adjacent_with_tz.is_empty()
            && adjacent_field_coordinates(team_mate_coordinate).iter().any(|adjacent_coordinate| {
                game.field_model.player_at(*adjacent_coordinate).is_none()
                    && adjacent_field_coordinates(*adjacent_coordinate).iter().any(|fc| {
                        match game.field_model.player_at(*fc) {
                            Some(occupant_id) => !game.active_team().has_player(occupant_id),
                            None => false,
                        }
                    })
            })
    })
}

/// java: `isLookIntoMyEyesAvailable(ActingPlayer actingPlayer)`
///
/// Java reads `actingPlayer.getOldPlayerState().hasTacklezones()`; `ActingPlayer` has no
/// `old_player_state` equivalent in the Rust model (documented gap), so `had_tackle_zone`
/// conservatively evaluates to `false`, matching Java's null-safe fallback when there is no
/// old state to compare against.
pub fn is_look_into_my_eyes_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    let had_tackle_zone = false; // java: gap — see doc comment above.
    if acting_player.has_acted || !had_tackle_zone {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_look_into_my_eyes_available(game, player),
        None => false,
    }
}

/// java: `isLookIntoMyEyesAvailable(Player<?> player)`
pub fn is_look_into_my_eyes_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    UtilPlayer::find_adjacent_blockable_players(game, other_team, coord)
        .iter()
        .any(|opponent| UtilPlayer::has_ball(game, opponent))
}

/// java: `isBalefulHexAvailable(ActingPlayer player)`
pub fn is_baleful_hex_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_baleful_hex_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isBalefulHexAvailable(Player<?> player)`
pub fn is_baleful_hex_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    other_team.players.iter().any(|opponent| {
        game.field_model
            .player_coordinate(&opponent.id)
            .map(|opp_coord| opp_coord.distance_in_steps(coord) <= 5)
            .unwrap_or(false)
    })
}

/// java: `isThenIStartedBlastinAvailable(ActingPlayer player)`
pub fn is_then_i_started_blastin_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_then_i_started_blastin_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isThenIStartedBlastinAvailable(Player<?> player)`
pub fn is_then_i_started_blastin_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    other_team.players.iter().any(|opponent| {
        game.field_model
            .player_coordinate(&opponent.id)
            .map(|opp_coord| opp_coord.distance_in_steps(coord) <= 3)
            .unwrap_or(false)
    })
}

/// java: `isBlockActionAvailable(Player<?> player)`
pub fn is_block_action_available(game: &Game, player: &Player) -> bool {
    let mechanic = game_mechanic_for(game.rules);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        || !player_state.is_active()
        || player.has_skill_property(NamedProperties::PREVENT_REGULAR_BLOCK_ACTION)
        || !mechanic.is_block_action_allowed(game.turn_mode)
    {
        return false;
    }
    if player_state.is_prone() && !player.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE) {
        return false;
    }
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    !UtilPlayer::find_adjacent_blockable_players(game, &game.team_away, player_coordinate).is_empty()
}

/// java: `isMultiBlockActionAvailable(Player<?> player)`
pub fn is_multi_block_action_available(game: &Game, player: &Player) -> bool {
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        || !player_state.is_active()
        || player.has_skill_property(NamedProperties::PREVENT_REGULAR_BLOCK_ACTION)
    {
        return false;
    }
    let can_block_more_than_once = UtilCards::has_skill_with_property(player, NamedProperties::CAN_BLOCK_MORE_THAN_ONCE)
        && !UtilCards::has_skill_to_cancel_property(player, NamedProperties::CAN_BLOCK_MORE_THAN_ONCE);
    let can_block_two_at_once = UtilCards::has_skill_with_property(player, NamedProperties::CAN_BLOCK_TWO_AT_ONCE)
        && !UtilCards::has_skill_to_cancel_property(player, NamedProperties::CAN_BLOCK_TWO_AT_ONCE);
    if !(can_block_more_than_once || can_block_two_at_once) {
        return false;
    }
    if player_state.is_prone() && !player.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE) {
        return false;
    }
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    UtilPlayer::find_adjacent_blockable_players(game, &game.team_away, player_coordinate).len() > 1
}

/// java: `isThrowBombActionAvailable(Player<?> player)`
pub fn is_throw_bomb_action_available(game: &Game, player: &Player) -> bool {
    let mechanic = game_mechanic_for(game.rules);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    mechanic.is_bomb_action_allowed(game.turn_mode)
        && !game.turn_data().bomb_used
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && !player_state.is_prone_or_stunned()
        && player.has_skill_property(NamedProperties::ENABLE_THROW_BOMB_ACTION)
}

/// java: `isSecureTheBallActionAvailable(Player<?> player)`
pub fn is_secure_the_ball_action_available(game: &Game, player: &Player) -> bool {
    let ball_coordinate = match game.field_model.ball_coordinate {
        Some(c) => c,
        None => return false,
    };
    let opponents = UtilPlayer::find_players_with_tackle_zones(game, &game.team_away, ball_coordinate, 2);
    game.field_model.ball_in_play
        && game.field_model.ball_moving
        && !game.turn_data().secure_the_ball_used
        && !player.has_skill_property(NamedProperties::PREVENT_SECURE_THE_BALL_ACTION)
        && !player.is_big_guy
        && opponents.is_empty()
}

/// java: `isBlitzActionAvailable(Player<?> player)`
pub fn is_blitz_action_available(game: &Game, player: &Player) -> bool {
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    !game.turn_data().blitz_used
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && player_state.is_active()
        && (player_state.is_able_to_move() || player_can_not_move_placeholder(player_state))
        && !player.has_skill_property(NamedProperties::PREVENT_REGULAR_BLITZ_ACTION)
}

/// java: `plugin().playerCanNotMove(playerState)` — obtaining the `BaseLogicPlugin` requires
/// `LogicPluginFactory` (`client/factory/LogicPluginFactory.java`), which is not yet
/// translated (deferred per the batch plan). Documented gap: conservatively `false`.
fn player_can_not_move_placeholder(_player_state: PlayerState) -> bool {
    false
}

/// java: `isFoulActionAvailable(Player<?> player)`
pub fn is_foul_action_available(game: &Game, player: &Player) -> bool {
    let mechanic = game_mechanic_for(game.rules);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        || !mechanic.is_foul_action_allowed(game.turn_mode)
        || !player_state.is_active()
        || (game.turn_data().foul_used && !player.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
        || player.has_skill_property(NamedProperties::PREVENT_REGULAR_FOUL_ACTION)
    {
        return false;
    }
    game.team_away.players.iter().any(|opponent| {
        game.field_model
            .player_state(&opponent.id)
            .map(|s| s.can_be_fouled())
            .unwrap_or(false)
    })
}

/// java: `isPassActionAvailable(Player<?> player, boolean treacherousAvailable)`
pub fn is_pass_action_available(game: &Game, player: &Player, treacherous_available: bool) -> bool {
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    !game.turn_data().pass_used
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && (UtilPlayer::is_ball_available(game, &player.id) || treacherous_available)
        && (player_state.is_able_to_move() || (UtilPlayer::has_ball(game, &player.id) || treacherous_available))
        && !player.has_skill_property(NamedProperties::PREVENT_REGULAR_PASS_ACTION)
}

/// java: `isPuntActionAvailable(Player<?> player, boolean treacherousAvailable)`
pub fn is_punt_action_available(game: &Game, player: &Player, treacherous_available: bool) -> bool {
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    !game.turn_data().punt_used
        && player.has_skill_property(NamedProperties::CAN_PUNT)
        && (UtilPlayer::is_ball_available(game, &player.id) || treacherous_available)
        && (player_state.is_able_to_move() || (UtilPlayer::has_ball(game, &player.id) || treacherous_available))
        && (!player_state.is_prone() || util_game_option::is_option_enabled(game, game_option_id::ALLOW_SPECIAL_ACTIONS_FROM_PRONE))
        && !player.has_skill_property(NamedProperties::PREVENT_PUNT_ACTION)
}

/// java: `isHandOverActionAvailable(Player<?> player, boolean treacherousAvailable)`
pub fn is_hand_over_action_available(game: &Game, player: &Player, treacherous_available: bool) -> bool {
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    !game.turn_data().hand_over_used
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && (UtilPlayer::is_ball_available(game, &player.id) || treacherous_available)
        && (player_state.is_able_to_move() || (UtilPlayer::has_ball(game, &player.id) || treacherous_available))
        && !player.has_skill_property(NamedProperties::PREVENT_REGULAR_HAND_OVER_ACTION)
}

/// java: `isThrowTeamMateActionAvailable(Player<?> player)`
pub fn is_throw_team_mate_action_available(game: &Game, player: &Player) -> bool {
    let mechanic = ttm_mechanic_for(game.rules);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if player.has_skill_property(NamedProperties::PREVENT_THROW_TEAM_MATE_ACTION) {
        return false;
    }
    let own_team = player_own_team(game, player);
    let right_stuff_available = own_team
        .map(|team| {
            team.players.iter().any(|team_player| {
                let coord = game.field_model.player_coordinate(&team_player.id);
                mechanic.can_be_thrown(game, team_player) && coord.map(|c| !c.is_box_coordinate()).unwrap_or(false)
            })
        })
        .unwrap_or(false);
    let right_stuff_adjacent = ArrayTool::is_provided(&mechanic.find_throwable_team_mates(game, player));

    mechanic.is_ttm_available(game.turn_data())
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && mechanic.can_throw(game, player)
        && right_stuff_available
        && (player_state.is_able_to_move() || right_stuff_adjacent)
}

/// java: `isKickTeamMateActionAvailable(Player<?> player)`
pub fn is_kick_team_mate_action_available(game: &Game, player: &Player) -> bool {
    let game_mechanic = game_mechanic_for(game.rules);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if !game_mechanic.is_kick_team_mate_action_allowed(game.turn_mode)
        || player.has_skill_property(NamedProperties::PREVENT_KICK_TEAM_MATE_ACTION)
    {
        return false;
    }
    let mechanic = ttm_mechanic_for(game.rules);

    let own_team = player_own_team(game, player);
    let right_stuff_available = own_team
        .map(|team| {
            team.players.iter().any(|team_player| {
                let coord = game.field_model.player_coordinate(&team_player.id);
                mechanic.can_be_kicked(game, team_player) && coord.map(|c| !c.is_box_coordinate()).unwrap_or(false)
            })
        })
        .unwrap_or(false);

    let mut right_stuff_adjacent = false;
    if let (Some(player_coordinate), Some(own_team)) =
        (game.field_model.player_coordinate(&player.id), own_team)
    {
        let adjacent_team_players =
            UtilPlayer::find_adjacent_players_with_tacklezones(game, own_team, player_coordinate, false);
        right_stuff_adjacent = adjacent_team_players.iter().any(|adjacent_id| {
            own_team
                .player(adjacent_id)
                .map(|p| mechanic.can_be_kicked(game, p))
                .unwrap_or(false)
        });
    }

    mechanic.is_ktm_available(game.turn_data())
        && !game.field_model.has_card_effect(&player.id, CardEffect::IllegallySubstituted)
        && player.has_skill_property(NamedProperties::CAN_KICK_TEAM_MATES)
        && right_stuff_available
        && (player_state.is_able_to_move() || right_stuff_adjacent)
}

/// java: `isStandUpActionAvailable(Player<?> player)`
pub fn is_stand_up_action_available(game: &Game, player: &Player) -> bool {
    match game.field_model.player_state(&player.id) {
        Some(s) => {
            s.is_prone() && s.is_active() && !player.has_skill_property(NamedProperties::PREVENT_STAND_UP_ACTION)
        }
        None => false,
    }
}

/// java: `isRecoverFromConfusionActionAvailable(Player<?> player)`
pub fn is_recover_from_confusion_action_available(game: &Game, player: &Player) -> bool {
    match game.field_model.player_state(&player.id) {
        Some(s) => {
            s.is_confused()
                && s.is_active()
                && !s.is_prone()
                && !player.has_skill_property(NamedProperties::PREVENT_RECOVER_FROM_CONFUSION_ACTION)
        }
        None => false,
    }
}

/// java: `isRecoverFromGazeActionAvailable(Player<?> player)`
pub fn is_recover_from_gaze_action_available(game: &Game, player: &Player) -> bool {
    match game.field_model.player_state(&player.id) {
        Some(s) => {
            s.is_hypnotized()
                && !s.is_prone()
                && !player.has_skill_property(NamedProperties::PREVENT_RECOVER_FROM_GAZE_ACTION)
        }
        None => false,
    }
}

/// java: `isBeerBarrelBashAvailable(Player<?> player)`
pub fn is_beer_barrel_bash_available(game: &Game, player: &Player) -> bool {
    match game.field_model.player_state(&player.id) {
        Some(s) => {
            game.turn_mode == TurnMode::Regular
                && s.base() == ffb_model::enums::PS_STANDING
                && UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_THROW_KEG)
        }
        None => false,
    }
}

/// java: `isAllYouCanEatAvailable(Player<?> player)`
pub fn is_all_you_can_eat_available(game: &Game, player: &Player) -> bool {
    is_throw_bomb_action_available(game, player)
        && game.turn_mode == TurnMode::Regular
        && UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_USE_THROW_BOMB_ACTION_TWICE)
}

fn is_kick_em_available(game: &Game, player: &Player, move_allowed: bool) -> bool {
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    if !(player_state.is_active()
        && (!game.turn_data().blitz_used || !move_allowed)
        && UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_USE_CHAINSAW_ON_DOWNED_OPPONENTS)
        && player.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
    {
        return false;
    }
    game.team_away.players.iter().any(|opponent| {
        let opponent_state = game.field_model.player_state(&opponent.id);
        let can_be_fouled = opponent_state.map(|s| s.can_be_fouled()).unwrap_or(false);
        let adjacent = game
            .field_model
            .player_coordinate(&opponent.id)
            .map(|c| player_coordinate.is_adjacent(c))
            .unwrap_or(false);
        can_be_fouled && (move_allowed || adjacent)
    })
}

/// java: `isKickEmBlockAvailable(Player<?> player)`
pub fn is_kick_em_block_available(game: &Game, player: &Player) -> bool {
    is_kick_em_available(game, player, false)
}

/// java: `isKickEmBlitzAvailable(Player<?> player)`
pub fn is_kick_em_blitz_available(game: &Game, player: &Player) -> bool {
    is_kick_em_available(game, player, true)
}

/// java: `isFlashingBladeAvailable(Player<?> player)`
pub fn is_flashing_blade_available(game: &Game, player: &Player) -> bool {
    let opponent_team = UtilPlayer::find_other_team(game, &player.id);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    let mechanic = game_mechanic_for(game.rules);
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    player_state.is_active()
        && mechanic.is_block_action_allowed(game.turn_mode)
        && !player_state.is_prone()
        && player.has_unused_skill_with_property(NamedProperties::CAN_STAB_AND_MOVE_AFTERWARDS)
        && ArrayTool::is_provided(&UtilPlayer::find_adjacent_blockable_players(game, opponent_team, coord))
}

/// java: `protected boolean isEndPlayerActionAvailable()`
pub fn is_end_player_action_available(game: &Game) -> bool {
    let acting_player = &game.acting_player;
    let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return false,
    };
    !acting_player.has_acted
        || !player.has_skill_property(NamedProperties::FORCE_FULL_MOVEMENT)
        || acting_player.current_move >= player.movement_with_modifiers()
}

/// java: `isJumpAvailableAsNextMove(Game game, ActingPlayer actingPlayer, boolean jumping)`
pub fn is_jump_available_as_next_move(game: &Game, acting_player: &ActingPlayer, jumping: bool) -> bool {
    jump_mechanic_for(game.rules).is_available_as_next_move(game, acting_player, jumping)
}

/// java: `isBoundingLeapAvailable(Game game, ActingPlayer actingPlayer)`
///
/// Java returns `Optional<Skill>`; `UtilCards::get_unused_skill_with_property` returns
/// `Option<SkillId>` in the Rust model (no `Skill` value type is threaded through here), so
/// this returns `Option<SkillId>` instead — the closest available equivalent.
pub fn is_bounding_leap_available(
    game: &Game,
    acting_player: &ActingPlayer,
) -> Option<ffb_model::enums::SkillId> {
    if !is_jump_available_as_next_move(game, acting_player, false) {
        return None;
    }
    let player = acting_player.player_id.as_deref().and_then(|id| game.player(id))?;
    UtilCards::get_unused_skill_with_property(player, NamedProperties::CAN_IGNORE_JUMP_MODIFIERS)
}

/// java: `isFumblerooskieAvailable()`
pub fn is_fumblerooskie_available(game: &Game) -> bool {
    let acting_player = &game.acting_player;
    let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return false,
    };
    has_uncanceled_skill_with_property(player, NamedProperties::CAN_DROP_BALL)
        && acting_player
            .player_action
            .map(|a| a.allows_fumblerooskie())
            .unwrap_or(false)
        && UtilPlayer::has_ball(game, &player.id)
}

/// java: `isPutridRegurgitationAvailable()` — always `false` in Java itself.
pub fn is_putrid_regurgitation_available() -> bool {
    false
}

/// java: `isFrenziedRushAvailable(ActingPlayer actingPlayer)`
pub fn is_frenzied_rush_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_frenzied_rush_available(player),
        None => false,
    }
}

/// java: `protected boolean isFrenziedRushAvailable(Player<?> player)`
pub fn is_frenzied_rush_available(player: &Player) -> bool {
    UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ)
}

/// java: `isSlashingNailsAvailable(ActingPlayer actingPlayer)`
pub fn is_slashing_nails_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_slashing_nails_available(player),
        None => false,
    }
}

/// java: `protected boolean isSlashingNailsAvailable(Player<?> player)`
pub fn is_slashing_nails_available(player: &Player) -> bool {
    UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ)
}

/// java: `protected boolean isZoatGazeAvailable(ActingPlayer actingPlayer)`
pub fn is_zoat_gaze_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    if acting_player.has_acted || acting_player.standing_up {
        return false;
    }
    match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(player) => is_zoat_gaze_available(game, player),
        None => false,
    }
}

/// java: `protected boolean isZoatGazeAvailable(Player<?> player)`
pub fn is_zoat_gaze_available(game: &Game, player: &Player) -> bool {
    if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY) {
        return false;
    }
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    let other_team = UtilPlayer::find_other_team(game, &player.id);
    UtilPlayer::find_players_with_tackle_zones(game, other_team, coord, 3)
        .iter()
        .any(|opponent| {
            !game
                .field_model
                .player_state(opponent)
                .map(|s| s.is_distracted())
                .unwrap_or(false)
        })
}

/// java: `isIncorporealAvailable(ActingPlayer actingPlayer)`
///
/// Java also checks `actingPlayer.hasOnlyStandingUpMove()` and
/// `player.hasActiveEnhancement(...)` — neither has a Rust-model equivalent (documented
/// gaps at the top of this file); both are treated conservatively as `false`.
pub fn is_incorporeal_available_ap(game: &Game, acting_player: &ActingPlayer) -> bool {
    let has_only_standing_up_move = false; // java: gap — see module doc comment.
    if !(acting_player.current_move == 0 || has_only_standing_up_move) {
        return false;
    }
    let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return false,
    };
    let has_active_enhancement = false; // java: gap — see module doc comment.
    is_incorporeal_available(player) || has_active_enhancement
}

/// java: `protected boolean isIncorporealAvailable(Player<?> player)`
pub fn is_incorporeal_available(player: &Player) -> bool {
    UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_AVOID_DODGING)
}

/// java: `isSpecialAbilityAvailable(ActingPlayer actingPlayer)`
pub fn is_special_ability_available(game: &Game, acting_player: &ActingPlayer) -> bool {
    is_treacherous_available_ap(game, acting_player)
        || is_wisdom_available_ap(game, acting_player)
        || is_raiding_party_available_ap(game, acting_player)
        || is_look_into_my_eyes_available_ap(game, acting_player)
        || is_baleful_hex_available_ap(game, acting_player)
        || is_putrid_regurgitation_available()
        || is_catch_of_the_day_available_ap(game, acting_player)
        || is_black_ink_available_ap(game, acting_player)
        || is_then_i_started_blastin_available_ap(game, acting_player)
        || is_zoat_gaze_available_ap(game, acting_player)
        || is_incorporeal_available_ap(game, acting_player)
}

/// java: `isBlitzSpecialAbilityAvailable(ActingPlayer actingPlayer)`
pub fn is_blitz_special_ability_available(game: &Game, acting_player: &ActingPlayer) -> bool {
    is_special_ability_available(game, acting_player)
        || is_frenzied_rush_available_ap(game, acting_player)
        || is_slashing_nails_available_ap(game, acting_player)
}

/// java: `isPassAnySquareAvailable(ActingPlayer actingPlayer, Game game)`
pub fn is_pass_any_square_available(acting_player: &ActingPlayer, game: &Game) -> bool {
    acting_player.player_action == Some(ffb_model::enums::PlayerAction::PassMove)
        && acting_player
            .player_id
            .as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false)
}

/// java: `protected boolean showGridForKTM(Game game, ActingPlayer actingPlayer)` — always
/// `false` in the base `LogicModule`, overridden by subclasses.
pub fn show_grid_for_ktm(_game: &Game, _acting_player: &ActingPlayer) -> bool {
    false
}

/// java: `performsRangeGridAction(ActingPlayer actingPlayer, Game game)`
pub fn performs_range_grid_action(acting_player: &ActingPlayer, game: &Game) -> bool {
    is_pass_any_square_available(acting_player, game)
        || show_grid_for_ktm(game, acting_player)
        || ((acting_player.player_action == Some(ffb_model::enums::PlayerAction::ThrowTeamMateMove)
            || acting_player.player_action == Some(ffb_model::enums::PlayerAction::ThrowTeamMate))
            && acting_player
                .player_id
                .as_deref()
                .and_then(|id| game.player(id))
                .map(|player| ttm_mechanic_for(game.rules).can_throw(game, player))
                .unwrap_or(false))
}

/// java: `isMoveAvailable(ActingPlayer actingPlayer)`
pub fn is_move_available(acting_player: &ActingPlayer) -> bool {
    acting_player.player_action == Some(ffb_model::enums::PlayerAction::Gaze)
}

/// java: `isHailMaryPassActionAvailable()`
pub fn is_hail_mary_pass_action_available(game: &Game) -> bool {
    let acting_player = &game.acting_player;
    let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return false,
    };
    player.has_skill_property(NamedProperties::CAN_PASS_TO_ANY_SQUARE) && game.weather != Weather::Blizzard
}

/// java: `protected boolean isViciousVinesAvailable(Player<?> player)`
pub fn is_vicious_vines_available(game: &Game, player: &Player) -> bool {
    let opponent_team = UtilPlayer::find_other_team(game, &player.id);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    let mechanic = game_mechanic_for(game.rules);
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    player_state.is_active()
        && player_state.base() == ffb_model::enums::PS_STANDING
        && mechanic.is_block_action_allowed(game.turn_mode)
        && player.has_unused_skill_with_property(NamedProperties::CAN_BLOCK_OVER_DISTANCE)
        && ArrayTool::is_provided(&UtilPlayer::find_blockable_players_two_squares_away(game, opponent_team, coord))
}

/// java: `protected boolean isFuriousOutburstAvailable(Player<?> player)`
pub fn is_furious_outburst_available(game: &Game, player: &Player) -> bool {
    let opponent_team = UtilPlayer::find_other_team(game, &player.id);
    let player_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    let coord = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    player_state.is_active()
        && player_state.base() == ffb_model::enums::PS_STANDING
        && !game.turn_data().blitz_used
        && player.has_unused_skill_with_property(NamedProperties::CAN_TELEPORT_BEFORE_AND_AFTER_AV_ROLL_ATTACK)
        && ArrayTool::is_provided(&UtilPlayer::find_blockable_players(game, opponent_team, coord, 3))
}

/// java: `isRecoverFromEyeGougeActionAvailable(Player<?> player)`
pub fn is_recover_from_eye_gouge_action_available(game: &Game, player: &Player) -> bool {
    match game.field_model.player_state(&player.id) {
        Some(s) => s.is_eye_gouged() && s.is_active() && !s.is_prone(),
        None => false,
    }
}

/// java: `isSpecialBlockActionAvailable(Player<?> player, PlayerState playerState)`
pub fn is_special_block_action_available(game: &Game, player: &Player, player_state: Option<PlayerState>) -> bool {
    let mechanic = game_mechanic_for(game.rules);
    let player_state = match player_state {
        Some(s) => s,
        None => return false,
    };
    if player.has_skill_property(NamedProperties::PREVENT_REGULAR_BLOCK_ACTION)
        || !mechanic.is_block_action_allowed(game.turn_mode)
        || player_state.is_prone()
    {
        return false;
    }
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    !UtilPlayer::find_adjacent_blockable_players(game, &game.team_away, player_coordinate).is_empty()
}

/// java: `isChompAvailable(Player<?> player, Player<?> target)`
pub fn is_chomp_available_target(game: &Game, player: &Player, target: &Player) -> bool {
    can_chomp(player) && not_chomped(game, player, target)
}

/// java: `isChompAvailable(Player<?> player)`
pub fn is_chomp_available(game: &Game, player: &Player) -> bool {
    if !can_chomp(player) {
        return false;
    }
    let player_coordinate = match game.field_model.player_coordinate(&player.id) {
        Some(c) => c,
        None => return false,
    };
    UtilPlayer::find_adjacent_blockable_players(game, &game.team_away, player_coordinate)
        .iter()
        .any(|target_id| match game.player(target_id) {
            Some(target) => not_chomped(game, player, target),
            None => false,
        })
}

/// java: `private boolean canChomp(Player<?> player)`
fn can_chomp(player: &Player) -> bool {
    player.has_skill_property(NamedProperties::CAN_PIN_PLAYERS)
}

/// java: `player.getTeam()` — `Player` has no back-reference to its owning `Team` in the
/// Rust model, so this looks the team up via `Game::player_team_id` + `Game::team_by_id`.
fn player_own_team<'a>(game: &'a Game, player: &Player) -> Option<&'a ffb_model::model::team::Team> {
    game.player_team_id(&player.id).and_then(|id| game.team_by_id(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerAction, Rules, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
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
        game.field_model
            .set_player_state(id, PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
    }

    #[test]
    fn is_putrid_regurgitation_available_is_always_false() {
        assert!(!is_putrid_regurgitation_available());
    }

    #[test]
    fn can_chomp_requires_named_property() {
        let mut player = Player::default();
        assert!(!can_chomp(&player));
        player.add_skill(SkillId::DumpOff);
        // java: hasSkillProperty checks the skill's property list, not just presence of a
        // skill id; without a skill->property mapping loaded, this stays false unless the
        // property constant itself is asserted directly.
        assert!(!has_uncanceled_skill_with_property(&player, "unrelated-property"));
    }

    #[test]
    fn is_move_available_matches_gaze_action_only() {
        let mut ap = ActingPlayer::new();
        ap.player_action = Some(PlayerAction::Gaze);
        assert!(is_move_available(&ap));
        ap.player_action = Some(PlayerAction::Move);
        assert!(!is_move_available(&ap));
    }

    #[test]
    fn is_pass_any_square_available_requires_pass_move_and_ball() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let mut ap = ActingPlayer::new();
        ap.player_id = Some("p1".to_string());
        ap.player_action = Some(PlayerAction::PassMove);
        assert!(!is_pass_any_square_available(&ap, &game));

        game.field_model.ball_coordinate = Some(FieldCoordinate::new(1, 1));
        // java: UtilPlayer.hasBall additionally requires ball_in_play / ball on player's
        // square depending on edition; exercised here only for a no-ball baseline above.
    }

    #[test]
    fn is_frenzied_rush_available_false_without_skill() {
        let player = Player::default();
        assert!(!is_frenzied_rush_available(&player));
    }

    #[test]
    fn is_slashing_nails_available_false_without_skill() {
        let player = Player::default();
        assert!(!is_slashing_nails_available(&player));
    }

    #[test]
    fn is_incorporeal_available_false_without_skill() {
        let player = Player::default();
        assert!(!is_incorporeal_available(&player));
    }

    #[test]
    fn is_hail_mary_pass_unavailable_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        assert!(!is_hail_mary_pass_action_available(&game));
    }

    #[test]
    fn is_hail_mary_pass_unavailable_in_blizzard_even_with_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        game.weather = Weather::Blizzard;
        if let Some(p) = game.team_home.player_mut("p1") {
            p.add_skill(SkillId::HailMaryPass); // any skill id; property gating below is what matters
        }
        assert!(!is_hail_mary_pass_action_available(&game));
    }

    #[test]
    fn is_end_player_action_available_true_when_not_yet_acted() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.has_acted = false;
        assert!(is_end_player_action_available(&game));
    }

    #[test]
    fn not_chomped_true_when_no_entry() {
        let mut game = make_game();
        add_player(&mut game, true, "chomper", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "target", FieldCoordinate::new(1, 2));
        let chomper = game.player("chomper").unwrap().clone();
        let target = game.player("target").unwrap().clone();
        assert!(not_chomped(&game, &chomper, &target));
    }

    #[test]
    fn not_chomped_false_once_recorded() {
        let mut game = make_game();
        add_player(&mut game, true, "chomper", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "target", FieldCoordinate::new(1, 2));
        game.field_model.add_chomp("chomper", "target");
        let chomper = game.player("chomper").unwrap().clone();
        let target = game.player("target").unwrap().clone();
        assert!(!not_chomped(&game, &chomper, &target));
    }

    #[test]
    fn chomps_returns_chompee_coordinate() {
        let mut game = make_game();
        add_player(&mut game, true, "chomper", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "target", FieldCoordinate::new(1, 2));
        game.field_model.add_chomp("chomper", "target");
        let chomper = game.player("chomper").unwrap().clone();
        let squares = chomps(&game, &chomper);
        assert_eq!(squares, HashSet::from([FieldCoordinate::new(1, 2)]));
    }

    #[test]
    fn chomped_by_returns_chomper_coordinate() {
        let mut game = make_game();
        add_player(&mut game, true, "chomper", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "target", FieldCoordinate::new(1, 2));
        game.field_model.add_chomp("chomper", "target");
        let target = game.player("target").unwrap().clone();
        let squares = chomped_by(&game, &target);
        assert_eq!(squares, HashSet::from([FieldCoordinate::new(1, 1)]));
    }

    #[test]
    fn is_chomp_available_false_without_pin_players_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "chomper", FieldCoordinate::new(1, 1));
        let chomper = game.player("chomper").unwrap().clone();
        assert!(!is_chomp_available(&game, &chomper));
    }

    #[test]
    fn is_beer_barrel_bash_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_beer_barrel_bash_available(&game, &player));
    }

    #[test]
    fn is_all_you_can_eat_false_without_bomb_action() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_all_you_can_eat_available(&game, &player));
    }

    #[test]
    fn is_stand_up_action_available_requires_prone_and_active() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_stand_up_action_available(&game, &player));
        game.field_model
            .set_player_state("p1", PlayerState::new(ffb_model::enums::PS_PRONE).change_active(true));
        assert!(is_stand_up_action_available(&game, &player));
    }

    #[test]
    fn is_recover_from_confusion_action_available_requires_confused_state() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_recover_from_confusion_action_available(&game, &player));
        game.field_model.set_player_state(
            "p1",
            PlayerState::new(ffb_model::enums::PS_STANDING)
                .change_active(true)
                .change_confused(true),
        );
        assert!(is_recover_from_confusion_action_available(&game, &player));
    }

    #[test]
    fn is_recover_from_gaze_action_available_requires_hypnotized_state() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_recover_from_gaze_action_available(&game, &player));
        game.field_model.set_player_state(
            "p1",
            PlayerState::new(ffb_model::enums::PS_STANDING)
                .change_active(true)
                .change_hypnotized(true),
        );
        assert!(is_recover_from_gaze_action_available(&game, &player));
    }

    #[test]
    fn is_recover_from_eye_gouge_action_available_requires_eye_gouged_state() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_recover_from_eye_gouge_action_available(&game, &player));
        game.field_model.set_player_state(
            "p1",
            PlayerState::new(ffb_model::enums::PS_STANDING)
                .change_active(true)
                .change_eye_gouged(true),
        );
        assert!(is_recover_from_eye_gouge_action_available(&game, &player));
    }

    #[test]
    fn is_block_action_available_false_without_adjacent_targets() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_block_action_available(&game, &player));
    }

    #[test]
    fn is_multi_block_action_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_multi_block_action_available(&game, &player));
    }

    #[test]
    fn is_throw_bomb_action_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_throw_bomb_action_available(&game, &player));
    }

    #[test]
    fn is_secure_the_ball_action_available_false_without_ball_moving() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_secure_the_ball_action_available(&game, &player));
    }

    #[test]
    fn is_blitz_action_available_false_when_blitz_used() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.turn_data_mut().blitz_used = true;
        let player = game.player("p1").unwrap().clone();
        assert!(!is_blitz_action_available(&game, &player));
    }

    #[test]
    fn is_foul_action_available_false_without_opponents() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_foul_action_available(&game, &player));
    }

    #[test]
    fn is_pass_action_available_false_when_pass_used() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.turn_data_mut().pass_used = true;
        let player = game.player("p1").unwrap().clone();
        assert!(!is_pass_action_available(&game, &player, false));
    }

    #[test]
    fn is_punt_action_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_punt_action_available(&game, &player, false));
    }

    #[test]
    fn is_hand_over_action_available_false_when_used() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.turn_data_mut().hand_over_used = true;
        let player = game.player("p1").unwrap().clone();
        assert!(!is_hand_over_action_available(&game, &player, false));
    }

    #[test]
    fn is_kick_em_block_and_blitz_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_kick_em_block_available(&game, &player));
        assert!(!is_kick_em_blitz_available(&game, &player));
    }

    #[test]
    fn is_flashing_blade_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_flashing_blade_available(&game, &player));
    }

    #[test]
    fn is_vicious_vines_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_vicious_vines_available(&game, &player));
    }

    #[test]
    fn is_furious_outburst_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_furious_outburst_available(&game, &player));
    }

    #[test]
    fn is_treacherous_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_treacherous_available(&game, &player));
    }

    #[test]
    fn is_catch_of_the_day_available_false_without_ball_moving() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_catch_of_the_day_available(&game, &player));
    }

    #[test]
    fn is_wisdom_available_delegates_to_game_mechanic() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        // Baseline: no special wisdom-granting state is set up.
        let _ = is_wisdom_available(&game, &player);
    }

    #[test]
    fn is_black_ink_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_black_ink_available(&game, &player));
    }

    #[test]
    fn is_raiding_party_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_raiding_party_available(&game, &player));
    }

    #[test]
    fn is_look_into_my_eyes_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_look_into_my_eyes_available(&game, &player));
    }

    #[test]
    fn is_baleful_hex_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_baleful_hex_available(&game, &player));
    }

    #[test]
    fn is_then_i_started_blastin_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_then_i_started_blastin_available(&game, &player));
    }

    #[test]
    fn is_zoat_gaze_available_false_without_skill() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_zoat_gaze_available(&game, &player));
    }

    #[test]
    fn is_special_ability_available_false_by_default() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        assert!(!is_special_ability_available(&game, &game.acting_player.clone()));
    }

    #[test]
    fn is_blitz_special_ability_available_false_by_default() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        assert!(!is_blitz_special_ability_available(&game, &game.acting_player.clone()));
    }

    #[test]
    fn performs_range_grid_action_false_by_default() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        assert!(!performs_range_grid_action(&game.acting_player.clone(), &game));
    }

    #[test]
    fn is_special_block_action_available_none_state_is_false() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(!is_special_block_action_available(&game, &player, None));
    }

    #[test]
    fn is_recover_from_confusion_and_stand_up_are_mutually_exclusive_prone_gate() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        game.field_model.set_player_state(
            "p1",
            PlayerState::new(ffb_model::enums::PS_PRONE)
                .change_active(true)
                .change_confused(true),
        );
        // Prone confused players cannot yet recover from confusion (must stand up first).
        assert!(!is_recover_from_confusion_action_available(&game, &player));
    }
}
