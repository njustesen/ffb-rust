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
}
