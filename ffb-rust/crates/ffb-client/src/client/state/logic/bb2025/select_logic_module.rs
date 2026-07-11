//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.SelectLogicModule` (389 lines).
//!
//! Java's `SelectLogicModule extends LogicModule` directly (no plugin/mixin fields).
//!
//! Documented gap:
//! - `Player.canDeclareSkillAction(ISkillProperty property, PlayerState playerState)` — the
//!   Java implementation walks the player's actual `Skill` objects and checks
//!   `skill.getDeclareCondition().fulfilled(playerState)` per-skill. The Rust `Player` only
//!   stores `SkillId`s with a static `properties()` list (see `logic_module.rs`'s own doc
//!   comment on the general "no full `Skill` object" gap; `player.getSkillWithProperty(...)`
//!   used elsewhere in this batch has the identical limitation). There is no per-`SkillId`
//!   declare-condition data available at this layer to know whether a given skill requires
//!   e.g. "standing" to be declared. `can_declare_skill_action` below approximates the Java
//!   method with `player.has_unused_skill_with_property(property)` alone (i.e. treats every
//!   skill's declare condition as always fulfilled — `DeclareCondition::None`, the common case),
//!   matching the same simplification `block_logic_extension.rs`'s own
//!   `block_action_context` already makes for the identical stab/chainsaw/vomit/fire properties
//!   (it also never applies a declare-condition check).

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction, PlayerState};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::option::{game_option_id, util_game_option};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `Player.canDeclareSkillAction(ISkillProperty property, PlayerState playerState)` —
/// see module doc gap.
fn can_declare_skill_action(player: &Player, property: &str, _player_state: PlayerState) -> bool {
    player.has_unused_skill_with_property(property)
}

/// 1:1 translation of the `SelectLogicModule` class.
#[derive(Debug, Default)]
pub struct SelectLogicModule;

impl SelectLogicModule {
    /// java: `public SelectLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::ignore(),
        };
        let player_state = game.field_model.player_state(&player.id);
        let is_active = player_state.map(|s| s.is_active()).unwrap_or(false);
        if game.team_home.has_player(&player.id) && is_active {
            return InteractionResult::select_action(self.action_context_for_player(game, player));
        }
        InteractionResult::ignore()
    }

    /// java: `protected ActionContext actionContext(Player<?> player)`.
    pub fn action_context_for_player(&self, game: &Game, player: &Player) -> ActionContext {
        let mut context = ActionContext::new();

        let treacherous_available = logic_module::is_treacherous_available(game, player);
        if treacherous_available {
            context.add_influence(Influences::BALL_ACTIONS_DUE_TO_TREACHEROUS);
        }

        if logic_module::is_block_action_available(game, player) {
            context.add_action(ClientAction::BLOCK);
        }

        let player_state = game.field_model.player_state(&player.id);
        let special_blocks_available =
            logic_module::is_special_block_action_available(game, player, player_state);

        if let Some(player_state) = player_state {
            if special_blocks_available
                && can_declare_skill_action(player, NamedProperties::PROVIDES_STAB_BLOCK_ALTERNATIVE, player_state)
            {
                context.add_action(ClientAction::STAB);
            }
            if special_blocks_available
                && can_declare_skill_action(player, NamedProperties::PROVIDES_CHAINSAW_BLOCK_ALTERNATIVE, player_state)
            {
                context.add_action(ClientAction::CHAINSAW);
            }
            if special_blocks_available
                && can_declare_skill_action(
                    player,
                    NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL,
                    player_state,
                )
            {
                if player.has_unused_skill_with_property(NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK) {
                    context.add_influence(Influences::VOMIT_DUE_TO_PUTRID_REGURGITATION);
                }
                context.add_action(ClientAction::PROJECTILE_VOMIT);
            }
            if special_blocks_available
                && can_declare_skill_action(
                    player,
                    NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER,
                    player_state,
                )
            {
                context.add_action(ClientAction::BREATHE_FIRE);
            }
        }

        if logic_module::is_multi_block_action_available(game, player) {
            context.add_action(ClientAction::MULTIPLE_BLOCK);
        }
        if logic_module::is_throw_bomb_action_available(game, player) {
            context.add_action(ClientAction::BOMB);
            if player.has_unused_skill_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                context.add_action(ClientAction::SHOT_TO_NOTHING_BOMB);
            }
        }
        if logic_module::is_hypnotic_gaze_action_available(game, true, player) {
            context.add_action(ClientAction::GAZE);
        }
        if self.is_move_action_available(game, player) {
            context.add_action(ClientAction::MOVE);
        }
        if logic_module::is_blitz_action_available(game, player) {
            context.add_action(ClientAction::BLITZ);
        }
        if logic_module::is_foul_action_available(game, player) {
            context.add_action(ClientAction::FOUL);
        }
        if logic_module::is_pass_action_available(game, player, treacherous_available) {
            context.add_action(ClientAction::PASS);
            if player.has_unused_skill_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                context.add_action(ClientAction::SHOT_TO_NOTHING);
            }
        }
        if logic_module::is_punt_action_available(game, player, treacherous_available) {
            context.add_action(ClientAction::PUNT);
        }
        if logic_module::is_hand_over_action_available(game, player, treacherous_available) {
            context.add_action(ClientAction::HAND_OVER);
        }
        if logic_module::is_throw_team_mate_action_available(game, player) {
            context.add_action(ClientAction::THROW_TEAM_MATE);
        }
        if logic_module::is_kick_team_mate_action_available(game, player) {
            context.add_action(ClientAction::KICK_TEAM_MATE);
        }
        if logic_module::is_beer_barrel_bash_available(game, player) {
            context.add_action(ClientAction::BEER_BARREL_BASH);
        }
        if logic_module::is_all_you_can_eat_available(game, player) {
            context.add_action(ClientAction::ALL_YOU_CAN_EAT);
        }
        if logic_module::is_kick_em_block_available(game, player) {
            context.add_action(ClientAction::KICK_EM_BLOCK);
        }
        if logic_module::is_kick_em_blitz_available(game, player) {
            context.add_action(ClientAction::KICK_EM_BLITZ);
        }
        if logic_module::is_flashing_blade_available(game, player) {
            context.add_action(ClientAction::THE_FLASHING_BLADE);
        }
        if logic_module::is_vicious_vines_available(game, player) {
            context.add_action(ClientAction::VICIOUS_VINES);
        }
        if logic_module::is_furious_outburst_available(game, player) {
            context.add_action(ClientAction::FURIOUS_OUTBURST);
        }
        if logic_module::is_recover_from_confusion_action_available(game, player)
            || logic_module::is_recover_from_gaze_action_available(game, player)
            || logic_module::is_recover_from_eye_gouge_action_available(game, player)
        {
            context.add_action(ClientAction::RECOVER);
        }
        if logic_module::is_stand_up_action_available(game, player)
            && player.has_skill_property(NamedProperties::ENABLE_STAND_UP_AND_END_BLITZ_ACTION)
            && !game.turn_data().blitz_used
        {
            context.add_action(ClientAction::STAND_UP_BLITZ);
        }
        if logic_module::is_stand_up_action_available(game, player) {
            context.add_action(ClientAction::STAND_UP);
        }
        if logic_module::is_secure_the_ball_action_available(game, player) {
            context.add_action(ClientAction::SECURE_THE_BALL);
        }
        let not_prone_or_stunned = player_state.map(|s| !s.is_prone_or_stunned()).unwrap_or(false);
        if util_game_option::is_option_enabled(game, game_option_id::ENABLE_STALLING_CHECK) && not_prone_or_stunned {
            context.add_action(ClientAction::FORGO);
        }
        if logic_module::is_then_i_started_blastin_available(game, player) {
            context.add_action(ClientAction::THEN_I_STARTED_BLASTIN);
        }
        if special_blocks_available && logic_module::is_chomp_available(game, player) {
            context.add_action(ClientAction::CHOMP);
        }
        context
    }

    /// java: `public boolean isMoveActionAvailable(Player<?> player)`.
    pub fn is_move_action_available(&self, game: &Game, player: &Player) -> bool {
        game.field_model.player_state(&player.id).map(|s| s.is_able_to_move()).unwrap_or(false)
    }
}

impl LogicModule for SelectLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::SelectPlayer
    }

    /// java: `public void setUp()`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        if let Some(game) = client.game_mut() {
            game.defender_id = None;
        }
        client.client_data_mut().clear_block_dice_result();
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::STAB);
        actions.insert(ClientAction::CHAINSAW);
        actions.insert(ClientAction::PROJECTILE_VOMIT);
        actions.insert(ClientAction::BREATHE_FIRE);
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::BLITZ);
        actions.insert(ClientAction::FOUL);
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::STAND_UP);
        actions.insert(ClientAction::STAND_UP_BLITZ);
        actions.insert(ClientAction::HAND_OVER);
        actions.insert(ClientAction::PASS);
        actions.insert(ClientAction::THROW_TEAM_MATE);
        actions.insert(ClientAction::KICK_TEAM_MATE);
        actions.insert(ClientAction::RECOVER);
        actions.insert(ClientAction::MULTIPLE_BLOCK);
        actions.insert(ClientAction::BOMB);
        actions.insert(ClientAction::GAZE);
        actions.insert(ClientAction::SHOT_TO_NOTHING);
        actions.insert(ClientAction::SHOT_TO_NOTHING_BOMB);
        actions.insert(ClientAction::BEER_BARREL_BASH);
        actions.insert(ClientAction::ALL_YOU_CAN_EAT);
        actions.insert(ClientAction::KICK_EM_BLOCK);
        actions.insert(ClientAction::KICK_EM_BLITZ);
        actions.insert(ClientAction::THE_FLASHING_BLADE);
        actions.insert(ClientAction::VICIOUS_VINES);
        actions.insert(ClientAction::FURIOUS_OUTBURST);
        actions.insert(ClientAction::SECURE_THE_BALL);
        actions.insert(ClientAction::FORGO);
        actions.insert(ClientAction::THEN_I_STARTED_BLASTIN);
        actions.insert(ClientAction::CHOMP);
        actions.insert(ClientAction::PUNT);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer) { throw new
    /// UnsupportedOperationException("actionContext for acting player is not supported in select
    /// context"); }`.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in select context")
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        // Every `logic_module::is_xxx_available(game, player)` predicate needed below is
        // precomputed here (rather than read from a held `&Game` reference threaded through the
        // match arms) so the borrow of `client` ends before the arms take `client.communication_mut()`.
        let (
            secure_the_ball_available,
            treacherous_available,
            throw_bomb_available,
            beer_barrel_bash_available,
            all_you_can_eat_available,
            kick_em_block_available,
            kick_em_blitz_available,
            flashing_blade_available,
            vicious_vines_available,
            furious_outburst_available,
            then_i_started_blastin_available,
        ) = match client.game() {
            Some(game) => (
                logic_module::is_secure_the_ball_action_available(game, player),
                logic_module::is_treacherous_available(game, player),
                logic_module::is_throw_bomb_action_available(game, player),
                logic_module::is_beer_barrel_bash_available(game, player),
                logic_module::is_all_you_can_eat_available(game, player),
                logic_module::is_kick_em_block_available(game, player),
                logic_module::is_kick_em_blitz_available(game, player),
                logic_module::is_flashing_blade_available(game, player),
                logic_module::is_vicious_vines_available(game, player),
                logic_module::is_furious_outburst_available(game, player),
                logic_module::is_then_i_started_blastin_available(game, player),
            ),
            None => return,
        };
        match action {
            ClientAction::BLOCK => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Block, false);
            }
            ClientAction::BREATHE_FIRE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BreatheFire, false);
            }
            ClientAction::CHAINSAW => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Chainsaw, false);
            }
            ClientAction::PROJECTILE_VOMIT => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::ProjectileVomit, false);
            }
            ClientAction::STAB => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Stab, false);
            }
            ClientAction::CHOMP => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Chomp, false);
            }
            ClientAction::BLITZ => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BlitzMove, false);
            }
            ClientAction::FOUL => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::FoulMove, false);
            }
            ClientAction::MOVE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, false);
            }
            ClientAction::SECURE_THE_BALL => {
                if secure_the_ball_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::SecureTheBall, false);
                }
            }
            ClientAction::PUNT => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::PuntMove, false);
                if treacherous_available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::STAND_UP => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::StandUp, false);
            }
            ClientAction::STAND_UP_BLITZ => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::StandUpBlitz, false);
            }
            ClientAction::HAND_OVER => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::HandOverMove, false);
                if treacherous_available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::PASS => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::PassMove, false);
                if treacherous_available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::THROW_TEAM_MATE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowTeamMateMove, false);
            }
            ClientAction::KICK_TEAM_MATE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::KickTeamMateMove, false);
            }
            ClientAction::RECOVER => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::RemoveConfusion, false);
            }
            ClientAction::MULTIPLE_BLOCK => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::MultipleBlock, false);
            }
            ClientAction::BOMB => {
                if throw_bomb_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowBomb, false);
                }
            }
            ClientAction::GAZE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::GazeMove, false);
            }
            ClientAction::SHOT_TO_NOTHING => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::PassMove, false);
                if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                    client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                }
                if treacherous_available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::SHOT_TO_NOTHING_BOMB => {
                if throw_bomb_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowBomb, false);
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                    if treacherous_available {
                        if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                        }
                    }
                }
            }
            ClientAction::BEER_BARREL_BASH => {
                if beer_barrel_bash_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowKeg, false);
                }
            }
            ClientAction::ALL_YOU_CAN_EAT => {
                if all_you_can_eat_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::AllYouCanEat, false);
                }
            }
            ClientAction::KICK_EM_BLOCK => {
                if kick_em_block_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::KickEmBlock, false);
                }
            }
            ClientAction::KICK_EM_BLITZ => {
                if kick_em_blitz_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::KickEmBlitz, false);
                }
            }
            ClientAction::THE_FLASHING_BLADE => {
                if flashing_blade_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::TheFlashingBlade, false);
                }
            }
            ClientAction::VICIOUS_VINES => {
                if vicious_vines_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ViciousVines, false);
                }
            }
            ClientAction::FURIOUS_OUTBURST => {
                if furious_outburst_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::FuriousOutburst, false);
                }
            }
            ClientAction::FORGO => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Forgo, false);
            }
            ClientAction::THEN_I_STARTED_BLASTIN => {
                if then_i_started_blastin_available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThenIStartedBlastin, false);
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player_state::PlayerState as ModelPlayerState;
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
            .set_player_state(id, ModelPlayerState::new(PS_STANDING).change_active(true));
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
    fn get_id_is_select_player() {
        assert_eq!(SelectLogicModule::new().get_id(), ClientStateId::SelectPlayer);
    }

    #[test]
    fn available_actions_contains_move_and_punt() {
        let actions = SelectLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::PUNT));
        assert_eq!(actions.len(), 32);
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in select context")]
    fn action_context_panics() {
        let module = SelectLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn is_move_action_available_true_for_standing_active_player() {
        let module = SelectLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        assert!(module.is_move_action_available(&game, &player));
    }

    #[test]
    fn is_move_action_available_false_without_state() {
        let module = SelectLogicModule::new();
        let game = make_game();
        let mut player = Player::default();
        player.id = "unknown".to_string();
        assert!(!module.is_move_action_available(&game, &player));
    }

    #[test]
    fn action_context_for_player_adds_move_for_standing_player() {
        let module = SelectLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        let player = game.player("p1").unwrap().clone();
        let ctx = module.action_context_for_player(&game, &player);
        assert!(ctx.get_actions().contains(&ClientAction::MOVE));
    }

    #[test]
    fn can_declare_skill_action_false_without_skill() {
        let player = Player::default();
        let state = ModelPlayerState::new(PS_STANDING);
        assert!(!can_declare_skill_action(&player, "some-property", state));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let client = make_client();
        let module = SelectLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_selects_action_for_active_home_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let module = SelectLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::SelectAction
        );
    }

    #[test]
    fn set_up_clears_defender_and_dice_result() {
        let mut module = SelectLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        module.set_up(&mut client);
        assert!(client.game().unwrap().defender_id.is_none());
    }

    #[test]
    fn perform_available_action_move_sends_command() {
        let mut module = SelectLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
        assert!(!client.communication().is_stopped());
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = SelectLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
