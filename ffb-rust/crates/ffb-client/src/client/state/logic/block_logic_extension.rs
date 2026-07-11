//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.BlockLogicExtension` (238 lines).
//!
//! Java's `BlockLogicExtension` is a `LogicModule` subclass used as a shared "block sub-logic"
//! mixed into other logic modules (edition-specific block/blitz logic modules, a later batch).
//! Its constructor resolves a `BlockLogicExtensionPlugin` via `LogicPluginFactory`
//! (`client/factory/LogicPluginFactory.java`), which is not yet translated (documented gap,
//! same as `MoveLogicModule`). Consequently there is no `plugin` field here; every call site
//! that would delegate to `plugin.xxx(...)` is left with a `// java: plugin — gap` comment and
//! a conservative fallback (empty/no-op/pass-through), matching the convention already used in
//! `logic_module.rs` (see `player_can_not_move_placeholder`) and
//! `abstract_block_logic_module.rs`.
//!
//! Other documented gaps:
//! - `player.getSkillWithProperty(property)` returns a full `Skill` in Java; the Rust `Player`
//!   only exposes `skill_id_with_property(&str) -> Option<SkillId>` (see `logic_module.rs`'s own
//!   doc comment). Since `ClientCommunication::send_use_skill` only actually serializes
//!   `skill.get_name()` onto the wire, `skill_placeholder(SkillId)` below builds a minimal
//!   `Skill` carrying just the matching name (via `SkillId::class_name()`) — sufficient for the
//!   network command, though it does not carry the full skill-property/modifier data a
//!   fully-loaded `Skill` from the (currently un-wired) skill data registry would.
//! - `LogicModule::player_interaction`'s trait-default signature (`fn player_interaction(&self,
//!   player: &Player) -> InteractionResult`) has no `client`/mutability parameter, so it cannot
//!   express Java's `playerInteraction(Player<?>)` (which needs `&mut FantasyFootballClient` to
//!   send commands). Per the same pattern already used for `MoveLogicModule`, this is
//!   translated as inherent methods (`player_interaction`/`player_interaction_full`) that shadow
//!   the trait method by taking the extra parameters Java's real logic requires.

use ffb_model::enums::{PlayerAction, SkillCategory};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::dice_decoration::DiceDecoration;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::player_state::PlayerState;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{
    is_baleful_hex_available_ap, is_black_ink_available_ap, is_catch_of_the_day_available_ap,
    is_look_into_my_eyes_available_ap, is_raiding_party_available_ap, is_treacherous_available_ap,
    is_wisdom_available_ap, is_zoat_gaze_available_ap, LogicModule,
};

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), SkillCategory::General)
}

/// java: `plugin().playerCanNotMove(playerState)` — see module doc gap.
fn player_can_not_move_placeholder(_player_state: Option<PlayerState>) -> bool {
    false
}

/// 1:1 translation of the `BlockLogicExtension` class.
#[derive(Debug, Default)]
pub struct BlockLogicExtension;

impl BlockLogicExtension {
    /// java: `public BlockLogicExtension(FantasyFootballClient client)` — see module doc gap
    /// regarding the un-translated `plugin` field.
    pub fn new() -> Self {
        Self
    }

    /// java: `public ActionContext blockActionContext(ActingPlayer actingPlayer, boolean multiBlock)`.
    pub fn block_action_context(
        &self,
        client: &FantasyFootballClient,
        acting_player: &ActingPlayer,
        multi_block: bool,
    ) -> ActionContext {
        let mut action_context = ActionContext::new();
        let attacker = match acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)) {
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

        if self.is_gored_available(client) {
            action_context.add_action(ClientAction::GORED_BY_THE_BULL);
        }
        action_context.add_action(ClientAction::BLOCK);

        // java: plugin.blockActionContext(actingPlayer, multiBlock, actionContext, this) — gap;
        // pass through unchanged.
        action_context
    }

    /// java: `public void performAvailableAction(Player<?> player, ClientAction action)`.
    pub fn perform_available_action(
        &mut self,
        client: &mut FantasyFootballClient,
        player: &Player,
        action: ClientAction,
    ) {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };
        let acting_player_id = acting_player.player_id.clone();
        match action {
            ClientAction::BLOCK => {
                if let Some(id) = &acting_player_id {
                    self.block(client, id, Some(player), false, false, false, false, false);
                }
            }
            ClientAction::STAB => {
                if let Some(id) = &acting_player_id {
                    self.block(client, id, Some(player), true, false, false, false, false);
                }
            }
            ClientAction::CHAINSAW => {
                if let Some(id) = &acting_player_id {
                    self.block(client, id, Some(player), false, true, false, false, false);
                }
            }
            ClientAction::PROJECTILE_VOMIT => {
                if let Some(id) = &acting_player_id {
                    self.block(client, id, Some(player), false, false, true, false, false);
                }
            }
            ClientAction::TREACHEROUS => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::WISDOM => {
                client.communication_mut().send_use_wisdom();
            }
            ClientAction::RAIDING_PARTY => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) =
                        UtilCards::get_unused_skill_with_property(&attacker, NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) = attacker.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            ClientAction::BREATHE_FIRE => {
                if let Some(id) = &acting_player_id {
                    self.block(client, id, Some(player), false, false, false, true, false);
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                if let Some(attacker) = acting_player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned()
                {
                    if let Some(skill_id) = attacker
                        .skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, attacker.id);
                    }
                }
            }
            _ => {
                // java: plugin.performAvailableAction(action, actingPlayer, this, communication, player) — gap.
            }
        }
    }

    /// java: `public void block(String pActingPlayerId, Player<?> pDefender, boolean pUsingStab,
    /// boolean usingChainsaw, boolean usingVomit, boolean usingBreatheFire, boolean usingChomp)`.
    #[allow(clippy::too_many_arguments)]
    pub fn block(
        &self,
        client: &mut FantasyFootballClient,
        acting_player_id: &str,
        defender: Option<&Player>,
        using_stab: bool,
        using_chainsaw: bool,
        using_vomit: bool,
        using_breathe_fire: bool,
        using_chomp: bool,
    ) {
        client.communication_mut().send_block(
            acting_player_id,
            defender,
            using_stab,
            using_chainsaw,
            using_vomit,
            using_breathe_fire,
            using_chomp,
        );
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, defender: Option<&Player>) -> InteractionResult {
        self.player_interaction_full(client, defender, false, false)
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pDefender, boolean pDoBlitz, boolean multiBlock)`.
    pub fn player_interaction_full(
        &self,
        client: &mut FantasyFootballClient,
        defender: Option<&Player>,
        do_blitz: bool,
        multi_block: bool,
    ) -> InteractionResult {
        let defender = match defender {
            Some(d) => d,
            None => return InteractionResult::ignore(),
        };

        let (acting_player, player_state) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let player_state = acting_player.player_id.as_deref().and_then(|id| game.field_model.player_state(id));
                (acting_player, player_state)
            }
            None => return InteractionResult::ignore(),
        };

        let is_blockable = match client.game() {
            Some(game) => self.is_blockable(game, defender),
            None => false,
        };

        if is_blockable
            && (!do_blitz
                || player_can_not_move_placeholder(player_state)
                || client.game().map(|g| ffb_model::util::util_player::UtilPlayer::is_next_move_possible(g, false)).unwrap_or(false))
        {
            let defender_coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&defender.id));

            let attacker = acting_player.player_id.as_deref().and_then(|id| client.game()?.player(id)).cloned();
            let provides_block_alternative = attacker
                .as_ref()
                .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::PROVIDES_BLOCK_ALTERNATIVE))
                .unwrap_or(false);
            let gored_and_blitz = self.is_gored_available(client) && do_blitz;

            if provides_block_alternative || gored_and_blitz {
                let context = self.block_action_context(client, &acting_player, multi_block);
                return InteractionResult::select_action(context);
            } else if let Some(defender_coordinate) = defender_coordinate {
                let has_decoration = client
                    .game()
                    .and_then(|g| g.field_model.get_dice_decoration_at(defender_coordinate))
                    .is_some();
                if has_decoration {
                    if let Some(acting_player_id) = &acting_player.player_id {
                        self.block(client, acting_player_id, Some(defender), false, false, false, false, false);
                    }
                    return InteractionResult::handled();
                }
            }
        }
        InteractionResult::ignore()
    }

    /// java: `public boolean isBlockable(Game game, Player<?> pPlayer)`.
    pub fn is_blockable(&self, game: &Game, player: &Player) -> bool {
        let acting_player = &game.acting_player;
        let defender_coordinate = game.field_model.player_coordinate(&player.id);
        let attacker_coordinate = acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
        if !self.is_valid_blitz_target(game, Some(player)) {
            return false;
        }
        let adjacent_or_vicious_vines = match (defender_coordinate, attacker_coordinate) {
            (Some(d), Some(a)) => d.is_adjacent(a),
            _ => false,
        } || acting_player.player_action == Some(PlayerAction::ViciousVines);
        if !adjacent_or_vicious_vines {
            return false;
        }
        match defender_coordinate {
            Some(coord) => game.field_model.get_dice_decoration_at(coord).is_some(),
            None => false,
        }
    }

    /// java: `public boolean isValidBlitzTarget(Game game, Player<?> pPlayer)`.
    pub fn is_valid_blitz_target(&self, game: &Game, player: Option<&Player>) -> bool {
        let player = match player {
            Some(p) => p,
            None => return false,
        };
        let field_model = &game.field_model;
        let defender_state = match field_model.player_state(&player.id) {
            Some(s) => s,
            None => return false,
        };
        let defender_coordinate = field_model.player_coordinate(&player.id);
        defender_state.can_be_blocked()
            && game.team_away.has_player(&player.id)
            && defender_coordinate.is_some()
            && (field_model.target_selection_state.is_none()
                || field_model
                    .target_selection_state
                    .as_ref()
                    .and_then(|s| s.get_selected_player_id())
                    .map(|id| id == &player.id)
                    .unwrap_or(false))
    }

    /// java: `public boolean isGoredAvailable()`.
    pub fn is_gored_available(&self, client: &FantasyFootballClient) -> bool {
        let game = match client.game() {
            Some(g) => g,
            None => return false,
        };
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
        let dice_decoration: Option<&DiceDecoration> = target_coordinate.and_then(|c| game.field_model.get_dice_decoration_at(c));
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
}

impl LogicModule for BlockLogicExtension {
    /// java: `public ClientStateId getId() { throw new UnsupportedOperationException(...); }`.
    fn get_id(&self) -> ffb_model::enums::ClientStateId {
        unimplemented!("getId not implemented for extensions")
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::STAB);
        actions.insert(ClientAction::CHAINSAW);
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::BREATHE_FIRE);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        // java: plugin.availableActions() — gap, contributes nothing extra.
        actions
    }

    /// java: `public ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        if is_treacherous_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::TREACHEROUS);
        }
        if is_wisdom_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::WISDOM);
        }
        if is_raiding_party_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::RAIDING_PARTY);
        }
        if is_look_into_my_eyes_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
        }
        if is_baleful_hex_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BALEFUL_HEX);
        }
        if is_black_ink_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BLACK_INK);
        }
        if is_catch_of_the_day_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
        }
        if is_zoat_gaze_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
        }
        // java: plugin.actionContext(actingPlayer, actionContext, this) — gap; pass through
        // unchanged.
        action_context
    }

    /// java: `public void performAvailableAction(Player<?> player, ClientAction action)` — the
    /// inherent method above implements the real logic; this trait method is unused because
    /// `BlockLogicExtension` is a mixin, not dispatched through `LogicModule::perform` directly
    /// in Java either (its `getId()` throws), matching `logic_module::show_grid_for_ktm`-style
    /// unreachable defaults.
    fn perform_available_action(
        &mut self,
        client: &mut FantasyFootballClient,
        player: &Player,
        action: ClientAction,
    ) {
        BlockLogicExtension::perform_available_action(self, client, player, action);
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

    #[test]
    fn get_id_panics_for_extension() {
        let ext = BlockLogicExtension::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ext.get_id()));
        assert!(result.is_err());
    }

    #[test]
    fn available_actions_contains_block_and_treacherous() {
        let ext = BlockLogicExtension::new();
        let actions = ext.available_actions();
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::TREACHEROUS));
        assert!(actions.contains(&ClientAction::AUTO_GAZE_ZOAT));
    }

    #[test]
    fn action_context_empty_without_any_skills() {
        let ext = BlockLogicExtension::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = ext.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn is_valid_blitz_target_false_for_none() {
        let ext = BlockLogicExtension::new();
        let game = make_game();
        assert!(!ext.is_valid_blitz_target(&game, None));
    }

    #[test]
    fn is_valid_blitz_target_requires_away_team_membership() {
        let ext = BlockLogicExtension::new();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let player = game.player("h1").unwrap().clone();
        // h1 is on the home team, not away -> not a valid blitz target.
        assert!(!ext.is_valid_blitz_target(&game, Some(&player)));
    }

    #[test]
    fn is_blockable_false_without_adjacency() {
        let ext = BlockLogicExtension::new();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10));
        let defender = game.player("a1").unwrap().clone();
        assert!(!ext.is_blockable(&game, &defender));
    }

    #[test]
    fn is_gored_available_false_without_target_selection_state() {
        let client_params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let mut client = FantasyFootballClient::new(client_params);
        client.set_game(make_game());
        let ext = BlockLogicExtension::new();
        assert!(!ext.is_gored_available(&client));
    }

    #[test]
    fn player_interaction_full_ignores_when_defender_none() {
        let client_params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let mut client = FantasyFootballClient::new(client_params);
        client.set_game(make_game());
        let ext = BlockLogicExtension::new();
        let result = ext.player_interaction_full(&mut client, None, false, false);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
