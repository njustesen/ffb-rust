//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.PutridRegurgitationBlitzLogicModule`
//! (105 lines).
//!
//! Java's `PutridRegurgitationBlitzLogicModule extends BlitzLogicModule`. Per the established
//! composition convention (see `blitz_logic_module.rs`, `kick_em_blitz_logic_module.rs`), this
//! struct holds a `BlitzLogicModule` value plus its own `BlockLogicExtension` instance for
//! `extension.isBlockable(...)`/`extension.block(...)`.
//!
//! Note: `LogicModule::isPutridRegurgitationAvailable()` is always `false` in the base Java class
//! (see `logic_module::is_putrid_regurgitation_available`), but `PutridRegurgitationBlitzLogicModule`
//! genuinely overrides it with real logic — that override is translated below as its own method
//! (not reusing the base's always-`false` free function, since this class's behavior differs).
//! Likewise `isMoveAvailable(ActingPlayer)` is overridden here with different semantics than the
//! base `logic_module::is_move_available` free function, so it is reimplemented locally rather
//! than reused.

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::array_tool::ArrayTool;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::blitz_logic_module::BlitzLogicModule;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `Game.getOtherTeam(Team team)` — no equivalent on the Rust `Game`; the acting/other
/// team is looked up by comparing team ids against `game.team_home`/`game.team_away` directly,
/// matching `logic_module.rs`'s own `player_own_team` helper convention.
fn other_team<'a>(game: &'a Game, team_id: &str) -> &'a ffb_model::model::team::Team {
    if game.team_home.id == team_id {
        &game.team_away
    } else {
        &game.team_home
    }
}

/// 1:1 translation of the `PutridRegurgitationBlitzLogicModule` class.
#[derive(Debug, Default)]
pub struct PutridRegurgitationBlitzLogicModule {
    blitz: BlitzLogicModule,
    extension: BlockLogicExtension,
}

impl PutridRegurgitationBlitzLogicModule {
    /// java: `public PutridRegurgitationBlitzLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { blitz: BlitzLogicModule::new(), extension: BlockLogicExtension::new() }
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
                let vomit_skill = acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK))
                    .unwrap_or(false);
                acting_player.player_action == Some(PlayerAction::PutridRegurgitationBlitz)
                    && vomit_skill
                    && self.extension.is_blockable(game, player)
            }
            None => false,
        };

        if extra_available {
            if let Some(id) = acting_player.player_id.clone() {
                self.extension.block(client, &id, Some(player), false, false, true, false, false);
            }
            return InteractionResult::handled();
        }

        InteractionResult::ignore()
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions_impl(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::JUMP);
        actions.insert(ClientAction::BOUNDING_LEAP);
        actions
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action_impl(
        &mut self,
        client: &mut FantasyFootballClient,
        player: &Player,
        action: ClientAction,
    ) {
        let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
        match action {
            ClientAction::PROJECTILE_VOMIT => {
                client.communication_mut().send_acting_player(
                    Some(player),
                    PlayerAction::PutridRegurgitationBlitz,
                    jumping,
                );
            }
            ClientAction::MOVE => {
                client.communication_mut().send_acting_player(
                    Some(player),
                    PlayerAction::PutridRegurgitationMove,
                    jumping,
                );
            }
            _ => {
                <BlitzLogicModule as LogicModule>::perform_available_action(&mut self.blitz, client, player, action);
            }
        }
    }

    /// java: `public boolean isPutridRegurgitationAvailable()` — a genuine override (see module
    /// doc comment); distinct from `logic_module::is_putrid_regurgitation_available` (always
    /// `false` in the base class).
    pub fn is_putrid_regurgitation_available(&self, game: &Game) -> bool {
        let acting_player = &game.acting_player;
        if self.is_move_available(acting_player) || !acting_player.has_blocked {
            return false;
        }
        let player = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return false,
        };
        if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK) {
            return false;
        }
        let coord = match game.field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return false,
        };
        let active_team_id = game.active_team().id.clone();
        let opponent_team = other_team(game, &active_team_id);
        ArrayTool::is_provided(&UtilPlayer::find_adjacent_blockable_players(game, opponent_team, coord))
    }

    /// java: `public boolean isMoveAvailable(ActingPlayer actingPlayer)` — a genuine override
    /// (see module doc comment); distinct from `logic_module::is_move_available`.
    pub fn is_move_available(&self, acting_player: &ActingPlayer) -> bool {
        acting_player.player_action == Some(PlayerAction::PutridRegurgitationBlitz)
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::reset(),
        };
        if game.acting_player.player_action == Some(PlayerAction::PutridRegurgitationBlitz)
            && self.extension.is_blockable(game, player)
        {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for PutridRegurgitationBlitzLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::PutridRegurgitationBlitz
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        self.available_actions_impl()
    }

    /// java: (not overridden) inherited unchanged from `BlitzLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.blitz, game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.perform_available_action_impl(client, player, action)
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
    fn get_id_is_putrid_regurgitation_blitz() {
        assert_eq!(
            PutridRegurgitationBlitzLogicModule::new().get_id(),
            ClientStateId::PutridRegurgitationBlitz
        );
    }

    #[test]
    fn available_actions_has_expected_set() {
        let actions = PutridRegurgitationBlitzLogicModule::new().available_actions();
        assert_eq!(actions.len(), 5);
        assert!(actions.contains(&ClientAction::PROJECTILE_VOMIT));
        assert!(actions.contains(&ClientAction::BOUNDING_LEAP));
    }

    #[test]
    fn is_move_available_matches_own_action_only() {
        let module = PutridRegurgitationBlitzLogicModule::new();
        let mut ap = ActingPlayer::new();
        ap.player_action = Some(PlayerAction::PutridRegurgitationBlitz);
        assert!(module.is_move_available(&ap));
        ap.player_action = Some(PlayerAction::Move);
        assert!(!module.is_move_available(&ap));
    }

    #[test]
    fn is_putrid_regurgitation_available_false_when_move_available() {
        let module = PutridRegurgitationBlitzLogicModule::new();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::PutridRegurgitationBlitz);
        assert!(!module.is_putrid_regurgitation_available(&game));
    }

    #[test]
    fn is_putrid_regurgitation_available_false_without_block() {
        let module = PutridRegurgitationBlitzLogicModule::new();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.acting_player.has_blocked = false;
        assert!(!module.is_putrid_regurgitation_available(&game));
    }

    #[test]
    fn player_peek_resets_without_game() {
        let client = make_client();
        let module = PutridRegurgitationBlitzLogicModule::new();
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
        let module = PutridRegurgitationBlitzLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn other_team_returns_away_for_home_id() {
        let game = make_game();
        let opp = other_team(&game, &game.team_home.id.clone());
        assert_eq!(opp.id, game.team_away.id);
    }
}
