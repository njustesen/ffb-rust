use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.option.GameOptionId`.
///
/// All option name strings as constants (Java enum `getName()` return values).
pub const RULESVERSION: &str = "rulesVersion";
pub const CHECK_OWNERSHIP: &str = "checkOwnership";
pub const TEST_MODE: &str = "testMode";
pub const OVERTIME: &str = "overtime";
pub const TURNTIME: &str = "turntime";
pub const ALLOW_CONCESSIONS: &str = "allowConcessions";

pub const PETTY_CASH: &str = "pettyCash";
pub const INDUCEMENTS: &str = "inducements";
pub const MAX_NR_OF_CARDS: &str = "maxNrOfCards";

pub const MAX_PLAYERS_ON_FIELD: &str = "maxPlayersOnField";
pub const MAX_PLAYERS_IN_WIDE_ZONE: &str = "maxPlayersInWideZone";
pub const MIN_PLAYERS_ON_LOS: &str = "minPlayersOnLos";

pub const ALLOW_STAR_ON_BOTH_TEAMS: &str = "allowStarOnBothTeams";
pub const ALLOW_STAFF_ON_BOTH_TEAMS: &str = "allowStaffOnBothTeams";
pub const FORCE_TREASURY_TO_PETTY_CASH: &str = "forceTreasuryToPettyCash";
pub const USE_PREDEFINED_INDUCEMENTS: &str = "usePredefinedInducements";

pub const ALLOW_KTM_REROLL: &str = "allowKtmReroll";
pub const CLAW_DOES_NOT_STACK: &str = "clawDoesNotStack";
pub const FOUL_BONUS: &str = "foulBonus";
pub const FOUL_BONUS_OUTSIDE_TACKLEZONE: &str = "foulBonusOutsideTacklezone";
pub const FREE_INDUCEMENT_CASH: &str = "freeInducementCash";
pub const FREE_CARD_CASH: &str = "freeCardCash";
pub const PILING_ON_DOES_NOT_STACK: &str = "pilingOnDoesNotStack";
pub const PILING_ON_INJURY_ONLY: &str = "pilingOnInjuryOnly";
pub const PILING_ON_ARMOR_ONLY: &str = "pilingOnArmorOnly";
pub const PILING_ON_TO_KO_ON_DOUBLE: &str = "pilingOnToKoOnDouble";
pub const PILING_ON_USES_A_TEAM_REROLL: &str = "pilingOnUsesATeamReroll";
pub const RIGHT_STUFF_CANCELS_TACKLE: &str = "rightStuffCancelsTackle";
pub const SNEAKY_GIT_AS_FOUL_GUARD: &str = "sneakyGitAsFoulGuard";
pub const SNEAKY_GIT_BAN_TO_KO: &str = "sneakyGitBanToKo";
pub const STAND_FIRM_NO_DROP_ON_FAILED_DODGE: &str = "standFirmNoDropOnFailedDodge";
pub const SPIKED_BALL: &str = "spikedBall";
pub const DIVING_TACKLE_LEAVING_TZ_ONLY: &str = "divingTackleLeavingTzOnly";
pub const ALLOW_SPECIAL_ACTIONS_FROM_PRONE: &str = "allowSpecialActionsFromProne";
pub const ALLOW_BRAWLER_ON_BOTH_BLOCKS: &str = "allowBrawlerOnBothBlocks";
pub const ASK_FOR_KICK_AFTER_ROLL: &str = "askForKickAfterRoll";

pub const ARGUE_THE_CALL: &str = "argueTheCall";
pub const MVP_NOMINATIONS: &str = "mvpNominations";
pub const PETTY_CASH_AFFECTS_TV: &str = "pettyCashAffectsTv";
pub const WIZARD_AVAILABLE: &str = "wizardAvailable";
pub const EXTRA_MVP: &str = "extraMvp";

pub const CARDS_MISCELLANEOUS_MAYHEM_COST: &str = "cardsMiscellaneousMayhemCost";
pub const CARDS_MISCELLANEOUS_MAYHEM_MAX: &str = "cardsMiscellaneousMayhemMax";
pub const CARDS_SPECIAL_TEAM_PLAY_COST: &str = "cardsSpecialTeamPlayCost";
pub const CARDS_SPECIAL_TEAM_PLAY_MAX: &str = "cardsSpecialTeamPlayMax";
pub const CARDS_MAGIC_ITEM_COST: &str = "cardsMagicItemCost";
pub const CARDS_MAGIC_ITEM_MAX: &str = "cardsMagicItemMax";
pub const CARDS_DIRTY_TRICK_COST: &str = "cardsDirtyTrickCost";
pub const CARDS_DIRTY_TRICK_MAX: &str = "cardsDirtyTrickMax";
pub const CARDS_GOOD_KARMA_COST: &str = "cardsGoodKarmaCost";
pub const CARDS_GOOD_KARMA_MAX: &str = "cardsGoodKarmaMax";
pub const CARDS_RANDOM_EVENT_COST: &str = "cardsRandomEventCost";
pub const CARDS_RANDOM_EVENT_MAX: &str = "cardsRandomEventMax";
pub const CARDS_DESPERATE_MEASURE_COST: &str = "cardsDesperateMeasureCost";
pub const CARDS_DESPERATE_MEASURE_MAX: &str = "cardsDesperateMeasureMax";
pub const CARDS_SPECIAL_PLAY_COST: &str = "cardsSpecialPlayCost";

pub const INDUCEMENT_APOS_COST: &str = "inducementAposCost";
pub const INDUCEMENT_APOS_MAX: &str = "inducementAposMax";
pub const INDUCEMENT_BRIBES_COST: &str = "inducementBribesCost";
pub const INDUCEMENT_BRIBES_REDUCED_COST: &str = "inducementBribesReducedCost";
pub const INDUCEMENT_BRIBES_MAX: &str = "inducementBribesMax";
pub const INDUCEMENT_BRIBES_REDUCED_MAX: &str = "inducementBribesReducedMax";
pub const INDUCEMENT_CHEFS_COST: &str = "inducementChefsCost";
pub const INDUCEMENT_CHEFS_REDUCED_COST: &str = "inducementChefsReducedCost";
pub const INDUCEMENT_CHEFS_MAX: &str = "inducementChefsMax";
pub const INDUCEMENT_CHEFS_REDUCED_MAX: &str = "inducementChefsReducedMax";
pub const INDUCEMENT_EXTRA_TRAINING_COST: &str = "inducementExtraTrainingCost";
pub const INDUCEMENT_EXTRA_TRAINING_MAX: &str = "inducementExtraTrainingMax";
pub const INDUCEMENT_IGORS_COST: &str = "inducementIgorsCost";
pub const INDUCEMENT_IGORS_MAX: &str = "inducementIgorsMax";
pub const INDUCEMENT_MORTUARY_ASSISTANTS_COST: &str = "inducementMortuaryAssistantsCost";
pub const INDUCEMENT_MORTUARY_ASSISTANTS_MAX: &str = "inducementMortuaryAssistantsMax";
pub const INDUCEMENT_PLAGUE_DOCTORS_COST: &str = "inducementPlagueDoctorsCost";
pub const INDUCEMENT_PLAGUE_DOCTORS_MAX: &str = "inducementPlagueDoctorsMax";
pub const INDUCEMENT_KEGS_COST: &str = "inducementKegsCost";
pub const INDUCEMENT_KEGS_MAX: &str = "inducementKegsMax";
pub const INDUCEMENT_MASCOT_MAX: &str = "inducementMascotMax";
pub const INDUCEMENT_MASCOT_COST: &str = "inducementMascotCost";
pub const INDUCEMENT_MERCENARIES_EXTRA_COST: &str = "inducementMercenariesExtraCost";
pub const INDUCEMENT_MERCENARIES_SKILL_COST: &str = "inducementMercenariesSkillCost";
pub const INDUCEMENT_MERCENARIES_MAX: &str = "inducementMercenariesMax";
pub const INDUCEMENT_PRAYERS_COST: &str = "inducementPrayersCost";
pub const INDUCEMENT_PRAYERS_MAX: &str = "inducementPrayersMax";
pub const INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE: &str = "inducementPrayersUseLeagueTable";
pub const INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG: &str = "inducementPrayersAvailableForUnderdog";
pub const INDUCEMENT_RIOTOUS_ROOKIES_COST: &str = "inducementRiotousRookiesCost";
pub const INDUCEMENT_RIOTOUS_ROOKIES_MAX: &str = "inducementRiotousRookiesMax";
pub const INDUCEMENT_STARS_MAX: &str = "inducementStarsMax";
pub const INDUCEMENT_STAFF_MAX: &str = "inducementStaffMax";
pub const INDUCEMENT_WIZARDS_COST: &str = "inducementWizardsCost";
pub const INDUCEMENT_WIZARDS_MAX: &str = "inducementWizardsMax";
pub const INDUCEMENT_BIASED_REF_COST: &str = "inducementBiasedRefCost";
pub const INDUCEMENT_BIASED_REF_REDUCED_COST: &str = "inducementBiasedRefReducedCost";
pub const INDUCEMENT_BIASED_REF_MAX: &str = "inducementBiasedRefMax";
pub const INDUCEMENT_BIASED_REF_REDUCED_MAX: &str = "inducementBiasedRefReducedMax";
pub const INDUCEMENT_TEMP_CHEERLEADER_MAX: &str = "inducementTempCheerleaderMax";
pub const INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX: &str = "inducementTempCheerleaderTotalMax";
pub const INDUCEMENT_TEMP_CHEERLEADER_COST: &str = "inducementTempCheerleaderCost";
pub const INDUCEMENT_PART_TIME_COACH_MAX: &str = "inducementPartTimeCoachMax";
pub const INDUCEMENT_PART_TIME_COACH_TOTAL_MAX: &str = "inducementPartTimeCoachTotalMax";
pub const INDUCEMENT_PART_TIME_COACH_COST: &str = "inducementPartTimeCoachCost";
pub const INDUCEMENT_WEATHER_MAGE_MAX: &str = "inducementWeatherMageMax";
pub const INDUCEMENT_WEATHER_MAGE_COST: &str = "inducementWeatherMageCost";
pub const INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV: &str = "inducementsAllowSpendingTreasuryOnEqualCTV";
pub const INDUCEMENTS_ALWAYS_USE_TREASURY: &str = "inducementsAlwaysUseTreasury";
pub const INDUCEMENTS_ALLOW_OVERDOG_SPENDING: &str = "inducementsAllowOverdogSpending";
pub const INDUCEMENT_DUMMY: &str = "inducementDummy";

pub const ENABLE_STALLING_CHECK: &str = "enableStallingCheck";
pub const ALLOW_BALL_AND_CHAIN_RE_ROLL: &str = "allowBallAndChainReRoll";
pub const END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM: &str = "endTurnWhenHittingAnyPlayerWithTtm";
pub const SWOOP_DISTANCE: &str = "swoopDistance";
pub const ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN: &str = "allowSpecialBlocksWithBallAndChain";
pub const MB_STACKS_AGAINST_CHAINSAW: &str = "mbStacksAgainstChainsaw";
/// Legacy, kept around to make sure old replays or running games do not break after update.
pub const CHAINSAW_TURNOVER_ON_AV_BREAK: &str = "chainsawTurnoverOnAvBreak";
pub const CHAINSAW_TURNOVER: &str = "chainsawTurnover";
pub const BOMBER_PLACED_PRONE_IGNORES_TURNOVER: &str = "bomberPlacedProneIgnoresTurnover";
pub const BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER: &str = "bombTeamMateKnockDownCausesTurnover";
pub const SNEAKY_GIT_CAN_MOVE_AFTER_FOUL: &str = "sneakyGitCanMoveAfterFoul";
pub const BOMB_USES_MB: &str = "bombUsesMb";
pub const CATCH_WORKS_FOR_BOMBS: &str = "catchWorksForBombs";
pub const ONLY_ONE_BRIBE_PER_SEND_OFF: &str = "onlyOneBribePerSendOff";
pub const OVERTIME_GOLDEN_GOAL: &str = "overtimeGoldenGoal";
pub const OVERTIME_KICK_OFF_RESULTS: &str = "overtimeKickOffResults";
pub const ENABLE_TACKLEZONE_OVERLAYS: &str = "enableTacklezoneOverlays";
pub const BOMB_BOUNCES_ON_EMPTY_SQUARES: &str = "bombBouncesOnEmptySquares";
pub const ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION: &str = "animalSavageryLashOutEndsActivation";

pub const PITCH_URL: &str = "pitchUrl";

/// 1:1 translation of the `com.fumbbl.ffb.option.GameOptionId` enum (implements
/// `INamedObject`: `getName()`). Variant order matches the Java source exactly.
/// `get_name()` reuses the string constants declared above.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameOptionId {
    RULESVERSION,
    CHECK_OWNERSHIP,
    TEST_MODE,
    OVERTIME,
    TURNTIME,
    ALLOW_CONCESSIONS,

    PETTY_CASH,
    INDUCEMENTS,
    MAX_NR_OF_CARDS,

    MAX_PLAYERS_ON_FIELD,
    MAX_PLAYERS_IN_WIDE_ZONE,
    MIN_PLAYERS_ON_LOS,

    ALLOW_STAR_ON_BOTH_TEAMS,
    ALLOW_STAFF_ON_BOTH_TEAMS,
    FORCE_TREASURY_TO_PETTY_CASH,
    USE_PREDEFINED_INDUCEMENTS,

    ALLOW_KTM_REROLL,
    CLAW_DOES_NOT_STACK,
    FOUL_BONUS,
    FOUL_BONUS_OUTSIDE_TACKLEZONE,
    FREE_INDUCEMENT_CASH,
    FREE_CARD_CASH,
    PILING_ON_DOES_NOT_STACK,
    PILING_ON_INJURY_ONLY,
    PILING_ON_ARMOR_ONLY,
    PILING_ON_TO_KO_ON_DOUBLE,
    PILING_ON_USES_A_TEAM_REROLL,
    RIGHT_STUFF_CANCELS_TACKLE,
    SNEAKY_GIT_AS_FOUL_GUARD,
    SNEAKY_GIT_BAN_TO_KO,
    STAND_FIRM_NO_DROP_ON_FAILED_DODGE,
    SPIKED_BALL,
    DIVING_TACKLE_LEAVING_TZ_ONLY,
    ALLOW_SPECIAL_ACTIONS_FROM_PRONE,
    ALLOW_BRAWLER_ON_BOTH_BLOCKS,
    ASK_FOR_KICK_AFTER_ROLL,

    ARGUE_THE_CALL,
    MVP_NOMINATIONS,
    PETTY_CASH_AFFECTS_TV,
    WIZARD_AVAILABLE,

    EXTRA_MVP,

    CARDS_MISCELLANEOUS_MAYHEM_COST,
    CARDS_MISCELLANEOUS_MAYHEM_MAX,
    CARDS_SPECIAL_TEAM_PLAY_COST,
    CARDS_SPECIAL_TEAM_PLAY_MAX,
    CARDS_MAGIC_ITEM_COST,
    CARDS_MAGIC_ITEM_MAX,
    CARDS_DIRTY_TRICK_COST,
    CARDS_DIRTY_TRICK_MAX,
    CARDS_GOOD_KARMA_COST,
    CARDS_GOOD_KARMA_MAX,
    CARDS_RANDOM_EVENT_COST,
    CARDS_RANDOM_EVENT_MAX,
    CARDS_DESPERATE_MEASURE_COST,
    CARDS_DESPERATE_MEASURE_MAX,
    CARDS_SPECIAL_PLAY_COST,

    INDUCEMENT_APOS_COST,
    INDUCEMENT_APOS_MAX,
    INDUCEMENT_BRIBES_COST,
    INDUCEMENT_BRIBES_REDUCED_COST,
    INDUCEMENT_BRIBES_MAX,
    INDUCEMENT_BRIBES_REDUCED_MAX,
    INDUCEMENT_CHEFS_COST,
    INDUCEMENT_CHEFS_REDUCED_COST,
    INDUCEMENT_CHEFS_MAX,
    INDUCEMENT_CHEFS_REDUCED_MAX,
    INDUCEMENT_EXTRA_TRAINING_COST,
    INDUCEMENT_EXTRA_TRAINING_MAX,
    INDUCEMENT_IGORS_COST,
    INDUCEMENT_IGORS_MAX,
    INDUCEMENT_MORTUARY_ASSISTANTS_COST,
    INDUCEMENT_MORTUARY_ASSISTANTS_MAX,
    INDUCEMENT_PLAGUE_DOCTORS_COST,
    INDUCEMENT_PLAGUE_DOCTORS_MAX,
    INDUCEMENT_KEGS_COST,
    INDUCEMENT_KEGS_MAX,
    INDUCEMENT_MASCOT_MAX,
    INDUCEMENT_MASCOT_COST,
    INDUCEMENT_MERCENARIES_EXTRA_COST,
    INDUCEMENT_MERCENARIES_SKILL_COST,
    INDUCEMENT_MERCENARIES_MAX,
    INDUCEMENT_PRAYERS_COST,
    INDUCEMENT_PRAYERS_MAX,
    INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE,
    INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG,
    INDUCEMENT_RIOTOUS_ROOKIES_COST,
    INDUCEMENT_RIOTOUS_ROOKIES_MAX,
    INDUCEMENT_STARS_MAX,
    INDUCEMENT_STAFF_MAX,
    INDUCEMENT_WIZARDS_COST,
    INDUCEMENT_WIZARDS_MAX,
    INDUCEMENT_BIASED_REF_COST,
    INDUCEMENT_BIASED_REF_REDUCED_COST,
    INDUCEMENT_BIASED_REF_MAX,
    INDUCEMENT_BIASED_REF_REDUCED_MAX,
    INDUCEMENT_TEMP_CHEERLEADER_MAX,
    INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX,
    INDUCEMENT_TEMP_CHEERLEADER_COST,
    INDUCEMENT_PART_TIME_COACH_MAX,
    INDUCEMENT_PART_TIME_COACH_TOTAL_MAX,
    INDUCEMENT_PART_TIME_COACH_COST,
    INDUCEMENT_WEATHER_MAGE_MAX,
    INDUCEMENT_WEATHER_MAGE_COST,
    INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV,
    INDUCEMENTS_ALWAYS_USE_TREASURY,
    INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
    INDUCEMENT_DUMMY,

    ENABLE_STALLING_CHECK,
    ALLOW_BALL_AND_CHAIN_RE_ROLL,
    END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM,
    SWOOP_DISTANCE,
    ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN,
    MB_STACKS_AGAINST_CHAINSAW,
    /// legacy, keep around to make sure old replays or running games do not break after update
    CHAINSAW_TURNOVER_ON_AV_BREAK,
    CHAINSAW_TURNOVER,
    BOMBER_PLACED_PRONE_IGNORES_TURNOVER,
    BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER,
    SNEAKY_GIT_CAN_MOVE_AFTER_FOUL,
    BOMB_USES_MB,
    CATCH_WORKS_FOR_BOMBS,
    ONLY_ONE_BRIBE_PER_SEND_OFF,
    OVERTIME_GOLDEN_GOAL,
    OVERTIME_KICK_OFF_RESULTS,
    ENABLE_TACKLEZONE_OVERLAYS,
    BOMB_BOUNCES_ON_EMPTY_SQUARES,
    ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION,

    PITCH_URL,
}

impl GameOptionId {
    /// Java: `getName()` — returns `fName`.
    pub fn get_name(self) -> &'static str {
        match self {
            Self::RULESVERSION => RULESVERSION,
            Self::CHECK_OWNERSHIP => CHECK_OWNERSHIP,
            Self::TEST_MODE => TEST_MODE,
            Self::OVERTIME => OVERTIME,
            Self::TURNTIME => TURNTIME,
            Self::ALLOW_CONCESSIONS => ALLOW_CONCESSIONS,
            Self::PETTY_CASH => PETTY_CASH,
            Self::INDUCEMENTS => INDUCEMENTS,
            Self::MAX_NR_OF_CARDS => MAX_NR_OF_CARDS,
            Self::MAX_PLAYERS_ON_FIELD => MAX_PLAYERS_ON_FIELD,
            Self::MAX_PLAYERS_IN_WIDE_ZONE => MAX_PLAYERS_IN_WIDE_ZONE,
            Self::MIN_PLAYERS_ON_LOS => MIN_PLAYERS_ON_LOS,
            Self::ALLOW_STAR_ON_BOTH_TEAMS => ALLOW_STAR_ON_BOTH_TEAMS,
            Self::ALLOW_STAFF_ON_BOTH_TEAMS => ALLOW_STAFF_ON_BOTH_TEAMS,
            Self::FORCE_TREASURY_TO_PETTY_CASH => FORCE_TREASURY_TO_PETTY_CASH,
            Self::USE_PREDEFINED_INDUCEMENTS => USE_PREDEFINED_INDUCEMENTS,
            Self::ALLOW_KTM_REROLL => ALLOW_KTM_REROLL,
            Self::CLAW_DOES_NOT_STACK => CLAW_DOES_NOT_STACK,
            Self::FOUL_BONUS => FOUL_BONUS,
            Self::FOUL_BONUS_OUTSIDE_TACKLEZONE => FOUL_BONUS_OUTSIDE_TACKLEZONE,
            Self::FREE_INDUCEMENT_CASH => FREE_INDUCEMENT_CASH,
            Self::FREE_CARD_CASH => FREE_CARD_CASH,
            Self::PILING_ON_DOES_NOT_STACK => PILING_ON_DOES_NOT_STACK,
            Self::PILING_ON_INJURY_ONLY => PILING_ON_INJURY_ONLY,
            Self::PILING_ON_ARMOR_ONLY => PILING_ON_ARMOR_ONLY,
            Self::PILING_ON_TO_KO_ON_DOUBLE => PILING_ON_TO_KO_ON_DOUBLE,
            Self::PILING_ON_USES_A_TEAM_REROLL => PILING_ON_USES_A_TEAM_REROLL,
            Self::RIGHT_STUFF_CANCELS_TACKLE => RIGHT_STUFF_CANCELS_TACKLE,
            Self::SNEAKY_GIT_AS_FOUL_GUARD => SNEAKY_GIT_AS_FOUL_GUARD,
            Self::SNEAKY_GIT_BAN_TO_KO => SNEAKY_GIT_BAN_TO_KO,
            Self::STAND_FIRM_NO_DROP_ON_FAILED_DODGE => STAND_FIRM_NO_DROP_ON_FAILED_DODGE,
            Self::SPIKED_BALL => SPIKED_BALL,
            Self::DIVING_TACKLE_LEAVING_TZ_ONLY => DIVING_TACKLE_LEAVING_TZ_ONLY,
            Self::ALLOW_SPECIAL_ACTIONS_FROM_PRONE => ALLOW_SPECIAL_ACTIONS_FROM_PRONE,
            Self::ALLOW_BRAWLER_ON_BOTH_BLOCKS => ALLOW_BRAWLER_ON_BOTH_BLOCKS,
            Self::ASK_FOR_KICK_AFTER_ROLL => ASK_FOR_KICK_AFTER_ROLL,
            Self::ARGUE_THE_CALL => ARGUE_THE_CALL,
            Self::MVP_NOMINATIONS => MVP_NOMINATIONS,
            Self::PETTY_CASH_AFFECTS_TV => PETTY_CASH_AFFECTS_TV,
            Self::WIZARD_AVAILABLE => WIZARD_AVAILABLE,
            Self::EXTRA_MVP => EXTRA_MVP,
            Self::CARDS_MISCELLANEOUS_MAYHEM_COST => CARDS_MISCELLANEOUS_MAYHEM_COST,
            Self::CARDS_MISCELLANEOUS_MAYHEM_MAX => CARDS_MISCELLANEOUS_MAYHEM_MAX,
            Self::CARDS_SPECIAL_TEAM_PLAY_COST => CARDS_SPECIAL_TEAM_PLAY_COST,
            Self::CARDS_SPECIAL_TEAM_PLAY_MAX => CARDS_SPECIAL_TEAM_PLAY_MAX,
            Self::CARDS_MAGIC_ITEM_COST => CARDS_MAGIC_ITEM_COST,
            Self::CARDS_MAGIC_ITEM_MAX => CARDS_MAGIC_ITEM_MAX,
            Self::CARDS_DIRTY_TRICK_COST => CARDS_DIRTY_TRICK_COST,
            Self::CARDS_DIRTY_TRICK_MAX => CARDS_DIRTY_TRICK_MAX,
            Self::CARDS_GOOD_KARMA_COST => CARDS_GOOD_KARMA_COST,
            Self::CARDS_GOOD_KARMA_MAX => CARDS_GOOD_KARMA_MAX,
            Self::CARDS_RANDOM_EVENT_COST => CARDS_RANDOM_EVENT_COST,
            Self::CARDS_RANDOM_EVENT_MAX => CARDS_RANDOM_EVENT_MAX,
            Self::CARDS_DESPERATE_MEASURE_COST => CARDS_DESPERATE_MEASURE_COST,
            Self::CARDS_DESPERATE_MEASURE_MAX => CARDS_DESPERATE_MEASURE_MAX,
            Self::CARDS_SPECIAL_PLAY_COST => CARDS_SPECIAL_PLAY_COST,
            Self::INDUCEMENT_APOS_COST => INDUCEMENT_APOS_COST,
            Self::INDUCEMENT_APOS_MAX => INDUCEMENT_APOS_MAX,
            Self::INDUCEMENT_BRIBES_COST => INDUCEMENT_BRIBES_COST,
            Self::INDUCEMENT_BRIBES_REDUCED_COST => INDUCEMENT_BRIBES_REDUCED_COST,
            Self::INDUCEMENT_BRIBES_MAX => INDUCEMENT_BRIBES_MAX,
            Self::INDUCEMENT_BRIBES_REDUCED_MAX => INDUCEMENT_BRIBES_REDUCED_MAX,
            Self::INDUCEMENT_CHEFS_COST => INDUCEMENT_CHEFS_COST,
            Self::INDUCEMENT_CHEFS_REDUCED_COST => INDUCEMENT_CHEFS_REDUCED_COST,
            Self::INDUCEMENT_CHEFS_MAX => INDUCEMENT_CHEFS_MAX,
            Self::INDUCEMENT_CHEFS_REDUCED_MAX => INDUCEMENT_CHEFS_REDUCED_MAX,
            Self::INDUCEMENT_EXTRA_TRAINING_COST => INDUCEMENT_EXTRA_TRAINING_COST,
            Self::INDUCEMENT_EXTRA_TRAINING_MAX => INDUCEMENT_EXTRA_TRAINING_MAX,
            Self::INDUCEMENT_IGORS_COST => INDUCEMENT_IGORS_COST,
            Self::INDUCEMENT_IGORS_MAX => INDUCEMENT_IGORS_MAX,
            Self::INDUCEMENT_MORTUARY_ASSISTANTS_COST => INDUCEMENT_MORTUARY_ASSISTANTS_COST,
            Self::INDUCEMENT_MORTUARY_ASSISTANTS_MAX => INDUCEMENT_MORTUARY_ASSISTANTS_MAX,
            Self::INDUCEMENT_PLAGUE_DOCTORS_COST => INDUCEMENT_PLAGUE_DOCTORS_COST,
            Self::INDUCEMENT_PLAGUE_DOCTORS_MAX => INDUCEMENT_PLAGUE_DOCTORS_MAX,
            Self::INDUCEMENT_KEGS_COST => INDUCEMENT_KEGS_COST,
            Self::INDUCEMENT_KEGS_MAX => INDUCEMENT_KEGS_MAX,
            Self::INDUCEMENT_MASCOT_MAX => INDUCEMENT_MASCOT_MAX,
            Self::INDUCEMENT_MASCOT_COST => INDUCEMENT_MASCOT_COST,
            Self::INDUCEMENT_MERCENARIES_EXTRA_COST => INDUCEMENT_MERCENARIES_EXTRA_COST,
            Self::INDUCEMENT_MERCENARIES_SKILL_COST => INDUCEMENT_MERCENARIES_SKILL_COST,
            Self::INDUCEMENT_MERCENARIES_MAX => INDUCEMENT_MERCENARIES_MAX,
            Self::INDUCEMENT_PRAYERS_COST => INDUCEMENT_PRAYERS_COST,
            Self::INDUCEMENT_PRAYERS_MAX => INDUCEMENT_PRAYERS_MAX,
            Self::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE => INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE,
            Self::INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG => INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG,
            Self::INDUCEMENT_RIOTOUS_ROOKIES_COST => INDUCEMENT_RIOTOUS_ROOKIES_COST,
            Self::INDUCEMENT_RIOTOUS_ROOKIES_MAX => INDUCEMENT_RIOTOUS_ROOKIES_MAX,
            Self::INDUCEMENT_STARS_MAX => INDUCEMENT_STARS_MAX,
            Self::INDUCEMENT_STAFF_MAX => INDUCEMENT_STAFF_MAX,
            Self::INDUCEMENT_WIZARDS_COST => INDUCEMENT_WIZARDS_COST,
            Self::INDUCEMENT_WIZARDS_MAX => INDUCEMENT_WIZARDS_MAX,
            Self::INDUCEMENT_BIASED_REF_COST => INDUCEMENT_BIASED_REF_COST,
            Self::INDUCEMENT_BIASED_REF_REDUCED_COST => INDUCEMENT_BIASED_REF_REDUCED_COST,
            Self::INDUCEMENT_BIASED_REF_MAX => INDUCEMENT_BIASED_REF_MAX,
            Self::INDUCEMENT_BIASED_REF_REDUCED_MAX => INDUCEMENT_BIASED_REF_REDUCED_MAX,
            Self::INDUCEMENT_TEMP_CHEERLEADER_MAX => INDUCEMENT_TEMP_CHEERLEADER_MAX,
            Self::INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX => INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX,
            Self::INDUCEMENT_TEMP_CHEERLEADER_COST => INDUCEMENT_TEMP_CHEERLEADER_COST,
            Self::INDUCEMENT_PART_TIME_COACH_MAX => INDUCEMENT_PART_TIME_COACH_MAX,
            Self::INDUCEMENT_PART_TIME_COACH_TOTAL_MAX => INDUCEMENT_PART_TIME_COACH_TOTAL_MAX,
            Self::INDUCEMENT_PART_TIME_COACH_COST => INDUCEMENT_PART_TIME_COACH_COST,
            Self::INDUCEMENT_WEATHER_MAGE_MAX => INDUCEMENT_WEATHER_MAGE_MAX,
            Self::INDUCEMENT_WEATHER_MAGE_COST => INDUCEMENT_WEATHER_MAGE_COST,
            Self::INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV => INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV,
            Self::INDUCEMENTS_ALWAYS_USE_TREASURY => INDUCEMENTS_ALWAYS_USE_TREASURY,
            Self::INDUCEMENTS_ALLOW_OVERDOG_SPENDING => INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
            Self::INDUCEMENT_DUMMY => INDUCEMENT_DUMMY,
            Self::ENABLE_STALLING_CHECK => ENABLE_STALLING_CHECK,
            Self::ALLOW_BALL_AND_CHAIN_RE_ROLL => ALLOW_BALL_AND_CHAIN_RE_ROLL,
            Self::END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM => END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM,
            Self::SWOOP_DISTANCE => SWOOP_DISTANCE,
            Self::ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN => ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN,
            Self::MB_STACKS_AGAINST_CHAINSAW => MB_STACKS_AGAINST_CHAINSAW,
            Self::CHAINSAW_TURNOVER_ON_AV_BREAK => CHAINSAW_TURNOVER_ON_AV_BREAK,
            Self::CHAINSAW_TURNOVER => CHAINSAW_TURNOVER,
            Self::BOMBER_PLACED_PRONE_IGNORES_TURNOVER => BOMBER_PLACED_PRONE_IGNORES_TURNOVER,
            Self::BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER => BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER,
            Self::SNEAKY_GIT_CAN_MOVE_AFTER_FOUL => SNEAKY_GIT_CAN_MOVE_AFTER_FOUL,
            Self::BOMB_USES_MB => BOMB_USES_MB,
            Self::CATCH_WORKS_FOR_BOMBS => CATCH_WORKS_FOR_BOMBS,
            Self::ONLY_ONE_BRIBE_PER_SEND_OFF => ONLY_ONE_BRIBE_PER_SEND_OFF,
            Self::OVERTIME_GOLDEN_GOAL => OVERTIME_GOLDEN_GOAL,
            Self::OVERTIME_KICK_OFF_RESULTS => OVERTIME_KICK_OFF_RESULTS,
            Self::ENABLE_TACKLEZONE_OVERLAYS => ENABLE_TACKLEZONE_OVERLAYS,
            Self::BOMB_BOUNCES_ON_EMPTY_SQUARES => BOMB_BOUNCES_ON_EMPTY_SQUARES,
            Self::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION => ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION,
            Self::PITCH_URL => PITCH_URL,
        }
    }

    /// Java: `GameOptionIdFactory.forName(String)` — case-insensitive match against
    /// `values()`, plus legacy backwards-compatibility aliases for old Fumbbl option names.
    pub fn for_name(name: &str) -> Option<Self> {
        for id in Self::values() {
            if name.eq_ignore_ascii_case(id.get_name()) {
                return Some(*id);
            }
        }
        // backwards compatibility (wrong Fumbbl option names)
        match name.to_lowercase().as_str() {
            "maxcards" => Some(Self::MAX_NR_OF_CARDS),
            "cardgold" => Some(Self::FREE_CARD_CASH),
            "freeinducementmoney" => Some(Self::FREE_INDUCEMENT_CASH),
            "widezoneplayers" => Some(Self::MAX_PLAYERS_IN_WIDE_ZONE),
            "playersonfield" => Some(Self::MAX_PLAYERS_ON_FIELD),
            "playersonlos" => Some(Self::MIN_PLAYERS_ON_LOS),
            "clawnostack" => Some(Self::CLAW_DOES_NOT_STACK),
            "pilingonnostack" => Some(Self::PILING_ON_DOES_NOT_STACK),
            "pilingonkodouble" => Some(Self::PILING_ON_TO_KO_ON_DOUBLE),
            "sneakyasfoul" => Some(Self::SNEAKY_GIT_AS_FOUL_GUARD),
            "sneakybantoko" => Some(Self::SNEAKY_GIT_BAN_TO_KO),
            "standfirmnofall" => Some(Self::STAND_FIRM_NO_DROP_ON_FAILED_DODGE),
            "rightstuffcanceltackle" => Some(Self::RIGHT_STUFF_CANCELS_TACKLE),
            _ => None,
        }
    }

    /// Java: `GameOptionId.values()` — every variant, in declaration order.
    pub fn values() -> &'static [Self] {
        &[
            Self::RULESVERSION, Self::CHECK_OWNERSHIP, Self::TEST_MODE, Self::OVERTIME,
            Self::TURNTIME, Self::ALLOW_CONCESSIONS, Self::PETTY_CASH, Self::INDUCEMENTS,
            Self::MAX_NR_OF_CARDS, Self::MAX_PLAYERS_ON_FIELD, Self::MAX_PLAYERS_IN_WIDE_ZONE,
            Self::MIN_PLAYERS_ON_LOS, Self::ALLOW_STAR_ON_BOTH_TEAMS, Self::ALLOW_STAFF_ON_BOTH_TEAMS,
            Self::FORCE_TREASURY_TO_PETTY_CASH, Self::USE_PREDEFINED_INDUCEMENTS,
            Self::ALLOW_KTM_REROLL, Self::CLAW_DOES_NOT_STACK, Self::FOUL_BONUS,
            Self::FOUL_BONUS_OUTSIDE_TACKLEZONE, Self::FREE_INDUCEMENT_CASH, Self::FREE_CARD_CASH,
            Self::PILING_ON_DOES_NOT_STACK, Self::PILING_ON_INJURY_ONLY, Self::PILING_ON_ARMOR_ONLY,
            Self::PILING_ON_TO_KO_ON_DOUBLE, Self::PILING_ON_USES_A_TEAM_REROLL,
            Self::RIGHT_STUFF_CANCELS_TACKLE, Self::SNEAKY_GIT_AS_FOUL_GUARD, Self::SNEAKY_GIT_BAN_TO_KO,
            Self::STAND_FIRM_NO_DROP_ON_FAILED_DODGE, Self::SPIKED_BALL, Self::DIVING_TACKLE_LEAVING_TZ_ONLY,
            Self::ALLOW_SPECIAL_ACTIONS_FROM_PRONE, Self::ALLOW_BRAWLER_ON_BOTH_BLOCKS,
            Self::ASK_FOR_KICK_AFTER_ROLL, Self::ARGUE_THE_CALL, Self::MVP_NOMINATIONS,
            Self::PETTY_CASH_AFFECTS_TV, Self::WIZARD_AVAILABLE, Self::EXTRA_MVP,
            Self::CARDS_MISCELLANEOUS_MAYHEM_COST, Self::CARDS_MISCELLANEOUS_MAYHEM_MAX,
            Self::CARDS_SPECIAL_TEAM_PLAY_COST, Self::CARDS_SPECIAL_TEAM_PLAY_MAX,
            Self::CARDS_MAGIC_ITEM_COST, Self::CARDS_MAGIC_ITEM_MAX, Self::CARDS_DIRTY_TRICK_COST,
            Self::CARDS_DIRTY_TRICK_MAX, Self::CARDS_GOOD_KARMA_COST, Self::CARDS_GOOD_KARMA_MAX,
            Self::CARDS_RANDOM_EVENT_COST, Self::CARDS_RANDOM_EVENT_MAX,
            Self::CARDS_DESPERATE_MEASURE_COST, Self::CARDS_DESPERATE_MEASURE_MAX,
            Self::CARDS_SPECIAL_PLAY_COST, Self::INDUCEMENT_APOS_COST, Self::INDUCEMENT_APOS_MAX,
            Self::INDUCEMENT_BRIBES_COST, Self::INDUCEMENT_BRIBES_REDUCED_COST,
            Self::INDUCEMENT_BRIBES_MAX, Self::INDUCEMENT_BRIBES_REDUCED_MAX,
            Self::INDUCEMENT_CHEFS_COST, Self::INDUCEMENT_CHEFS_REDUCED_COST,
            Self::INDUCEMENT_CHEFS_MAX, Self::INDUCEMENT_CHEFS_REDUCED_MAX,
            Self::INDUCEMENT_EXTRA_TRAINING_COST, Self::INDUCEMENT_EXTRA_TRAINING_MAX,
            Self::INDUCEMENT_IGORS_COST, Self::INDUCEMENT_IGORS_MAX,
            Self::INDUCEMENT_MORTUARY_ASSISTANTS_COST, Self::INDUCEMENT_MORTUARY_ASSISTANTS_MAX,
            Self::INDUCEMENT_PLAGUE_DOCTORS_COST, Self::INDUCEMENT_PLAGUE_DOCTORS_MAX,
            Self::INDUCEMENT_KEGS_COST, Self::INDUCEMENT_KEGS_MAX, Self::INDUCEMENT_MASCOT_MAX,
            Self::INDUCEMENT_MASCOT_COST, Self::INDUCEMENT_MERCENARIES_EXTRA_COST,
            Self::INDUCEMENT_MERCENARIES_SKILL_COST, Self::INDUCEMENT_MERCENARIES_MAX,
            Self::INDUCEMENT_PRAYERS_COST, Self::INDUCEMENT_PRAYERS_MAX,
            Self::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE, Self::INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG,
            Self::INDUCEMENT_RIOTOUS_ROOKIES_COST, Self::INDUCEMENT_RIOTOUS_ROOKIES_MAX,
            Self::INDUCEMENT_STARS_MAX, Self::INDUCEMENT_STAFF_MAX, Self::INDUCEMENT_WIZARDS_COST,
            Self::INDUCEMENT_WIZARDS_MAX, Self::INDUCEMENT_BIASED_REF_COST,
            Self::INDUCEMENT_BIASED_REF_REDUCED_COST, Self::INDUCEMENT_BIASED_REF_MAX,
            Self::INDUCEMENT_BIASED_REF_REDUCED_MAX, Self::INDUCEMENT_TEMP_CHEERLEADER_MAX,
            Self::INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX, Self::INDUCEMENT_TEMP_CHEERLEADER_COST,
            Self::INDUCEMENT_PART_TIME_COACH_MAX, Self::INDUCEMENT_PART_TIME_COACH_TOTAL_MAX,
            Self::INDUCEMENT_PART_TIME_COACH_COST, Self::INDUCEMENT_WEATHER_MAGE_MAX,
            Self::INDUCEMENT_WEATHER_MAGE_COST, Self::INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV,
            Self::INDUCEMENTS_ALWAYS_USE_TREASURY, Self::INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
            Self::INDUCEMENT_DUMMY, Self::ENABLE_STALLING_CHECK, Self::ALLOW_BALL_AND_CHAIN_RE_ROLL,
            Self::END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM, Self::SWOOP_DISTANCE,
            Self::ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN, Self::MB_STACKS_AGAINST_CHAINSAW,
            Self::CHAINSAW_TURNOVER_ON_AV_BREAK, Self::CHAINSAW_TURNOVER,
            Self::BOMBER_PLACED_PRONE_IGNORES_TURNOVER, Self::BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER,
            Self::SNEAKY_GIT_CAN_MOVE_AFTER_FOUL, Self::BOMB_USES_MB, Self::CATCH_WORKS_FOR_BOMBS,
            Self::ONLY_ONE_BRIBE_PER_SEND_OFF, Self::OVERTIME_GOLDEN_GOAL, Self::OVERTIME_KICK_OFF_RESULTS,
            Self::ENABLE_TACKLEZONE_OVERLAYS, Self::BOMB_BOUNCES_ON_EMPTY_SQUARES,
            Self::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION, Self::PITCH_URL,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_players_on_field_key_matches_java() {
        assert_eq!(MAX_PLAYERS_ON_FIELD, "maxPlayersOnField");
    }

    #[test]
    fn max_players_in_wide_zone_key_matches_java() {
        assert_eq!(MAX_PLAYERS_IN_WIDE_ZONE, "maxPlayersInWideZone");
    }

    #[test]
    fn min_players_on_los_key_matches_java() {
        assert_eq!(MIN_PLAYERS_ON_LOS, "minPlayersOnLos");
    }

    #[test]
    fn overtime_kick_off_results_key_matches_java() {
        assert_eq!(OVERTIME_KICK_OFF_RESULTS, "overtimeKickOffResults");
    }

    #[test]
    fn stand_firm_no_drop_key_matches_java() {
        assert_eq!(STAND_FIRM_NO_DROP_ON_FAILED_DODGE, "standFirmNoDropOnFailedDodge");
    }

    #[test]
    fn values_has_127_variants() {
        assert_eq!(GameOptionId::values().len(), 127);
    }

    #[test]
    fn get_name_matches_java_for_sample_variants() {
        assert_eq!(GameOptionId::RULESVERSION.get_name(), "rulesVersion");
        assert_eq!(GameOptionId::PITCH_URL.get_name(), "pitchUrl");
        assert_eq!(GameOptionId::MAX_PLAYERS_ON_FIELD.get_name(), "maxPlayersOnField");
        assert_eq!(
            GameOptionId::CHAINSAW_TURNOVER_ON_AV_BREAK.get_name(),
            "chainsawTurnoverOnAvBreak"
        );
    }

    #[test]
    fn for_name_round_trips_all_variants() {
        for id in GameOptionId::values() {
            let name = id.get_name();
            assert_eq!(GameOptionId::for_name(name), Some(*id), "round trip failed for {:?}", id);
        }
    }

    #[test]
    fn for_name_is_case_insensitive() {
        assert_eq!(GameOptionId::for_name("pitchUrl"), Some(GameOptionId::PITCH_URL));
        assert_eq!(GameOptionId::for_name("PITCHURL"), Some(GameOptionId::PITCH_URL));
        assert_eq!(GameOptionId::for_name("PitchUrl"), Some(GameOptionId::PITCH_URL));
        assert_eq!(GameOptionId::for_name("notARealOption"), None);
    }

    #[test]
    fn for_name_legacy_aliases() {
        assert_eq!(GameOptionId::for_name("maxCards"), Some(GameOptionId::MAX_NR_OF_CARDS));
        assert_eq!(GameOptionId::for_name("cardGold"), Some(GameOptionId::FREE_CARD_CASH));
        assert_eq!(
            GameOptionId::for_name("freeInducementMoney"),
            Some(GameOptionId::FREE_INDUCEMENT_CASH)
        );
        assert_eq!(
            GameOptionId::for_name("wideZonePlayers"),
            Some(GameOptionId::MAX_PLAYERS_IN_WIDE_ZONE)
        );
        assert_eq!(GameOptionId::for_name("playersOnField"), Some(GameOptionId::MAX_PLAYERS_ON_FIELD));
        assert_eq!(GameOptionId::for_name("playersOnLos"), Some(GameOptionId::MIN_PLAYERS_ON_LOS));
        assert_eq!(GameOptionId::for_name("clawNoStack"), Some(GameOptionId::CLAW_DOES_NOT_STACK));
        assert_eq!(
            GameOptionId::for_name("pilingOnNoStack"),
            Some(GameOptionId::PILING_ON_DOES_NOT_STACK)
        );
        assert_eq!(
            GameOptionId::for_name("pilingOnKoDouble"),
            Some(GameOptionId::PILING_ON_TO_KO_ON_DOUBLE)
        );
        assert_eq!(
            GameOptionId::for_name("sneakyAsFoul"),
            Some(GameOptionId::SNEAKY_GIT_AS_FOUL_GUARD)
        );
        assert_eq!(GameOptionId::for_name("sneakyBanToKo"), Some(GameOptionId::SNEAKY_GIT_BAN_TO_KO));
        assert_eq!(
            GameOptionId::for_name("standFirmNoFall"),
            Some(GameOptionId::STAND_FIRM_NO_DROP_ON_FAILED_DODGE)
        );
        assert_eq!(
            GameOptionId::for_name("rightStuffCancelTackle"),
            Some(GameOptionId::RIGHT_STUFF_CANCELS_TACKLE)
        );
        assert_eq!(GameOptionId::for_name("totallyUnknown"), None);
    }
}
