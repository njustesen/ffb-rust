//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.FoulLogicModule` (211 lines).
//!
//! Java's `FoulLogicModule extends MoveLogicModule`, overriding `getId`/`playerInteraction`/
//! `playerPeek`/`availableActions`/`performAvailableAction` and adding `playerSelected`/`foul`/
//! `foulActionContext`/`bloodlustActionContext`. Per the `MoveLogicModule` convention (see that
//! module's own doc comment), the inherited `playerInteraction` needs `&mut
//! FantasyFootballClient`, so this is translated as a struct composing `MoveLogicModule` and
//! delegating via its inherent (non-trait) method.
//!
//! Documented gaps:
//! - `UtilPlayer.isFoulable(Game, Player<?>)` has no equivalent yet in the Rust
//!   `ffb_model::util::util_player::UtilPlayer` (only `isNextMoveGoingForIt`/`hasBall`/etc. are
//!   translated there so far). Reimplemented here as a private `is_foulable` helper mirroring
//!   `UtilPlayer.java` exactly (same pattern already used for `UtilRangeRuler.createRangeRuler`
//!   in `dump_off_logic_module.rs`).
//! - `Player.hasActiveEnhancement(skill)` — no enhancement-tracking exists on the Rust `Player`
//!   (same gap as `logic_module.rs`'s `is_incorporeal_available_ap`); conservatively `false`, so
//!   `INCORPOREAL`'s toggle always requests activation (`!active` == `true`).

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// java: `UtilPlayer.isFoulable(Game pGame, Player<?> pPlayer)` — see module doc gap.
pub fn is_foulable(game: &Game, player: Option<&Player>) -> bool {
    let player = match player {
        Some(p) => p,
        None => return false,
    };
    let acting_player = &game.acting_player;
    let attacker_id = match acting_player.player_id.as_deref() {
        Some(id) => id,
        None => return false,
    };
    let defender_state = match game.field_model.player_state(&player.id) {
        Some(s) => s,
        None => return false,
    };
    let defender_coordinate = game.field_model.player_coordinate(&player.id);
    let attacker_coordinate = game.field_model.player_coordinate(attacker_id);

    (defender_state.base() == ffb_model::enums::PS_PRONE || defender_state.base() == ffb_model::enums::PS_STUNNED)
        && game.team_away.has_player(&player.id)
        && match (defender_coordinate, attacker_coordinate) {
            (Some(d), Some(a)) => d.is_adjacent(a),
            _ => false,
        }
        && !player.has_skill_property(NamedProperties::PREVENT_BEING_FOULED)
}

/// 1:1 translation of the `FoulLogicModule` class.
#[derive(Debug, Default)]
pub struct FoulLogicModule {
    move_logic: MoveLogicModule,
}

impl FoulLogicModule {
    /// java: `public FoulLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (acting_player_id, suffering_blood_lust, is_next_move_going_for_it, goes_for_it) =
            match client.game() {
                Some(game) => (
                    game.acting_player.player_id.clone(),
                    game.acting_player.suffering_blood_lust,
                    UtilPlayer::is_next_move_going_for_it(game),
                    game.acting_player.goes_for_it,
                ),
                None => return InteractionResult::ignore(),
            };

        if acting_player_id.as_deref() == Some(player.id.as_str()) {
            if suffering_blood_lust {
                let ap = client.game().unwrap().acting_player.clone();
                return InteractionResult::select_action(bloodlust_action_context(&ap));
            }
            return self.move_logic.player_interaction(client, player);
        }

        if is_next_move_going_for_it && !goes_for_it {
            let (game_ref, ap) = match client.game() {
                Some(game) => (game, game.acting_player.clone()),
                None => return InteractionResult::ignore(),
            };
            let ctx = self.action_context(game_ref, &ap);
            InteractionResult::select_action(ctx)
        } else {
            self.player_selected(client, player)
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let foulable = match client.game() {
            Some(game) => is_foulable(game, Some(player)),
            None => false,
        };
        if foulable {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerSelected(Player<?> defender)`.
    pub fn player_selected(&self, client: &mut FantasyFootballClient, defender: &Player) -> InteractionResult {
        let (do_foul, provides_alternative) = match client.game() {
            Some(game) => {
                let foulable = is_foulable(game, Some(defender));
                let provides_alternative = game
                    .acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::PROVIDES_FOULING_ALTERNATIVE))
                    .unwrap_or(false);
                (foulable, provides_alternative)
            }
            None => return InteractionResult::ignore(),
        };
        if do_foul {
            if provides_alternative {
                let ctx = match client.game() {
                    Some(game) => {
                        let ap = game.acting_player.clone();
                        foul_action_context(game, &ap)
                    }
                    None => ActionContext::new(),
                };
                InteractionResult::select_action(ctx)
            } else {
                self.foul(client, defender, false);
                InteractionResult::handled()
            }
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public void foul(Player<?> defender, boolean usingChainsaw)`.
    pub fn foul(&self, client: &mut FantasyFootballClient, defender: &Player, using_chainsaw: bool) {
        let acting_player_id = match client.game() {
            Some(game) => game.acting_player.player_id.clone(),
            None => return,
        };
        if let Some(id) = acting_player_id {
            client.communication_mut().send_foul(id, Some(defender), using_chainsaw);
        }
    }
}

/// java: `private ActionContext foulActionContext(ActingPlayer actingPlayer)`.
fn foul_action_context(game: &Game, acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
    let mut action_context = ActionContext::new();
    let provides_chainsaw_alternative = acting_player
        .player_id
        .as_deref()
        .and_then(|id| game.player(id))
        .map(|p| p.has_skill_property(NamedProperties::PROVIDES_CHAINSAW_FOULING_ALTERNATIVE))
        .unwrap_or(false);
    if provides_chainsaw_alternative {
        action_context.add_action(ClientAction::CHAINSAW);
    }
    action_context.add_action(ClientAction::FOUL);
    action_context
}

/// java: `private ActionContext bloodlustActionContext(ActingPlayer actingPlayer)`.
fn bloodlust_action_context(acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
    let mut action_context = ActionContext::new();
    if acting_player.suffering_blood_lust {
        action_context.add_action(ClientAction::MOVE);
        action_context.add_action(ClientAction::END_MOVE);
    }
    action_context
}

impl LogicModule for FoulLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Foul
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::JUMP);
        actions.insert(ClientAction::MOVE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::BOUNDING_LEAP);
        actions.insert(ClientAction::FOUL);
        actions.insert(ClientAction::CHAINSAW);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        actions.insert(ClientAction::INCORPOREAL);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — inherited
    /// unchanged from `MoveLogicModule` (not overridden in Java).
    fn action_context(&self, game: &Game, acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return,
        };

        match action {
            ClientAction::END_MOVE => {
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s documented gap.
            }
            ClientAction::JUMP => {
                let jump_ok = client
                    .game()
                    .map(|g| logic_module::is_jump_available_as_next_move(g, &acting_player, false))
                    .unwrap_or(false);
                if jump_ok {
                    if let Some(pa) = acting_player.player_action {
                        client.communication_mut().send_acting_player(Some(player), pa, !acting_player.jumping);
                    }
                }
            }
            ClientAction::MOVE => {
                if acting_player.suffering_blood_lust {
                    client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, acting_player.jumping);
                }
            }
            ClientAction::FOUL => {
                self.foul(client, player, false);
            }
            ClientAction::CHAINSAW => {
                self.foul(client, player, true);
            }
            ClientAction::TREACHEROUS => {
                let available =
                    client.game().map(|g| logic_module::is_treacherous_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                let available =
                    client.game().map(|g| logic_module::is_wisdom_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                let available =
                    client.game().map(|g| logic_module::is_raiding_party_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                let available =
                    client.game().map(|g| logic_module::is_look_into_my_eyes_available(g, player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                let available =
                    client.game().map(|g| logic_module::is_baleful_hex_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                let available =
                    client.game().map(|g| logic_module::is_black_ink_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                let available =
                    client.game().map(|g| logic_module::is_catch_of_the_day_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::BOUNDING_LEAP => {
                let skill_id = client.game().and_then(|g| logic_module::is_bounding_leap_available(g, &acting_player));
                if let Some(skill_id) = skill_id {
                    if let Some(id) = acting_player.player_id.clone() {
                        send_skill(client, skill_id, id);
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                let available =
                    client.game().map(|g| logic_module::is_zoat_gaze_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) =
                        player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        send_skill(client, skill_id, player.id.clone());
                    }
                }
            }
            ClientAction::INCORPOREAL => {
                let available =
                    client.game().map(|g| logic_module::is_incorporeal_available_ap(g, &acting_player)).unwrap_or(false);
                if available {
                    if let Some(skill_id) = acting_player
                        .player_id
                        .as_deref()
                        .and_then(|id| client.game().and_then(|g| g.player(id)))
                        .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_AVOID_DODGING))
                    {
                        // java: `player.hasActiveEnhancement(skill)` — see module doc gap;
                        // conservatively `false`.
                        let incorporeal_active = false;
                        if let Some(id) = acting_player.player_id.clone() {
                            send_skill_used(client, skill_id, id, !incorporeal_active);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// java: `player.getSkillWithProperty(property)` — see `move_logic_module.rs`'s own doc gap;
/// `SkillId::class_name()` used as a name-only placeholder since `send_use_skill` only
/// serializes `skill.get_name()` onto the wire.
fn send_skill(client: &mut FantasyFootballClient, skill_id: ffb_model::enums::SkillId, player_id: String) {
    let skill = ffb_model::model::skill::skill::Skill::new(skill_id.class_name(), ffb_model::enums::SkillCategory::General);
    client.communication_mut().send_use_skill(&skill, true, player_id);
}

fn send_skill_used(
    client: &mut FantasyFootballClient,
    skill_id: ffb_model::enums::SkillId,
    player_id: String,
    used: bool,
) {
    let skill = ffb_model::model::skill::skill::Skill::new(skill_id.class_name(), ffb_model::enums::SkillCategory::General);
    client.communication_mut().send_use_skill(&skill, used, player_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_PRONE, PS_STANDING};
    use ffb_model::model::acting_player::ActingPlayer;
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
    fn get_id_is_foul() {
        assert_eq!(FoulLogicModule::new().get_id(), ClientStateId::Foul);
    }

    #[test]
    fn is_foulable_false_without_player() {
        let game = make_game();
        assert!(!is_foulable(&game, None));
    }

    #[test]
    fn is_foulable_true_for_prone_adjacent_away_player() {
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "defender", FieldCoordinate::new(2, 1));
        game.field_model.set_player_state("defender", PlayerState::new(PS_PRONE));
        game.acting_player.player_id = Some("attacker".to_string());
        let defender = game.player("defender").unwrap().clone();
        assert!(is_foulable(&game, Some(&defender)));
    }

    #[test]
    fn is_foulable_false_for_standing_player() {
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "defender", FieldCoordinate::new(2, 1));
        game.acting_player.player_id = Some("attacker".to_string());
        let defender = game.player("defender").unwrap().clone();
        assert!(!is_foulable(&game, Some(&defender)));
    }

    #[test]
    fn bloodlust_action_context_empty_when_not_suffering() {
        let ap = ActingPlayer::new();
        let ctx = bloodlust_action_context(&ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn bloodlust_action_context_has_move_and_end_move() {
        let mut ap = ActingPlayer::new();
        ap.suffering_blood_lust = true;
        let ctx = bloodlust_action_context(&ap);
        assert_eq!(ctx.get_actions(), &vec![ClientAction::MOVE, ClientAction::END_MOVE]);
    }

    #[test]
    fn available_actions_contains_foul_and_chainsaw() {
        let module = FoulLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::FOUL));
        assert!(actions.contains(&ClientAction::CHAINSAW));
        assert!(actions.contains(&ClientAction::INCORPOREAL));
    }

    #[test]
    fn player_peek_ignores_without_game() {
        let client = make_client();
        let module = FoulLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = FoulLogicModule::new();
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
        let mut module = FoulLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::FOUL);
    }
}
