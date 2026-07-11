//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.KickEmBlockLogicModule` (43 lines).
//!
//! Java's `KickEmBlockLogicModule extends BlockLogicModule` (the `mixed` package's own
//! `BlockLogicModule`, translated by a sibling batch as
//! `crate::client::state::logic::mixed::block_logic_module::BlockLogicModule`). Per the
//! established composition convention (see `blitz_logic_module.rs` composing `MoveLogicModule`,
//! `kick_em_blitz_logic_module.rs` composing `BlitzLogicModule`), this struct holds a
//! `BlockLogicModule` value for delegating unmodified inherited behavior
//! (`availableActions`/`actionContext`/`performAvailableAction`), plus its own
//! `BlockLogicExtension` instance to call `extension.block(...)` directly (mirroring
//! `BlockLogicModule`'s own independent `extension` field rather than reaching into a sibling
//! struct's private field).
//!
//! NOTE — cross-worktree dependency: `mixed::block_logic_module::BlockLogicModule` is owned by a
//! sibling agent translating the other half of this `mixed/` package concurrently in a separate
//! worktree, so it does not exist in this worktree and this file cannot be compiled/tested here in
//! isolation (expected per the batch plan; verified once both halves are merged). The API assumed
//! here — `BlockLogicModule::new()`, an inherent `player_interaction(&self, client, player)` method
//! (matching the same "trait-default-shadowing inherent method" convention used by
//! `MoveLogicModule`/`BlitzLogicModule`/`BlockLogicExtension` for methods that need client access),
//! and a `LogicModule` trait impl providing `get_id`/`available_actions`/`action_context`/
//! `perform_available_action` — mirrors `BlockLogicModule.java`'s own shape (`extends
//! AbstractBlockLogicModule`, holds a `BlockLogicExtension extension` field, overrides
//! `playerInteraction`/`playerPeek`/`availableActions`/`actionContext`/`performAvailableAction`).
//!
//! Documented gap:
//! - `UtilPlayer.isKickable(Game, Player<?>)` is not translated in `ffb-model`'s `UtilPlayer`;
//!   reimplemented here as a local free function (same gap/approach as
//!   `kick_em_blitz_logic_module.rs`).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::mixed::block_logic_module::BlockLogicModule;

/// java: `UtilPlayer.isKickable(Game pGame, Player<?> pPlayer)` — see module doc gap.
fn is_kickable(game: &Game, player: &Player) -> bool {
    let acting_player = &game.acting_player;
    let defender_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    let defender_coordinate = game.field_model.player_coordinate(&player.id);
    let attacker_coordinate =
        acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
    defender_state.is_prone_or_stunned()
        && game.team_away.has_player(&player.id)
        && defender_coordinate.is_some()
        && match (defender_coordinate, attacker_coordinate) {
            (Some(d), Some(a)) => d.is_adjacent(a),
            _ => false,
        }
}

/// 1:1 translation of the `KickEmBlockLogicModule` class.
#[derive(Debug, Default)]
pub struct KickEmBlockLogicModule {
    block_logic: BlockLogicModule,
    extension: BlockLogicExtension,
}

impl KickEmBlockLogicModule {
    /// java: `public KickEmBlockLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { block_logic: BlockLogicModule::new(), extension: BlockLogicExtension::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            return self.block_logic.player_interaction(client, player);
        }

        let extra_available = match client.game() {
            Some(game) => {
                let attacker = acting_player.player_id.as_deref().and_then(|id| game.player(id));
                let chainsaw = attacker
                    .map(|p| {
                        UtilCards::has_unused_skill_with_property(
                            p,
                            NamedProperties::CAN_USE_CHAINSAW_ON_DOWNED_OPPONENTS,
                        )
                    })
                    .unwrap_or(false);
                let prone_or_stunned =
                    game.field_model.player_state(&player.id).map(|s| s.is_prone_or_stunned()).unwrap_or(false);
                let adjacent = match (
                    acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id)),
                    game.field_model.player_coordinate(&player.id),
                ) {
                    (Some(a), Some(b)) => a.is_adjacent(b),
                    _ => false,
                };
                chainsaw && prone_or_stunned && adjacent
            }
            None => false,
        };

        if extra_available {
            if let Some(id) = acting_player.player_id.clone() {
                self.extension.block(client, &id, Some(player), false, true, false, false, false);
            }
        }
        // java: no `return InteractionResult.perform()` here — the block call is a side effect
        // and the method always falls through to `ignore()`, exactly matching the Java source.
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        match client.game() {
            Some(game) if is_kickable(game, player) => InteractionResult::perform(),
            _ => InteractionResult::reset(),
        }
    }
}

impl LogicModule for KickEmBlockLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::KickEmBlock
    }

    /// java: (not overridden) inherited unchanged from `BlockLogicModule`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        LogicModule::available_actions(&self.block_logic)
    }

    /// java: (not overridden) inherited unchanged from `BlockLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.block_logic, game, acting_player)
    }

    /// java: (not overridden) inherited unchanged from `BlockLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        LogicModule::perform_available_action(&mut self.block_logic, client, player, action)
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
    fn get_id_is_kick_em_block() {
        assert_eq!(KickEmBlockLogicModule::new().get_id(), ClientStateId::KickEmBlock);
    }

    #[test]
    fn is_kickable_false_without_defender_state() {
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!is_kickable(&game, &player));
    }

    #[test]
    fn is_kickable_requires_away_team() {
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let player = game.player("h1").unwrap().clone();
        // h1 is home team -> never kickable regardless of state.
        assert!(!is_kickable(&game, &player));
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = KickEmBlockLogicModule::new();
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
        let module = KickEmBlockLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_ignores_when_target_not_kickable() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10));
        game.acting_player.player_id = Some("h1".to_string());
        client.set_game(game);
        let module = KickEmBlockLogicModule::new();
        let target = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_interaction(&mut client, &target);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
