//! Real dispatch layer over the 91 genuine 1:1-translated `ClientCommand*` structs.
//! 1:1 translation of the dispatch role played by `NetCommandId.createNetCommand()`
//! (the client-command half of the switch) plus `NetCommand`/`NetCommandFactory`.
//!
//! This is additive: the pre-existing `client_commands::ClientCommand` (a hand-rolled,
//! not-1:1 simplification used by the live WebSocket layer today) is untouched.

use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;
use crate::commands::client_command_acting_player::ClientCommandActingPlayer;
use crate::commands::client_command_add_sketch::ClientCommandAddSketch;
use crate::commands::client_command_apothecary_choice::ClientCommandApothecaryChoice;
use crate::commands::client_command_argue_the_call::ClientCommandArgueTheCall;
use crate::commands::client_command_blitz_move::ClientCommandBlitzMove;
use crate::commands::client_command_block::ClientCommandBlock;
use crate::commands::client_command_block_choice::ClientCommandBlockChoice;
use crate::commands::client_command_block_or_re_roll_choice_for_target::ClientCommandBlockOrReRollChoiceForTarget;
use crate::commands::client_command_bloodlust_action::ClientCommandBloodlustAction;
use crate::commands::client_command_buy_card::ClientCommandBuyCard;
use crate::commands::client_command_buy_inducements::ClientCommandBuyInducements;
use crate::commands::client_command_clear_sketches::ClientCommandClearSketches;
use crate::commands::client_command_close_session::ClientCommandCloseSession;
use crate::commands::client_command_coin_choice::ClientCommandCoinChoice;
use crate::commands::client_command_concede_game::ClientCommandConcedeGame;
use crate::commands::client_command_confirm::ClientCommandConfirm;
use crate::commands::client_command_debug_client_state::ClientCommandDebugClientState;
use crate::commands::client_command_end_turn::ClientCommandEndTurn;
use crate::commands::client_command_field_coordinate::ClientCommandFieldCoordinate;
use crate::commands::client_command_followup_choice::ClientCommandFollowupChoice;
use crate::commands::client_command_foul::ClientCommandFoul;
use crate::commands::client_command_gaze::ClientCommandGaze;
use crate::commands::client_command_hand_over::ClientCommandHandOver;
use crate::commands::client_command_illegal_procedure::ClientCommandIllegalProcedure;
use crate::commands::client_command_interceptor_choice::ClientCommandInterceptorChoice;
use crate::commands::client_command_join::ClientCommandJoin;
use crate::commands::client_command_join_replay::ClientCommandJoinReplay;
use crate::commands::client_command_journeymen::ClientCommandJourneymen;
use crate::commands::client_command_keyword_selection::ClientCommandKeywordSelection;
use crate::commands::client_command_kick_off_result_choice::ClientCommandKickOffResultChoice;
use crate::commands::client_command_kick_team_mate::ClientCommandKickTeamMate;
use crate::commands::client_command_kickoff::ClientCommandKickoff;
use crate::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings;
use crate::commands::client_command_move::ClientCommandMove;
use crate::commands::client_command_pass::ClientCommandPass;
use crate::commands::client_command_password_challenge::ClientCommandPasswordChallenge;
use crate::commands::client_command_petty_cash::ClientCommandPettyCash;
use crate::commands::client_command_pick_up_choice::ClientCommandPickUpChoice;
use crate::commands::client_command_pile_driver::ClientCommandPileDriver;
use crate::commands::client_command_ping::ClientCommandPing;
use crate::commands::client_command_player_choice::ClientCommandPlayerChoice;
use crate::commands::client_command_position_selection::ClientCommandPositionSelection;
use crate::commands::client_command_punt_to_crowd::ClientCommandPuntToCrowd;
use crate::commands::client_command_pushback::ClientCommandPushback;
use crate::commands::client_command_receive_choice::ClientCommandReceiveChoice;
use crate::commands::client_command_remove_sketches::ClientCommandRemoveSketches;
use crate::commands::client_command_replay::ClientCommandReplay;
use crate::commands::client_command_replay_status::ClientCommandReplayStatus;
use crate::commands::client_command_request_version::ClientCommandRequestVersion;
use crate::commands::client_command_select_card_to_buy::ClientCommandSelectCardToBuy;
use crate::commands::client_command_select_weather::ClientCommandSelectWeather;
use crate::commands::client_command_set_block_target_selection::ClientCommandSetBlockTargetSelection;
use crate::commands::client_command_set_marker::ClientCommandSetMarker;
use crate::commands::client_command_set_prevent_sketching::ClientCommandSetPreventSketching;
use crate::commands::client_command_setup_player::ClientCommandSetupPlayer;
use crate::commands::client_command_sketch_add_coordinate::ClientCommandSketchAddCoordinate;
use crate::commands::client_command_sketch_set_color::ClientCommandSketchSetColor;
use crate::commands::client_command_sketch_set_label::ClientCommandSketchSetLabel;
use crate::commands::client_command_skill_selection::ClientCommandSkillSelection;
use crate::commands::client_command_start_game::ClientCommandStartGame;
use crate::commands::client_command_swoop::ClientCommandSwoop;
use crate::commands::client_command_synchronous_multi_block::ClientCommandSynchronousMultiBlock;
use crate::commands::client_command_talk::ClientCommandTalk;
use crate::commands::client_command_target_selected::ClientCommandTargetSelected;
use crate::commands::client_command_team_setup_delete::ClientCommandTeamSetupDelete;
use crate::commands::client_command_team_setup_load::ClientCommandTeamSetupLoad;
use crate::commands::client_command_team_setup_save::ClientCommandTeamSetupSave;
use crate::commands::client_command_throw_keg::ClientCommandThrowKeg;
use crate::commands::client_command_throw_team_mate::ClientCommandThrowTeamMate;
use crate::commands::client_command_touchback::ClientCommandTouchback;
use crate::commands::client_command_transfer_replay_control::ClientCommandTransferReplayControl;
use crate::commands::client_command_unset_block_target_selection::ClientCommandUnsetBlockTargetSelection;
use crate::commands::client_command_update_player_markings::ClientCommandUpdatePlayerMarkings;
use crate::commands::client_command_use_apothecaries::ClientCommandUseApothecaries;
use crate::commands::client_command_use_apothecary::ClientCommandUseApothecary;
use crate::commands::client_command_use_brawler::ClientCommandUseBrawler;
use crate::commands::client_command_use_chainsaw::ClientCommandUseChainsaw;
use crate::commands::client_command_use_consummate_re_roll_for_block::ClientCommandUseConsummateReRollForBlock;
use crate::commands::client_command_use_fumblerooskie::ClientCommandUseFumblerooskie;
use crate::commands::client_command_use_hatred::ClientCommandUseHatred;
use crate::commands::client_command_use_igors::ClientCommandUseIgors;
use crate::commands::client_command_use_inducement::ClientCommandUseInducement;
use crate::commands::client_command_use_multi_block_dice_re_roll::ClientCommandUseMultiBlockDiceReRoll;
use crate::commands::client_command_use_pro_re_roll_for_block::ClientCommandUseProReRollForBlock;
use crate::commands::client_command_use_re_roll::ClientCommandUseReRoll;
use crate::commands::client_command_use_re_roll_for_target::ClientCommandUseReRollForTarget;
use crate::commands::client_command_use_single_block_die_re_roll::ClientCommandUseSingleBlockDieReRoll;
use crate::commands::client_command_use_skill::ClientCommandUseSkill;
use crate::commands::client_command_use_team_mates_wisdom::ClientCommandUseTeamMatesWisdom;
use crate::commands::client_command_user_settings::ClientCommandUserSettings;
use crate::commands::client_command_wizard_spell::ClientCommandWizardSpell;

/// Sum type over every genuine `ClientCommand*` struct, keyed the same way Java's
/// `NetCommandId.createNetCommand()` switch keys its instantiation.
#[derive(Debug, Clone)]
pub enum AnyClientCommand {
    ClientActingPlayer(ClientCommandActingPlayer),
    ClientAddSketch(ClientCommandAddSketch),
    ClientApothecaryChoice(ClientCommandApothecaryChoice),
    ClientArgueTheCall(ClientCommandArgueTheCall),
    ClientBlitzMove(ClientCommandBlitzMove),
    ClientBlock(ClientCommandBlock),
    ClientBlockChoice(ClientCommandBlockChoice),
    ClientBlockOrReRollChoiceForTarget(ClientCommandBlockOrReRollChoiceForTarget),
    ClientBloodlustAction(ClientCommandBloodlustAction),
    ClientBuyCard(ClientCommandBuyCard),
    ClientBuyInducements(ClientCommandBuyInducements),
    ClientClearSketches(ClientCommandClearSketches),
    ClientCloseSession(ClientCommandCloseSession),
    ClientCoinChoice(ClientCommandCoinChoice),
    ClientConcedeGame(ClientCommandConcedeGame),
    ClientConfirm(ClientCommandConfirm),
    ClientDebugClientState(ClientCommandDebugClientState),
    ClientEndTurn(ClientCommandEndTurn),
    ClientFieldCoordinate(ClientCommandFieldCoordinate),
    ClientFollowupChoice(ClientCommandFollowupChoice),
    ClientFoul(ClientCommandFoul),
    ClientGaze(ClientCommandGaze),
    ClientHandOver(ClientCommandHandOver),
    ClientIllegalProcedure(ClientCommandIllegalProcedure),
    ClientInterceptorChoice(ClientCommandInterceptorChoice),
    ClientJoin(ClientCommandJoin),
    ClientJoinReplay(ClientCommandJoinReplay),
    ClientJourneymen(ClientCommandJourneymen),
    ClientKeywordSelection(ClientCommandKeywordSelection),
    ClientKickOffResultChoice(ClientCommandKickOffResultChoice),
    ClientKickTeamMate(ClientCommandKickTeamMate),
    ClientKickoff(ClientCommandKickoff),
    ClientLoadAutomaticPlayerMarkings(ClientCommandLoadAutomaticPlayerMarkings),
    ClientMove(ClientCommandMove),
    ClientPass(ClientCommandPass),
    ClientPasswordChallenge(ClientCommandPasswordChallenge),
    ClientPettyCash(ClientCommandPettyCash),
    ClientPickUpChoice(ClientCommandPickUpChoice),
    ClientPileDriver(ClientCommandPileDriver),
    ClientPing(ClientCommandPing),
    ClientPlayerChoice(ClientCommandPlayerChoice),
    ClientPositionSelection(ClientCommandPositionSelection),
    ClientPuntToCrowd(ClientCommandPuntToCrowd),
    ClientPushback(ClientCommandPushback),
    ClientReceiveChoice(ClientCommandReceiveChoice),
    ClientRemoveSketches(ClientCommandRemoveSketches),
    ClientReplay(ClientCommandReplay),
    ClientReplayStatus(ClientCommandReplayStatus),
    ClientRequestVersion(ClientCommandRequestVersion),
    ClientSelectCardToBuy(ClientCommandSelectCardToBuy),
    ClientSelectWeather(ClientCommandSelectWeather),
    ClientSetBlockTargetSelection(ClientCommandSetBlockTargetSelection),
    ClientSetMarker(ClientCommandSetMarker),
    ClientSetPreventSketching(ClientCommandSetPreventSketching),
    ClientSetupPlayer(ClientCommandSetupPlayer),
    ClientSketchAddCoordinate(ClientCommandSketchAddCoordinate),
    ClientSketchSetColor(ClientCommandSketchSetColor),
    ClientSketchSetLabel(ClientCommandSketchSetLabel),
    ClientPrayerSelection(ClientCommandSkillSelection),
    ClientStartGame(ClientCommandStartGame),
    ClientSwoop(ClientCommandSwoop),
    ClientSynchronousMultiBlock(ClientCommandSynchronousMultiBlock),
    ClientTalk(ClientCommandTalk),
    ClientTargetSelected(ClientCommandTargetSelected),
    ClientTeamSetupDelete(ClientCommandTeamSetupDelete),
    ClientTeamSetupLoad(ClientCommandTeamSetupLoad),
    ClientTeamSetupSave(ClientCommandTeamSetupSave),
    ClientThrowKeg(ClientCommandThrowKeg),
    ClientThrowTeamMate(ClientCommandThrowTeamMate),
    ClientTouchback(ClientCommandTouchback),
    ClientTransferReplayControl(ClientCommandTransferReplayControl),
    ClientUnsetBlockTargetSelection(ClientCommandUnsetBlockTargetSelection),
    ClientUpdatePlayerMarkings(ClientCommandUpdatePlayerMarkings),
    ClientUseApothecaries(ClientCommandUseApothecaries),
    ClientUseApothecary(ClientCommandUseApothecary),
    ClientUseBrawler(ClientCommandUseBrawler),
    ClientUseChainsaw(ClientCommandUseChainsaw),
    ClientUseConsummateReRollForBlock(ClientCommandUseConsummateReRollForBlock),
    ClientUseFumblerooskie(ClientCommandUseFumblerooskie),
    ClientUseHatred(ClientCommandUseHatred),
    ClientUseIgors(ClientCommandUseIgors),
    ClientUseInducement(ClientCommandUseInducement),
    ClientUseMultiBlockDiceReRoll(ClientCommandUseMultiBlockDiceReRoll),
    ClientUseProReRollForBlock(ClientCommandUseProReRollForBlock),
    ClientUseReRoll(ClientCommandUseReRoll),
    ClientUseReRollForTarget(ClientCommandUseReRollForTarget),
    ClientUseSingleBlockDieReRoll(ClientCommandUseSingleBlockDieReRoll),
    ClientUseSkill(ClientCommandUseSkill),
    ClientUseTeamMatesWisdom(ClientCommandUseTeamMatesWisdom),
    ClientUserSettings(ClientCommandUserSettings),
    ClientWizardSpell(ClientCommandWizardSpell),
}

impl NetCommand for AnyClientCommand {
    fn get_id(&self) -> NetCommandId {
        match self {
            AnyClientCommand::ClientActingPlayer(_) => NetCommandId::ClientActingPlayer,
            AnyClientCommand::ClientAddSketch(_) => NetCommandId::ClientAddSketch,
            AnyClientCommand::ClientApothecaryChoice(_) => NetCommandId::ClientApothecaryChoice,
            AnyClientCommand::ClientArgueTheCall(_) => NetCommandId::ClientArgueTheCall,
            AnyClientCommand::ClientBlitzMove(_) => NetCommandId::ClientBlitzMove,
            AnyClientCommand::ClientBlock(_) => NetCommandId::ClientBlock,
            AnyClientCommand::ClientBlockChoice(_) => NetCommandId::ClientBlockChoice,
            AnyClientCommand::ClientBlockOrReRollChoiceForTarget(_) => NetCommandId::ClientBlockOrReRollChoiceForTarget,
            AnyClientCommand::ClientBloodlustAction(_) => NetCommandId::ClientBloodlustAction,
            AnyClientCommand::ClientBuyCard(_) => NetCommandId::ClientBuyCard,
            AnyClientCommand::ClientBuyInducements(_) => NetCommandId::ClientBuyInducements,
            AnyClientCommand::ClientClearSketches(_) => NetCommandId::ClientClearSketches,
            AnyClientCommand::ClientCloseSession(_) => NetCommandId::ClientCloseSession,
            AnyClientCommand::ClientCoinChoice(_) => NetCommandId::ClientCoinChoice,
            AnyClientCommand::ClientConcedeGame(_) => NetCommandId::ClientConcedeGame,
            AnyClientCommand::ClientConfirm(_) => NetCommandId::ClientConfirm,
            AnyClientCommand::ClientDebugClientState(_) => NetCommandId::ClientDebugClientState,
            AnyClientCommand::ClientEndTurn(_) => NetCommandId::ClientEndTurn,
            AnyClientCommand::ClientFieldCoordinate(_) => NetCommandId::ClientFieldCoordinate,
            AnyClientCommand::ClientFollowupChoice(_) => NetCommandId::ClientFollowupChoice,
            AnyClientCommand::ClientFoul(_) => NetCommandId::ClientFoul,
            AnyClientCommand::ClientGaze(_) => NetCommandId::ClientGaze,
            AnyClientCommand::ClientHandOver(_) => NetCommandId::ClientHandOver,
            AnyClientCommand::ClientIllegalProcedure(_) => NetCommandId::ClientIllegalProcedure,
            AnyClientCommand::ClientInterceptorChoice(_) => NetCommandId::ClientInterceptorChoice,
            AnyClientCommand::ClientJoin(_) => NetCommandId::ClientJoin,
            AnyClientCommand::ClientJoinReplay(_) => NetCommandId::ClientJoinReplay,
            AnyClientCommand::ClientJourneymen(_) => NetCommandId::ClientJourneymen,
            AnyClientCommand::ClientKeywordSelection(_) => NetCommandId::ClientKeywordSelection,
            AnyClientCommand::ClientKickOffResultChoice(_) => NetCommandId::ClientKickOffResultChoice,
            AnyClientCommand::ClientKickTeamMate(_) => NetCommandId::ClientKickTeamMate,
            AnyClientCommand::ClientKickoff(_) => NetCommandId::ClientKickoff,
            AnyClientCommand::ClientLoadAutomaticPlayerMarkings(_) => NetCommandId::ClientLoadAutomaticPlayerMarkings,
            AnyClientCommand::ClientMove(_) => NetCommandId::ClientMove,
            AnyClientCommand::ClientPass(_) => NetCommandId::ClientPass,
            AnyClientCommand::ClientPasswordChallenge(_) => NetCommandId::ClientPasswordChallenge,
            AnyClientCommand::ClientPettyCash(_) => NetCommandId::ClientPettyCash,
            AnyClientCommand::ClientPickUpChoice(_) => NetCommandId::ClientPickUpChoice,
            AnyClientCommand::ClientPileDriver(_) => NetCommandId::ClientPileDriver,
            AnyClientCommand::ClientPing(_) => NetCommandId::ClientPing,
            AnyClientCommand::ClientPlayerChoice(_) => NetCommandId::ClientPlayerChoice,
            AnyClientCommand::ClientPositionSelection(_) => NetCommandId::ClientPositionSelection,
            AnyClientCommand::ClientPuntToCrowd(_) => NetCommandId::ClientPuntToCrowd,
            AnyClientCommand::ClientPushback(_) => NetCommandId::ClientPushback,
            AnyClientCommand::ClientReceiveChoice(_) => NetCommandId::ClientReceiveChoice,
            AnyClientCommand::ClientRemoveSketches(_) => NetCommandId::ClientRemoveSketches,
            AnyClientCommand::ClientReplay(_) => NetCommandId::ClientReplay,
            AnyClientCommand::ClientReplayStatus(_) => NetCommandId::ClientReplayStatus,
            AnyClientCommand::ClientRequestVersion(_) => NetCommandId::ClientRequestVersion,
            AnyClientCommand::ClientSelectCardToBuy(_) => NetCommandId::ClientSelectCardToBuy,
            AnyClientCommand::ClientSelectWeather(_) => NetCommandId::ClientSelectWeather,
            AnyClientCommand::ClientSetBlockTargetSelection(_) => NetCommandId::ClientSetBlockTargetSelection,
            AnyClientCommand::ClientSetMarker(_) => NetCommandId::ClientSetMarker,
            AnyClientCommand::ClientSetPreventSketching(_) => NetCommandId::ClientSetPreventSketching,
            AnyClientCommand::ClientSetupPlayer(_) => NetCommandId::ClientSetupPlayer,
            AnyClientCommand::ClientSketchAddCoordinate(_) => NetCommandId::ClientSketchAddCoordinate,
            AnyClientCommand::ClientSketchSetColor(_) => NetCommandId::ClientSketchSetColor,
            AnyClientCommand::ClientSketchSetLabel(_) => NetCommandId::ClientSketchSetLabel,
            AnyClientCommand::ClientPrayerSelection(_) => NetCommandId::ClientPrayerSelection,
            AnyClientCommand::ClientStartGame(_) => NetCommandId::ClientStartGame,
            AnyClientCommand::ClientSwoop(_) => NetCommandId::ClientSwoop,
            AnyClientCommand::ClientSynchronousMultiBlock(_) => NetCommandId::ClientSynchronousMultiBlock,
            AnyClientCommand::ClientTalk(_) => NetCommandId::ClientTalk,
            AnyClientCommand::ClientTargetSelected(_) => NetCommandId::ClientTargetSelected,
            AnyClientCommand::ClientTeamSetupDelete(_) => NetCommandId::ClientTeamSetupDelete,
            AnyClientCommand::ClientTeamSetupLoad(_) => NetCommandId::ClientTeamSetupLoad,
            AnyClientCommand::ClientTeamSetupSave(_) => NetCommandId::ClientTeamSetupSave,
            AnyClientCommand::ClientThrowKeg(_) => NetCommandId::ClientThrowKeg,
            AnyClientCommand::ClientThrowTeamMate(_) => NetCommandId::ClientThrowTeamMate,
            AnyClientCommand::ClientTouchback(_) => NetCommandId::ClientTouchback,
            AnyClientCommand::ClientTransferReplayControl(_) => NetCommandId::ClientTransferReplayControl,
            AnyClientCommand::ClientUnsetBlockTargetSelection(_) => NetCommandId::ClientUnsetBlockTargetSelection,
            AnyClientCommand::ClientUpdatePlayerMarkings(_) => NetCommandId::ClientUpdatePlayerMarkings,
            AnyClientCommand::ClientUseApothecaries(_) => NetCommandId::ClientUseApothecaries,
            AnyClientCommand::ClientUseApothecary(_) => NetCommandId::ClientUseApothecary,
            AnyClientCommand::ClientUseBrawler(_) => NetCommandId::ClientUseBrawler,
            AnyClientCommand::ClientUseChainsaw(_) => NetCommandId::ClientUseChainsaw,
            AnyClientCommand::ClientUseConsummateReRollForBlock(_) => NetCommandId::ClientUseConsummateReRollForBlock,
            AnyClientCommand::ClientUseFumblerooskie(_) => NetCommandId::ClientUseFumblerooskie,
            AnyClientCommand::ClientUseHatred(_) => NetCommandId::ClientUseHatred,
            AnyClientCommand::ClientUseIgors(_) => NetCommandId::ClientUseIgors,
            AnyClientCommand::ClientUseInducement(_) => NetCommandId::ClientUseInducement,
            AnyClientCommand::ClientUseMultiBlockDiceReRoll(_) => NetCommandId::ClientUseMultiBlockDiceReRoll,
            AnyClientCommand::ClientUseProReRollForBlock(_) => NetCommandId::ClientUseProReRollForBlock,
            AnyClientCommand::ClientUseReRoll(_) => NetCommandId::ClientUseReRoll,
            AnyClientCommand::ClientUseReRollForTarget(_) => NetCommandId::ClientUseReRollForTarget,
            AnyClientCommand::ClientUseSingleBlockDieReRoll(_) => NetCommandId::ClientUseSingleBlockDieReRoll,
            AnyClientCommand::ClientUseSkill(_) => NetCommandId::ClientUseSkill,
            AnyClientCommand::ClientUseTeamMatesWisdom(_) => NetCommandId::ClientUseTeamMatesWisdom,
            AnyClientCommand::ClientUserSettings(_) => NetCommandId::ClientUserSettings,
            AnyClientCommand::ClientWizardSpell(_) => NetCommandId::ClientWizardSpell,
        }
    }
}

impl AnyClientCommand {
    /// Java: `NetCommand.toJsonValue()` dispatched to the concrete subclass.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            AnyClientCommand::ClientActingPlayer(c) => c.to_json_value(),
            AnyClientCommand::ClientAddSketch(c) => c.to_json_value(),
            AnyClientCommand::ClientApothecaryChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientArgueTheCall(c) => c.to_json_value(),
            AnyClientCommand::ClientBlitzMove(c) => c.to_json_value(),
            AnyClientCommand::ClientBlock(c) => c.to_json_value(),
            AnyClientCommand::ClientBlockChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientBlockOrReRollChoiceForTarget(c) => c.to_json_value(),
            AnyClientCommand::ClientBloodlustAction(c) => c.to_json_value(),
            AnyClientCommand::ClientBuyCard(c) => c.to_json_value(),
            AnyClientCommand::ClientBuyInducements(c) => c.to_json_value(),
            AnyClientCommand::ClientClearSketches(c) => c.to_json_value(),
            AnyClientCommand::ClientCloseSession(c) => c.to_json_value(),
            AnyClientCommand::ClientCoinChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientConcedeGame(c) => c.to_json_value(),
            AnyClientCommand::ClientConfirm(c) => c.to_json_value(),
            AnyClientCommand::ClientDebugClientState(c) => c.to_json_value(),
            AnyClientCommand::ClientEndTurn(c) => c.to_json_value(),
            AnyClientCommand::ClientFieldCoordinate(c) => c.to_json_value(),
            AnyClientCommand::ClientFollowupChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientFoul(c) => c.to_json_value(),
            AnyClientCommand::ClientGaze(c) => c.to_json_value(),
            AnyClientCommand::ClientHandOver(c) => c.to_json_value(),
            AnyClientCommand::ClientIllegalProcedure(c) => c.to_json_value(),
            AnyClientCommand::ClientInterceptorChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientJoin(c) => c.to_json_value(),
            AnyClientCommand::ClientJoinReplay(c) => c.to_json_value(),
            AnyClientCommand::ClientJourneymen(c) => c.to_json_value(),
            AnyClientCommand::ClientKeywordSelection(c) => c.to_json_value(),
            AnyClientCommand::ClientKickOffResultChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientKickTeamMate(c) => c.to_json_value(),
            AnyClientCommand::ClientKickoff(c) => c.to_json_value(),
            AnyClientCommand::ClientLoadAutomaticPlayerMarkings(c) => c.to_json_value(),
            AnyClientCommand::ClientMove(c) => c.to_json_value(),
            AnyClientCommand::ClientPass(c) => c.to_json_value(),
            AnyClientCommand::ClientPasswordChallenge(c) => c.to_json_value(),
            AnyClientCommand::ClientPettyCash(c) => c.to_json_value(),
            AnyClientCommand::ClientPickUpChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientPileDriver(c) => c.to_json_value(),
            AnyClientCommand::ClientPing(c) => c.to_json_value(),
            AnyClientCommand::ClientPlayerChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientPositionSelection(c) => c.to_json_value(),
            AnyClientCommand::ClientPuntToCrowd(c) => c.to_json_value(),
            AnyClientCommand::ClientPushback(c) => c.to_json_value(),
            AnyClientCommand::ClientReceiveChoice(c) => c.to_json_value(),
            AnyClientCommand::ClientRemoveSketches(c) => c.to_json_value(),
            AnyClientCommand::ClientReplay(c) => c.to_json_value(),
            AnyClientCommand::ClientReplayStatus(c) => c.to_json_value(),
            AnyClientCommand::ClientRequestVersion(c) => c.to_json_value(),
            AnyClientCommand::ClientSelectCardToBuy(c) => c.to_json_value(),
            AnyClientCommand::ClientSelectWeather(c) => c.to_json_value(),
            AnyClientCommand::ClientSetBlockTargetSelection(c) => c.to_json_value(),
            AnyClientCommand::ClientSetMarker(c) => c.to_json_value(),
            AnyClientCommand::ClientSetPreventSketching(c) => c.to_json_value(),
            AnyClientCommand::ClientSetupPlayer(c) => c.to_json_value(),
            AnyClientCommand::ClientSketchAddCoordinate(c) => c.to_json_value(),
            AnyClientCommand::ClientSketchSetColor(c) => c.to_json_value(),
            AnyClientCommand::ClientSketchSetLabel(c) => c.to_json_value(),
            AnyClientCommand::ClientPrayerSelection(c) => c.to_json_value(),
            AnyClientCommand::ClientStartGame(c) => c.to_json_value(),
            AnyClientCommand::ClientSwoop(c) => c.to_json_value(),
            AnyClientCommand::ClientSynchronousMultiBlock(c) => c.to_json_value(),
            AnyClientCommand::ClientTalk(c) => c.to_json_value(),
            AnyClientCommand::ClientTargetSelected(c) => c.to_json_value(),
            AnyClientCommand::ClientTeamSetupDelete(c) => c.to_json_value(),
            AnyClientCommand::ClientTeamSetupLoad(c) => c.to_json_value(),
            AnyClientCommand::ClientTeamSetupSave(c) => c.to_json_value(),
            AnyClientCommand::ClientThrowKeg(c) => c.to_json_value(),
            AnyClientCommand::ClientThrowTeamMate(c) => c.to_json_value(),
            AnyClientCommand::ClientTouchback(c) => c.to_json_value(),
            AnyClientCommand::ClientTransferReplayControl(c) => c.to_json_value(),
            AnyClientCommand::ClientUnsetBlockTargetSelection(c) => c.to_json_value(),
            AnyClientCommand::ClientUpdatePlayerMarkings(c) => c.to_json_value(),
            AnyClientCommand::ClientUseApothecaries(c) => c.to_json_value(),
            AnyClientCommand::ClientUseApothecary(c) => c.to_json_value(),
            AnyClientCommand::ClientUseBrawler(c) => c.to_json_value(),
            AnyClientCommand::ClientUseChainsaw(c) => c.to_json_value(),
            AnyClientCommand::ClientUseConsummateReRollForBlock(c) => c.to_json_value(),
            AnyClientCommand::ClientUseFumblerooskie(c) => c.to_json_value(),
            AnyClientCommand::ClientUseHatred(c) => c.to_json_value(),
            AnyClientCommand::ClientUseIgors(c) => c.to_json_value(),
            AnyClientCommand::ClientUseInducement(c) => c.to_json_value(),
            AnyClientCommand::ClientUseMultiBlockDiceReRoll(c) => c.to_json_value(),
            AnyClientCommand::ClientUseProReRollForBlock(c) => c.to_json_value(),
            AnyClientCommand::ClientUseReRoll(c) => c.to_json_value(),
            AnyClientCommand::ClientUseReRollForTarget(c) => c.to_json_value(),
            AnyClientCommand::ClientUseSingleBlockDieReRoll(c) => c.to_json_value(),
            AnyClientCommand::ClientUseSkill(c) => c.to_json_value(),
            AnyClientCommand::ClientUseTeamMatesWisdom(c) => c.to_json_value(),
            AnyClientCommand::ClientUserSettings(c) => c.to_json_value(),
            AnyClientCommand::ClientWizardSpell(c) => c.to_json_value(),
        }
    }

    /// Java: `NetCommandId.createNetCommand()` + `NetCommand.initFrom(...)`.
    pub fn from_json(id: NetCommandId, json: &serde_json::Value) -> Option<Self> {
        Some(match id {
            NetCommandId::ClientActingPlayer => AnyClientCommand::ClientActingPlayer(ClientCommandActingPlayer::from_json(json)),
            NetCommandId::ClientAddSketch => AnyClientCommand::ClientAddSketch(ClientCommandAddSketch::from_json(json)),
            NetCommandId::ClientApothecaryChoice => AnyClientCommand::ClientApothecaryChoice(ClientCommandApothecaryChoice::from_json(json)),
            NetCommandId::ClientArgueTheCall => AnyClientCommand::ClientArgueTheCall(ClientCommandArgueTheCall::from_json(json)),
            NetCommandId::ClientBlitzMove => AnyClientCommand::ClientBlitzMove(ClientCommandBlitzMove::from_json(json)),
            NetCommandId::ClientBlock => AnyClientCommand::ClientBlock(ClientCommandBlock::from_json(json)),
            NetCommandId::ClientBlockChoice => AnyClientCommand::ClientBlockChoice(ClientCommandBlockChoice::from_json(json)),
            NetCommandId::ClientBlockOrReRollChoiceForTarget => AnyClientCommand::ClientBlockOrReRollChoiceForTarget(ClientCommandBlockOrReRollChoiceForTarget::from_json(json)),
            NetCommandId::ClientBloodlustAction => AnyClientCommand::ClientBloodlustAction(ClientCommandBloodlustAction::from_json(json)),
            NetCommandId::ClientBuyCard => AnyClientCommand::ClientBuyCard(ClientCommandBuyCard::from_json(json)),
            NetCommandId::ClientBuyInducements => AnyClientCommand::ClientBuyInducements(ClientCommandBuyInducements::from_json(json)),
            NetCommandId::ClientClearSketches => AnyClientCommand::ClientClearSketches(ClientCommandClearSketches::from_json(json)),
            NetCommandId::ClientCloseSession => AnyClientCommand::ClientCloseSession(ClientCommandCloseSession::from_json(json)),
            NetCommandId::ClientCoinChoice => AnyClientCommand::ClientCoinChoice(ClientCommandCoinChoice::from_json(json)),
            NetCommandId::ClientConcedeGame => AnyClientCommand::ClientConcedeGame(ClientCommandConcedeGame::from_json(json)),
            NetCommandId::ClientConfirm => AnyClientCommand::ClientConfirm(ClientCommandConfirm::from_json(json)),
            NetCommandId::ClientDebugClientState => AnyClientCommand::ClientDebugClientState(ClientCommandDebugClientState::from_json(json)),
            NetCommandId::ClientEndTurn => AnyClientCommand::ClientEndTurn(ClientCommandEndTurn::from_json(json)),
            NetCommandId::ClientFieldCoordinate => AnyClientCommand::ClientFieldCoordinate(ClientCommandFieldCoordinate::from_json(json)),
            NetCommandId::ClientFollowupChoice => AnyClientCommand::ClientFollowupChoice(ClientCommandFollowupChoice::from_json(json)),
            NetCommandId::ClientFoul => AnyClientCommand::ClientFoul(ClientCommandFoul::from_json(json)),
            NetCommandId::ClientGaze => AnyClientCommand::ClientGaze(ClientCommandGaze::from_json(json)),
            NetCommandId::ClientHandOver => AnyClientCommand::ClientHandOver(ClientCommandHandOver::from_json(json)),
            NetCommandId::ClientIllegalProcedure => AnyClientCommand::ClientIllegalProcedure(ClientCommandIllegalProcedure::from_json(json)),
            NetCommandId::ClientInterceptorChoice => AnyClientCommand::ClientInterceptorChoice(ClientCommandInterceptorChoice::from_json(json)),
            NetCommandId::ClientJoin => AnyClientCommand::ClientJoin(ClientCommandJoin::from_json(json)),
            NetCommandId::ClientJoinReplay => AnyClientCommand::ClientJoinReplay(ClientCommandJoinReplay::from_json(json)),
            NetCommandId::ClientJourneymen => AnyClientCommand::ClientJourneymen(ClientCommandJourneymen::from_json(json)),
            NetCommandId::ClientKeywordSelection => AnyClientCommand::ClientKeywordSelection(ClientCommandKeywordSelection::from_json(json)),
            NetCommandId::ClientKickOffResultChoice => AnyClientCommand::ClientKickOffResultChoice(ClientCommandKickOffResultChoice::from_json(json)),
            NetCommandId::ClientKickTeamMate => AnyClientCommand::ClientKickTeamMate(ClientCommandKickTeamMate::from_json(json)),
            NetCommandId::ClientKickoff => AnyClientCommand::ClientKickoff(ClientCommandKickoff::from_json(json)),
            NetCommandId::ClientLoadAutomaticPlayerMarkings => AnyClientCommand::ClientLoadAutomaticPlayerMarkings(ClientCommandLoadAutomaticPlayerMarkings::from_json(json)),
            NetCommandId::ClientMove => AnyClientCommand::ClientMove(ClientCommandMove::from_json(json)),
            NetCommandId::ClientPass => AnyClientCommand::ClientPass(ClientCommandPass::from_json(json)),
            NetCommandId::ClientPasswordChallenge => AnyClientCommand::ClientPasswordChallenge(ClientCommandPasswordChallenge::from_json(json)),
            NetCommandId::ClientPettyCash => AnyClientCommand::ClientPettyCash(ClientCommandPettyCash::from_json(json)),
            NetCommandId::ClientPickUpChoice => AnyClientCommand::ClientPickUpChoice(ClientCommandPickUpChoice::from_json(json)),
            NetCommandId::ClientPileDriver => AnyClientCommand::ClientPileDriver(ClientCommandPileDriver::from_json(json)),
            NetCommandId::ClientPing => AnyClientCommand::ClientPing(ClientCommandPing::from_json(json)),
            NetCommandId::ClientPlayerChoice => AnyClientCommand::ClientPlayerChoice(ClientCommandPlayerChoice::from_json(json)),
            NetCommandId::ClientPositionSelection => AnyClientCommand::ClientPositionSelection(ClientCommandPositionSelection::from_json(json)),
            NetCommandId::ClientPuntToCrowd => AnyClientCommand::ClientPuntToCrowd(ClientCommandPuntToCrowd::from_json(json)),
            NetCommandId::ClientPushback => AnyClientCommand::ClientPushback(ClientCommandPushback::from_json(json)),
            NetCommandId::ClientReceiveChoice => AnyClientCommand::ClientReceiveChoice(ClientCommandReceiveChoice::from_json(json)),
            NetCommandId::ClientRemoveSketches => AnyClientCommand::ClientRemoveSketches(ClientCommandRemoveSketches::from_json(json)),
            NetCommandId::ClientReplay => AnyClientCommand::ClientReplay(ClientCommandReplay::from_json(json)),
            NetCommandId::ClientReplayStatus => AnyClientCommand::ClientReplayStatus(ClientCommandReplayStatus::from_json(json)),
            NetCommandId::ClientRequestVersion => AnyClientCommand::ClientRequestVersion(ClientCommandRequestVersion::from_json(json)),
            NetCommandId::ClientSelectCardToBuy => AnyClientCommand::ClientSelectCardToBuy(ClientCommandSelectCardToBuy::from_json(json)),
            NetCommandId::ClientSelectWeather => AnyClientCommand::ClientSelectWeather(ClientCommandSelectWeather::from_json(json)),
            NetCommandId::ClientSetBlockTargetSelection => AnyClientCommand::ClientSetBlockTargetSelection(ClientCommandSetBlockTargetSelection::from_json(json)),
            NetCommandId::ClientSetMarker => AnyClientCommand::ClientSetMarker(ClientCommandSetMarker::from_json(json)),
            NetCommandId::ClientSetPreventSketching => AnyClientCommand::ClientSetPreventSketching(ClientCommandSetPreventSketching::from_json(json)),
            NetCommandId::ClientSetupPlayer => AnyClientCommand::ClientSetupPlayer(ClientCommandSetupPlayer::from_json(json)),
            NetCommandId::ClientSketchAddCoordinate => AnyClientCommand::ClientSketchAddCoordinate(ClientCommandSketchAddCoordinate::from_json(json)),
            NetCommandId::ClientSketchSetColor => AnyClientCommand::ClientSketchSetColor(ClientCommandSketchSetColor::from_json(json)),
            NetCommandId::ClientSketchSetLabel => AnyClientCommand::ClientSketchSetLabel(ClientCommandSketchSetLabel::from_json(json)),
            NetCommandId::ClientPrayerSelection => AnyClientCommand::ClientPrayerSelection(ClientCommandSkillSelection::from_json(json)),
            NetCommandId::ClientStartGame => AnyClientCommand::ClientStartGame(ClientCommandStartGame::from_json(json)),
            NetCommandId::ClientSwoop => AnyClientCommand::ClientSwoop(ClientCommandSwoop::from_json(json)),
            NetCommandId::ClientSynchronousMultiBlock => AnyClientCommand::ClientSynchronousMultiBlock(ClientCommandSynchronousMultiBlock::from_json(json)),
            NetCommandId::ClientTalk => AnyClientCommand::ClientTalk(ClientCommandTalk::from_json(json)),
            NetCommandId::ClientTargetSelected => AnyClientCommand::ClientTargetSelected(ClientCommandTargetSelected::from_json(json)),
            NetCommandId::ClientTeamSetupDelete => AnyClientCommand::ClientTeamSetupDelete(ClientCommandTeamSetupDelete::from_json(json)),
            NetCommandId::ClientTeamSetupLoad => AnyClientCommand::ClientTeamSetupLoad(ClientCommandTeamSetupLoad::from_json(json)),
            NetCommandId::ClientTeamSetupSave => AnyClientCommand::ClientTeamSetupSave(ClientCommandTeamSetupSave::from_json(json)),
            NetCommandId::ClientThrowKeg => AnyClientCommand::ClientThrowKeg(ClientCommandThrowKeg::from_json(json)),
            NetCommandId::ClientThrowTeamMate => AnyClientCommand::ClientThrowTeamMate(ClientCommandThrowTeamMate::from_json(json)),
            NetCommandId::ClientTouchback => AnyClientCommand::ClientTouchback(ClientCommandTouchback::from_json(json)),
            NetCommandId::ClientTransferReplayControl => AnyClientCommand::ClientTransferReplayControl(ClientCommandTransferReplayControl::from_json(json)),
            NetCommandId::ClientUnsetBlockTargetSelection => AnyClientCommand::ClientUnsetBlockTargetSelection(ClientCommandUnsetBlockTargetSelection::from_json(json)),
            NetCommandId::ClientUpdatePlayerMarkings => AnyClientCommand::ClientUpdatePlayerMarkings(ClientCommandUpdatePlayerMarkings::from_json(json)),
            NetCommandId::ClientUseApothecaries => AnyClientCommand::ClientUseApothecaries(ClientCommandUseApothecaries::from_json(json)),
            NetCommandId::ClientUseApothecary => AnyClientCommand::ClientUseApothecary(ClientCommandUseApothecary::from_json(json)),
            NetCommandId::ClientUseBrawler => AnyClientCommand::ClientUseBrawler(ClientCommandUseBrawler::from_json(json)),
            NetCommandId::ClientUseChainsaw => AnyClientCommand::ClientUseChainsaw(ClientCommandUseChainsaw::from_json(json)),
            NetCommandId::ClientUseConsummateReRollForBlock => AnyClientCommand::ClientUseConsummateReRollForBlock(ClientCommandUseConsummateReRollForBlock::from_json(json)),
            NetCommandId::ClientUseFumblerooskie => AnyClientCommand::ClientUseFumblerooskie(ClientCommandUseFumblerooskie::from_json(json)),
            NetCommandId::ClientUseHatred => AnyClientCommand::ClientUseHatred(ClientCommandUseHatred::from_json(json)),
            NetCommandId::ClientUseIgors => AnyClientCommand::ClientUseIgors(ClientCommandUseIgors::from_json(json)),
            NetCommandId::ClientUseInducement => AnyClientCommand::ClientUseInducement(ClientCommandUseInducement::from_json(json)),
            NetCommandId::ClientUseMultiBlockDiceReRoll => AnyClientCommand::ClientUseMultiBlockDiceReRoll(ClientCommandUseMultiBlockDiceReRoll::from_json(json)),
            NetCommandId::ClientUseProReRollForBlock => AnyClientCommand::ClientUseProReRollForBlock(ClientCommandUseProReRollForBlock::from_json(json)),
            NetCommandId::ClientUseReRoll => AnyClientCommand::ClientUseReRoll(ClientCommandUseReRoll::from_json(json)),
            NetCommandId::ClientUseReRollForTarget => AnyClientCommand::ClientUseReRollForTarget(ClientCommandUseReRollForTarget::from_json(json)),
            NetCommandId::ClientUseSingleBlockDieReRoll => AnyClientCommand::ClientUseSingleBlockDieReRoll(ClientCommandUseSingleBlockDieReRoll::from_json(json)),
            NetCommandId::ClientUseSkill => AnyClientCommand::ClientUseSkill(ClientCommandUseSkill::from_json(json)),
            NetCommandId::ClientUseTeamMatesWisdom => AnyClientCommand::ClientUseTeamMatesWisdom(ClientCommandUseTeamMatesWisdom::from_json(json)),
            NetCommandId::ClientUserSettings => AnyClientCommand::ClientUserSettings(ClientCommandUserSettings::from_json(json)),
            NetCommandId::ClientWizardSpell => AnyClientCommand::ClientWizardSpell(ClientCommandWizardSpell::from_json(json)),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_matches_wrapped_variant() {
        let cmd = AnyClientCommand::ClientEndTurn(ClientCommandEndTurn::new());
        assert_eq!(cmd.get_id(), NetCommandId::ClientEndTurn);
    }

    #[test]
    fn to_json_value_delegates_to_wrapped_command() {
        let cmd = AnyClientCommand::ClientEndTurn(ClientCommandEndTurn::new());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientEndTurn");
    }

    #[test]
    fn from_json_round_trip_for_known_id() {
        let original = AnyClientCommand::ClientEndTurn(ClientCommandEndTurn::new());
        let json = original.to_json_value();
        let restored = AnyClientCommand::from_json(NetCommandId::ClientEndTurn, &json).unwrap();
        assert_eq!(restored.get_id(), NetCommandId::ClientEndTurn);
    }

    #[test]
    fn from_json_returns_none_for_a_server_only_id() {
        let json = serde_json::json!({});
        assert!(AnyClientCommand::from_json(NetCommandId::ServerGameTime, &json).is_none());
    }

    #[test]
    fn from_json_dispatches_to_the_matching_struct_type() {
        let json = serde_json::json!({"netCommandId": "clientJoin"});
        let restored = AnyClientCommand::from_json(NetCommandId::ClientJoin, &json).unwrap();
        assert!(matches!(restored, AnyClientCommand::ClientJoin(_)));
    }
}
