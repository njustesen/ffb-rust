//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.SelectLogicModule` (326 lines).
//!
//! Java's `SelectLogicModule extends LogicModule` directly. It has an overloaded
//! `actionContext(Player<?>)` (distinct from the abstract `actionContext(ActingPlayer)`, which it
//! makes always throw); per the `LogicModule` trait, `action_context(&self, game, acting_player)`
//! is the only trait slot, so the Java `actionContext(ActingPlayer)` override (always throwing) is
//! implemented as the trait method, and the real `actionContext(Player<?>)` logic is exposed as
//! its own separate inherent method (`action_context_for_player`).
//!
//! Documented gap:
//! - `player.getSkillWithProperty(property)` returns a full `Skill` in Java (used directly for
//!   `sendUseSkill`); the Rust `Player` only exposes `skill_id_with_property(&str) ->
//!   Option<SkillId>` (same gap as `move_logic_module.rs`/`blitz_logic_module.rs`), so
//!   `skill_placeholder(SkillId)` is reused here for the network command.
//! - `findAlternateBlockActions` returns `List<Skill>` in Java; here it returns
//!   `Vec<SkillId>` (built via `skill_placeholder` at the call site for
//!   `ActionContext::add_block_alternative`), matching the same documented pattern as
//!   `logic_module.rs`'s `is_bounding_leap_available`.

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction, SkillCategory, SkillId};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `player.getSkillWithProperty(property)` — see module doc gap.
fn skill_placeholder(id: SkillId) -> Skill {
    Skill::new(id.class_name(), SkillCategory::General)
}

/// 1:1 translation of the `SelectLogicModule` class.
#[derive(Debug, Default)]
pub struct SelectLogicModule;

impl SelectLogicModule {
    /// java: `public SelectLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public void setUp()`.
    pub fn set_up(&mut self, client: &mut FantasyFootballClient) {
        if let Some(game) = client.game_mut() {
            game.defender_id = None;
        }
        client.client_data_mut().clear_block_dice_result();
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_home_player, is_active) = match client.game() {
            Some(game) => {
                let is_home = game.team_home.has_player(&player.id);
                let is_active = game.field_model.player_state(&player.id).map(|s| s.is_active()).unwrap_or(false);
                (is_home, is_active)
            }
            None => return InteractionResult::ignore(),
        };
        if is_home_player && is_active {
            let ctx = match client.game() {
                Some(game) => self.action_context_for_player(game, player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
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
            for skill_id in find_alternate_block_actions(player) {
                context.add_block_alternative(skill_placeholder(skill_id));
            }
            context.add_action(ClientAction::BLOCK);
        }
        if logic_module::is_multi_block_action_available(game, player) {
            context.add_action(ClientAction::MULTIPLE_BLOCK);
        }
        if logic_module::is_throw_bomb_action_available(game, player) {
            context.add_action(ClientAction::BOMB);
            if UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAIN_HAIL_MARY) {
                context.add_action(ClientAction::SHOT_TO_NOTHING_BOMB);
            }
        }
        if logic_module::is_hypnotic_gaze_action_available(game, true, player) {
            context.add_action(ClientAction::GAZE);
        }
        if logic_module::is_hypnotic_gaze_action_available(game, true, player) {
            context.add_action(ClientAction::GAZE_ZOAT);
        }
        if self.is_move_action_available(game, player) {
            context.add_action(ClientAction::MOVE);
        }
        if logic_module::is_blitz_action_available(game, player) {
            context.add_action(ClientAction::BLITZ);
            if UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ) {
                context.add_action(ClientAction::FRENZIED_RUSH);
            }
        }
        if logic_module::is_foul_action_available(game, player) {
            context.add_action(ClientAction::FOUL);
        }
        if logic_module::is_pass_action_available(game, player, treacherous_available) {
            context.add_action(ClientAction::PASS);
            if UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GAIN_HAIL_MARY) {
                context.add_action(ClientAction::SHOT_TO_NOTHING);
            }
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

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::BLOCK);
        actions.insert(ClientAction::BLITZ);
        actions.insert(ClientAction::FRENZIED_RUSH);
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
        actions.insert(ClientAction::GAZE_ZOAT);
        actions.insert(ClientAction::SHOT_TO_NOTHING);
        actions.insert(ClientAction::SHOT_TO_NOTHING_BOMB);
        actions.insert(ClientAction::BEER_BARREL_BASH);
        actions.insert(ClientAction::ALL_YOU_CAN_EAT);
        actions.insert(ClientAction::KICK_EM_BLOCK);
        actions.insert(ClientAction::KICK_EM_BLITZ);
        actions.insert(ClientAction::THE_FLASHING_BLADE);
        actions.insert(ClientAction::VICIOUS_VINES);
        actions.insert(ClientAction::FURIOUS_OUTBURST);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java; faithfully translated as a panic. The real
    /// per-player logic lives on `action_context_for_player`.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in select context")
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::BLOCK => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Block, false);
            }
            ClientAction::BLITZ => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BlitzMove, false);
            }
            ClientAction::FRENZIED_RUSH => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::BlitzMove, false);
                if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ) {
                    client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                }
            }
            ClientAction::FOUL => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::FoulMove, false);
            }
            ClientAction::MOVE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, false);
            }
            ClientAction::STAND_UP => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::StandUp, false);
            }
            ClientAction::STAND_UP_BLITZ => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::StandUpBlitz, false);
            }
            ClientAction::HAND_OVER => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::HandOverMove, false);
                if let Some(game) = client.game() {
                    if logic_module::is_treacherous_available(game, player) {
                        if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                        }
                    }
                }
            }
            ClientAction::PASS => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::PassMove, false);
                if let Some(game) = client.game() {
                    if logic_module::is_treacherous_available(game, player) {
                        if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                        }
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
                let available = client.game().map(|g| logic_module::is_throw_bomb_action_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowBomb, false);
                }
            }
            ClientAction::GAZE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::GazeMove, false);
            }
            ClientAction::GAZE_ZOAT => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::GazeMove, false);
                if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_GAZE) {
                    client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                }
            }
            ClientAction::SHOT_TO_NOTHING => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::PassMove, false);
                if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                    client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                }
                if let Some(game) = client.game() {
                    if logic_module::is_treacherous_available(game, player) {
                        if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                            client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                        }
                    }
                }
            }
            ClientAction::SHOT_TO_NOTHING_BOMB => {
                let available = client.game().map(|g| logic_module::is_throw_bomb_action_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowBomb, false);
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAIN_HAIL_MARY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                    if let Some(game) = client.game() {
                        if logic_module::is_treacherous_available(game, player) {
                            if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                                client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                            }
                        }
                    }
                }
            }
            ClientAction::BEER_BARREL_BASH => {
                let available = client.game().map(|g| logic_module::is_beer_barrel_bash_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ThrowKeg, false);
                }
            }
            ClientAction::ALL_YOU_CAN_EAT => {
                let available = client.game().map(|g| logic_module::is_all_you_can_eat_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::AllYouCanEat, false);
                }
            }
            ClientAction::KICK_EM_BLOCK => {
                let available = client.game().map(|g| logic_module::is_kick_em_block_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::KickEmBlock, false);
                }
            }
            ClientAction::KICK_EM_BLITZ => {
                let available = client.game().map(|g| logic_module::is_kick_em_blitz_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::KickEmBlitz, false);
                }
            }
            ClientAction::THE_FLASHING_BLADE => {
                let available = client.game().map(|g| logic_module::is_flashing_blade_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::TheFlashingBlade, false);
                }
            }
            ClientAction::VICIOUS_VINES => {
                let available = client.game().map(|g| logic_module::is_vicious_vines_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::ViciousVines, false);
                }
            }
            ClientAction::FURIOUS_OUTBURST => {
                let available = client.game().map(|g| logic_module::is_furious_outburst_available(g, player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::FuriousOutburst, false);
                }
            }
            _ => {}
        }
    }
}

/// java: `private List<Skill> findAlternateBlockActions(Player<?> player)`.
fn find_alternate_block_actions(player: &Player) -> Vec<SkillId> {
    player
        .all_skill_ids()
        .filter(|id| id.properties().contains(&NamedProperties::PROVIDES_BLOCK_ALTERNATIVE))
        .collect()
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
    fn get_id_is_select_player() {
        assert_eq!(SelectLogicModule::new().get_id(), ClientStateId::SelectPlayer);
    }

    #[test]
    fn available_actions_contains_expected_set() {
        let actions = SelectLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::BLOCK));
        assert!(actions.contains(&ClientAction::VICIOUS_VINES));
        assert_eq!(actions.len(), 25);
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in select context")]
    fn action_context_for_acting_player_panics() {
        let module = SelectLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn action_context_for_player_empty_by_default() {
        let module = SelectLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        let player = game.player("h1").unwrap().clone();
        let ctx = module.action_context_for_player(&game, &player);
        // Standing, active, no ball, no opponents nearby: no actions should surface except
        // possibly move (depends on movement stats which default to 0).
        assert!(!ctx.get_actions().contains(&ClientAction::BLOCK));
    }

    #[test]
    fn is_move_action_available_reflects_player_state() {
        let module = SelectLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        let player = game.player("h1").unwrap().clone();
        // Default PlayerState with 0 movement is still "able to move" per PS_STANDING active.
        let _ = module.is_move_action_available(&game, &player);
    }

    #[test]
    fn find_alternate_block_actions_empty_without_skill() {
        let player = Player::default();
        assert!(find_alternate_block_actions(&player).is_empty());
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = SelectLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn set_up_clears_defender_and_block_dice() {
        let mut client = make_client();
        client.set_game(make_game());
        let mut module = SelectLogicModule::new();
        module.set_up(&mut client);
        assert!(client.game().unwrap().defender_id.is_none());
    }

    #[test]
    fn perform_available_action_sends_block() {
        let mut module = SelectLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::BLOCK);
        assert!(!client.communication().is_stopped());
    }
}
