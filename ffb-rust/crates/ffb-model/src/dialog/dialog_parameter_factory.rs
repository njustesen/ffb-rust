//! 1:1 translation of `com.fumbbl.ffb.dialog.DialogParameterFactory`, plus the polymorphic
//! `IDialogParameter` value (`DialogParameter`, one variant per concrete Java class the factory
//! creates) with the Java JSON semantics (BACKLOG §O): `for_json_value` = `forJsonValue`, an
//! unknown or missing dialog id -> `None` (the whole parameter is null), as in Java.

use super::dialog_id::DialogId;
use super::dialog_apothecary_choice_parameter::DialogApothecaryChoiceParameter;
use super::dialog_receive_choice_parameter::DialogReceiveChoiceParameter;
use super::dialog_re_roll_parameter::DialogReRollParameter;
use super::dialog_re_roll_properties_parameter::DialogReRollPropertiesParameter;
use super::dialog_re_roll_for_targets_parameter::DialogReRollForTargetsParameter;
use super::dialog_skill_use_parameter::DialogSkillUseParameter;
use super::dialog_use_apothecary_parameter::DialogUseApothecaryParameter;
use super::dialog_block_roll_parameter::DialogBlockRollParameter;
use super::dialog_block_roll_partial_re_roll_parameter::DialogBlockRollPartialReRollParameter;
use super::dialog_block_roll_properties_parameter::DialogBlockRollPropertiesParameter;
use super::dialog_player_choice_parameter::DialogPlayerChoiceParameter;
use super::dialog_interception_parameter::DialogInterceptionParameter;
use super::dialog_winnings_re_roll_parameter::DialogWinningsReRollParameter;
use super::dialog_bribes_parameter::DialogBribesParameter;
use super::dialog_game_statistics_parameter::DialogGameStatisticsParameter;
use super::dialog_join_parameter::DialogJoinParameter;
use super::dialog_start_game_parameter::DialogStartGameParameter;
use super::dialog_team_setup_parameter::DialogTeamSetupParameter;
use super::dialog_setup_error_parameter::DialogSetupErrorParameter;
use super::dialog_touchback_parameter::DialogTouchbackParameter;
use super::dialog_defender_action_parameter::DialogDefenderActionParameter;
use super::dialog_coin_choice_parameter::DialogCoinChoiceParameter;
use super::dialog_followup_choice_parameter::DialogFollowupChoiceParameter;
use super::dialog_concede_game_parameter::DialogConcedeGameParameter;
use super::dialog_piling_on_parameter::DialogPilingOnParameter;
use super::dialog_buy_inducements_parameter::DialogBuyInducementsParameter;
use super::dialog_journeymen_parameter::DialogJourneymenParameter;
use super::dialog_kick_skill_parameter::DialogKickSkillParameter;
use super::dialog_use_igor_parameter::DialogUseIgorParameter;
use super::dialog_kickoff_return_parameter::DialogKickoffReturnParameter;
use super::dialog_petty_cash_parameter::DialogPettyCashParameter;
use super::dialog_wizard_spell_parameter::DialogWizardSpellParameter;
use super::dialog_use_inducement_parameter::DialogUseInducementParameter;
use super::dialog_pass_block_parameter::DialogPassBlockParameter;
use super::dialog_buy_cards_parameter::DialogBuyCardsParameter;
use super::dialog_argue_the_call_parameter::DialogArgueTheCallParameter;
use super::dialog_swarming_players_parameter::DialogSwarmingPlayersParameter;
use super::dialog_swarming_error_parameter::DialogSwarmingErrorParameter;
use super::dialog_buy_cards_and_inducements_parameter::DialogBuyCardsAndInducementsParameter;
use super::dialog_select_blitz_target_parameter::DialogSelectBlitzTargetParameter;
use super::dialog_select_gaze_target_parameter::DialogSelectGazeTargetParameter;
use super::dialog_re_roll_block_for_targets_parameter::DialogReRollBlockForTargetsParameter;
use super::dialog_opponent_block_selection_parameter::DialogOpponentBlockSelectionParameter;
use super::dialog_use_apothecaries_parameter::DialogUseApothecariesParameter;
use super::dialog_use_igors_parameter::DialogUseIgorsParameter;
use super::dialog_pile_driver_parameter::DialogPileDriverParameter;
use super::dialog_use_chainsaw_parameter::DialogUseChainsawParameter;
use super::dialog_invalid_solid_defence_parameter::DialogInvalidSolidDefenceParameter;
use super::dialog_select_skill_parameter::DialogSelectSkillParameter;
use super::dialog_bribery_and_corruption_parameter::DialogBriberyAndCorruptionParameter;
use super::dialog_confirm_end_action_parameter::DialogConfirmEndActionParameter;
use super::dialog_select_weather_parameter::DialogSelectWeatherParameter;
use super::dialog_information_okay_parameter::DialogInformationOkayParameter;
use super::dialog_use_mortuary_assistant_parameter::DialogUseMortuaryAssistantParameter;
use super::dialog_use_mortuary_assistants_parameter::DialogUseMortuaryAssistantsParameter;
use super::dialog_kick_off_result_parameter::DialogKickOffResultParameter;
use super::dialog_bloodlust_action_parameter::DialogBloodlustActionParameter;
use super::dialog_penalty_shootout_parameter::DialogPenaltyShootoutParameter;
use super::dialog_buy_prayers_and_inducements_parameter::DialogBuyPrayersAndInducementsParameter;
use super::dialog_re_roll_block_for_targets_properties_parameter::DialogReRollBlockForTargetsPropertiesParameter;
use super::dialog_opponent_block_selection_properties_parameter::DialogOpponentBlockSelectionPropertiesParameter;
use super::dialog_pick_up_choice_parameter::DialogPickUpChoiceParameter;
use super::dialog_select_keyword_parameter::DialogSelectKeywordParameter;
use super::dialog_select_position_parameter::DialogSelectPositionParameter;
use super::dialog_re_roll_regeneration_multiple_parameter::DialogReRollRegenerationMultipleParameter;
use super::dialog_punt_to_crowd_parameter::DialogPuntToCrowdParameter;
use crate::enums::Rules;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct DialogParameterFactory;

/// A Java `IDialogParameter` of one of the classes `DialogParameterFactory` creates.
#[derive(Debug, Clone)]
pub enum DialogParameter {
    ApothecaryChoice(DialogApothecaryChoiceParameter),
    ReceiveChoice(DialogReceiveChoiceParameter),
    ReRoll(DialogReRollParameter),
    ReRollProperties(DialogReRollPropertiesParameter),
    ReRollForTargets(DialogReRollForTargetsParameter),
    SkillUse(DialogSkillUseParameter),
    UseApothecary(DialogUseApothecaryParameter),
    BlockRoll(DialogBlockRollParameter),
    BlockRollPartialReRoll(DialogBlockRollPartialReRollParameter),
    BlockRollProperties(DialogBlockRollPropertiesParameter),
    PlayerChoice(DialogPlayerChoiceParameter),
    Interception(DialogInterceptionParameter),
    WinningsReRoll(DialogWinningsReRollParameter),
    Bribes(DialogBribesParameter),
    GameStatistics(DialogGameStatisticsParameter),
    Join(DialogJoinParameter),
    StartGame(DialogStartGameParameter),
    TeamSetup(DialogTeamSetupParameter),
    SetupError(DialogSetupErrorParameter),
    Touchback(DialogTouchbackParameter),
    DefenderAction(DialogDefenderActionParameter),
    CoinChoice(DialogCoinChoiceParameter),
    FollowupChoice(DialogFollowupChoiceParameter),
    ConcedeGame(DialogConcedeGameParameter),
    PilingOn(DialogPilingOnParameter),
    BuyInducements(DialogBuyInducementsParameter),
    Journeymen(DialogJourneymenParameter),
    KickSkill(DialogKickSkillParameter),
    UseIgor(DialogUseIgorParameter),
    KickoffReturn(DialogKickoffReturnParameter),
    PettyCash(DialogPettyCashParameter),
    WizardSpell(DialogWizardSpellParameter),
    UseInducement(DialogUseInducementParameter),
    PassBlock(DialogPassBlockParameter),
    BuyCards(DialogBuyCardsParameter),
    ArgueTheCall(DialogArgueTheCallParameter),
    SwarmingPlayers(DialogSwarmingPlayersParameter),
    SwarmingError(DialogSwarmingErrorParameter),
    BuyCardsAndInducements(DialogBuyCardsAndInducementsParameter),
    SelectBlitzTarget(DialogSelectBlitzTargetParameter),
    SelectGazeTarget(DialogSelectGazeTargetParameter),
    ReRollBlockForTargets(DialogReRollBlockForTargetsParameter),
    OpponentBlockSelection(DialogOpponentBlockSelectionParameter),
    UseApothecaries(DialogUseApothecariesParameter),
    UseIgors(DialogUseIgorsParameter),
    PileDriver(DialogPileDriverParameter),
    UseChainsaw(DialogUseChainsawParameter),
    InvalidSolidDefence(DialogInvalidSolidDefenceParameter),
    SelectSkill(DialogSelectSkillParameter),
    BriberyAndCorruption(DialogBriberyAndCorruptionParameter),
    ConfirmEndAction(DialogConfirmEndActionParameter),
    SelectWeather(DialogSelectWeatherParameter),
    InformationOkay(DialogInformationOkayParameter),
    UseMortuaryAssistant(DialogUseMortuaryAssistantParameter),
    UseMortuaryAssistants(DialogUseMortuaryAssistantsParameter),
    KickOffResult(DialogKickOffResultParameter),
    BloodlustAction(DialogBloodlustActionParameter),
    PenaltyShootout(DialogPenaltyShootoutParameter),
    BuyPrayersAndInducements(DialogBuyPrayersAndInducementsParameter),
    ReRollBlockForTargetsProperties(DialogReRollBlockForTargetsPropertiesParameter),
    OpponentBlockSelectionProperties(DialogOpponentBlockSelectionPropertiesParameter),
    PickUpChoice(DialogPickUpChoiceParameter),
    SelectKeyword(DialogSelectKeywordParameter),
    SelectPosition(DialogSelectPositionParameter),
    ReRollRegenerationMultiple(DialogReRollRegenerationMultipleParameter),
    PuntToCrowd(DialogPuntToCrowdParameter),
}

impl DialogParameter {
    /// Java `IDialogParameter.getId()`.
    pub fn get_id(&self) -> DialogId {
        match self {
            DialogParameter::ApothecaryChoice(_) => DialogId::APOTHECARY_CHOICE,
            DialogParameter::ReceiveChoice(_) => DialogId::RECEIVE_CHOICE,
            DialogParameter::ReRoll(_) => DialogId::RE_ROLL,
            DialogParameter::ReRollProperties(_) => DialogId::RE_ROLL_PROPERTIES,
            DialogParameter::ReRollForTargets(_) => DialogId::RE_ROLL_FOR_TARGETS,
            DialogParameter::SkillUse(_) => DialogId::SKILL_USE,
            DialogParameter::UseApothecary(_) => DialogId::USE_APOTHECARY,
            DialogParameter::BlockRoll(_) => DialogId::BLOCK_ROLL,
            DialogParameter::BlockRollPartialReRoll(_) => DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL,
            DialogParameter::BlockRollProperties(_) => DialogId::BLOCK_ROLL_PROPERTIES,
            DialogParameter::PlayerChoice(_) => DialogId::PLAYER_CHOICE,
            DialogParameter::Interception(_) => DialogId::INTERCEPTION,
            DialogParameter::WinningsReRoll(_) => DialogId::WINNINGS_RE_ROLL,
            DialogParameter::Bribes(_) => DialogId::BRIBES,
            DialogParameter::GameStatistics(_) => DialogId::GAME_STATISTICS,
            DialogParameter::Join(_) => DialogId::JOIN,
            DialogParameter::StartGame(_) => DialogId::START_GAME,
            DialogParameter::TeamSetup(_) => DialogId::TEAM_SETUP,
            DialogParameter::SetupError(_) => DialogId::SETUP_ERROR,
            DialogParameter::Touchback(_) => DialogId::TOUCHBACK,
            DialogParameter::DefenderAction(_) => DialogId::DEFENDER_ACTION,
            DialogParameter::CoinChoice(_) => DialogId::COIN_CHOICE,
            DialogParameter::FollowupChoice(_) => DialogId::FOLLOWUP_CHOICE,
            DialogParameter::ConcedeGame(_) => DialogId::CONCEDE_GAME,
            DialogParameter::PilingOn(_) => DialogId::PILING_ON,
            DialogParameter::BuyInducements(_) => DialogId::BUY_INDUCEMENTS,
            DialogParameter::Journeymen(_) => DialogId::JOURNEYMEN,
            DialogParameter::KickSkill(_) => DialogId::KICK_SKILL,
            DialogParameter::UseIgor(_) => DialogId::USE_IGOR,
            DialogParameter::KickoffReturn(_) => DialogId::KICKOFF_RETURN,
            DialogParameter::PettyCash(_) => DialogId::PETTY_CASH,
            DialogParameter::WizardSpell(_) => DialogId::WIZARD_SPELL,
            DialogParameter::UseInducement(_) => DialogId::USE_INDUCEMENT,
            DialogParameter::PassBlock(_) => DialogId::PASS_BLOCK,
            DialogParameter::BuyCards(_) => DialogId::BUY_CARDS,
            DialogParameter::ArgueTheCall(_) => DialogId::ARGUE_THE_CALL,
            DialogParameter::SwarmingPlayers(_) => DialogId::SWARMING,
            DialogParameter::SwarmingError(_) => DialogId::SWARMING_ERROR,
            DialogParameter::BuyCardsAndInducements(_) => DialogId::BUY_CARDS_AND_INDUCEMENTS,
            DialogParameter::SelectBlitzTarget(_) => DialogId::SELECT_BLITZ_TARGET,
            DialogParameter::SelectGazeTarget(_) => DialogId::SELECT_GAZE_TARGET,
            DialogParameter::ReRollBlockForTargets(_) => DialogId::RE_ROLL_BLOCK_FOR_TARGETS,
            DialogParameter::OpponentBlockSelection(_) => DialogId::OPPONENT_BLOCK_SELECTION,
            DialogParameter::UseApothecaries(_) => DialogId::USE_APOTHECARIES,
            DialogParameter::UseIgors(_) => DialogId::USE_IGORS,
            DialogParameter::PileDriver(_) => DialogId::PILE_DRIVER,
            DialogParameter::UseChainsaw(_) => DialogId::USE_CHAINSAW,
            DialogParameter::InvalidSolidDefence(_) => DialogId::INVALID_SOLID_DEFENCE,
            DialogParameter::SelectSkill(_) => DialogId::SELECT_SKILL,
            DialogParameter::BriberyAndCorruption(_) => DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL,
            DialogParameter::ConfirmEndAction(_) => DialogId::CONFIRM_END_ACTION,
            DialogParameter::SelectWeather(_) => DialogId::SELECT_WEATHER,
            DialogParameter::InformationOkay(_) => DialogId::INFORMATION_OKAY,
            DialogParameter::UseMortuaryAssistant(_) => DialogId::USE_MORTUARY_ASSISTANT,
            DialogParameter::UseMortuaryAssistants(_) => DialogId::USE_MORTUARY_ASSISTANTS,
            DialogParameter::KickOffResult(_) => DialogId::KICK_OFF_RESULT,
            DialogParameter::BloodlustAction(_) => DialogId::BLOODLUST_ACTION,
            DialogParameter::PenaltyShootout(_) => DialogId::PENALTY_SHOOTOUT,
            DialogParameter::BuyPrayersAndInducements(_) => DialogId::BUY_PRAYERS_AND_INDUCEMENTS,
            DialogParameter::ReRollBlockForTargetsProperties(_) => DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES,
            DialogParameter::OpponentBlockSelectionProperties(_) => DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES,
            DialogParameter::PickUpChoice(_) => DialogId::PICK_UP_CHOICE,
            DialogParameter::SelectKeyword(_) => DialogId::SELECT_KEYWORD,
            DialogParameter::SelectPosition(_) => DialogId::SELECT_POSITION,
            DialogParameter::ReRollRegenerationMultiple(_) => DialogId::RE_ROLL_REGENERATION_MULTIPLE,
            DialogParameter::PuntToCrowd(_) => DialogId::PUNT_TO_CROWD,
        }
    }

    /// Java `IDialogParameter.initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: Rules, json: &Value) -> Result<(), String> {
        match self {
            DialogParameter::ApothecaryChoice(p) => p.init_from(rules, json),
            DialogParameter::ReceiveChoice(p) => p.init_from(rules, json),
            DialogParameter::ReRoll(p) => p.init_from(rules, json),
            DialogParameter::ReRollProperties(p) => p.init_from(rules, json),
            DialogParameter::ReRollForTargets(p) => p.init_from(rules, json),
            DialogParameter::SkillUse(p) => p.init_from(rules, json),
            DialogParameter::UseApothecary(p) => p.init_from(rules, json),
            DialogParameter::BlockRoll(p) => p.init_from(rules, json),
            DialogParameter::BlockRollPartialReRoll(p) => p.init_from(rules, json),
            DialogParameter::BlockRollProperties(p) => p.init_from(rules, json),
            DialogParameter::PlayerChoice(p) => p.init_from(rules, json),
            DialogParameter::Interception(p) => p.init_from(rules, json),
            DialogParameter::WinningsReRoll(p) => p.init_from(rules, json),
            DialogParameter::Bribes(p) => p.init_from(rules, json),
            DialogParameter::GameStatistics(p) => p.init_from(rules, json),
            DialogParameter::Join(p) => p.init_from(rules, json),
            DialogParameter::StartGame(p) => p.init_from(rules, json),
            DialogParameter::TeamSetup(p) => p.init_from(rules, json),
            DialogParameter::SetupError(p) => p.init_from(rules, json),
            DialogParameter::Touchback(p) => p.init_from(rules, json),
            DialogParameter::DefenderAction(p) => p.init_from(rules, json),
            DialogParameter::CoinChoice(p) => p.init_from(rules, json),
            DialogParameter::FollowupChoice(p) => p.init_from(rules, json),
            DialogParameter::ConcedeGame(p) => p.init_from(rules, json),
            DialogParameter::PilingOn(p) => p.init_from(rules, json),
            DialogParameter::BuyInducements(p) => p.init_from(rules, json),
            DialogParameter::Journeymen(p) => p.init_from(rules, json),
            DialogParameter::KickSkill(p) => p.init_from(rules, json),
            DialogParameter::UseIgor(p) => p.init_from(rules, json),
            DialogParameter::KickoffReturn(p) => p.init_from(rules, json),
            DialogParameter::PettyCash(p) => p.init_from(rules, json),
            DialogParameter::WizardSpell(p) => p.init_from(rules, json),
            DialogParameter::UseInducement(p) => p.init_from(rules, json),
            DialogParameter::PassBlock(p) => p.init_from(rules, json),
            DialogParameter::BuyCards(p) => p.init_from(rules, json),
            DialogParameter::ArgueTheCall(p) => p.init_from(rules, json),
            DialogParameter::SwarmingPlayers(p) => p.init_from(rules, json),
            DialogParameter::SwarmingError(p) => p.init_from(rules, json),
            DialogParameter::BuyCardsAndInducements(p) => p.init_from(rules, json),
            DialogParameter::SelectBlitzTarget(p) => p.init_from(rules, json),
            DialogParameter::SelectGazeTarget(p) => p.init_from(rules, json),
            DialogParameter::ReRollBlockForTargets(p) => p.init_from(rules, json),
            DialogParameter::OpponentBlockSelection(p) => p.init_from(rules, json),
            DialogParameter::UseApothecaries(p) => p.init_from(rules, json),
            DialogParameter::UseIgors(p) => p.init_from(rules, json),
            DialogParameter::PileDriver(p) => p.init_from(rules, json),
            DialogParameter::UseChainsaw(p) => p.init_from(rules, json),
            DialogParameter::InvalidSolidDefence(p) => p.init_from(rules, json),
            DialogParameter::SelectSkill(p) => p.init_from(rules, json),
            DialogParameter::BriberyAndCorruption(p) => p.init_from(rules, json),
            DialogParameter::ConfirmEndAction(p) => p.init_from(rules, json),
            DialogParameter::SelectWeather(p) => p.init_from(rules, json),
            DialogParameter::InformationOkay(p) => p.init_from(rules, json),
            DialogParameter::UseMortuaryAssistant(p) => p.init_from(rules, json),
            DialogParameter::UseMortuaryAssistants(p) => p.init_from(rules, json),
            DialogParameter::KickOffResult(p) => p.init_from(rules, json),
            DialogParameter::BloodlustAction(p) => p.init_from(rules, json),
            DialogParameter::PenaltyShootout(p) => p.init_from(rules, json),
            DialogParameter::BuyPrayersAndInducements(p) => p.init_from(rules, json),
            DialogParameter::ReRollBlockForTargetsProperties(p) => p.init_from(rules, json),
            DialogParameter::OpponentBlockSelectionProperties(p) => p.init_from(rules, json),
            DialogParameter::PickUpChoice(p) => p.init_from(rules, json),
            DialogParameter::SelectKeyword(p) => p.init_from(rules, json),
            DialogParameter::SelectPosition(p) => p.init_from(rules, json),
            DialogParameter::ReRollRegenerationMultiple(p) => p.init_from(rules, json),
            DialogParameter::PuntToCrowd(p) => p.init_from(rules, json),
        }
    }

    /// Java `IDialogParameter.toJsonValue()`.
    pub fn to_json_value(&self, rules: Rules) -> Result<Value, String> {
        match self {
            DialogParameter::ApothecaryChoice(p) => p.to_json_value(rules),
            DialogParameter::ReceiveChoice(p) => p.to_json_value(rules),
            DialogParameter::ReRoll(p) => p.to_json_value(rules),
            DialogParameter::ReRollProperties(p) => p.to_json_value(rules),
            DialogParameter::ReRollForTargets(p) => p.to_json_value(rules),
            DialogParameter::SkillUse(p) => p.to_json_value(rules),
            DialogParameter::UseApothecary(p) => p.to_json_value(rules),
            DialogParameter::BlockRoll(p) => p.to_json_value(rules),
            DialogParameter::BlockRollPartialReRoll(p) => p.to_json_value(rules),
            DialogParameter::BlockRollProperties(p) => p.to_json_value(rules),
            DialogParameter::PlayerChoice(p) => p.to_json_value(rules),
            DialogParameter::Interception(p) => p.to_json_value(rules),
            DialogParameter::WinningsReRoll(p) => p.to_json_value(rules),
            DialogParameter::Bribes(p) => p.to_json_value(rules),
            DialogParameter::GameStatistics(p) => p.to_json_value(rules),
            DialogParameter::Join(p) => p.to_json_value(rules),
            DialogParameter::StartGame(p) => p.to_json_value(rules),
            DialogParameter::TeamSetup(p) => p.to_json_value(rules),
            DialogParameter::SetupError(p) => p.to_json_value(rules),
            DialogParameter::Touchback(p) => p.to_json_value(rules),
            DialogParameter::DefenderAction(p) => p.to_json_value(rules),
            DialogParameter::CoinChoice(p) => p.to_json_value(rules),
            DialogParameter::FollowupChoice(p) => p.to_json_value(rules),
            DialogParameter::ConcedeGame(p) => p.to_json_value(rules),
            DialogParameter::PilingOn(p) => p.to_json_value(rules),
            DialogParameter::BuyInducements(p) => p.to_json_value(rules),
            DialogParameter::Journeymen(p) => p.to_json_value(rules),
            DialogParameter::KickSkill(p) => p.to_json_value(rules),
            DialogParameter::UseIgor(p) => p.to_json_value(rules),
            DialogParameter::KickoffReturn(p) => p.to_json_value(rules),
            DialogParameter::PettyCash(p) => p.to_json_value(rules),
            DialogParameter::WizardSpell(p) => p.to_json_value(rules),
            DialogParameter::UseInducement(p) => p.to_json_value(rules),
            DialogParameter::PassBlock(p) => p.to_json_value(rules),
            DialogParameter::BuyCards(p) => p.to_json_value(rules),
            DialogParameter::ArgueTheCall(p) => p.to_json_value(rules),
            DialogParameter::SwarmingPlayers(p) => p.to_json_value(rules),
            DialogParameter::SwarmingError(p) => p.to_json_value(rules),
            DialogParameter::BuyCardsAndInducements(p) => p.to_json_value(rules),
            DialogParameter::SelectBlitzTarget(p) => p.to_json_value(rules),
            DialogParameter::SelectGazeTarget(p) => p.to_json_value(rules),
            DialogParameter::ReRollBlockForTargets(p) => p.to_json_value(rules),
            DialogParameter::OpponentBlockSelection(p) => p.to_json_value(rules),
            DialogParameter::UseApothecaries(p) => p.to_json_value(rules),
            DialogParameter::UseIgors(p) => p.to_json_value(rules),
            DialogParameter::PileDriver(p) => p.to_json_value(rules),
            DialogParameter::UseChainsaw(p) => p.to_json_value(rules),
            DialogParameter::InvalidSolidDefence(p) => p.to_json_value(rules),
            DialogParameter::SelectSkill(p) => p.to_json_value(rules),
            DialogParameter::BriberyAndCorruption(p) => p.to_json_value(rules),
            DialogParameter::ConfirmEndAction(p) => p.to_json_value(rules),
            DialogParameter::SelectWeather(p) => p.to_json_value(rules),
            DialogParameter::InformationOkay(p) => p.to_json_value(rules),
            DialogParameter::UseMortuaryAssistant(p) => p.to_json_value(rules),
            DialogParameter::UseMortuaryAssistants(p) => p.to_json_value(rules),
            DialogParameter::KickOffResult(p) => p.to_json_value(rules),
            DialogParameter::BloodlustAction(p) => p.to_json_value(rules),
            DialogParameter::PenaltyShootout(p) => p.to_json_value(rules),
            DialogParameter::BuyPrayersAndInducements(p) => p.to_json_value(rules),
            DialogParameter::ReRollBlockForTargetsProperties(p) => p.to_json_value(rules),
            DialogParameter::OpponentBlockSelectionProperties(p) => p.to_json_value(rules),
            DialogParameter::PickUpChoice(p) => p.to_json_value(rules),
            DialogParameter::SelectKeyword(p) => p.to_json_value(rules),
            DialogParameter::SelectPosition(p) => p.to_json_value(rules),
            DialogParameter::ReRollRegenerationMultiple(p) => p.to_json_value(rules),
            DialogParameter::PuntToCrowd(p) => p.to_json_value(rules),
        }
    }
}

impl DialogParameterFactory {
    pub fn new() -> Self {
        Self
    }

    /// Java `createDialogParameter(DialogId)`: null id or a dialog without parameter -> `None`.
    pub fn create_dialog_parameter(&self, dialog_id: Option<DialogId>) -> Option<DialogParameter> {
        match dialog_id? {
            DialogId::APOTHECARY_CHOICE => Some(DialogParameter::ApothecaryChoice(DialogApothecaryChoiceParameter::default())),
            DialogId::RECEIVE_CHOICE => Some(DialogParameter::ReceiveChoice(DialogReceiveChoiceParameter::default())),
            DialogId::RE_ROLL => Some(DialogParameter::ReRoll(DialogReRollParameter::default())),
            DialogId::RE_ROLL_PROPERTIES => Some(DialogParameter::ReRollProperties(DialogReRollPropertiesParameter::default())),
            DialogId::RE_ROLL_FOR_TARGETS => Some(DialogParameter::ReRollForTargets(DialogReRollForTargetsParameter::default())),
            DialogId::SKILL_USE => Some(DialogParameter::SkillUse(DialogSkillUseParameter::default())),
            DialogId::USE_APOTHECARY => Some(DialogParameter::UseApothecary(DialogUseApothecaryParameter::default())),
            DialogId::BLOCK_ROLL => Some(DialogParameter::BlockRoll(DialogBlockRollParameter::default())),
            DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL => Some(DialogParameter::BlockRollPartialReRoll(DialogBlockRollPartialReRollParameter::default())),
            DialogId::BLOCK_ROLL_PROPERTIES => Some(DialogParameter::BlockRollProperties(DialogBlockRollPropertiesParameter::default())),
            DialogId::PLAYER_CHOICE => Some(DialogParameter::PlayerChoice(DialogPlayerChoiceParameter::default())),
            DialogId::INTERCEPTION => Some(DialogParameter::Interception(DialogInterceptionParameter::default())),
            DialogId::WINNINGS_RE_ROLL => Some(DialogParameter::WinningsReRoll(DialogWinningsReRollParameter::default())),
            DialogId::BRIBES => Some(DialogParameter::Bribes(DialogBribesParameter::default())),
            DialogId::GAME_STATISTICS => Some(DialogParameter::GameStatistics(DialogGameStatisticsParameter::default())),
            DialogId::JOIN => Some(DialogParameter::Join(DialogJoinParameter::default())),
            DialogId::START_GAME => Some(DialogParameter::StartGame(DialogStartGameParameter::default())),
            DialogId::TEAM_SETUP => Some(DialogParameter::TeamSetup(DialogTeamSetupParameter::default())),
            DialogId::SETUP_ERROR => Some(DialogParameter::SetupError(DialogSetupErrorParameter::default())),
            DialogId::TOUCHBACK => Some(DialogParameter::Touchback(DialogTouchbackParameter::default())),
            DialogId::DEFENDER_ACTION => Some(DialogParameter::DefenderAction(DialogDefenderActionParameter::default())),
            DialogId::COIN_CHOICE => Some(DialogParameter::CoinChoice(DialogCoinChoiceParameter::default())),
            DialogId::FOLLOWUP_CHOICE => Some(DialogParameter::FollowupChoice(DialogFollowupChoiceParameter::default())),
            DialogId::CONCEDE_GAME => Some(DialogParameter::ConcedeGame(DialogConcedeGameParameter::default())),
            DialogId::PILING_ON => Some(DialogParameter::PilingOn(DialogPilingOnParameter::default())),
            DialogId::BUY_INDUCEMENTS => Some(DialogParameter::BuyInducements(DialogBuyInducementsParameter::default())),
            DialogId::JOURNEYMEN => Some(DialogParameter::Journeymen(DialogJourneymenParameter::default())),
            DialogId::KICK_SKILL => Some(DialogParameter::KickSkill(DialogKickSkillParameter::default())),
            DialogId::USE_IGOR => Some(DialogParameter::UseIgor(DialogUseIgorParameter::default())),
            DialogId::KICKOFF_RETURN => Some(DialogParameter::KickoffReturn(DialogKickoffReturnParameter::default())),
            DialogId::PETTY_CASH => Some(DialogParameter::PettyCash(DialogPettyCashParameter::default())),
            DialogId::WIZARD_SPELL => Some(DialogParameter::WizardSpell(DialogWizardSpellParameter::default())),
            DialogId::USE_INDUCEMENT => Some(DialogParameter::UseInducement(DialogUseInducementParameter::default())),
            DialogId::PASS_BLOCK => Some(DialogParameter::PassBlock(DialogPassBlockParameter::default())),
            DialogId::BUY_CARDS => Some(DialogParameter::BuyCards(DialogBuyCardsParameter::default())),
            DialogId::ARGUE_THE_CALL => Some(DialogParameter::ArgueTheCall(DialogArgueTheCallParameter::default())),
            DialogId::SWARMING => Some(DialogParameter::SwarmingPlayers(DialogSwarmingPlayersParameter::default())),
            DialogId::SWARMING_ERROR => Some(DialogParameter::SwarmingError(DialogSwarmingErrorParameter::default())),
            DialogId::BUY_CARDS_AND_INDUCEMENTS => Some(DialogParameter::BuyCardsAndInducements(DialogBuyCardsAndInducementsParameter::default())),
            DialogId::SELECT_BLITZ_TARGET => Some(DialogParameter::SelectBlitzTarget(DialogSelectBlitzTargetParameter::default())),
            DialogId::SELECT_GAZE_TARGET => Some(DialogParameter::SelectGazeTarget(DialogSelectGazeTargetParameter::default())),
            DialogId::RE_ROLL_BLOCK_FOR_TARGETS => Some(DialogParameter::ReRollBlockForTargets(DialogReRollBlockForTargetsParameter::default())),
            DialogId::OPPONENT_BLOCK_SELECTION => Some(DialogParameter::OpponentBlockSelection(DialogOpponentBlockSelectionParameter::default())),
            DialogId::USE_APOTHECARIES => Some(DialogParameter::UseApothecaries(DialogUseApothecariesParameter::default())),
            DialogId::USE_IGORS => Some(DialogParameter::UseIgors(DialogUseIgorsParameter::default())),
            DialogId::PILE_DRIVER => Some(DialogParameter::PileDriver(DialogPileDriverParameter::default())),
            DialogId::USE_CHAINSAW => Some(DialogParameter::UseChainsaw(DialogUseChainsawParameter::default())),
            DialogId::INVALID_SOLID_DEFENCE => Some(DialogParameter::InvalidSolidDefence(DialogInvalidSolidDefenceParameter::default())),
            DialogId::SELECT_SKILL => Some(DialogParameter::SelectSkill(DialogSelectSkillParameter::default())),
            DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL => Some(DialogParameter::BriberyAndCorruption(DialogBriberyAndCorruptionParameter::default())),
            DialogId::CONFIRM_END_ACTION => Some(DialogParameter::ConfirmEndAction(DialogConfirmEndActionParameter::default())),
            DialogId::SELECT_WEATHER => Some(DialogParameter::SelectWeather(DialogSelectWeatherParameter::default())),
            DialogId::INFORMATION_OKAY => Some(DialogParameter::InformationOkay(DialogInformationOkayParameter::default())),
            DialogId::USE_MORTUARY_ASSISTANT => Some(DialogParameter::UseMortuaryAssistant(DialogUseMortuaryAssistantParameter::default())),
            DialogId::USE_MORTUARY_ASSISTANTS => Some(DialogParameter::UseMortuaryAssistants(DialogUseMortuaryAssistantsParameter::default())),
            DialogId::KICK_OFF_RESULT => Some(DialogParameter::KickOffResult(DialogKickOffResultParameter::default())),
            DialogId::BLOODLUST_ACTION => Some(DialogParameter::BloodlustAction(DialogBloodlustActionParameter::default())),
            DialogId::PENALTY_SHOOTOUT => Some(DialogParameter::PenaltyShootout(DialogPenaltyShootoutParameter::default())),
            DialogId::BUY_PRAYERS_AND_INDUCEMENTS => Some(DialogParameter::BuyPrayersAndInducements(DialogBuyPrayersAndInducementsParameter::default())),
            DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES => Some(DialogParameter::ReRollBlockForTargetsProperties(DialogReRollBlockForTargetsPropertiesParameter::default())),
            DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES => Some(DialogParameter::OpponentBlockSelectionProperties(DialogOpponentBlockSelectionPropertiesParameter::default())),
            DialogId::PICK_UP_CHOICE => Some(DialogParameter::PickUpChoice(DialogPickUpChoiceParameter::default())),
            DialogId::SELECT_KEYWORD => Some(DialogParameter::SelectKeyword(DialogSelectKeywordParameter::default())),
            DialogId::SELECT_POSITION => Some(DialogParameter::SelectPosition(DialogSelectPositionParameter::default())),
            DialogId::RE_ROLL_REGENERATION_MULTIPLE => Some(DialogParameter::ReRollRegenerationMultiple(DialogReRollRegenerationMultipleParameter::default())),
            DialogId::PUNT_TO_CROWD => Some(DialogParameter::PuntToCrowd(DialogPuntToCrowdParameter::default())),
            _ => None,
        }
    }

    /// Java `forJsonValue(IFactorySource, JsonValue)`: JSON null -> `None`; the dialog id is read with
    /// `DialogIdFactory.forName` (case-insensitive), then the created parameter `initFrom`s the value.
    pub fn for_json_value(&self, rules: Rules, json: &Value) -> Result<Option<DialogParameter>, String> {
        if json.is_null() {
            return Ok(None);
        }
        let obj = crate::util::java_json::to_json_object(json)?;
        let dialog_id = super::util_dialog_parameter::get_dialog_id(obj)?;
        let mut dialog_parameter = self.create_dialog_parameter(dialog_id);
        if let Some(p) = dialog_parameter.as_mut() {
            p.init_from(rules, json)?;
        }
        Ok(dialog_parameter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn known_dialog_id_creates_its_parameter() {
        let f = DialogParameterFactory::new();
        assert_eq!(f.create_dialog_parameter(Some(DialogId::BLOCK_ROLL)).map(|p| p.get_id()), Some(DialogId::BLOCK_ROLL));
        assert_eq!(f.create_dialog_parameter(Some(DialogId::PUNT_TO_CROWD)).map(|p| p.get_id()), Some(DialogId::PUNT_TO_CROWD));
    }

    #[test]
    fn dialog_without_parameter_class_and_null_id_create_nothing() {
        let f = DialogParameterFactory::new();
        assert!(f.create_dialog_parameter(Some(DialogId::INFORMATION)).is_none());
        assert!(f.create_dialog_parameter(None).is_none());
    }

    #[test]
    fn for_json_value_unknown_id_is_null_and_dialog_id_is_case_insensitive() {
        let f = DialogParameterFactory::new();
        // a newer server's dialog id (FUMBBL replays): the whole parameter decodes to null
        assert!(f.for_json_value(Rules::Bb2020, &json!({"dialogId": "reRollModifierChoice"})).unwrap().is_none());
        assert!(f.for_json_value(Rules::Bb2020, &Value::Null).unwrap().is_none());
        let p = f.for_json_value(Rules::Bb2020, &json!({"dialogId": "COINCHOICE"})).unwrap().unwrap();
        assert_eq!(p.get_id(), DialogId::COIN_CHOICE);
        assert_eq!(p.to_json_value(Rules::Bb2020).unwrap(), json!({"dialogId": "coinChoice"}));
    }
}
