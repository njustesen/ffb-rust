//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.SynchronousMultiBlockLogicModule`
//! (245 lines).
//!
//! Java's `SynchronousMultiBlockLogicModule extends LogicModule` directly and mixes in a
//! `BlockLogicExtension` (`extension` field), plus two mutable maps (`selectedPlayers`,
//! `originalPlayerStates`) tracking in-progress block-target selection state — translated as
//! plain struct fields (mutated via `&mut self`, matching Rust ownership rather than Java's
//! implicit `this` mutation).
//!
//! Documented gap:
//! - `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap;
//!   `skill_placeholder(SkillId)` is reused here for the network command.
//!
//! KNOWN PITFALL worked around here: `LogicModule::player_interaction` has a `&self` trait
//! default, but this class's own `playerInteraction` needs to mutate `selectedPlayers`/
//! `originalPlayerStates`. An inherent `&mut self` method of the same name would be silently
//! shadowed by the trait's `&self` default at every call site (Rust's method-resolution probes
//! the `&Self` receiver step — where the trait default matches — before ever reaching the
//! `&mut Self` step). Per the established workaround, the two maps are wrapped in `RefCell` so
//! `player_interaction` (and everything it calls transitively) can stay `&self`, letting the
//! inherent method correctly take priority over the trait default at the same receiver step.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::block_kind::BlockKind;
use ffb_model::model::block_target::BlockTarget;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::player_state::PlayerState;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// 1:1 translation of the `SynchronousMultiBlockLogicModule` class.
#[derive(Debug, Default)]
pub struct SynchronousMultiBlockLogicModule {
    selected_players: RefCell<HashMap<String, BlockKind>>,
    original_player_states: RefCell<HashMap<String, Option<PlayerState>>>,
    extension: BlockLogicExtension,
}

impl SynchronousMultiBlockLogicModule {
    /// java: `public SynchronousMultiBlockLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            selected_players: RefCell::new(HashMap::new()),
            original_player_states: RefCell::new(HashMap::new()),
            extension: BlockLogicExtension::new(),
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            let ctx = client
                .game()
                .map(|g| self.action_context(g, &acting_player))
                .unwrap_or_else(ActionContext::new);
            InteractionResult::select_action(ctx)
        } else {
            self.handle_player_selection(client, player)
        }
    }

    /// java: `public InteractionResult handlePlayerSelection(Player<?> player)`.
    pub fn handle_player_selection(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let was_selected = self.selected_players.borrow_mut().remove(&player.id).is_some();
        if was_selected {
            self.original_player_states.borrow_mut().remove(&player.id);
            client.communication_mut().send_unset_block_target(player.id.clone());
            InteractionResult::handled()
        } else {
            self.handle_defender_selection(client, player)
        }
    }

    /// java: `private InteractionResult handleDefenderSelection(Player<?> defender)`.
    fn handle_defender_selection(&self, client: &mut FantasyFootballClient, defender: &Player) -> InteractionResult {
        let (is_blockable, defender_coordinate, has_multi_block_alt) = match client.game() {
            Some(game) => {
                let is_blockable = self.extension.is_blockable(game, defender);
                let defender_coordinate = game.field_model.player_coordinate(&defender.id);
                let has_multi_block_alt = game
                    .acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::PROVIDES_MULTIPLE_BLOCK_ALTERNATIVE))
                    .unwrap_or(false);
                (is_blockable, defender_coordinate, has_multi_block_alt)
            }
            None => return InteractionResult::ignore(),
        };

        if is_blockable {
            if has_multi_block_alt {
                return self.extension.player_interaction_full(client, Some(defender), false, true);
            }
            let has_decoration = defender_coordinate
                .and_then(|c| client.game().and_then(|g| g.field_model.get_dice_decoration_at(c)))
                .is_some();
            if has_decoration {
                self.select_player(client, defender);
                return InteractionResult::handled();
            }
        }
        InteractionResult::ignore()
    }

    /// java: `private void selectPlayer(Player<?> player)`.
    fn select_player(&self, client: &mut FantasyFootballClient, player: &Player) {
        if self.selected_players.borrow().len() < 2 {
            self.selected_players.borrow_mut().insert(player.id.clone(), BlockKind::BLOCK);
            let state = client.game().and_then(|g| g.field_model.player_state(&player.id));
            self.original_player_states.borrow_mut().insert(player.id.clone(), state);
            client.communication_mut().send_set_block_target(player.id.clone(), BlockKind::BLOCK);
            self.send_if_selection_complete(client);
        }
    }

    /// java: `private void sendIfSelectionComplete()`.
    fn send_if_selection_complete(&self, client: &mut FantasyFootballClient) {
        let selected_players = self.selected_players.borrow();
        if selected_players.len() == 2 {
            let original_player_states = self.original_player_states.borrow();
            let mut block_targets: Vec<BlockTarget> = selected_players
                .iter()
                .map(|(player_id, kind)| {
                    let original_state = original_player_states.get(player_id).copied().flatten();
                    BlockTarget::new(player_id.clone(), *kind, original_state)
                })
                .collect();
            block_targets.sort_by(|a, b| a.get_player_id().cmp(&b.get_player_id()));
            drop(selected_players);
            drop(original_player_states);
            client.communication_mut().send_block_targets(block_targets);
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let is_blockable = client.game().map(|g| self.extension.is_blockable(g, player)).unwrap_or(false);
        if is_blockable {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for SynchronousMultiBlockLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SynchronousMultiBlock
    }

    /// java: `public void setUp()`.
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {
        self.selected_players.borrow_mut().clear();
        self.original_player_states.borrow_mut().clear();
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::BLOCK);
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
        if acting_player.suffering_blood_lust {
            action_context.add_action(ClientAction::MOVE);
        }
        action_context.add_action(ClientAction::END_MOVE);

        let player = acting_player.player_id.as_deref().and_then(|id| game.player(id));
        if let Some(player) = player {
            use crate::client::state::logic::logic_module::{
                is_baleful_hex_available, is_black_ink_available, is_catch_of_the_day_available,
                is_look_into_my_eyes_available, is_raiding_party_available, is_treacherous_available,
                is_wisdom_available, is_zoat_gaze_available,
            };
            if is_treacherous_available(game, player) {
                action_context.add_action(ClientAction::TREACHEROUS);
            }
            if is_wisdom_available(game, player) {
                action_context.add_action(ClientAction::WISDOM);
            }
            if is_raiding_party_available(game, player) {
                action_context.add_action(ClientAction::RAIDING_PARTY);
            }
            if is_look_into_my_eyes_available(game, player) {
                action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
            }
            if is_baleful_hex_available(game, player) {
                action_context.add_action(ClientAction::BALEFUL_HEX);
            }
            if is_black_ink_available(game, player) {
                action_context.add_action(ClientAction::BLACK_INK);
            }
            if is_catch_of_the_day_available(game, player) {
                action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
            }
            if is_zoat_gaze_available(game, player) {
                action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
            }
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                let ids: Vec<String> = self.selected_players.borrow().keys().cloned().collect();
                for id in ids {
                    client.communication_mut().send_unset_block_target(id);
                }
                self.selected_players.borrow_mut().clear();
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s own documented gap.
            }
            ClientAction::BLOCK => {
                self.select_player(client, player);
            }
            ClientAction::TREACHEROUS => {
                let available =
                    client.game().map(|g| crate::client::state::logic::logic_module::is_treacherous_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                let available =
                    client.game().map(|g| crate::client::state::logic::logic_module::is_wisdom_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                let available = client
                    .game()
                    .map(|g| crate::client::state::logic::logic_module::is_raiding_party_available(g, player))
                    .unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                let available = client
                    .game()
                    .map(|g| crate::client::state::logic::logic_module::is_look_into_my_eyes_available(g, player))
                    .unwrap_or(false);
                if available {
                    if let Some(skill_id) = ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                let available =
                    client.game().map(|g| crate::client::state::logic::logic_module::is_baleful_hex_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                let available =
                    client.game().map(|g| crate::client::state::logic::logic_module::is_black_ink_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                let available = client
                    .game()
                    .map(|g| crate::client::state::logic::logic_module::is_catch_of_the_day_available(g, player))
                    .unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                let available =
                    client.game().map(|g| crate::client::state::logic::logic_module::is_zoat_gaze_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player
                        .skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// java: `public void endTurn()`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };
        if let Some(player) = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned() {
            self.perform(client, &player, ClientAction::END_MOVE);
        }
        let turn_mode = client.game().map(|g| g.turn_mode);
        if let Some(turn_mode) = turn_mode {
            client.communication_mut().send_end_turn(turn_mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
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
    fn get_id_is_synchronous_multi_block() {
        assert_eq!(SynchronousMultiBlockLogicModule::new().get_id(), ClientStateId::SynchronousMultiBlock);
    }

    #[test]
    fn available_actions_contains_block_and_move() {
        let actions = SynchronousMultiBlockLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::MOVE));
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn set_up_clears_selection_state() {
        let mut module = SynchronousMultiBlockLogicModule::new();
        module.selected_players.borrow_mut().insert("p1".to_string(), BlockKind::BLOCK);
        let mut client = make_client();
        module.set_up(&mut client);
        assert!(module.selected_players.borrow().is_empty());
        assert!(module.original_player_states.borrow().is_empty());
    }

    #[test]
    fn action_context_adds_move_when_suffering_blood_lust() {
        let module = SynchronousMultiBlockLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.suffering_blood_lust = true;
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::MOVE));
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn select_player_stores_block_kind_and_state() {
        let module = SynchronousMultiBlockLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("d1").unwrap().clone();
        module.select_player(&mut client, &player);
        assert_eq!(module.selected_players.borrow().get("d1"), Some(&BlockKind::BLOCK));
        assert!(module.original_player_states.borrow().contains_key("d1"));
    }

    #[test]
    fn select_player_ignored_once_two_are_selected() {
        let module = SynchronousMultiBlockLogicModule::new();
        module.selected_players.borrow_mut().insert("a".to_string(), BlockKind::BLOCK);
        module.selected_players.borrow_mut().insert("b".to_string(), BlockKind::BLOCK);
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("d1").unwrap().clone();
        module.select_player(&mut client, &player);
        assert_eq!(module.selected_players.borrow().len(), 2);
        assert!(!module.selected_players.borrow().contains_key("d1"));
    }

    #[test]
    fn handle_player_selection_deselects_when_already_selected() {
        let module = SynchronousMultiBlockLogicModule::new();
        module.selected_players.borrow_mut().insert("d1".to_string(), BlockKind::BLOCK);
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("d1").unwrap().clone();
        let result = module.handle_player_selection(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
        assert!(!module.selected_players.borrow().contains_key("d1"));
    }

    #[test]
    fn player_peek_resets_when_not_blockable() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(10, 10));
        client.set_game(game);
        let module = SynchronousMultiBlockLogicModule::new();
        let player = client.game().unwrap().player("d1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = SynchronousMultiBlockLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_end_move_clears_selection() {
        let mut module = SynchronousMultiBlockLogicModule::new();
        module.selected_players.borrow_mut().insert("a".to_string(), BlockKind::BLOCK);
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
        assert!(module.selected_players.borrow().is_empty());
    }
}
