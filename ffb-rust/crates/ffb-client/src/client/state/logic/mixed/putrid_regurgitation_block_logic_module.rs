//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.PutridRegurgitationBlockLogicModule`
//! (77 lines).
//!
//! Java's `PutridRegurgitationBlockLogicModule extends BlockLogicModule` (the `mixed` package's
//! own `BlockLogicModule`, owned by a sibling translation batch — see the cross-worktree
//! dependency note in `kick_em_block_logic_module.rs`, which applies identically here).
//! Composition mirrors that file: a `BlockLogicModule` value for the inherited/unmodified
//! `getId` fallback path (not applicable — `getId` is overridden — but kept for `super.playerInteraction`
//! and the inherited `endTurn`/`performAvailableAction(END_MOVE)` behavior), plus its own
//! `BlockLogicExtension` instance for `extension.isBlockable(...)`/`extension.block(...)`.

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction};
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

/// 1:1 translation of the `PutridRegurgitationBlockLogicModule` class.
#[derive(Debug, Default)]
pub struct PutridRegurgitationBlockLogicModule {
    block_logic: BlockLogicModule,
    extension: BlockLogicExtension,
}

impl PutridRegurgitationBlockLogicModule {
    /// java: `public PutridRegurgitationBlockLogicModule(FantasyFootballClient pClient)`.
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

        let should_block = match client.game() {
            Some(game) => {
                let vomit_skill = acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK))
                    .unwrap_or(false);
                vomit_skill && self.extension.is_blockable(game, player)
            }
            None => false,
        };

        if should_block {
            if let Some(id) = acting_player.player_id.clone() {
                self.extension.block(client, &id, Some(player), false, false, true, false, false);
            }
            return InteractionResult::handled();
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::reset(),
        };
        if game.acting_player.player_action == Some(PlayerAction::PutridRegurgitationBlock)
            && self.extension.is_blockable(game, player)
        {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for PutridRegurgitationBlockLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::PutridRegurgitationBlock
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::END_MOVE);
        actions
    }

    /// java: (not overridden) inherited unchanged from `BlockLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.block_logic, game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::PROJECTILE_VOMIT => {
                let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
                client.communication_mut().send_acting_player(
                    Some(player),
                    PlayerAction::PutridRegurgitationBlitz,
                    jumping,
                );
            }
            ClientAction::END_MOVE => {
                <BlockLogicModule as LogicModule>::perform_available_action(
                    &mut self.block_logic,
                    client,
                    player,
                    action,
                );
            }
            _ => {
                // java: default: break;
            }
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[allow(dead_code)]
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
    fn get_id_is_putrid_regurgitation_block() {
        assert_eq!(
            PutridRegurgitationBlockLogicModule::new().get_id(),
            ClientStateId::PutridRegurgitationBlock
        );
    }

    #[test]
    fn available_actions_has_expected_set() {
        let actions = PutridRegurgitationBlockLogicModule::new().available_actions();
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&ClientAction::PROJECTILE_VOMIT));
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = PutridRegurgitationBlockLogicModule::new();
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
        let module = PutridRegurgitationBlockLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_for_unknown_action() {
        let mut module = PutridRegurgitationBlockLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
