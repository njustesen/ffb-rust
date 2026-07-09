/// 1:1 translation of `com.fumbbl.ffb.dialog.DialogParameterFactory`.
///
/// Factory that maps a `DialogId` to an empty parameter object.  In the Rust
/// codebase dialog parameters are thin string-keyed maps, so each variant
/// just returns a `DialogParameters` (a type alias for
/// `std::collections::HashMap<String,String>`) — the same approach used
/// elsewhere in ffb-model.
///
/// NOTE: The Java source returns concrete `DialogXxxParameter` objects; we
/// represent them generically here with a shared enum so no external crate
/// dependency on heavy dialog structs is needed.
#[derive(Debug, Clone, Default)]
pub struct DialogParameterFactory;

/// Lightweight representation of a dialog parameter container.
/// Each variant corresponds to a `DialogXxxParameter` class in the Java source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogParameters {
    ApothecaryChoice,
    ReceiveChoice,
    ReRoll,
    ReRollProperties,
    ReRollForTargets,
    SkillUse,
    UseApothecary,
    BlockRoll,
    BlockRollPartialReRoll,
    BlockRollProperties,
    PlayerChoice,
    Interception,
    WinningsReRoll,
    Bribes,
    GameStatistics,
    Join,
    StartGame,
    TeamSetup,
    SetupError,
    Touchback,
    DefenderAction,
    CoinChoice,
    FollowupChoice,
    ConcedeGame,
    PilingOn,
    BuyInducements,
    Journeymen,
    KickSkill,
    UseIgor,
    KickoffReturn,
    PettyCash,
    WizardSpell,
    UseInducement,
    PassBlock,
    BuyCards,
    ArgueTheCall,
    Swarming,
    SwarmingError,
    BuyCardsAndInducements,
    SelectBlitzTarget,
    SelectGazeTarget,
    ReRollBlockForTargets,
    OpponentBlockSelection,
    UseApothecaries,
    UseIgors,
    PileDriver,
    UseChainsaw,
    InvalidSolidDefence,
    SelectSkill,
    BriberyAndCorruption,
    ConfirmEndAction,
    SelectWeather,
    InformationOkay,
    UseMortuaryAssistant,
    UseMortuaryAssistants,
    KickOffResult,
    BloodlustAction,
    PenaltyShootout,
    BuyPrayersAndInducements,
    ReRollBlockForTargetsProperties,
    OpponentBlockSelectionProperties,
    PickUpChoice,
    SelectKeyword,
    SelectPosition,
    ReRollRegenerationMultiple,
    PuntToCrowd,
}

impl DialogParameterFactory {
    pub fn new() -> Self { Self }

    /// Java: `createDialogParameter(DialogId)`.
    /// Returns `None` if `dialog_id` is not recognised.
    pub fn create_dialog_parameter(&self, dialog_id: &str) -> Option<DialogParameters> {
        match dialog_id {
            "APOTHECARY_CHOICE"                     => Some(DialogParameters::ApothecaryChoice),
            "RECEIVE_CHOICE"                        => Some(DialogParameters::ReceiveChoice),
            "RE_ROLL"                               => Some(DialogParameters::ReRoll),
            "RE_ROLL_PROPERTIES"                    => Some(DialogParameters::ReRollProperties),
            "RE_ROLL_FOR_TARGETS"                   => Some(DialogParameters::ReRollForTargets),
            "SKILL_USE"                             => Some(DialogParameters::SkillUse),
            "USE_APOTHECARY"                        => Some(DialogParameters::UseApothecary),
            "BLOCK_ROLL"                            => Some(DialogParameters::BlockRoll),
            "BLOCK_ROLL_PARTIAL_RE_ROLL"            => Some(DialogParameters::BlockRollPartialReRoll),
            "BLOCK_ROLL_PROPERTIES"                 => Some(DialogParameters::BlockRollProperties),
            "PLAYER_CHOICE"                         => Some(DialogParameters::PlayerChoice),
            "INTERCEPTION"                          => Some(DialogParameters::Interception),
            "WINNINGS_RE_ROLL"                      => Some(DialogParameters::WinningsReRoll),
            "BRIBES"                                => Some(DialogParameters::Bribes),
            "GAME_STATISTICS"                       => Some(DialogParameters::GameStatistics),
            "JOIN"                                  => Some(DialogParameters::Join),
            "START_GAME"                            => Some(DialogParameters::StartGame),
            "TEAM_SETUP"                            => Some(DialogParameters::TeamSetup),
            "SETUP_ERROR"                           => Some(DialogParameters::SetupError),
            "TOUCHBACK"                             => Some(DialogParameters::Touchback),
            "DEFENDER_ACTION"                       => Some(DialogParameters::DefenderAction),
            "COIN_CHOICE"                           => Some(DialogParameters::CoinChoice),
            "FOLLOWUP_CHOICE"                       => Some(DialogParameters::FollowupChoice),
            "CONCEDE_GAME"                          => Some(DialogParameters::ConcedeGame),
            "PILING_ON"                             => Some(DialogParameters::PilingOn),
            "BUY_INDUCEMENTS"                       => Some(DialogParameters::BuyInducements),
            "JOURNEYMEN"                            => Some(DialogParameters::Journeymen),
            "KICK_SKILL"                            => Some(DialogParameters::KickSkill),
            "USE_IGOR"                              => Some(DialogParameters::UseIgor),
            "KICKOFF_RETURN"                        => Some(DialogParameters::KickoffReturn),
            "PETTY_CASH"                            => Some(DialogParameters::PettyCash),
            "WIZARD_SPELL"                          => Some(DialogParameters::WizardSpell),
            "USE_INDUCEMENT"                        => Some(DialogParameters::UseInducement),
            "PASS_BLOCK"                            => Some(DialogParameters::PassBlock),
            "BUY_CARDS"                             => Some(DialogParameters::BuyCards),
            "ARGUE_THE_CALL"                        => Some(DialogParameters::ArgueTheCall),
            "SWARMING"                              => Some(DialogParameters::Swarming),
            "SWARMING_ERROR"                        => Some(DialogParameters::SwarmingError),
            "BUY_CARDS_AND_INDUCEMENTS"             => Some(DialogParameters::BuyCardsAndInducements),
            "SELECT_BLITZ_TARGET"                   => Some(DialogParameters::SelectBlitzTarget),
            "SELECT_GAZE_TARGET"                    => Some(DialogParameters::SelectGazeTarget),
            "RE_ROLL_BLOCK_FOR_TARGETS"             => Some(DialogParameters::ReRollBlockForTargets),
            "OPPONENT_BLOCK_SELECTION"              => Some(DialogParameters::OpponentBlockSelection),
            "USE_APOTHECARIES"                      => Some(DialogParameters::UseApothecaries),
            "USE_IGORS"                             => Some(DialogParameters::UseIgors),
            "PILE_DRIVER"                           => Some(DialogParameters::PileDriver),
            "USE_CHAINSAW"                          => Some(DialogParameters::UseChainsaw),
            "INVALID_SOLID_DEFENCE"                 => Some(DialogParameters::InvalidSolidDefence),
            "SELECT_SKILL"                          => Some(DialogParameters::SelectSkill),
            "BRIBERY_AND_CORRUPTION_RE_ROLL"        => Some(DialogParameters::BriberyAndCorruption),
            "CONFIRM_END_ACTION"                    => Some(DialogParameters::ConfirmEndAction),
            "SELECT_WEATHER"                        => Some(DialogParameters::SelectWeather),
            "INFORMATION_OKAY"                      => Some(DialogParameters::InformationOkay),
            "USE_MORTUARY_ASSISTANT"                => Some(DialogParameters::UseMortuaryAssistant),
            "USE_MORTUARY_ASSISTANTS"               => Some(DialogParameters::UseMortuaryAssistants),
            "KICK_OFF_RESULT"                       => Some(DialogParameters::KickOffResult),
            "BLOODLUST_ACTION"                      => Some(DialogParameters::BloodlustAction),
            "PENALTY_SHOOTOUT"                      => Some(DialogParameters::PenaltyShootout),
            "BUY_PRAYERS_AND_INDUCEMENTS"           => Some(DialogParameters::BuyPrayersAndInducements),
            "RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES"  => Some(DialogParameters::ReRollBlockForTargetsProperties),
            "OPPONENT_BLOCK_SELECTION_PROPERTIES"   => Some(DialogParameters::OpponentBlockSelectionProperties),
            "PICK_UP_CHOICE"                        => Some(DialogParameters::PickUpChoice),
            "SELECT_KEYWORD"                        => Some(DialogParameters::SelectKeyword),
            "SELECT_POSITION"                       => Some(DialogParameters::SelectPosition),
            "RE_ROLL_REGENERATION_MULTIPLE"         => Some(DialogParameters::ReRollRegenerationMultiple),
            "PUNT_TO_CROWD"                         => Some(DialogParameters::PuntToCrowd),
            _                                       => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dialog_id_returns_some() {
        let f = DialogParameterFactory::new();
        assert_eq!(f.create_dialog_parameter("BLOCK_ROLL"), Some(DialogParameters::BlockRoll));
    }

    #[test]
    fn unknown_dialog_id_returns_none() {
        let f = DialogParameterFactory::new();
        assert!(f.create_dialog_parameter("UNKNOWN_DIALOG").is_none());
    }

    #[test]
    fn punt_to_crowd_is_last_variant() {
        let f = DialogParameterFactory::new();
        assert_eq!(f.create_dialog_parameter("PUNT_TO_CROWD"), Some(DialogParameters::PuntToCrowd));
    }
}
