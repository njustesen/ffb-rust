//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.BlockKindLogicModule` (78 lines).
//!
//! Java's `BlockKindLogicModule extends LogicModule` and mixes in a `BlockLogicExtension`
//! (`extension` field), matching the same composition pattern already established for
//! `BlitzLogicModule` (struct holding a `BlockLogicExtension` value, delegating to its
//! methods and to `logic_module` free functions).
//!
//! Documented gaps:
//! - `protected ActionContext actionContext(ActingPlayer actingPlayer)` calls
//!   `extension.blockActionContext(actingPlayer, multiBlock)`, which in Java reads
//!   `client.getGame()` via the extension's own held `client` reference. The Rust
//!   `LogicModule::action_context` trait signature only provides `&Game` (no client), while
//!   `BlockLogicExtension::block_action_context`/`is_gored_available` (this crate's existing
//!   translation, not owned by this batch) require `&FantasyFootballClient` explicitly. Since
//!   both only ever read `client.game()` internally, `block_action_context_from_game` and
//!   `is_gored_available_from_game` below are game-only reimplementations of those exact
//!   same bodies (mirroring the Java source 1:1), used only where the trait signature can't
//!   thread a client through — matching the precedent already set by
//!   `dump_off_logic_module.rs`'s locally-reimplemented `create_range_ruler`.
//! - `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap;
//!   `skill_placeholder(SkillId)` is reused for the network command.
//! - `if (player != null)` in `performAvailableAction` — the trait signature always passes
//!   a live `&Player` reference (no null variant in Rust), so this guard is vacuous and
//!   omitted, matching the convention already used in `blitz_logic_module.rs`.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::dice_decoration::DiceDecoration;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::block_logic_extension::BlockLogicExtension;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `BlockLogicExtension.isGoredAvailable()` — see module doc gap (game-only variant).
fn is_gored_available_from_game(game: &Game) -> bool {
    let acting_player = &game.acting_player;
    let target_selection_state = match &game.field_model.target_selection_state {
        Some(s) => s,
        None => return false,
    };
    let acting_attacker = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return false,
    };
    if !UtilCards::has_unused_skill_with_property(acting_attacker, NamedProperties::CAN_ADD_BLOCK_DIE) {
        return false;
    }
    let selected_id = match target_selection_state.get_selected_player_id() {
        Some(id) => id.clone(),
        None => return false,
    };
    let target_coordinate = game.field_model.player_coordinate(&selected_id);
    let player_coordinate = acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
    let dice_decoration: Option<&DiceDecoration> =
        target_coordinate.and_then(|c| game.field_model.get_dice_decoration_at(c));
    let defender = game.player(&selected_id);
    let opponent_can_move = defender
        .map(|d| UtilCards::has_unused_skill_with_property(d, NamedProperties::CAN_MOVE_BEFORE_BEING_BLOCKED))
        .unwrap_or(false);

    match (dice_decoration, target_coordinate, player_coordinate) {
        (Some(decoration), Some(target_coordinate), Some(player_coordinate)) => {
            (decoration.nr_of_dice == 1 || decoration.nr_of_dice == 2 || (decoration.nr_of_dice == 3 && opponent_can_move))
                && target_coordinate.is_adjacent(player_coordinate)
        }
        _ => false,
    }
}

/// java: `BlockLogicExtension.blockActionContext(ActingPlayer, boolean)` — see module doc gap
/// (game-only variant).
fn block_action_context_from_game(game: &Game, acting_player: &ActingPlayer, multi_block: bool) -> ActionContext {
    let mut action_context = ActionContext::new();
    let attacker = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        Some(p) => p,
        None => return action_context,
    };
    if attacker.has_skill_property(NamedProperties::PROVIDES_STAB_BLOCK_ALTERNATIVE) {
        action_context.add_action(ClientAction::STAB);
    }
    if attacker.has_skill_property(NamedProperties::PROVIDES_CHAINSAW_BLOCK_ALTERNATIVE) && !multi_block {
        action_context.add_action(ClientAction::CHAINSAW);
    }
    let vomit_skill = UtilCards::get_unused_skill_with_property(
        attacker,
        NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL,
    );
    if let Some(vomit_skill_id) = vomit_skill {
        if !multi_block {
            if skill_placeholder(vomit_skill_id).has_skill_property(NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK) {
                action_context.add_influence(Influences::VOMIT_DUE_TO_PUTRID_REGURGITATION);
            }
            action_context.add_action(ClientAction::PROJECTILE_VOMIT);
        }
    }

    let fire_skill = UtilCards::get_unused_skill_with_property(
        attacker,
        NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER,
    );
    if fire_skill.is_some() && !multi_block {
        action_context.add_action(ClientAction::BREATHE_FIRE);
    }

    if is_gored_available_from_game(game) {
        action_context.add_action(ClientAction::GORED_BY_THE_BULL);
    }
    action_context.add_action(ClientAction::BLOCK);

    // java: plugin.blockActionContext(actingPlayer, multiBlock, actionContext, this) — gap;
    // pass through unchanged.
    action_context
}

/// 1:1 translation of the `BlockKindLogicModule` class.
#[derive(Debug, Default)]
pub struct BlockKindLogicModule {
    extension: BlockLogicExtension,
}

impl BlockKindLogicModule {
    /// java: `public BlockKindLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { extension: BlockLogicExtension::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &FantasyFootballClient, _player: &Player) -> InteractionResult {
        match client.game() {
            Some(game) => InteractionResult::select_action(self.action_context(game, &game.acting_player)),
            None => InteractionResult::ignore(),
        }
    }
}

impl LogicModule for BlockKindLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SelectBlockKind
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::STAB);
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::CHAINSAW);
        actions.insert(ClientAction::BREATHE_FIRE);
        actions.insert(ClientAction::GORED_BY_THE_BULL);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — see module
    /// doc gap regarding the game-only reimplementation.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        block_action_context_from_game(game, acting_player, acting_player.player_action == Some(PlayerAction::MultipleBlock))
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let (home_playing, acting_player) = match client.game() {
            Some(game) => (game.home_playing, game.acting_player.clone()),
            None => return,
        };
        if !home_playing {
            return;
        }
        match action {
            ClientAction::GORED_BY_THE_BULL => {
                let gored_available = self.extension.is_gored_available(client);
                if gored_available {
                    if let Some(attacker) =
                        acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                    {
                        if let Some(skill_id) =
                            UtilCards::get_unused_skill_with_property(&attacker, NamedProperties::CAN_ADD_BLOCK_DIE)
                        {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                        }
                    }
                }
                if let Some(id) = acting_player.player_id.clone() {
                    client.communication_mut().send_block(id, Some(player), false, false, false, false, false);
                }
            }
            _ => {
                self.extension.perform_available_action(client, player, action);
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
    fn get_id_is_select_block_kind() {
        assert_eq!(BlockKindLogicModule::new().get_id(), ClientStateId::SelectBlockKind);
    }

    #[test]
    fn available_actions_contains_expected_variants() {
        let actions = BlockKindLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::STAB));
        assert!(actions.contains(&ClientAction::GORED_BY_THE_BULL));
        assert_eq!(actions.len(), 6);
    }

    #[test]
    fn action_context_contains_block_when_attacker_resolves() {
        let module = BlockKindLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        let mut ap = ActingPlayer::new();
        ap.player_id = Some("attacker".to_string());
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::BLOCK));
        assert!(!ctx.get_actions().contains(&ClientAction::STAB));
    }

    #[test]
    fn action_context_empty_when_attacker_does_not_resolve() {
        let module = BlockKindLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let client = make_client();
        let module = BlockKindLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_selects_action_with_game() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = BlockKindLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::SelectAction
        );
    }

    #[test]
    fn perform_available_action_skips_when_away_playing() {
        let mut module = BlockKindLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.home_playing = false;
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        client.set_game(game);
        let player = client.game().unwrap().player("a1").unwrap().clone();
        module.perform_available_action(&mut client, &player, ClientAction::GORED_BY_THE_BULL);
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = BlockKindLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::BLOCK);
    }
}
