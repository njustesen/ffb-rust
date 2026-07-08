/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFactory.
///
/// Routes incoming `ClientCommand`s to the appropriate handler:
/// - Join / Ping / SocketClosed → dedicated handlers
/// - Gameplay commands → decoded to `Action` and fed to the engine
use std::sync::{Arc, Mutex};
use ffb_engine::action::{Action, PlayerActionChoice};
use ffb_engine::legal_actions::TeamSide;
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_protocol::client_commands::ClientCommand;
use crate::game_cache::GameCache;
use crate::model::received_command::ReceivedCommand;
use crate::net::session_manager::SessionManager;
use crate::net::wire::{OutgoingModelSync, events_to_reports};

/// Errors returned when a `ClientCommand` cannot be decoded to an `Action`.
#[derive(Debug)]
pub enum DecodeError {
    UnknownSkill(String),
    NotImplemented(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnknownSkill(s) => write!(f, "unknown skill: {s}"),
            DecodeError::NotImplemented(s) => write!(f, "not implemented: {s}"),
        }
    }
}

/// Java: `ServerCommandHandlerFactory`
pub struct ServerCommandHandlerFactory {
    pub game_cache: Arc<Mutex<GameCache>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerFactory {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: `handleCommand(ReceivedCommand)` — the main dispatch entry point.
    pub fn handle_command(&self, received: ReceivedCommand) {
        let session_id = received.session_id;
        let game_id = {
            let sm = self.session_manager.lock().unwrap();
            sm.get_game_id_for_session(session_id)
        };

        match &received.command {
            ClientCommand::ClientPing(ping) => {
                let mut sm = self.session_manager.lock().unwrap();
                sm.set_last_ping(session_id, ping.timestamp);
                return;
            }
            ClientCommand::ClientJoin(_) => {
                // Join is handled upstream in command_socket.rs before being enqueued.
                log::warn!("session {} sent ClientJoin after already joined", session_id);
                return;
            }
            _ => {}
        }

        let side = {
            let sm = self.session_manager.lock().unwrap();
            if sm.get_session_of_home_coach(game_id) == Some(session_id) {
                TeamSide::Home
            } else {
                TeamSide::Away
            }
        };

        let action = match decode_command(received.command, side) {
            Ok(a) => a,
            Err(e) => {
                log::warn!("session {} decode error: {}", session_id, e);
                return;
            }
        };

        let events = {
            let mut gc = self.game_cache.lock().unwrap();
            match gc.get_game_state_by_id_mut(game_id) {
                Some(gs) => match gs.handle_action(side, action) {
                    Ok(evts) => evts,
                    Err(e) => {
                        log::warn!("engine rejected action from session {}: {}", session_id, e);
                        return;
                    }
                },
                None => {
                    log::warn!("session {} sent command but game {} not found", session_id, game_id);
                    return;
                }
            }
        };

        let command_nr = {
            let mut gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id_mut(game_id).map(|gs| gs.generate_command_nr()).unwrap_or(0)
        };

        let reports = events_to_reports(&events);
        let sync = OutgoingModelSync::new(command_nr, reports);
        match serde_json::to_string(&sync) {
            Ok(json) => {
                let sm = self.session_manager.lock().unwrap();
                sm.send_all(game_id, &json);
            }
            Err(e) => log::error!("failed to serialize model sync: {}", e),
        }
    }
}

// ── ClientCommand → Action decoder ───────────────────────────────────────────

/// Server-side inverse of `ffb-client/src/network_encoder/mod.rs`.
///
/// Decodes a `ClientCommand` into an engine `Action`.  Gameplay-routing
/// commands that have no `Action` equivalent (e.g. `ClientJoin`, `ClientPing`)
/// should be handled *before* this function is called.
pub fn decode_command(cmd: ClientCommand, side: TeamSide) -> Result<Action, DecodeError> {
    match cmd {
        ClientCommand::ClientEndTurn(_) => Ok(Action::EndTurn),

        ClientCommand::ClientMove(m) => Ok(Action::Move { path: m.move_squares }),
        ClientCommand::ClientBlitzMove(m) => Ok(Action::Move { path: m.move_squares }),

        ClientCommand::ClientActingPlayer(a) => {
            let player_action = player_action_to_choice(a.player_action)
                .ok_or(DecodeError::NotImplemented(format!("player_action {:?}", a.player_action)))?;
            Ok(Action::ActivatePlayer {
                player_id: a.player_id,
                player_action,
                block_defender_id: None,
            })
        }

        ClientCommand::ClientBlock(b) => Ok(Action::Block { defender_id: b.defender_id }),
        ClientCommand::ClientBlockChoice(b) => Ok(Action::BlockChoice {
            die_index: b.selected_die_index as usize,
            target_id: None,
        }),
        ClientCommand::ClientPushback(p) => Ok(Action::PushTo { coord: p.pushback_square }),
        ClientCommand::ClientFollowupChoice(f) => Ok(Action::FollowUp { follow_up: f.follow_up }),

        ClientCommand::ClientKickoff(k) => Ok(Action::KickBall { coord: k.coordinate }),
        ClientCommand::ClientTouchback(t) => Ok(Action::Touchback { player_id: t.player_id }),

        ClientCommand::ClientPass(p) => Ok(Action::Pass { coord: p.target_coordinate }),
        ClientCommand::ClientHandOver(h) => Ok(Action::HandOff { receiver_id: h.target_player_id }),
        ClientCommand::ClientFoul(f) => Ok(Action::Foul { target_id: f.defender_id }),

        ClientCommand::ClientInterceptorChoice(i) => Ok(Action::Intercept { attempt: i.attempt_interception }),
        ClientCommand::ClientCoinChoice(c) => Ok(Action::CoinChoice { heads: c.home_choice }),
        ClientCommand::ClientReceiveChoice(r) => Ok(Action::ReceiveChoice { receive: r.receive }),

        ClientCommand::ClientUseReRoll(r) => Ok(Action::UseReRoll { use_reroll: r.use_reroll }),
        ClientCommand::ClientUseSkill(s) => {
            let skill_id = parse_skill_id(&s.skill)
                .ok_or_else(|| DecodeError::UnknownSkill(s.skill.clone()))?;
            Ok(Action::UseSkill { skill_id, use_skill: s.use_skill })
        }

        ClientCommand::ClientUseApothecary(a) => Ok(Action::UseApothecary {
            player_id: a.player_id,
            use_apothecary: a.use_apothecary,
        }),
        ClientCommand::ClientApothecaryChoice(a) => {
            use ffb_protocol::client_commands::ApothecaryChoice;
            Ok(Action::ApothecaryChoice {
                player_state: match a.choice {
                    ApothecaryChoice::Apothecary => 1,
                    ApothecaryChoice::RollResult => 0,
                },
                serious_injury: None,
            })
        }

        ClientCommand::ClientSetupPlayer(s) => Ok(Action::PlacePlayer {
            player_id: s.player_id,
            coord: s.coordinate,
        }),
        ClientCommand::ClientStartGame(_) => Ok(Action::StartGame { home: side == TeamSide::Home }),

        ClientCommand::ClientThrowTeamMate(t) => Ok(Action::ThrowTeamMate {
            player_id: t.player_id,
            coord: t.target_coordinate,
        }),
        ClientCommand::ClientKickTeamMate(k) => Ok(Action::KickTeamMate {
            player_id: k.player_id,
            coord: k.target_coordinate,
        }),
        ClientCommand::ClientSwoop(s) => Ok(Action::Pass { coord: s.target_coordinate }),

        ClientCommand::ClientGaze(g) => Ok(Action::HypnoticGaze { target_id: g.target_id }),
        ClientCommand::ClientConfirm(_) => Ok(Action::Acknowledge),
        ClientCommand::ClientArgueTheCall(a) => Ok(Action::ArgueTheCall { argue: a.use_argue }),

        ClientCommand::ClientPlayerChoice(p) => Ok(Action::SelectPlayer { player_id: p.player_id }),

        ClientCommand::ClientBloodlustAction(b) => Ok(Action::BloodlustAction {
            change: b.action.eq_ignore_ascii_case("change"),
        }),

        ClientCommand::ClientBuyInducements(b) => Ok(Action::BuyInducements {
            purchases: b.purchases.into_iter().map(|(id, count)| {
                ffb_engine::action::InducementPurchase { id, count: count as u32 }
            }).collect(),
        }),

        ClientCommand::ClientPettyCash(p) => Ok(Action::PettyCash {
            home: side == TeamSide::Home,
            amount: p.amount,
        }),

        ClientCommand::ClientKickOffResultChoice(_) => Ok(Action::Acknowledge),
        ClientCommand::ClientSelectWeather(s) => {
            // Parse weather name via serde camelCase
            let camel = to_camel_case(&s.weather);
            serde_json::from_str::<ffb_model::enums::Weather>(&format!("\"{}\"", camel))
                .map(|w| Action::SelectWeather { weather: w })
                .map_err(|_| DecodeError::NotImplemented(format!("weather {}", s.weather)))
        }

        ClientCommand::ClientPileDriver(_) => Err(DecodeError::NotImplemented("ClientPileDriver".into())),
        ClientCommand::ClientWizardSpell(_) => Err(DecodeError::NotImplemented("ClientWizardSpell".into())),
        ClientCommand::ClientJourneymen(_) => Err(DecodeError::NotImplemented("ClientJourneymen".into())),

        ClientCommand::ClientJoin(_) => Err(DecodeError::NotImplemented("ClientJoin".into())),
        ClientCommand::ClientPing(_) => Err(DecodeError::NotImplemented("ClientPing".into())),
    }
}

/// Map `PlayerAction` (wire/model enum) → `PlayerActionChoice` (engine enum).
///
/// This is the inverse of `choice_to_player_action` in `ffb-client/src/network_encoder/mod.rs`.
fn player_action_to_choice(action: PlayerAction) -> Option<PlayerActionChoice> {
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
        | PlayerAction::FoulMove | PlayerAction::HandOverMove | PlayerAction::GazeMove
        | PlayerAction::ThrowTeamMateMove | PlayerAction::KickTeamMateMove
        | PlayerAction::PutridRegurgitationMove => Some(PlayerActionChoice::Move),

        PlayerAction::Block | PlayerAction::MultipleBlock
        | PlayerAction::PutridRegurgitationBlock | PlayerAction::KickEmBlock => Some(PlayerActionChoice::Block),

        PlayerAction::Blitz | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz
        | PlayerAction::PutridRegurgitationBlitz | PlayerAction::KickEmBlitz => Some(PlayerActionChoice::Blitz),

        PlayerAction::Stab | PlayerAction::Chainsaw => Some(PlayerActionChoice::Stab),

        PlayerAction::Foul => Some(PlayerActionChoice::Foul),

        PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => Some(PlayerActionChoice::Pass),
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => Some(PlayerActionChoice::ThrowBomb),

        PlayerAction::HandOver => Some(PlayerActionChoice::HandOff),

        PlayerAction::StandUp | PlayerAction::RemoveConfusion => Some(PlayerActionChoice::StandUp),

        PlayerAction::ThrowTeamMate => Some(PlayerActionChoice::ThrowTeamMate),
        PlayerAction::KickTeamMate => Some(PlayerActionChoice::KickTeamMate),

        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::AutoGazeZoat => Some(PlayerActionChoice::HypnoticGaze),

        PlayerAction::Swoop => Some(PlayerActionChoice::Swoop),
        PlayerAction::Punt | PlayerAction::PuntMove => Some(PlayerActionChoice::Punt),
        PlayerAction::BreatheFire => Some(PlayerActionChoice::BreatheFire),
        PlayerAction::ProjectileVomit => Some(PlayerActionChoice::ProjectileVomit),
        PlayerAction::SecureTheBall => Some(PlayerActionChoice::SecureTheBall),

        _ => None,
    }
}

/// Parse a `SkillId` from the Debug-format string produced by `format!("{:?}", skill_id)`.
///
/// The `Debug` representation uses PascalCase (e.g. `"Block"`, `"SureHands"`).
/// `SkillId`'s serde impl uses `rename_all = "camelCase"`, so we lowercase the
/// first character before deserializing.
fn parse_skill_id(s: &str) -> Option<SkillId> {
    let camel = to_camel_case(s);
    serde_json::from_str::<SkillId>(&format!("\"{}\"", camel)).ok()
}

/// Lowercase the first ASCII character of a PascalCase string to produce camelCase.
fn to_camel_case(s: &str) -> String {
    let mut out = s.to_string();
    if let Some(first) = out.get_mut(0..1) {
        first.make_ascii_lowercase();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;
    use ffb_protocol::client_commands::*;

    fn coord(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate { x, y } }

    // ── decode_command: pre-game ───────────────────────────────────────────────

    #[test]
    fn decode_end_turn() {
        let r = decode_command(ClientCommand::ClientEndTurn(ClientEndTurn), TeamSide::Home);
        assert!(matches!(r, Ok(Action::EndTurn)));
    }

    #[test]
    fn decode_coin_choice_heads() {
        let r = decode_command(
            ClientCommand::ClientCoinChoice(ClientCoinChoice { home_choice: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::CoinChoice { heads: true })));
    }

    #[test]
    fn decode_coin_choice_tails() {
        let r = decode_command(
            ClientCommand::ClientCoinChoice(ClientCoinChoice { home_choice: false }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::CoinChoice { heads: false })));
    }

    #[test]
    fn decode_receive_choice_receive() {
        let r = decode_command(
            ClientCommand::ClientReceiveChoice(ClientReceiveChoice { receive: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ReceiveChoice { receive: true })));
    }

    #[test]
    fn decode_receive_choice_kick() {
        let r = decode_command(
            ClientCommand::ClientReceiveChoice(ClientReceiveChoice { receive: false }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::ReceiveChoice { receive: false })));
    }

    #[test]
    fn decode_start_game_home() {
        let r = decode_command(ClientCommand::ClientStartGame(ClientStartGame), TeamSide::Home);
        assert!(matches!(r, Ok(Action::StartGame { home: true })));
    }

    #[test]
    fn decode_start_game_away() {
        let r = decode_command(ClientCommand::ClientStartGame(ClientStartGame), TeamSide::Away);
        assert!(matches!(r, Ok(Action::StartGame { home: false })));
    }

    // ── decode_command: movement ───────────────────────────────────────────────

    #[test]
    fn decode_move() {
        let path = vec![coord(1, 1), coord(2, 2)];
        let r = decode_command(
            ClientCommand::ClientMove(ClientMove { move_squares: path.clone() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Move { path: p }) if p == path));
    }

    #[test]
    fn decode_blitz_move_as_move() {
        let path = vec![coord(3, 4)];
        let r = decode_command(
            ClientCommand::ClientBlitzMove(ClientBlitzMove { move_squares: path.clone() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Move { path: p }) if p == path));
    }

    #[test]
    fn decode_setup_player() {
        let r = decode_command(
            ClientCommand::ClientSetupPlayer(ClientSetupPlayer {
                player_id: "p1".into(),
                coordinate: coord(5, 5),
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::PlacePlayer { player_id, coord: c })
            if player_id == "p1" && c == coord(5, 5)));
    }

    // ── decode_command: block sequence ────────────────────────────────────────

    #[test]
    fn decode_block() {
        let r = decode_command(
            ClientCommand::ClientBlock(ClientBlock { defender_id: "defender".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Block { defender_id: d }) if d == "defender"));
    }

    #[test]
    fn decode_block_choice() {
        let r = decode_command(
            ClientCommand::ClientBlockChoice(ClientBlockChoice { selected_die_index: 2 }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::BlockChoice { die_index: 2, target_id: None })));
    }

    #[test]
    fn decode_pushback() {
        let r = decode_command(
            ClientCommand::ClientPushback(ClientPushback { pushback_square: coord(7, 3) }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::PushTo { coord: c }) if c == coord(7, 3)));
    }

    #[test]
    fn decode_followup_yes() {
        let r = decode_command(
            ClientCommand::ClientFollowupChoice(ClientFollowupChoice { follow_up: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::FollowUp { follow_up: true })));
    }

    #[test]
    fn decode_followup_no() {
        let r = decode_command(
            ClientCommand::ClientFollowupChoice(ClientFollowupChoice { follow_up: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::FollowUp { follow_up: false })));
    }

    // ── decode_command: acting player / activation ────────────────────────────

    #[test]
    fn decode_acting_player_move() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p1".into(),
                player_action: ffb_model::enums::PlayerAction::Move,
                standing_up: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer {
            player_id: ref pid,
            player_action: PlayerActionChoice::Move,
            ..
        }) if pid == "p1"));
    }

    #[test]
    fn decode_acting_player_block() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p2".into(),
                player_action: ffb_model::enums::PlayerAction::Block,
                standing_up: false,
            }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer { player_action: PlayerActionChoice::Block, .. })));
    }

    #[test]
    fn decode_acting_player_foul() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p3".into(),
                player_action: ffb_model::enums::PlayerAction::Foul,
                standing_up: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer { player_action: PlayerActionChoice::Foul, .. })));
    }

    // ── decode_command: pass / handoff / foul ─────────────────────────────────

    #[test]
    fn decode_pass() {
        let r = decode_command(
            ClientCommand::ClientPass(ClientPass { target_coordinate: coord(10, 5), hail_mary: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Pass { coord: c }) if c == coord(10, 5)));
    }

    #[test]
    fn decode_handover() {
        let r = decode_command(
            ClientCommand::ClientHandOver(ClientHandOver { target_player_id: "receiver".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::HandOff { receiver_id: ref id }) if id == "receiver"));
    }

    #[test]
    fn decode_foul() {
        let r = decode_command(
            ClientCommand::ClientFoul(ClientFoul { defender_id: "prone".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Foul { target_id: ref id }) if id == "prone"));
    }

    // ── decode_command: kickoff ────────────────────────────────────────────────

    #[test]
    fn decode_kickoff() {
        let r = decode_command(
            ClientCommand::ClientKickoff(ClientKickoff { coordinate: coord(8, 8) }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::KickBall { coord: c }) if c == coord(8, 8)));
    }

    #[test]
    fn decode_touchback() {
        let r = decode_command(
            ClientCommand::ClientTouchback(ClientTouchback { player_id: "catcher".into() }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::Touchback { player_id: ref id }) if id == "catcher"));
    }

    // ── decode_command: skill / reroll ────────────────────────────────────────

    #[test]
    fn decode_use_reroll_yes() {
        let r = decode_command(
            ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: true })));
    }

    #[test]
    fn decode_use_reroll_no() {
        let r = decode_command(
            ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: false })));
    }

    #[test]
    fn decode_use_skill_block() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "Block".into(),
                use_skill: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseSkill { skill_id: SkillId::Block, use_skill: true })));
    }

    #[test]
    fn decode_use_skill_sure_hands_declined() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "SureHands".into(),
                use_skill: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseSkill { skill_id: SkillId::SureHands, use_skill: false })));
    }

    #[test]
    fn decode_use_skill_unknown_returns_error() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "NotASkill".into(),
                use_skill: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::UnknownSkill(_))));
    }

    #[test]
    fn decode_use_apothecary() {
        let r = decode_command(
            ClientCommand::ClientUseApothecary(ClientUseApothecary {
                player_id: "hurt".into(),
                use_apothecary: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseApothecary { use_apothecary: true, .. })));
    }

    // ── decode_command: intercept / misc ──────────────────────────────────────

    #[test]
    fn decode_intercept_attempt() {
        let r = decode_command(
            ClientCommand::ClientInterceptorChoice(ClientInterceptorChoice { attempt_interception: true }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::Intercept { attempt: true })));
    }

    #[test]
    fn decode_confirm_acknowledges() {
        let r = decode_command(ClientCommand::ClientConfirm(ClientConfirm), TeamSide::Home);
        assert!(matches!(r, Ok(Action::Acknowledge)));
    }

    #[test]
    fn decode_argue_the_call() {
        let r = decode_command(
            ClientCommand::ClientArgueTheCall(ClientArgueTheCall {
                player_id: "captain".into(),
                use_argue: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ArgueTheCall { argue: true })));
    }

    #[test]
    fn decode_player_choice() {
        let r = decode_command(
            ClientCommand::ClientPlayerChoice(ClientPlayerChoice { player_id: "chosen".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::SelectPlayer { player_id: ref id }) if id == "chosen"));
    }

    #[test]
    fn decode_ping_returns_not_implemented() {
        let r = decode_command(
            ClientCommand::ClientPing(ClientPing { timestamp: 123 }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::NotImplemented(_))));
    }

    #[test]
    fn decode_join_returns_not_implemented() {
        let r = decode_command(
            ClientCommand::ClientJoin(ClientJoin {
                coach: "x".into(), team_id: "t".into(), game_id: "1".into(), password_hash: None,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::NotImplemented(_))));
    }

    // ── parse_skill_id ────────────────────────────────────────────────────────

    #[test]
    fn parse_skill_block() {
        assert_eq!(parse_skill_id("Block"), Some(SkillId::Block));
    }

    #[test]
    fn parse_skill_sure_hands() {
        assert_eq!(parse_skill_id("SureHands"), Some(SkillId::SureHands));
    }

    #[test]
    fn parse_skill_hail_mary_pass() {
        assert_eq!(parse_skill_id("HailMaryPass"), Some(SkillId::HailMaryPass));
    }

    #[test]
    fn parse_skill_unknown_returns_none() {
        assert_eq!(parse_skill_id("NotASkill"), None);
    }

    #[test]
    fn parse_skill_empty_returns_none() {
        assert_eq!(parse_skill_id(""), None);
    }

    // ── player_action_to_choice ────────────────────────────────────────────────

    #[test]
    fn player_action_move_maps_to_move() {
        assert_eq!(player_action_to_choice(PlayerAction::Move), Some(PlayerActionChoice::Move));
    }

    #[test]
    fn player_action_blitz_move_maps_to_move() {
        assert_eq!(player_action_to_choice(PlayerAction::BlitzMove), Some(PlayerActionChoice::Move));
    }

    #[test]
    fn player_action_blitz_maps_to_blitz() {
        assert_eq!(player_action_to_choice(PlayerAction::Blitz), Some(PlayerActionChoice::Blitz));
    }

    #[test]
    fn player_action_block_maps_to_block() {
        assert_eq!(player_action_to_choice(PlayerAction::Block), Some(PlayerActionChoice::Block));
    }

    #[test]
    fn player_action_foul_maps_to_foul() {
        assert_eq!(player_action_to_choice(PlayerAction::Foul), Some(PlayerActionChoice::Foul));
    }

    #[test]
    fn player_action_hand_over_maps_to_hand_off() {
        assert_eq!(player_action_to_choice(PlayerAction::HandOver), Some(PlayerActionChoice::HandOff));
    }

    #[test]
    fn player_action_gaze_maps_to_hypnotic_gaze() {
        assert_eq!(player_action_to_choice(PlayerAction::Gaze), Some(PlayerActionChoice::HypnoticGaze));
    }

    #[test]
    fn player_action_stand_up_maps_to_stand_up() {
        assert_eq!(player_action_to_choice(PlayerAction::StandUp), Some(PlayerActionChoice::StandUp));
    }

    #[test]
    fn player_action_throw_bomb_maps_to_throw_bomb() {
        assert_eq!(player_action_to_choice(PlayerAction::ThrowBomb), Some(PlayerActionChoice::ThrowBomb));
    }

    #[test]
    fn player_action_swoop_maps_to_swoop() {
        assert_eq!(player_action_to_choice(PlayerAction::Swoop), Some(PlayerActionChoice::Swoop));
    }

    #[test]
    fn player_action_punt_maps_to_punt() {
        assert_eq!(player_action_to_choice(PlayerAction::Punt), Some(PlayerActionChoice::Punt));
    }

    #[test]
    fn player_action_breathe_fire_maps() {
        assert_eq!(player_action_to_choice(PlayerAction::BreatheFire), Some(PlayerActionChoice::BreatheFire));
    }

    #[test]
    fn player_action_secure_the_ball_maps() {
        assert_eq!(player_action_to_choice(PlayerAction::SecureTheBall), Some(PlayerActionChoice::SecureTheBall));
    }

    #[test]
    fn player_action_unknown_variant_returns_none() {
        assert_eq!(player_action_to_choice(PlayerAction::Forgo), None);
    }

    // ── to_camel_case ─────────────────────────────────────────────────────────

    #[test]
    fn camel_case_lowercases_first_char() {
        assert_eq!(to_camel_case("Block"), "block");
        assert_eq!(to_camel_case("SureHands"), "sureHands");
        assert_eq!(to_camel_case("HailMaryPass"), "hailMaryPass");
    }

    #[test]
    fn camel_case_empty_is_safe() {
        assert_eq!(to_camel_case(""), "");
    }

    // ── handle_command routing (unit level) ───────────────────────────────────

    #[test]
    fn handle_command_ping_updates_last_ping() {
        use std::sync::{Arc, Mutex};
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;
        use crate::model::received_command::ReceivedCommand;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(42, 1, "TestCoach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc));
        factory.handle_command(ReceivedCommand {
            command: ClientCommand::ClientPing(ClientPing { timestamp: 9999 }),
            session_id: 42,
        });
        let sm = sm_arc.lock().unwrap();
        assert_eq!(sm.get_last_ping(42), 9999);
    }

    #[test]
    fn handle_command_unknown_session_does_not_panic() {
        use std::sync::{Arc, Mutex};
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm);
        // session 99 was never registered — should log a warning and return cleanly
        factory.handle_command(crate::model::received_command::ReceivedCommand {
            command: ClientCommand::ClientEndTurn(ClientEndTurn),
            session_id: 99,
        });
    }
}
