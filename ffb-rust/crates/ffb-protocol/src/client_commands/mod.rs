use serde::{Deserialize, Serialize};
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{NetCommandId, PlayerAction};

/// Commands sent from the Rust client to the Java server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "netCommandId", rename_all = "camelCase")]
pub enum ClientCommand {
    /// Join a game.
    ClientJoin(ClientJoin),
    /// Start the game.
    ClientStartGame(ClientStartGame),
    /// Place a player during setup.
    ClientSetupPlayer(ClientSetupPlayer),
    /// Activate a player with an action type.
    ClientActingPlayer(ClientActingPlayer),
    /// Move the active player.
    ClientMove(ClientMove),
    /// Blitz movement step.
    ClientBlitzMove(ClientBlitzMove),
    /// Perform a block.
    ClientBlock(ClientBlock),
    /// Choose a block die result.
    ClientBlockChoice(ClientBlockChoice),
    /// Choose where to push a player.
    ClientPushback(ClientPushback),
    /// Declare follow-up (or not) after a block.
    ClientFollowupChoice(ClientFollowupChoice),
    /// Kick the ball.
    ClientKickoff(ClientKickoff),
    /// Receive a touchback.
    ClientTouchback(ClientTouchback),
    /// Pass the ball.
    ClientPass(ClientPass),
    /// Hand off the ball.
    ClientHandOver(ClientHandOver),
    /// Foul a prone player.
    ClientFoul(ClientFoul),
    /// Attempt to intercept.
    ClientInterceptorChoice(ClientInterceptorChoice),
    /// Declare coin choice.
    ClientCoinChoice(ClientCoinChoice),
    /// Declare receive/kick choice.
    ClientReceiveChoice(ClientReceiveChoice),
    /// End the current team turn.
    ClientEndTurn(ClientEndTurn),
    /// Use (or decline) a re-roll.
    ClientUseReRoll(ClientUseReRoll),
    /// Use (or decline) a skill.
    ClientUseSkill(ClientUseSkill),
    /// Use (or decline) an apothecary.
    ClientUseApothecary(ClientUseApothecary),
    /// Confirm the apothecary choice (heal vs accept).
    ClientApothecaryChoice(ClientApothecaryChoice),
    /// Throw a team-mate.
    ClientThrowTeamMate(ClientThrowTeamMate),
    /// Kick a team-mate.
    ClientKickTeamMate(ClientKickTeamMate),
    /// Swoop.
    ClientSwoop(ClientSwoop),
    /// Buy inducements before the game.
    ClientBuyInducements(ClientBuyInducements),
    /// Petty cash spending.
    ClientPettyCash(ClientPettyCash),
    /// Select a player from a list.
    ClientPlayerChoice(ClientPlayerChoice),
    /// Gaze at a target.
    ClientGaze(ClientGaze),
    /// Confirm current decision.
    ClientConfirm(ClientConfirm),
    /// Use a pile driver.
    ClientPileDriver(ClientPileDriver),
    /// Argue the Call.
    ClientArgueTheCall(ClientArgueTheCall),
    /// Declare use of Wizard spell.
    ClientWizardSpell(ClientWizardSpell),
    /// Select weather (Weather Mage).
    ClientSelectWeather(ClientSelectWeather),
    /// Select journeymen positions.
    ClientJourneymen(ClientJourneymen),
    /// Bloodlust action choice.
    ClientBloodlustAction(ClientBloodlustAction),
    /// Select a kick-off result (some editions).
    ClientKickOffResultChoice(ClientKickOffResultChoice),
    /// Keep-alive ping.
    ClientPing(ClientPing),
    /// Chat message (game or replay session).
    ClientTalk(ClientTalk),
    /// Client-initiated session close.
    ClientCloseSession(ClientCloseSession),
    /// Transfer replay control to another coach.
    ClientTransferReplayControl(ClientTransferReplayControl),
    /// Request the server/client version and client properties.
    ClientRequestVersion(ClientRequestVersion),
    /// Request a password challenge for FUMBBL auth.
    ClientPasswordChallenge(ClientPasswordChallenge),
    /// Add a field sketch.
    ClientAddSketch(ClientAddSketch),
    /// Clear all field sketches for the sending session (and replay peers).
    ClientClearSketches(ClientClearSketches),
    /// Remove one or more field sketches by id.
    ClientRemoveSketches(ClientRemoveSketches),
    /// Append a path coordinate to an in-progress sketch.
    ClientSketchAddCoordinate(ClientSketchAddCoordinate),
    /// Set the color of one or more sketches.
    ClientSketchSetColor(ClientSketchSetColor),
    /// Set the label of one or more sketches.
    ClientSketchSetLabel(ClientSketchSetLabel),
    /// Set a field/player marker's text.
    ClientSetMarker(ClientSetMarker),
    /// Prevent (or allow) a coach from sketching during a replay.
    ClientSetPreventSketching(ClientSetPreventSketching),
    /// Update (or request) automatic/manual player markings.
    ClientUpdatePlayerMarkings(ClientUpdatePlayerMarkings),
    /// Load automatic player markings for a given game version.
    ClientLoadAutomaticPlayerMarkings(ClientLoadAutomaticPlayerMarkings),
}

// ── Individual client command structs ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientJoin {
    pub coach: String,
    pub team_id: String,
    pub game_id: String,
    pub password_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStartGame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSetupPlayer {
    pub player_id: PlayerId,
    pub coordinate: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientActingPlayer {
    pub player_id: PlayerId,
    pub player_action: PlayerAction,
    pub standing_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMove {
    pub move_squares: Vec<FieldCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBlitzMove {
    pub move_squares: Vec<FieldCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBlock {
    pub defender_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBlockChoice {
    pub selected_die_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPushback {
    pub pushback_square: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientFollowupChoice {
    pub follow_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKickoff {
    pub coordinate: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTouchback {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPass {
    pub target_coordinate: FieldCoordinate,
    pub hail_mary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHandOver {
    pub target_player_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientFoul {
    pub defender_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInterceptorChoice {
    pub attempt_interception: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCoinChoice {
    pub home_choice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientReceiveChoice {
    pub receive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEndTurn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUseReRoll {
    pub use_reroll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUseSkill {
    pub player_id: PlayerId,
    pub skill: String,
    pub use_skill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUseApothecary {
    pub player_id: PlayerId,
    pub use_apothecary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientApothecaryChoice {
    pub player_id: PlayerId,
    pub choice: ApothecaryChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApothecaryChoice {
    Apothecary,
    RollResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientThrowTeamMate {
    pub player_id: PlayerId,
    pub target_coordinate: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKickTeamMate {
    pub player_id: PlayerId,
    pub target_coordinate: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSwoop {
    pub player_id: PlayerId,
    pub target_coordinate: FieldCoordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBuyInducements {
    pub team_id: String,
    pub purchases: Vec<(String, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPettyCash {
    pub team_id: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPlayerChoice {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientGaze {
    pub target_id: PlayerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfirm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPileDriver {
    pub player_id: PlayerId,
    pub use_pile_driver: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientArgueTheCall {
    pub player_id: PlayerId,
    pub use_argue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWizardSpell {
    pub team_id: String,
    pub spell: String,
    pub target_coordinate: Option<FieldCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSelectWeather {
    pub weather: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientJourneymen {
    pub team_id: String,
    pub position_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBloodlustAction {
    pub player_id: PlayerId,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKickOffResultChoice {
    pub team_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPing {
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTalk {
    pub talk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCloseSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTransferReplayControl {
    pub coach: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequestVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPasswordChallenge {
    pub coach: Option<String>,
}

/// Mirrors `ffb_protocol::commands::client_command_add_sketch::ClientCommandAddSketch`'s
/// field shape (entropy dropped, matching the rest of this enum's variants).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAddSketch {
    pub sketch_id: Option<String>,
}

/// Mirrors `ClientCommandClearSketches` (no payload beyond entropy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientClearSketches;

/// Mirrors `ClientCommandRemoveSketches`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRemoveSketches {
    pub ids: Vec<String>,
}

/// Mirrors `ClientCommandSketchAddCoordinate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSketchAddCoordinate {
    pub sketch_id: Option<String>,
    pub coordinate: Option<FieldCoordinate>,
}

/// Mirrors `ClientCommandSketchSetColor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSketchSetColor {
    pub sketch_ids: Vec<String>,
    pub rgb: i32,
}

/// Mirrors `ClientCommandSketchSetLabel`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSketchSetLabel {
    pub sketch_ids: Vec<String>,
    pub label: Option<String>,
}

/// Mirrors `ClientCommandSetMarker`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSetMarker {
    pub player_id: Option<String>,
    pub coordinate: Option<FieldCoordinate>,
    pub text: Option<String>,
}

/// Mirrors `ClientCommandSetPreventSketching`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSetPreventSketching {
    pub coach: Option<String>,
    pub prevent_sketching: bool,
}

/// Mirrors `ClientCommandUpdatePlayerMarkings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUpdatePlayerMarkings {
    pub auto: bool,
    pub sort_mode_name: Option<String>,
}

/// Mirrors `ClientCommandLoadAutomaticPlayerMarkings`. The `game` field (full `Game` object)
/// is omitted here as it's optional in Java and not needed to build
/// `FumbblRequestLoadPlayerMarkingsForGameVersion`, whose Rust constructor was ported to take
/// only `index`/`coach` — see `ServerCommandHandlerLoadAutomaticPlayerMarkings::build_request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientLoadAutomaticPlayerMarkings {
    pub index: i32,
    pub coach: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerAction;

    fn rt(cmd: &ClientCommand) {
        let json = serde_json::to_string(cmd).unwrap();
        let _back: ClientCommand = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("round-trip failed: {e}\njson={json}"));
    }

    #[test]
    fn client_end_turn_round_trip() {
        rt(&ClientCommand::ClientEndTurn(ClientEndTurn));
    }

    #[test]
    fn client_move_round_trip() {
        rt(&ClientCommand::ClientMove(ClientMove {
            move_squares: vec![FieldCoordinate::new(10, 7), FieldCoordinate::new(11, 7)],
        }));
    }

    #[test]
    fn client_block_round_trip() {
        rt(&ClientCommand::ClientBlock(ClientBlock { defender_id: "p42".into() }));
    }

    #[test]
    fn client_acting_player_round_trip() {
        rt(&ClientCommand::ClientActingPlayer(ClientActingPlayer {
            player_id: "p1".into(),
            player_action: PlayerAction::Move,
            standing_up: false,
        }));
    }

    #[test]
    fn client_pass_round_trip() {
        rt(&ClientCommand::ClientPass(ClientPass {
            target_coordinate: FieldCoordinate::new(18, 7),
            hail_mary: false,
        }));
    }

    #[test]
    fn client_coin_choice_round_trip() {
        rt(&ClientCommand::ClientCoinChoice(ClientCoinChoice { home_choice: true }));
    }

    #[test]
    fn client_use_reroll_round_trip() {
        rt(&ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: true }));
        rt(&ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: false }));
    }

    #[test]
    fn client_buy_inducements_round_trip() {
        rt(&ClientCommand::ClientBuyInducements(ClientBuyInducements {
            team_id: "home".into(),
            purchases: vec![("wizard".into(), 1)],
        }));
    }

    #[test]
    fn client_tag_is_camel_case() {
        let json = serde_json::to_string(&ClientCommand::ClientEndTurn(ClientEndTurn)).unwrap();
        assert!(json.contains("clientEndTurn"), "tag must be camelCase, got: {json}");
    }

    #[test]
    fn client_join_round_trip() {
        rt(&ClientCommand::ClientJoin(ClientJoin {
            coach: "TestCoach".into(),
            team_id: "team1".into(),
            game_id: "game42".into(),
            password_hash: None,
        }));
    }

    #[test]
    fn client_talk_round_trip() {
        rt(&ClientCommand::ClientTalk(ClientTalk { talk: Some("hello".into()) }));
        rt(&ClientCommand::ClientTalk(ClientTalk { talk: None }));
    }

    #[test]
    fn client_close_session_round_trip() {
        rt(&ClientCommand::ClientCloseSession(ClientCloseSession));
    }

    #[test]
    fn client_transfer_replay_control_round_trip() {
        rt(&ClientCommand::ClientTransferReplayControl(ClientTransferReplayControl {
            coach: Some("coach1".into()),
        }));
    }

    #[test]
    fn client_request_version_round_trip() {
        rt(&ClientCommand::ClientRequestVersion(ClientRequestVersion));
    }

    #[test]
    fn client_password_challenge_round_trip() {
        rt(&ClientCommand::ClientPasswordChallenge(ClientPasswordChallenge {
            coach: Some("coach2".into()),
        }));
    }

    #[test]
    fn client_add_sketch_round_trip() {
        rt(&ClientCommand::ClientAddSketch(ClientAddSketch { sketch_id: Some("sk-1".into()) }));
    }

    #[test]
    fn client_clear_sketches_round_trip() {
        rt(&ClientCommand::ClientClearSketches(ClientClearSketches));
    }

    #[test]
    fn client_remove_sketches_round_trip() {
        rt(&ClientCommand::ClientRemoveSketches(ClientRemoveSketches { ids: vec!["sk-1".into()] }));
    }

    #[test]
    fn client_sketch_add_coordinate_round_trip() {
        rt(&ClientCommand::ClientSketchAddCoordinate(ClientSketchAddCoordinate {
            sketch_id: Some("sk-1".into()),
            coordinate: Some(FieldCoordinate::new(3, 4)),
        }));
    }

    #[test]
    fn client_sketch_set_color_round_trip() {
        rt(&ClientCommand::ClientSketchSetColor(ClientSketchSetColor {
            sketch_ids: vec!["sk-1".into()],
            rgb: 0xFF00FF,
        }));
    }

    #[test]
    fn client_sketch_set_label_round_trip() {
        rt(&ClientCommand::ClientSketchSetLabel(ClientSketchSetLabel {
            sketch_ids: vec!["sk-1".into()],
            label: Some("Arrow".into()),
        }));
    }

    #[test]
    fn client_set_marker_round_trip() {
        rt(&ClientCommand::ClientSetMarker(ClientSetMarker {
            player_id: Some("p1".into()),
            coordinate: Some(FieldCoordinate::new(1, 1)),
            text: Some("Nice job".into()),
        }));
    }

    #[test]
    fn client_set_prevent_sketching_round_trip() {
        rt(&ClientCommand::ClientSetPreventSketching(ClientSetPreventSketching {
            coach: Some("coach1".into()),
            prevent_sketching: true,
        }));
    }

    #[test]
    fn client_update_player_markings_round_trip() {
        rt(&ClientCommand::ClientUpdatePlayerMarkings(ClientUpdatePlayerMarkings {
            auto: true,
            sort_mode_name: Some("NAME".into()),
        }));
    }

    #[test]
    fn client_load_automatic_player_markings_round_trip() {
        rt(&ClientCommand::ClientLoadAutomaticPlayerMarkings(ClientLoadAutomaticPlayerMarkings {
            index: 1,
            coach: Some("Coach".into()),
        }));
    }
}
