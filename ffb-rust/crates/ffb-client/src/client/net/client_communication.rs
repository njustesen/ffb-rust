//! 1:1 translation of `com.fumbbl.ffb.client.net.ClientCommunication`.
//!
//! Java's `ClientCommunication` has two responsibilities:
//! 1. Outgoing command construction: ~90 `send*` methods, each building the matching
//!    `ClientCommand*` object and forwarding it to `CommandEndpoint.send()`.
//! 2. Incoming-command dispatch (`Runnable`/`INetCommandHandler`): a synchronized queue fed
//!    by `CommandEndpoint.onMessage()`, drained on a background thread that classifies each
//!    command (replayer vs not) before handing it to `getClient().getCommandHandlerFactory()`.
//!
//! DOCUMENTED GAP: `getClient()` (`FantasyFootballClient`) is the permanently-skipped GUI
//! shell (see `TRANSLATION_TRACKER.md`'s GUI-skip note), so:
//! - Values Java reads off the client (`getClient().getMode()`, `getParameters().getCoach()`,
//!   the `MouseEntropySource` used inside `send()`) are taken as explicit parameters here
//!   instead of being pulled from a client reference that doesn't exist.
//! - `getClient().getReplayer()` / `getCommandHandlerFactory().handleNetCommand(...)` in
//!   `run()` have no translated target yet, so only the classification logic (which
//!   `NetCommandId`s are excluded from the replayer) is translated, as the standalone,
//!   testable `should_skip_replay`. The actual downstream dispatch is a documented
//!   follow-up once the replayer/handler-factory land.
//! - `send()`'s real transport is `getClient().getCommandEndpoint().send(clientCommand)`
//!   (Java's `CommandEndpoint`), which corresponds to `crate::connection::ServerConnection`
//!   here — but `ServerConnection::send` only accepts the old, hand-rolled
//!   `ffb_protocol::client_commands::ClientCommand` enum (a not-1:1 simplification), not
//!   these newly translated `ffb_protocol::commands::client_command_*` structs. Rewiring
//!   `ServerConnection` to the real structs is a separate, wider change. Every `send*`
//!   method below still builds the *correct* `ClientCommand*` struct field-for-field and
//!   calls its real `.to_json_value()` (matching `NetCommand.toJsonValue()`); the resulting
//!   JSON is pushed onto `outbox` (`Vec<serde_json::Value>`) instead of a live socket, so the
//!   correctness of the command construction is fully testable today.
//! - `sendTeamSetupSave` reads `pTeamSetup.getPlayerNumbers()`/`getCoordinates()` off Java's
//!   `TeamSetup`; the ported `ffb_model::model::TeamSetup` only carries `name`/`coach`/
//!   `positions` (no player-number/coordinate fields), so those are taken as explicit
//!   parameters here instead.
//! - `sendAddSketch` reads `sketch.getId()` off Java's `Sketch`; the ported
//!   `ffb_model::model::sketch::Sketch` only carries `positions` (its own doc comment notes
//!   `id`/`rgb`/`label` were never ported), so the sketch id is taken as an explicit
//!   parameter here instead of `Sketch`.

use std::collections::{HashMap, VecDeque};

use ffb_model::enums::{
    ApothecaryType, KickoffResult, NetCommandId, PlayerAction, PlayerState, ReRollSource,
    SeriousInjuryKind, TurnMode,
};
use ffb_model::model::{
    BlockKind, BlockTarget, ClientMode, CommonProperty, ConcedeGameStatus, FieldModel, Game,
    Keyword, PlayerChoiceMode, ReRolledAction, SpecialEffect, Team,
};
use ffb_model::model::inducement_set::InducementSet;
use ffb_model::model::team_setup::TeamSetup;
use ffb_model::model::player::Player;
use ffb_model::model::skill::skill::Skill;
use ffb_model::marking::sort_mode::SortMode;
use ffb_model::model::pushback::Pushback;
use ffb_model::bb2020::injury_description::InjuryDescription;
use ffb_model::types::FieldCoordinate;
use ffb_model::inducement::card::Card;
use ffb_model::inducement::card_type::CardType;
use ffb_model::inducement::inducement_type::InducementType;

use ffb_protocol::commands::any_server_command::AnyServerCommand;
use ffb_protocol::commands::client_command_acting_player::ClientCommandActingPlayer;
use ffb_protocol::commands::client_command_add_sketch::ClientCommandAddSketch;
use ffb_protocol::commands::client_command_apothecary_choice::ClientCommandApothecaryChoice;
use ffb_protocol::commands::client_command_argue_the_call::ClientCommandArgueTheCall;
use ffb_protocol::commands::client_command_blitz_move::ClientCommandBlitzMove;
use ffb_protocol::commands::client_command_block::ClientCommandBlock;
use ffb_protocol::commands::client_command_block_choice::ClientCommandBlockChoice;
use ffb_protocol::commands::client_command_block_or_re_roll_choice_for_target::ClientCommandBlockOrReRollChoiceForTarget;
use ffb_protocol::commands::client_command_bloodlust_action::ClientCommandBloodlustAction;
use ffb_protocol::commands::client_command_buy_card::ClientCommandBuyCard;
use ffb_protocol::commands::client_command_buy_inducements::ClientCommandBuyInducements;
use ffb_protocol::commands::client_command_clear_sketches::ClientCommandClearSketches;
use ffb_protocol::commands::client_command_close_session::ClientCommandCloseSession;
use ffb_protocol::commands::client_command_coin_choice::ClientCommandCoinChoice;
use ffb_protocol::commands::client_command_concede_game::ClientCommandConcedeGame;
use ffb_protocol::commands::client_command_confirm::ClientCommandConfirm;
use ffb_protocol::commands::client_command_debug_client_state::ClientCommandDebugClientState;
use ffb_protocol::commands::client_command_end_turn::ClientCommandEndTurn;
use ffb_protocol::commands::client_command_field_coordinate::ClientCommandFieldCoordinate;
use ffb_protocol::commands::client_command_followup_choice::ClientCommandFollowupChoice;
use ffb_protocol::commands::client_command_foul::ClientCommandFoul;
use ffb_protocol::commands::client_command_gaze::ClientCommandGaze;
use ffb_protocol::commands::client_command_hand_over::ClientCommandHandOver;
use ffb_protocol::commands::client_command_illegal_procedure::ClientCommandIllegalProcedure;
use ffb_protocol::commands::client_command_interceptor_choice::ClientCommandInterceptorChoice;
use ffb_protocol::commands::client_command_join::ClientCommandJoin;
use ffb_protocol::commands::client_command_join_replay::ClientCommandJoinReplay;
use ffb_protocol::commands::client_command_journeymen::ClientCommandJourneymen;
use ffb_protocol::commands::client_command_keyword_selection::ClientCommandKeywordSelection;
use ffb_protocol::commands::client_command_kick_off_result_choice::ClientCommandKickOffResultChoice;
use ffb_protocol::commands::client_command_kick_team_mate::ClientCommandKickTeamMate;
use ffb_protocol::commands::client_command_kickoff::ClientCommandKickoff;
use ffb_protocol::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings;
use ffb_protocol::commands::client_command_move::ClientCommandMove;
use ffb_protocol::commands::client_command_pass::ClientCommandPass;
use ffb_protocol::commands::client_command_password_challenge::ClientCommandPasswordChallenge;
use ffb_protocol::commands::client_command_petty_cash::ClientCommandPettyCash;
use ffb_protocol::commands::client_command_pick_up_choice::ClientCommandPickUpChoice;
use ffb_protocol::commands::client_command_pile_driver::ClientCommandPileDriver;
use ffb_protocol::commands::client_command_ping::ClientCommandPing;
use ffb_protocol::commands::client_command_player_choice::ClientCommandPlayerChoice;
use ffb_protocol::commands::client_command_position_selection::ClientCommandPositionSelection;
use ffb_protocol::commands::client_command_punt_to_crowd::ClientCommandPuntToCrowd;
use ffb_protocol::commands::client_command_pushback::ClientCommandPushback;
use ffb_protocol::commands::client_command_receive_choice::ClientCommandReceiveChoice;
use ffb_protocol::commands::client_command_remove_sketches::ClientCommandRemoveSketches;
use ffb_protocol::commands::client_command_replay::ClientCommandReplay;
use ffb_protocol::commands::client_command_replay_status::ClientCommandReplayStatus;
use ffb_protocol::commands::client_command_request_version::ClientCommandRequestVersion;
use ffb_protocol::commands::client_command_select_card_to_buy::ClientCommandSelectCardToBuy;
use ffb_protocol::commands::client_command_select_weather::ClientCommandSelectWeather;
use ffb_protocol::commands::client_command_set_block_target_selection::ClientCommandSetBlockTargetSelection;
use ffb_protocol::commands::client_command_set_marker::ClientCommandSetMarker;
use ffb_protocol::commands::client_command_set_prevent_sketching::ClientCommandSetPreventSketching;
use ffb_protocol::commands::client_command_setup_player::ClientCommandSetupPlayer;
use ffb_protocol::commands::client_command_sketch_add_coordinate::ClientCommandSketchAddCoordinate;
use ffb_protocol::commands::client_command_sketch_set_color::ClientCommandSketchSetColor;
use ffb_protocol::commands::client_command_sketch_set_label::ClientCommandSketchSetLabel;
use ffb_protocol::commands::client_command_skill_selection::ClientCommandSkillSelection;
use ffb_protocol::commands::client_command_start_game::ClientCommandStartGame;
use ffb_protocol::commands::client_command_swoop::ClientCommandSwoop;
use ffb_protocol::commands::client_command_synchronous_multi_block::ClientCommandSynchronousMultiBlock;
use ffb_protocol::commands::client_command_talk::ClientCommandTalk;
use ffb_protocol::commands::client_command_target_selected::ClientCommandTargetSelected;
use ffb_protocol::commands::client_command_team_setup_delete::ClientCommandTeamSetupDelete;
use ffb_protocol::commands::client_command_team_setup_load::ClientCommandTeamSetupLoad;
use ffb_protocol::commands::client_command_team_setup_save::ClientCommandTeamSetupSave;
use ffb_protocol::commands::client_command_throw_keg::ClientCommandThrowKeg;
use ffb_protocol::commands::client_command_throw_team_mate::ClientCommandThrowTeamMate;
use ffb_protocol::commands::client_command_touchback::ClientCommandTouchback;
use ffb_protocol::commands::client_command_transfer_replay_control::ClientCommandTransferReplayControl;
use ffb_protocol::commands::client_command_unset_block_target_selection::ClientCommandUnsetBlockTargetSelection;
use ffb_protocol::commands::client_command_update_player_markings::ClientCommandUpdatePlayerMarkings;
use ffb_protocol::commands::client_command_use_apothecaries::ClientCommandUseApothecaries;
use ffb_protocol::commands::client_command_use_apothecary::ClientCommandUseApothecary;
use ffb_protocol::commands::client_command_use_brawler::ClientCommandUseBrawler;
use ffb_protocol::commands::client_command_use_chainsaw::ClientCommandUseChainsaw;
use ffb_protocol::commands::client_command_use_consummate_re_roll_for_block::ClientCommandUseConsummateReRollForBlock;
use ffb_protocol::commands::client_command_use_fumblerooskie::ClientCommandUseFumblerooskie;
use ffb_protocol::commands::client_command_use_hatred::ClientCommandUseHatred;
use ffb_protocol::commands::client_command_use_igors::ClientCommandUseIgors;
use ffb_protocol::commands::client_command_use_inducement::ClientCommandUseInducement;
use ffb_protocol::commands::client_command_use_multi_block_dice_re_roll::ClientCommandUseMultiBlockDiceReRoll;
use ffb_protocol::commands::client_command_use_pro_re_roll_for_block::ClientCommandUseProReRollForBlock;
use ffb_protocol::commands::client_command_use_re_roll::ClientCommandUseReRoll;
use ffb_protocol::commands::client_command_use_re_roll_for_target::ClientCommandUseReRollForTarget;
use ffb_protocol::commands::client_command_use_single_block_die_re_roll::ClientCommandUseSingleBlockDieReRoll;
use ffb_protocol::commands::client_command_use_skill::ClientCommandUseSkill;
use ffb_protocol::commands::client_command_use_team_mates_wisdom::ClientCommandUseTeamMatesWisdom;
use ffb_protocol::commands::client_command_user_settings::ClientCommandUserSettings;
use ffb_protocol::commands::client_command_wizard_spell::ClientCommandWizardSpell;

/// Java: `com.fumbbl.ffb.client.net.ClientCommunication`.
pub struct ClientCommunication {
    /// Java: `fStopped`.
    stopped: bool,
    /// Java: `fCommandQueue` (`List<NetCommand>`) — commands the wire actually delivers to
    /// this queue are server->client commands, so `AnyServerCommand` is used here.
    command_queue: VecDeque<AnyServerCommand>,
    /// Not present in Java: the correctly-constructed JSON for every `send*` call below,
    /// standing in for the not-yet-wired `CommandEndpoint.send()` transport (see module doc).
    pub outbox: Vec<serde_json::Value>,
}

impl ClientCommunication {
    /// Java: `ClientCommunication(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            stopped: false,
            command_queue: VecDeque::new(),
            outbox: Vec::new(),
        }
    }

    /// Java: `handleCommand(NetCommand pNetCommand)`.
    pub fn handle_command(&mut self, net_command: AnyServerCommand) {
        self.command_queue.push_back(net_command);
    }

    /// Java: `stop()`.
    pub fn stop(&mut self) {
        if !self.stopped {
            self.stopped = true;
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Java: `run()`'s dequeue step (`fCommandQueue.remove(0)`), without the
    /// wait/notify thread machinery — see module doc for what `run()` skips.
    pub fn dequeue(&mut self) -> Option<AnyServerCommand> {
        self.command_queue.pop_front()
    }

    /// Java: `run()`'s classification switch — `NetCommandId`s that are *not* added to
    /// the replayer (`getClient().getReplayer().add(...)`).
    pub fn should_skip_replay(id: NetCommandId) -> bool {
        matches!(
            id,
            NetCommandId::ServerPong
                | NetCommandId::ServerTalk
                | NetCommandId::ServerSound
                | NetCommandId::ServerReplay
                | NetCommandId::InternalServerSocketClosed
                | NetCommandId::ServerSketchAddCoordinate
                | NetCommandId::ServerSketchSetColor
                | NetCommandId::ServerSketchSetLabel
                | NetCommandId::ServerAddSketches
                | NetCommandId::ServerRemoveSketches
                | NetCommandId::ServerClearSketches
        )
    }

    /// Java: `protected void send(ClientCommand clientCommand)`.
    /// The `MouseEntropySource` entropy injection is skipped (untranslated Swing/AWT-coupled
    /// utility — see `crate::client::util::mod`'s own note); the transport is `outbox` (see
    /// module doc). A null check has no Rust equivalent since callers always pass a value.
    fn send(&mut self, json: serde_json::Value) {
        self.outbox.push(json);
    }

    /// Java: `sendDebugClientState(ClientStateId pClientStateId)`.
    pub fn send_debug_client_state(&mut self, client_state_id: ffb_model::enums::ClientStateId) {
        let cmd = ClientCommandDebugClientState { entropy: None, client_state_id: Some(client_state_id) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendJoin(String, String, long, String, String, String)`.
    /// `getClient().getMode()` is taken as an explicit `client_mode` parameter (see module doc).
    pub fn send_join(
        &mut self,
        client_mode: ClientMode,
        coach: impl Into<String>,
        password: impl Into<String>,
        game_id: i64,
        game_name: impl Into<String>,
        team_id: impl Into<String>,
        team_name: impl Into<String>,
    ) {
        let cmd = ClientCommandJoin {
            entropy: None,
            client_mode: Some(client_mode),
            coach: Some(coach.into()),
            password: Some(password.into()),
            game_id,
            game_name: Some(game_name.into()),
            team_id: Some(team_id.into()),
            team_name: Some(team_name.into()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendJourneymen(String[] pPositionsIds, int[] pSlots)`.
    pub fn send_journeymen(&mut self, position_ids: Vec<String>, slots: Vec<i32>) {
        let cmd = ClientCommandJourneymen { entropy: None, slots, position_ids };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTalk(String pTalk)`.
    pub fn send_talk(&mut self, talk: impl Into<String>) {
        let cmd = ClientCommandTalk { entropy: None, talk: Some(talk.into()) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPasswordChallenge()`.
    /// `getClient().getParameters().getCoach()` is taken as an explicit `coach` parameter.
    pub fn send_password_challenge(&mut self, coach: impl Into<String>) {
        let cmd = ClientCommandPasswordChallenge { entropy: None, coach: Some(coach.into()) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPing(long timestamp)`.
    pub fn send_ping(&mut self, timestamp: i64) {
        let cmd = ClientCommandPing::new(timestamp);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSetupPlayer(Player<?> pPlayer, FieldCoordinate pCoordinate)`.
    pub fn send_setup_player(&mut self, player: &Player, coordinate: FieldCoordinate) {
        let cmd = ClientCommandSetupPlayer {
            entropy: None,
            player_id: Some(player.id.clone()),
            coordinate: Some(coordinate),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTouchback(FieldCoordinate pBallCoordinate)`.
    pub fn send_touchback(&mut self, ball_coordinate: FieldCoordinate) {
        let cmd = ClientCommandTouchback::new(ball_coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPlayerMove(String, FieldCoordinate, FieldCoordinate[], String)`.
    pub fn send_player_move(
        &mut self,
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
        ball_and_chain_rr_setting: Option<String>,
    ) {
        let cmd = ClientCommandMove {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
            ball_and_chain_rr_setting,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPlayerBlitzMove(String, FieldCoordinate, FieldCoordinate[])`.
    pub fn send_player_blitz_move(
        &mut self,
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
    ) {
        let cmd = ClientCommandBlitzMove {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTargetSelected(String selectedPlayerId)`.
    pub fn send_target_selected(&mut self, selected_player_id: impl Into<String>) {
        let cmd = ClientCommandTargetSelected { entropy: None, target_player_id: Some(selected_player_id.into()) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendStartGame()`.
    pub fn send_start_game(&mut self) {
        let cmd = ClientCommandStartGame::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendEndTurn(TurnMode turnMode)`.
    /// Java's debug `logWithOutGameId(new Exception("Debug Exception"))` call is a
    /// no-op logging side effect off the skipped `FantasyFootballClient` — dropped.
    pub fn send_end_turn(&mut self, turn_mode: TurnMode) {
        let cmd = ClientCommandEndTurn { entropy: None, turn_mode: Some(turn_mode), player_coordinates: HashMap::new() };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendEndTurn(TurnMode turnMode, Team team, FieldModel fieldModel)`.
    pub fn send_end_turn_with_coordinates(&mut self, turn_mode: TurnMode, team: &Team, field_model: &FieldModel) {
        let cmd = ClientCommandEndTurn {
            entropy: None,
            turn_mode: Some(turn_mode),
            player_coordinates: Self::player_coordinates(team, field_model),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendConfirm()`.
    pub fn send_confirm(&mut self) {
        let cmd = ClientCommandConfirm::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendCloseSession()`.
    pub fn send_close_session(&mut self) {
        let cmd = ClientCommandCloseSession::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendConcedeGame(ConcedeGameStatus pStatus)`.
    pub fn send_concede_game(&mut self, status: ConcedeGameStatus) {
        let cmd = ClientCommandConcedeGame { entropy: None, concede_game_status: Some(status) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendIllegalProcedure()`.
    pub fn send_illegal_procedure(&mut self) {
        let cmd = ClientCommandIllegalProcedure::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendRequestVersion()`.
    pub fn send_request_version(&mut self) {
        let cmd = ClientCommandRequestVersion::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendCoinChoice(boolean pChoiceHeads)`.
    pub fn send_coin_choice(&mut self, choice_heads: bool) {
        let cmd = ClientCommandCoinChoice::new(choice_heads);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendReceiveChoice(boolean pChoiceReceive)`.
    pub fn send_receive_choice(&mut self, choice_receive: bool) {
        let cmd = ClientCommandReceiveChoice::new(choice_receive);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPlayerChoice(PlayerChoiceMode pMode, Player<?>[] pPlayers)`.
    pub fn send_player_choice(&mut self, mode: PlayerChoiceMode, players: &[Player]) {
        let cmd = ClientCommandPlayerChoice {
            entropy: None,
            player_choice_mode: Some(mode),
            player_ids: players.iter().map(|p| p.id.clone()).collect(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPettyCash(int pPettyCash)`.
    pub fn send_petty_cash(&mut self, petty_cash: i32) {
        let cmd = ClientCommandPettyCash::new(petty_cash);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendActingPlayer(Player<?> pPlayer, PlayerAction pPlayerAction, boolean jumping)`.
    pub fn send_acting_player(&mut self, player: Option<&Player>, player_action: PlayerAction, jumping: bool) {
        let player_id = player.map(|p| p.id.clone());
        let cmd = ClientCommandActingPlayer { entropy: None, player_id, player_action: Some(player_action), jumping };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseReRoll(ReRolledAction pReRolledAction, ReRollSource pReRollSource)`.
    pub fn send_use_re_roll(&mut self, re_rolled_action: &ReRolledAction, re_roll_source: &ReRollSource) {
        let cmd = ClientCommandUseReRoll::new(re_rolled_action.get_name(), re_roll_source.name.clone());
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseProReRollForBlock(int proIndex)`.
    pub fn send_use_pro_re_roll_for_block(&mut self, pro_index: i32) {
        let cmd = ClientCommandUseProReRollForBlock::new(pro_index);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseConsummateReRollForBlock(int proIndex)`.
    pub fn send_use_consummate_re_roll_for_block(&mut self, pro_index: i32) {
        let cmd = ClientCommandUseConsummateReRollForBlock::new(pro_index);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseSingleBlockDieReRollForBlock(int index, ReRollSource source)`.
    pub fn send_use_single_block_die_re_roll_for_block(&mut self, index: i32, source: ReRollSource) {
        let cmd = ClientCommandUseSingleBlockDieReRoll { entropy: None, die_index: index, re_roll_source: Some(source) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseMultiBlockDiceReRoll(int[] indexes)`.
    pub fn send_use_multi_block_dice_re_roll(&mut self, indexes: Vec<i32>) {
        let cmd = ClientCommandUseMultiBlockDiceReRoll::with_indexes(indexes);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseSkill(Skill, boolean, String)`.
    pub fn send_use_skill(&mut self, skill: &Skill, skill_used: bool, player_id: impl Into<String>) {
        self.send_use_skill_full(skill, skill_used, player_id, None, false);
    }

    /// Java: `sendUseSkill(Skill, boolean, String, ReRolledAction)`.
    pub fn send_use_skill_with_reroll(
        &mut self,
        skill: &Skill,
        skill_used: bool,
        player_id: impl Into<String>,
        re_rolled_action: ReRolledAction,
    ) {
        self.send_use_skill_full(skill, skill_used, player_id, Some(re_rolled_action), false);
    }

    /// Java: `sendUseSkill(Skill, boolean, String, ReRolledAction, boolean)`.
    pub fn send_use_skill_full(
        &mut self,
        skill: &Skill,
        skill_used: bool,
        player_id: impl Into<String>,
        re_rolled_action: Option<ReRolledAction>,
        never_use: bool,
    ) {
        let cmd = ClientCommandUseSkill {
            entropy: None,
            skill_name: Some(skill.get_name().to_string()),
            skill_used,
            never_use,
            player_id: Some(player_id.into()),
            re_rolled_action,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseSkill(Skill, boolean, String, boolean)`.
    pub fn send_use_skill_never_use(&mut self, skill: &Skill, skill_used: bool, player_id: impl Into<String>, never_use: bool) {
        self.send_use_skill_full(skill, skill_used, player_id, None, never_use);
    }

    /// Java: `sendUseWisdom()`.
    pub fn send_use_wisdom(&mut self) {
        let cmd = ClientCommandUseTeamMatesWisdom::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendKickoff(FieldCoordinate pBallCoordinate)`.
    pub fn send_kickoff(&mut self, ball_coordinate: FieldCoordinate) {
        let cmd = ClientCommandKickoff::new(ball_coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendHandOver(String pActingPlayerId, Player<?> pCatcher)`.
    pub fn send_hand_over(&mut self, acting_player_id: impl Into<String>, catcher: Option<&Player>) {
        let cmd = ClientCommandHandOver {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            catcher_id: catcher.map(|p| p.id.clone()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendGaze(String pActingPlayerId, Player<?> pVictim)`.
    pub fn send_gaze(&mut self, acting_player_id: impl Into<String>, victim: Option<&Player>) {
        let cmd = ClientCommandGaze {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            victim_id: victim.map(|p| p.id.clone()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPass(String pActingPlayerId, FieldCoordinate pTargetCoordinate)`.
    pub fn send_pass(&mut self, acting_player_id: impl Into<String>, target_coordinate: FieldCoordinate) {
        let cmd = ClientCommandPass::new(acting_player_id, target_coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBlock(String, Player<?>, boolean, boolean, boolean, boolean, boolean)`.
    #[allow(clippy::too_many_arguments)]
    pub fn send_block(
        &mut self,
        acting_player_id: impl Into<String>,
        defender: Option<&Player>,
        using_stab: bool,
        using_chainsaw: bool,
        using_vomit: bool,
        using_breathe_fire: bool,
        using_chomp: bool,
    ) {
        let cmd = ClientCommandBlock {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            defender_id: defender.map(|p| p.id.clone()),
            using_stab,
            using_chainsaw,
            using_vomit,
            using_breathe_fire,
            using_chomp,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendFoul(String pActingPlayerId, Player<?> pDefender, boolean usingChainsaw)`.
    pub fn send_foul(&mut self, acting_player_id: impl Into<String>, defender: Option<&Player>, using_chainsaw: bool) {
        let cmd = ClientCommandFoul {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            defender_id: defender.map(|p| p.id.clone()),
            using_chainsaw,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBlockChoice(int pDiceIndex)`.
    pub fn send_block_choice(&mut self, dice_index: i32) {
        let cmd = ClientCommandBlockChoice::new(dice_index);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseInducement(InducementType pInducement)`.
    pub fn send_use_inducement(&mut self, inducement: &InducementType) {
        let cmd = ClientCommandUseInducement {
            entropy: None,
            inducement_type_name: Some(inducement.get_name().to_string()),
            card_name: None,
            player_ids: Vec::new(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseInducement(Card pCard)`.
    pub fn send_use_inducement_card(&mut self, card: &Card) {
        let cmd = ClientCommandUseInducement {
            entropy: None,
            inducement_type_name: None,
            card_name: Some(card.get_name().to_string()),
            player_ids: Vec::new(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseInducement(InducementType pInducement, String pPlayerId)`.
    pub fn send_use_inducement_for_player(&mut self, inducement: &InducementType, player_id: impl Into<String>) {
        let cmd = ClientCommandUseInducement {
            entropy: None,
            inducement_type_name: Some(inducement.get_name().to_string()),
            card_name: None,
            player_ids: vec![player_id.into()],
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseInducement(InducementType pInducement, String[] pPlayerIds)`.
    pub fn send_use_inducement_for_players(&mut self, inducement: &InducementType, player_ids: Vec<String>) {
        let cmd = ClientCommandUseInducement {
            entropy: None,
            inducement_type_name: Some(inducement.get_name().to_string()),
            card_name: None,
            player_ids,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendArgueTheCall(String playerId)`.
    pub fn send_argue_the_call(&mut self, player_id: impl Into<String>) {
        let cmd = ClientCommandArgueTheCall::with_player_id(player_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendArgueTheCall(String[] playerIds)`.
    pub fn send_argue_the_call_many(&mut self, player_ids: Vec<String>) {
        let cmd = ClientCommandArgueTheCall::with_player_ids(player_ids);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseApothecaries(List<InjuryDescription> injuryDescriptions)`.
    /// `InjuryDescription` has no ported `toJsonValue()` (see the struct's own DEFERRED
    /// note), so a minimal JSON object is built here from its fields.
    pub fn send_use_apothecaries(&mut self, injury_descriptions: &[InjuryDescription]) {
        let cmd = ClientCommandUseApothecaries {
            entropy: None,
            injury_description_json: injury_descriptions.iter().map(Self::injury_description_json).collect(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseIgors(List<InjuryDescription> injuryDescriptions)`.
    pub fn send_use_igors(&mut self, injury_descriptions: &[InjuryDescription]) {
        let cmd = ClientCommandUseIgors {
            entropy: None,
            injury_description_json: injury_descriptions.iter().map(Self::injury_description_json).collect(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPushback(Pushback pPushback)`.
    pub fn send_pushback(&mut self, pushback: Pushback) {
        let cmd = ClientCommandPushback::with_pushback(pushback);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendFollowupChoice(boolean pFollowupChoice)`.
    pub fn send_followup_choice(&mut self, followup_choice: bool) {
        let cmd = ClientCommandFollowupChoice::new(followup_choice);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendInterceptorChoice(Player<?> pInterceptor, Skill interceptionSkill)`.
    pub fn send_interceptor_choice(&mut self, interceptor: Option<&Player>, interception_skill: Option<&Skill>) {
        let cmd = ClientCommandInterceptorChoice {
            entropy: None,
            interceptor_id: interceptor.map(|p| p.id.clone()),
            interception_skill_id: interception_skill.map(|s| s.get_name().to_string()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTeamSetupLoad(String pSetupName)`.
    pub fn send_team_setup_load(&mut self, setup_name: impl Into<String>) {
        let cmd = ClientCommandTeamSetupLoad::with_setup_name(setup_name);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTeamSetupDelete(String pSetupName)`.
    pub fn send_team_setup_delete(&mut self, setup_name: impl Into<String>) {
        let cmd = ClientCommandTeamSetupDelete::with_setup_name(setup_name);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTeamSetupSave(TeamSetup pTeamSetup)`.
    /// See module doc: the ported `TeamSetup` lacks `player_numbers`/`coordinates`, so they
    /// are taken as explicit parameters.
    pub fn send_team_setup_save(
        &mut self,
        team_setup: &TeamSetup,
        player_numbers: Vec<i32>,
        player_coordinates: Vec<FieldCoordinate>,
    ) {
        let cmd = ClientCommandTeamSetupSave {
            entropy: None,
            setup_name: Some(team_setup.name.clone()),
            player_numbers,
            player_coordinates,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseApothecary(String, boolean, ApothecaryType, SeriousInjury)`.
    pub fn send_use_apothecary(
        &mut self,
        player_id: impl Into<String>,
        apothecary_used: bool,
        apothecary_type: ApothecaryType,
        serious_injury: SeriousInjuryKind,
    ) {
        self.send_use_apothecary_full(player_id, apothecary_used, apothecary_type, serious_injury, None);
    }

    /// Java: `sendUseApothecary(String, boolean, ApothecaryType, SeriousInjury, PlayerState)`.
    pub fn send_use_apothecary_full(
        &mut self,
        player_id: impl Into<String>,
        apothecary_used: bool,
        apothecary_type: ApothecaryType,
        serious_injury: SeriousInjuryKind,
        player_state: Option<PlayerState>,
    ) {
        let cmd = ClientCommandUseApothecary {
            entropy: None,
            player_id: Some(player_id.into()),
            apothecary_used,
            apothecary_type: Some(apothecary_type),
            serious_injury: Some(serious_injury),
            player_state,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendApothecaryChoice(String, PlayerState, SeriousInjury, PlayerState)`.
    pub fn send_apothecary_choice(
        &mut self,
        player_id: impl Into<String>,
        player_state: PlayerState,
        serious_injury: SeriousInjuryKind,
        old_player_state: PlayerState,
    ) {
        let cmd = ClientCommandApothecaryChoice {
            entropy: None,
            player_id: Some(player_id.into()),
            player_state: Some(player_state),
            old_player_state: Some(old_player_state),
            serious_injury: Some(serious_injury),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUserSettings(CommonProperty[] pSettingNames, String[] pSettingValues)`.
    pub fn send_user_settings(&mut self, setting_names: &[CommonProperty], setting_values: &[String]) {
        let mut cmd = ClientCommandUserSettings::new();
        for (name, value) in setting_names.iter().zip(setting_values.iter()) {
            cmd.set(*name, value.clone());
        }
        self.send(cmd.to_json_value());
    }

    /// Java: `sendReplay(long pGameId, int pReplayToCommandNr, String coach)`.
    pub fn send_replay(&mut self, game_id: i64, replay_to_command_nr: i32, coach: impl Into<String>) {
        let cmd = ClientCommandReplay::with_params(game_id, replay_to_command_nr, coach);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendThrowTeamMate(String, FieldCoordinate)`.
    pub fn send_throw_team_mate(&mut self, acting_player_id: impl Into<String>, target_coordinate: FieldCoordinate) {
        let cmd = ClientCommandThrowTeamMate {
            entropy: None,
            target_coordinate: Some(target_coordinate),
            thrown_player_id: None,
            acting_player_id: Some(acting_player_id.into()),
            kicked: false,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendThrowTeamMate(String, FieldCoordinate, boolean)`.
    pub fn send_throw_team_mate_kicked(
        &mut self,
        acting_player_id: impl Into<String>,
        target_coordinate: FieldCoordinate,
        kicked: bool,
    ) {
        let cmd = ClientCommandThrowTeamMate {
            entropy: None,
            target_coordinate: Some(target_coordinate),
            thrown_player_id: None,
            acting_player_id: Some(acting_player_id.into()),
            kicked,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendKickTeamMate(String pActingPlayerId, String pPlayerId, int numDice)`.
    pub fn send_kick_team_mate(&mut self, acting_player_id: impl Into<String>, player_id: impl Into<String>, num_dice: i32) {
        let cmd = ClientCommandKickTeamMate {
            entropy: None,
            kicked_player_id: Some(player_id.into()),
            acting_player_id: Some(acting_player_id.into()),
            num_dice,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendThrowTeamMate(String pActingPlayerId, String pPlayerId)`.
    pub fn send_throw_team_mate_by_id(&mut self, acting_player_id: impl Into<String>, player_id: impl Into<String>) {
        let cmd = ClientCommandThrowTeamMate {
            entropy: None,
            target_coordinate: None,
            thrown_player_id: Some(player_id.into()),
            acting_player_id: Some(acting_player_id.into()),
            kicked: false,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendThrowTeamMate(String pActingPlayerId, String pPlayerId, boolean kicked)`.
    pub fn send_throw_team_mate_by_id_kicked(
        &mut self,
        acting_player_id: impl Into<String>,
        player_id: impl Into<String>,
        kicked: bool,
    ) {
        let cmd = ClientCommandThrowTeamMate {
            entropy: None,
            target_coordinate: None,
            thrown_player_id: Some(player_id.into()),
            acting_player_id: Some(acting_player_id.into()),
            kicked,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSwoop(String pActingPlayerId, FieldCoordinate pTargetCoordinate)`.
    pub fn send_swoop(&mut self, acting_player_id: impl Into<String>, target_coordinate: FieldCoordinate) {
        let cmd = ClientCommandSwoop::with_players_and_target(acting_player_id, target_coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBuyInducements(String, int, InducementSet, String[], String[], Skill[], String[])`.
    #[allow(clippy::too_many_arguments)]
    pub fn send_buy_inducements(
        &mut self,
        team_id: impl Into<String>,
        available_gold: i32,
        inducement_set: InducementSet,
        star_player_position_ids: Vec<String>,
        mercenary_position_ids: Vec<String>,
        mercenary_skills: &[Skill],
        staff_position_ids: Vec<String>,
    ) {
        let cmd = ClientCommandBuyInducements {
            entropy: None,
            team_id: Some(team_id.into()),
            available_gold,
            inducement_set: Some(inducement_set),
            star_player_position_ids,
            mercenary_position_ids,
            staff_position_ids,
            mercenary_skill_ids: mercenary_skills.iter().map(|s| s.get_name().to_string()).collect(),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBuyCard(CardType pType)`.
    pub fn send_buy_card(&mut self, card_type: &dyn CardType) {
        let cmd = ClientCommandBuyCard { entropy: None, card_type_name: Some(card_type.get_name().to_string()) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSetMarker(String pPlayerId, String pText)`.
    pub fn send_set_marker(&mut self, player_id: impl Into<String>, text: impl Into<String>) {
        let cmd = ClientCommandSetMarker {
            player_id: Some(player_id.into()),
            coordinate: None,
            text: Some(text.into()),
            entropy: None,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSetMarker(FieldCoordinate pCoordinate, String pText)`.
    pub fn send_set_marker_at(&mut self, coordinate: FieldCoordinate, text: impl Into<String>) {
        let cmd = ClientCommandSetMarker {
            player_id: None,
            coordinate: Some(coordinate),
            text: Some(text.into()),
            entropy: None,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendWizardSpell(SpecialEffect pWizardSpell, FieldCoordinate pCoordinate)`.
    pub fn send_wizard_spell(&mut self, wizard_spell: SpecialEffect, coordinate: FieldCoordinate) {
        let cmd = ClientCommandWizardSpell::with_spell(wizard_spell, coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendCardSelection(ClientCommandSelectCardToBuy.Selection selection)`.
    /// Java's inner `Selection` enum is already flattened into two booleans on the ported
    /// struct (see that struct's own doc comment), so both are taken as explicit parameters.
    pub fn send_card_selection(&mut self, initial_deck_choice: bool, first_card_choice: bool) {
        let cmd = ClientCommandSelectCardToBuy::new(initial_deck_choice, first_card_choice);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSetBlockTarget(String playerId, BlockKind kind)`.
    pub fn send_set_block_target(&mut self, player_id: impl Into<String>, kind: BlockKind) {
        let cmd = ClientCommandSetBlockTargetSelection::with_target(player_id, kind);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUnsetBlockTarget(String playerId)`.
    pub fn send_unset_block_target(&mut self, player_id: impl Into<String>) {
        let cmd = ClientCommandUnsetBlockTargetSelection::with_player_id(player_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBlockTargets(List<BlockTarget> blockTargets)`.
    pub fn send_block_targets(&mut self, block_targets: Vec<BlockTarget>) {
        let cmd = ClientCommandSynchronousMultiBlock::with_targets(block_targets);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseReRollForTarget(ReRolledAction, ReRollSource, String)`.
    pub fn send_use_re_roll_for_target(
        &mut self,
        re_rolled_action: &ReRolledAction,
        re_roll_source: &ReRollSource,
        target_id: impl Into<String>,
    ) {
        let cmd = ClientCommandUseReRollForTarget {
            entropy: None,
            target_id: Some(target_id.into()),
            re_rolled_action: Some(re_rolled_action.get_name().to_string()),
            re_roll_source: Some(re_roll_source.name.clone()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendBlockOrReRollChoiceForTarget(String, int, ReRollSource, int)`.
    pub fn send_block_or_re_roll_choice_for_target(
        &mut self,
        target_id: impl Into<String>,
        selected_index: i32,
        re_roll_source: ReRollSource,
        pro_index: i32,
    ) {
        self.send_block_or_re_roll_choice_for_target_with_dice(target_id, selected_index, re_roll_source, pro_index, Vec::new());
    }

    /// Java: `sendBlockOrReRollChoiceForTarget(String, int, ReRollSource, int, int[])`.
    pub fn send_block_or_re_roll_choice_for_target_with_dice(
        &mut self,
        target_id: impl Into<String>,
        selected_index: i32,
        re_roll_source: ReRollSource,
        pro_index: i32,
        any_dice_indexes: Vec<i32>,
    ) {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget {
            entropy: None,
            target_id: Some(target_id.into()),
            selected_index,
            pro_index,
            re_roll_source: Some(re_roll_source),
            any_dice_indexes,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPileDriver(String playerId)`.
    pub fn send_pile_driver(&mut self, player_id: impl Into<String>) {
        let cmd = ClientCommandPileDriver::with_player_id(player_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseChainsaw(boolean useChainsaw)`.
    pub fn send_use_chainsaw(&mut self, use_chainsaw: bool) {
        let cmd = ClientCommandUseChainsaw::new(use_chainsaw);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseBrawler(String targetId)`.
    pub fn send_use_brawler(&mut self, target_id: impl Into<String>) {
        let cmd = ClientCommandUseBrawler::with_target_id(target_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseHatred(String targetId)`.
    pub fn send_use_hatred(&mut self, target_id: impl Into<String>) {
        let cmd = ClientCommandUseHatred::with_target(target_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendFieldCoordinate(FieldCoordinate fieldCoordinate)`.
    pub fn send_field_coordinate(&mut self, field_coordinate: FieldCoordinate) {
        let cmd = ClientCommandFieldCoordinate::with_coordinate(field_coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUseFumblerooskie()`.
    pub fn send_use_fumblerooskie(&mut self) {
        let cmd = ClientCommandUseFumblerooskie::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSkillSelection(String playerId, Skill skill)`.
    pub fn send_skill_selection(&mut self, player_id: impl Into<String>, skill: &Skill) {
        let cmd = ClientCommandSkillSelection {
            entropy: None,
            player_id: Some(player_id.into()),
            skill_id: Some(skill.get_name().to_string()),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendKeywordSelection(String playerId, List<Keyword> keywords)`.
    pub fn send_keyword_selection(&mut self, player_id: impl Into<String>, keywords: Vec<Keyword>) {
        let mut cmd = ClientCommandKeywordSelection { entropy: None, player_id: Some(player_id.into()), keywords: Vec::new() };
        for keyword in keywords {
            cmd.add_keyword(keyword);
        }
        self.send(cmd.to_json_value());
    }

    /// Java: `sendThrowKeg(Player<?> player)`.
    pub fn send_throw_keg(&mut self, player: &Player) {
        let cmd = ClientCommandThrowKeg::with_player_id(player.id.clone());
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSelectedWeather(int modifier, String weatherName)`.
    pub fn send_selected_weather(&mut self, modifier: i32, weather_name: impl Into<String>) {
        let cmd = ClientCommandSelectWeather::with_fields(modifier, weather_name);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendUpdatePlayerMarkings(boolean auto, SortMode sortMode)`.
    pub fn send_update_player_markings(&mut self, auto: bool, sort_mode: SortMode) {
        let cmd = ClientCommandUpdatePlayerMarkings { entropy: None, auto, sort_mode_name: Some(sort_mode.name().to_string()) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendKickOffResultChoice(KickoffResult kickoffResult)`.
    pub fn send_kick_off_result_choice(&mut self, kickoff_result: KickoffResult) {
        let cmd = ClientCommandKickOffResultChoice { entropy: None, kickoff_result: Some(kickoff_result) };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendChangeBloodlustAction(boolean change)`.
    pub fn send_change_bloodlust_action(&mut self, change: bool) {
        let cmd = ClientCommandBloodlustAction::new(change);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendLoadPlayerMarkings(int index, Game game, String coach)`.
    pub fn send_load_player_markings(&mut self, index: i32, game: Game, coach: impl Into<String>) {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: None,
            index,
            coach: Some(coach.into()),
            game: Some(game),
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendReplayState(int commandNr, int speed, boolean running, boolean forward, boolean skip)`.
    pub fn send_replay_state(&mut self, command_nr: i32, speed: i32, running: bool, forward: bool, skip: bool) {
        let cmd = ClientCommandReplayStatus::with_params(command_nr, speed, running, forward, skip);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendJoinReplay(String replayName, String coach, long gameId)`.
    pub fn send_join_replay(&mut self, replay_name: impl Into<String>, coach: impl Into<String>, game_id: i64) {
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some(replay_name.into()),
            coach: Some(coach.into()),
            game_id,
        };
        self.send(cmd.to_json_value());
    }

    /// Java: `sendClearSketches()`.
    pub fn send_clear_sketches(&mut self) {
        let cmd = ClientCommandClearSketches::new();
        self.send(cmd.to_json_value());
    }

    /// Java: `sendRemoveSketches(List<String> ids)`.
    pub fn send_remove_sketches(&mut self, ids: Vec<String>) {
        let cmd = ClientCommandRemoveSketches::with_ids(ids);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendAddSketch(Sketch sketch)`. See module doc: takes `sketch_id` directly
    /// since the ported `Sketch` model has no `id` field.
    pub fn send_add_sketch(&mut self, sketch_id: impl Into<String>) {
        let cmd = ClientCommandAddSketch::with_sketch_id(sketch_id);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSketchAddCoordinate(String sketchId, FieldCoordinate coordinate)`.
    pub fn send_sketch_add_coordinate(&mut self, sketch_id: impl Into<String>, coordinate: FieldCoordinate) {
        let cmd = ClientCommandSketchAddCoordinate::with_sketch(sketch_id, coordinate);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSketchSetColor(List<String> sketchIds, int rgb)`.
    pub fn send_sketch_set_color(&mut self, sketch_ids: Vec<String>, rgb: i32) {
        let cmd = ClientCommandSketchSetColor::with_color(sketch_ids, rgb);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendSketchSetLabel(List<String> sketchId, String label)`.
    pub fn send_sketch_set_label(&mut self, sketch_ids: Vec<String>, label: impl Into<String>) {
        let cmd = ClientCommandSketchSetLabel::with_label(sketch_ids, label);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendTransferReplayControl(String coach)`.
    pub fn send_transfer_replay_control(&mut self, coach: impl Into<String>) {
        let cmd = ClientCommandTransferReplayControl::with_coach(coach);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPreventFromSketching(String coach, boolean prevent)`.
    pub fn send_prevent_from_sketching(&mut self, coach: impl Into<String>, prevent: bool) {
        let cmd = ClientCommandSetPreventSketching::with_fields(coach, prevent);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPickUpChoice(boolean attemptPickUp)`.
    pub fn send_pick_up_choice(&mut self, attempt_pick_up: bool) {
        let cmd = ClientCommandPickUpChoice::new(attempt_pick_up);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPositionSelection(String[] position, String teamId)`.
    pub fn send_position_selection(&mut self, position: Vec<String>, team_id: impl Into<String>) {
        let cmd = ClientCommandPositionSelection::with_team(team_id, position);
        self.send(cmd.to_json_value());
    }

    /// Java: `sendPuntToCrowd(boolean puntToCrowd)`.
    pub fn send_punt_to_crowd(&mut self, punt_to_crowd: bool) {
        let cmd = ClientCommandPuntToCrowd::new(punt_to_crowd);
        self.send(cmd.to_json_value());
    }

    /// Java: `private Map<String, FieldCoordinate> playerCoordinates(Team team, FieldModel fieldModel)`.
    fn player_coordinates(team: &Team, field_model: &FieldModel) -> HashMap<String, FieldCoordinate> {
        let mut player_coordinates = HashMap::new();
        for player in &team.players {
            if let Some(player_state) = field_model.player_states.get(&player.id) {
                if player_state.can_be_moved_during_setup() {
                    if let Some(coordinate) = field_model.player_coordinates.get(&player.id) {
                        player_coordinates.insert(player.id.clone(), *coordinate);
                    }
                }
            }
        }
        player_coordinates
    }

    /// Helper for `sendUseApothecaries`/`sendUseIgors`: minimal JSON encoding of
    /// `InjuryDescription` (which has no ported `toJsonValue()` of its own).
    fn injury_description_json(desc: &InjuryDescription) -> String {
        serde_json::json!({
            "playerId": desc.player_id,
            "playerState": desc.player_state,
            "seriousInjury": desc.serious_injury,
            "apothecaryTypes": desc.apothecary_types,
        })
        .to_string()
    }
}

impl Default for ClientCommunication {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn player(id: &str) -> Player {
        Player {
            id: id.to_string(),
            name: id.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn new_starts_empty() {
        let comm = ClientCommunication::new();
        assert!(!comm.is_stopped());
        assert!(comm.outbox.is_empty());
    }

    #[test]
    fn stop_sets_flag_once() {
        let mut comm = ClientCommunication::new();
        comm.stop();
        assert!(comm.is_stopped());
        comm.stop();
        assert!(comm.is_stopped());
    }

    #[test]
    fn handle_command_and_dequeue_is_fifo() {
        let mut comm = ClientCommunication::new();
        comm.handle_command(AnyServerCommand::ServerPong(Default::default()));
        comm.handle_command(AnyServerCommand::ServerClearSketches(Default::default()));
        assert!(matches!(comm.dequeue(), Some(AnyServerCommand::ServerPong(_))));
        assert!(matches!(comm.dequeue(), Some(AnyServerCommand::ServerClearSketches(_))));
        assert!(comm.dequeue().is_none());
    }

    #[test]
    fn should_skip_replay_matches_java_switch() {
        assert!(ClientCommunication::should_skip_replay(NetCommandId::ServerPong));
        assert!(ClientCommunication::should_skip_replay(NetCommandId::ServerClearSketches));
        assert!(!ClientCommunication::should_skip_replay(NetCommandId::ClientConfirm));
    }

    #[test]
    fn send_ping_builds_correct_json() {
        let mut comm = ClientCommunication::new();
        comm.send_ping(42);
        assert_eq!(comm.outbox.len(), 1);
        assert_eq!(comm.outbox[0]["netCommandId"], "clientPing");
        assert_eq!(comm.outbox[0]["timestamp"], 42);
    }

    #[test]
    fn send_start_game_flat_command() {
        let mut comm = ClientCommunication::new();
        comm.send_start_game();
        assert_eq!(comm.outbox[0]["netCommandId"], "clientStartGame");
    }

    #[test]
    fn send_join_populates_all_fields() {
        let mut comm = ClientCommunication::new();
        comm.send_join(ClientMode::PLAYER, "coach", "pw", 7, "game", "team1", "TeamName");
        let json = &comm.outbox[0];
        assert_eq!(json["netCommandId"], "clientJoin");
        assert_eq!(json["coach"], "coach");
        assert_eq!(json["gameId"], 7);
        assert_eq!(json["teamName"], "TeamName");
    }

    #[test]
    fn send_setup_player_uses_player_id_and_coordinate() {
        let mut comm = ClientCommunication::new();
        let p = player("p1");
        comm.send_setup_player(&p, FieldCoordinate { x: 1, y: 2 });
        let json = &comm.outbox[0];
        assert_eq!(json["netCommandId"], "clientSetupPlayer");
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn send_journeymen_carries_lists() {
        let mut comm = ClientCommunication::new();
        comm.send_journeymen(vec!["pos1".to_string(), "pos2".to_string()], vec![1, 2]);
        let json = &comm.outbox[0];
        assert_eq!(json["netCommandId"], "clientJourneymen");
        assert_eq!(json["positionIds"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn send_end_turn_with_coordinates_filters_by_can_be_moved_during_setup() {
        let mut comm = ClientCommunication::new();
        let mut team = Team {
            id: "t1".into(), name: "T".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![player("p1")], vampire_lord: false, necromancer: false,
        };
        team.players.push(player("p2"));
        let mut field_model = FieldModel::new();
        field_model.player_states.insert("p1".to_string(), PlayerState::new(1));
        field_model.player_coordinates.insert("p1".to_string(), FieldCoordinate { x: 3, y: 4 });
        // p2 has no recorded state -> excluded.
        comm.send_end_turn_with_coordinates(TurnMode::Regular, &team, &field_model);
        let json = &comm.outbox[0];
        let coords = json["playersAtCoordinates"].as_object().unwrap();
        assert_eq!(coords.len(), 1);
        assert!(coords.contains_key("p1"));
    }

    #[test]
    fn send_use_re_roll_stores_names() {
        let mut comm = ClientCommunication::new();
        let action = ReRolledAction::new("Dodge");
        let source = ReRollSource::new("TeamReRoll");
        comm.send_use_re_roll(&action, &source);
        let json = &comm.outbox[0];
        assert_eq!(json["netCommandId"], "clientUseReRoll");
        assert_eq!(json["reRolledAction"], "Dodge");
        assert_eq!(json["reRollSource"], "TeamReRoll");
    }

    #[test]
    fn send_use_apothecaries_builds_minimal_injury_json() {
        let mut comm = ClientCommunication::new();
        let desc = InjuryDescription::new();
        comm.send_use_apothecaries(&[desc]);
        let json = &comm.outbox[0];
        assert_eq!(json["netCommandId"], "clientUseApothecaries");
        assert_eq!(json["injuryDescriptions"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn multiple_sends_accumulate_in_order() {
        let mut comm = ClientCommunication::new();
        comm.send_confirm();
        comm.send_close_session();
        comm.send_illegal_procedure();
        assert_eq!(comm.outbox.len(), 3);
        assert_eq!(comm.outbox[0]["netCommandId"], "clientConfirm");
        assert_eq!(comm.outbox[1]["netCommandId"], "clientCloseSession");
        assert_eq!(comm.outbox[2]["netCommandId"], "clientIllegalProcedure");
    }
}
