//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.KickEmBlitzLogicModule` (63 lines).
//!
//! Java's `KickEmBlitzLogicModule extends BlitzLogicModule`. Per the established composition
//! convention (see `blitz_logic_module.rs`), this struct holds a `BlitzLogicModule` value for
//! delegating unmodified inherited behavior (`availableActions`/`actionContext`/
//! `performAvailableAction`), plus its own `BlockLogicExtension` instance to call `extension.block(...)`
//! directly (mirroring `BlitzLogicModule`'s own independent `extension` field rather than reaching
//! into a sibling struct's private field).
//!
//! Documented gap:
//! - `UtilPlayer.isKickable(Game, Player<?>)` is not translated in `ffb-model`'s `UtilPlayer`
//!   (only declared in `ffb-common`); reimplemented here as a local free function matching the
//!   Java body exactly (composed here, same pattern as `logic_module.rs`'s
//!   `has_uncanceled_skill_with_property`).

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::blitz_logic_module::BlitzLogicModule;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

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

/// 1:1 translation of the `KickEmBlitzLogicModule` class.
#[derive(Debug, Default)]
pub struct KickEmBlitzLogicModule {
    blitz: BlitzLogicModule,
    extension: BlockLogicExtension,
}

impl KickEmBlitzLogicModule {
    /// java: `public KickEmBlitzLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { blitz: BlitzLogicModule::new(), extension: BlockLogicExtension::new() }
    }

    /// java: `protected PlayerAction moveAction()`.
    pub fn move_action(&self) -> PlayerAction {
        PlayerAction::KickEmBlitz
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            return self.blitz.player_interaction(client, player);
        }

        let going_for_it = client
            .game()
            .map(|g| UtilPlayer::is_next_move_going_for_it(g) && !acting_player.goes_for_it)
            .unwrap_or(false);

        if going_for_it {
            let ctx = match client.game() {
                Some(game) => LogicModule::action_context(self, game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
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
            return InteractionResult::perform();
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::reset(),
        };
        if !game.acting_player.has_blocked && is_kickable(game, player) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for KickEmBlitzLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::KickEmBlitz
    }

    /// java: (not overridden) inherited unchanged from `BlitzLogicModule`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        LogicModule::available_actions(&self.blitz)
    }

    /// java: (not overridden) inherited unchanged from `BlitzLogicModule`/`MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.blitz, game, acting_player)
    }

    /// java: (not overridden) inherited unchanged from `BlitzLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        LogicModule::perform_available_action(&mut self.blitz, client, player, action)
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
    fn get_id_is_kick_em_blitz() {
        assert_eq!(KickEmBlitzLogicModule::new().get_id(), ClientStateId::KickEmBlitz);
    }

    #[test]
    fn move_action_is_kick_em_blitz() {
        assert_eq!(KickEmBlitzLogicModule::new().move_action(), PlayerAction::KickEmBlitz);
    }

    #[test]
    fn is_kickable_false_without_defender_state() {
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!is_kickable(&game, &player));
    }

    #[test]
    fn is_kickable_requires_away_team_and_adjacency() {
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        let player = game.player("a1").unwrap().clone();
        // Not prone/stunned by default (active standing) and no attacker present.
        assert!(!is_kickable(&game, &player));
    }

    #[test]
    fn available_actions_delegates_to_blitz() {
        let module = KickEmBlitzLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = KickEmBlitzLogicModule::new();
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
        let module = KickEmBlitzLogicModule::new();
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
        let mut module = KickEmBlitzLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
