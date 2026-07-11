use crate::option::game_option_boolean::GameOptionBoolean;
use crate::option::game_option_id::GameOptionId;
use crate::option::game_option_int::GameOptionInt;
use crate::option::game_option_string::{
    GameOptionString, CHAINSAW_TURNOVER_ALL_AV_BREAKS, CHAINSAW_TURNOVER_KICKBACK,
    CHAINSAW_TURNOVER_KICKBACK_AV_BREAK_ONLY, CHAINSAW_TURNOVER_NEVER, OVERTIME_KICK_OFF_ALL,
    OVERTIME_KICK_OFF_BLITZ, OVERTIME_KICK_OFF_BLITZ_OR_SOLID_DEFENCE,
    OVERTIME_KICK_OFF_RANDOM_BLITZ_OR_SOLID_DEFENCE, OVERTIME_KICK_OFF_SOLID_DEFENCE,
};
use crate::option::i_game_option::IGameOption;

/// 1:1 translation of com.fumbbl.ffb.factory.GameOptionFactory.
pub struct GameOptionFactory;

impl GameOptionFactory {
    pub fn new() -> Self {
        Self
    }

    /// Java: `createGameOption(GameOptionId pOptionId)` — builds a fresh `IGameOption`
    /// (with its default value and display message(s)) for the given id. Java's switch
    /// falls through to `default: return null` for ids it doesn't recognize, but every
    /// `GameOptionId` variant is in fact handled below (matches Java 1:1).
    pub fn create_game_option(&self, option_id: GameOptionId) -> Option<Box<dyn IGameOption>> {
        use GameOptionId::*;
        let name = option_id.get_name();
        let opt: Box<dyn IGameOption> = match option_id {
            ALLOW_CONCESSIONS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_false("Concessions have been disabled");
                Box::new(o)
            }
            ALLOW_KTM_REROLL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Kick Team-Mate can be rerolled.");
                Box::new(o)
            }
            ALLOW_STAR_ON_BOTH_TEAMS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("A star player may play for both teams.");
                Box::new(o)
            }
            ALLOW_STAFF_ON_BOTH_TEAMS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("An Infamous Staff member may be hired by both teams.");
                Box::new(o)
            }
            ARGUE_THE_CALL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_false("Calls may not be argued.");
                Box::new(o)
            }
            CARDS_DESPERATE_MEASURE_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(400_000)
                    .set_message("Desperate Measure cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_DESPERATE_MEASURE_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Desperate Measure cards.");
                Box::new(o)
            }
            CARDS_DIRTY_TRICK_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Dirty Trick cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_DIRTY_TRICK_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Dirty Trick cards.");
                Box::new(o)
            }
            CARDS_GOOD_KARMA_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000)
                    .set_message("Good Karma cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_GOOD_KARMA_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Good Karma cards.");
                Box::new(o)
            }
            CARDS_MAGIC_ITEM_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Magic Item cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_MAGIC_ITEM_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Magic Item cards.");
                Box::new(o)
            }
            CARDS_MISCELLANEOUS_MAYHEM_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Miscellaneous Mayhem cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_MISCELLANEOUS_MAYHEM_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Miscellaneous Mayhem cards.");
                Box::new(o)
            }
            CARDS_RANDOM_EVENT_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(200_000)
                    .set_message("Random Event cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_RANDOM_EVENT_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Random Event cards.");
                Box::new(o)
            }
            CARDS_SPECIAL_TEAM_PLAY_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Special Team Play cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CARDS_SPECIAL_TEAM_PLAY_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX)
                    .set_message("Coaches may purchase up to $1 Special Team Play cards.");
                Box::new(o)
            }
            CARDS_SPECIAL_PLAY_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000)
                    .set_message("Special Play cards can be bought for $1 gps each.");
                Box::new(o)
            }
            CHECK_OWNERSHIP => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_false("Team ownership is not checked.");
                Box::new(o)
            }
            CLAW_DOES_NOT_STACK => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_true(
                    "Claw does not stack with other skills that modify armour rolls.",
                );
                Box::new(o)
            }
            DIVING_TACKLE_LEAVING_TZ_ONLY => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_true("Diving Tackle only when the dodger leaves the tackler's TZ")
                    .set_message_false("Diving Tackle allowed even if the dodger stays adjacent");
                Box::new(o)
            }
            EXTRA_MVP => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("An extra MVP is awarded at the end of the match");
                Box::new(o)
            }
            FORCE_TREASURY_TO_PETTY_CASH => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Treasury is not automatically transferred to Petty Cash.");
                Box::new(o)
            }
            FOUL_BONUS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("+1 to armour roll for a foul.");
                Box::new(o)
            }
            FOUL_BONUS_OUTSIDE_TACKLEZONE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "+1 to armour roll for a foul, if fouler is not in an opposing tacklezone.",
                );
                Box::new(o)
            }
            FREE_CARD_CASH => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0)
                    .set_message("Both coaches get $1 extra gold to buy cards with.");
                Box::new(o)
            }
            FREE_INDUCEMENT_CASH => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0)
                    .set_message("Both coaches get $1 extra gold to buy inducements with.");
                Box::new(o)
            }
            INDUCEMENTS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_false("Inducements are not available.");
                Box::new(o)
            }
            INDUCEMENT_APOS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000)
                    .set_message("Wandering apothecaries can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_APOS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(2)
                    .set_message("Coaches may purchase up to $1 wandering apothecarie(s).");
                Box::new(o)
            }
            INDUCEMENT_BRIBES_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000).set_message("Bribes can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_BRIBES_REDUCED_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Bribes for reduced price can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_BRIBES_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(3).set_message("Coaches may purchase up to $1 bribe(s).");
                Box::new(o)
            }
            INDUCEMENT_BRIBES_REDUCED_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(6).set_message(
                    "Coaches may purchase up to $1 bribe(s) for reduced costs.",
                );
                Box::new(o)
            }
            INDUCEMENT_CHEFS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(300_000)
                    .set_message("Halfling Master Chefs can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_CHEFS_REDUCED_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000).set_message(
                    "Halfling Master Chefs for reduced price can be purchased for $1 gps each.",
                );
                Box::new(o)
            }
            INDUCEMENT_CHEFS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1)
                    .set_message("Coaches may purchase up to $1 Halfling Master Chef(s).");
                Box::new(o)
            }
            INDUCEMENT_CHEFS_REDUCED_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message(
                    "Coaches may purchase up to $1 Halfling Master Chef(s) for reduced costs.",
                );
                Box::new(o)
            }
            INDUCEMENT_IGORS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000).set_message("Igors can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_IGORS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches may purchase up to $1 Igor(s).");
                Box::new(o)
            }
            INDUCEMENT_MORTUARY_ASSISTANTS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000)
                    .set_message("Mortuary Assistants can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_MORTUARY_ASSISTANTS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1)
                    .set_message("Coaches may purchase up to $1 Mortuary Assistant(s).");
                Box::new(o)
            }
            INDUCEMENT_PLAGUE_DOCTORS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000).set_message("Plague Doctors can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_PLAGUE_DOCTORS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches may purchase up to $1 Plague Doctor(s).");
                Box::new(o)
            }
            INDUCEMENT_KEGS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000)
                    .set_message("Bloodweiser Kegs can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_KEGS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(2).set_message("Coaches may purchase up to $1 Bloodweiser Keg(s).");
                Box::new(o)
            }
            INDUCEMENT_MERCENARIES_EXTRA_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(30_000)
                    .set_message("Mercenaries can be purchased for an extra $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_MERCENARIES_SKILL_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000).set_message("Mercenaries can can gain an extra skill for $1 gps.");
                Box::new(o)
            }
            INDUCEMENT_MERCENARIES_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(i32::MAX).set_message("Coaches may purchase up to $1 Mercenaries.");
                Box::new(o)
            }
            INDUCEMENT_EXTRA_TRAINING_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000).set_message("Rerolls can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_EXTRA_TRAINING_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(4).set_message("Coaches may purchase up to $1 reroll(s).");
                Box::new(o)
            }
            INDUCEMENT_MASCOT_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(25_000).set_message("Team Mascots can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_MASCOT_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches may purchase up to $1 Team Mascot(s).");
                Box::new(o)
            }
            INDUCEMENT_STAFF_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(2).set_message(
                    "Coaches may purchase up to $1 infamous coaching staff member(s).",
                );
                Box::new(o)
            }
            INDUCEMENT_STARS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(2).set_message("Coaches may purchase up to $1 star(s).");
                Box::new(o)
            }
            INDUCEMENT_WIZARDS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(150_000).set_message("Wizards can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_WIZARDS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches may purchase up to $1 wizard(s).");
                Box::new(o)
            }
            INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("On equal CTV teams can buy inducements from treasury");
                Box::new(o)
            }
            INDUCEMENTS_ALWAYS_USE_TREASURY => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Teams will always use treasury instead of petty cash for inducements",
                );
                Box::new(o)
            }
            MAX_NR_OF_CARDS => {
                let mut o = GameOptionInt::new(name);
                o.set_default(5).set_message("A maximum of $1 cards can be bought.");
                Box::new(o)
            }
            MAX_PLAYERS_IN_WIDE_ZONE => {
                let mut o = GameOptionInt::new(name);
                o.set_default(2)
                    .set_message("A maximum of $1 players may be set up in a widezone.");
                Box::new(o)
            }
            MAX_PLAYERS_ON_FIELD => {
                let mut o = GameOptionInt::new(name);
                o.set_default(11)
                    .set_message("A maximum of $1 players may be set up on the field.");
                Box::new(o)
            }
            MIN_PLAYERS_ON_LOS => {
                let mut o = GameOptionInt::new(name);
                o.set_default(3).set_message(
                    "A minimum of $1 players must be set up on the line of scrimmage.",
                );
                Box::new(o)
            }
            MVP_NOMINATIONS => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0)
                    .set_message("$1 players may be nominated to receive the MVP award.");
                Box::new(o)
            }
            OVERTIME => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Game will go into overtime if there is a draw after 2nd half.");
                Box::new(o)
            }
            PETTY_CASH => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_false("Petty Cash is not available.");
                Box::new(o)
            }
            PETTY_CASH_AFFECTS_TV => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Petty Cash affects Team Value.");
                Box::new(o)
            }
            PILING_ON_ARMOR_ONLY => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Piling On lets you re-roll armour-rolls only.");
                Box::new(o)
            }
            PILING_ON_DOES_NOT_STACK => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Piling On does not stack with other skills that modify armour- or injury-rolls.",
                );
                Box::new(o)
            }
            PILING_ON_INJURY_ONLY => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Piling On lets you re-roll injury-rolls only.");
                Box::new(o)
            }
            PILING_ON_TO_KO_ON_DOUBLE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Piling On player knocks himself out when rolling a double on armour or injury.",
                );
                Box::new(o)
            }
            PILING_ON_USES_A_TEAM_REROLL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Piling On does not cost a Team Re-roll to use.");
                Box::new(o)
            }
            PITCH_URL => {
                let mut o = GameOptionString::new(name);
                o.set_message("Custom pitch set.");
                Box::new(o)
            }
            RIGHT_STUFF_CANCELS_TACKLE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Right Stuff prevents Tackle from negating Dodge for Pow/Pushback.",
                );
                Box::new(o)
            }
            RULESVERSION => {
                let mut o = GameOptionString::new(name);
                o.set_default("BB2016").set_message("Rules Version $1");
                Box::new(o)
            }
            SNEAKY_GIT_AS_FOUL_GUARD => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Sneaky Git works like Guard for fouling assists.");
                Box::new(o)
            }
            SNEAKY_GIT_BAN_TO_KO => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Sneaky Git players that get banned are sent to the KO box instead.",
                );
                Box::new(o)
            }
            SPIKED_BALL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "A Spiked Ball is used for play. Any failed Pickup or Catch roll results in the player being stabbed.",
                );
                Box::new(o)
            }
            STAND_FIRM_NO_DROP_ON_FAILED_DODGE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Stand Firm players do not drop on a failed dodge roll but end their move instead.",
                );
                Box::new(o)
            }
            TEST_MODE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true(
                    "Game is in TEST mode. No result will be uploaded. See help for available test commands.",
                );
                Box::new(o)
            }
            TURNTIME => {
                let mut o = GameOptionInt::new(name);
                o.set_default(240).set_message("Turntime is $1 sec.");
                Box::new(o)
            }
            USE_PREDEFINED_INDUCEMENTS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false).set_message_true("Inducements are predefined.");
                Box::new(o)
            }
            WIZARD_AVAILABLE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true).set_message_true("A wizard may be bought as an inducement.");
                Box::new(o)
            }
            INDUCEMENT_RIOTOUS_ROOKIES_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches my hire $1 groups of Riotous Rookies.");
                Box::new(o)
            }
            INDUCEMENT_RIOTOUS_ROOKIES_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(100_000)
                    .set_message("Groups of Riotous Rookies can be hired for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_PRAYERS_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(50_000).set_message("Prayers cost $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_PRAYERS_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0).set_message("Prayers are limited to $1.");
                Box::new(o)
            }
            INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Use Prayers from exhibition table.")
                    .set_message_true("Use Prayers from league table.");
                Box::new(o)
            }
            INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Underdog will not get Prayers during inducement phase.")
                    .set_message_true("Underdog will get Prayers during inducement phase.");
                Box::new(o)
            }
            ENABLE_STALLING_CHECK => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Stalling check is disabled")
                    .set_message_true("Stalling check is enabled");
                Box::new(o)
            }
            ALLOW_BALL_AND_CHAIN_RE_ROLL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Can't re-roll Ball & Chain movement")
                    .set_message_true("Can re-roll Ball & Chain movement");
                Box::new(o)
            }
            END_TURN_WHEN_HITTING_ANY_PLAYER_WITH_TTM => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Hitting a player with ttm is no turnover unless hitting a team-mate")
                    .set_message_true("Hitting any player with ttm is a turnover");
                Box::new(o)
            }
            SWOOP_DISTANCE => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0).set_message("Swoop players will fly exactly $1 squares.");
                Box::new(o)
            }
            ALLOW_SPECIAL_BLOCKS_WITH_BALL_AND_CHAIN => {
                let mut o = GameOptionBoolean::new(name);
                // Java: two chained setMessageFalse() calls (the second overwrites the first;
                // no setMessageTrue is ever called) — preserved verbatim from the source.
                o.set_default(false)
                    .set_message_false("Ball and Chain always performs regular blocks")
                    .set_message_false("Ball and Chain may use special block actions");
                Box::new(o)
            }
            INDUCEMENT_TEMP_CHEERLEADER_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(20_000).set_message("Temp Agency Cheerleaders cost $1 gps each");
                Box::new(o)
            }
            INDUCEMENT_TEMP_CHEERLEADER_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(4).set_message("Coaches may hire $1 Temp Agency Cheerleaders");
                Box::new(o)
            }
            INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(16).set_message(
                    "Coaches may hire Temp Agency Cheerleaders until a max of $1 cheerleaders",
                );
                Box::new(o)
            }
            INDUCEMENT_PART_TIME_COACH_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(20_000).set_message("Part-time Assistant Coaches cost $1 gps each");
                Box::new(o)
            }
            INDUCEMENT_PART_TIME_COACH_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(3).set_message("Coaches may hire $1 Part-time Assistant Coaches");
                Box::new(o)
            }
            INDUCEMENT_PART_TIME_COACH_TOTAL_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(9).set_message(
                    "Coaches may hire Part-time Assistant Coaches until a max of $1 assistant coaches",
                );
                Box::new(o)
            }
            INDUCEMENT_BIASED_REF_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches my hire $1 Biased Refs.");
                Box::new(o)
            }
            INDUCEMENT_BIASED_REF_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(120_000).set_message("Biased Refs can be hired for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_BIASED_REF_REDUCED_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1)
                    .set_message("Coaches my hire $1 Biased Refs for reduced costs.");
                Box::new(o)
            }
            INDUCEMENT_BIASED_REF_REDUCED_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(80_000)
                    .set_message("Biased Ref for reduced price can be purchased for $1 gps each.");
                Box::new(o)
            }
            INDUCEMENT_WEATHER_MAGE_MAX => {
                let mut o = GameOptionInt::new(name);
                o.set_default(1).set_message("Coaches my hire $1 Weather Mages.");
                Box::new(o)
            }
            INDUCEMENT_WEATHER_MAGE_COST => {
                let mut o = GameOptionInt::new(name);
                o.set_default(30_000).set_message("Weather Mages can be hired for $1 gps each.");
                Box::new(o)
            }
            CHAINSAW_TURNOVER => {
                let mut o = GameOptionString::new(name);
                o.set_default(CHAINSAW_TURNOVER_KICKBACK)
                    .set_message("Chainsaw causes turnover: $1")
                    .add_value_message(CHAINSAW_TURNOVER_NEVER, "Never")
                    .add_value_message(CHAINSAW_TURNOVER_KICKBACK_AV_BREAK_ONLY, "Only on kickback AV breaks")
                    .add_value_message(CHAINSAW_TURNOVER_KICKBACK, "On all kickbacks")
                    .add_value_message(CHAINSAW_TURNOVER_ALL_AV_BREAKS, "On all AV breaks");
                Box::new(o)
            }
            BOMBER_PLACED_PRONE_IGNORES_TURNOVER => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Bombardier placed prone causes turnover")
                    .set_message_true("Bombardier placed prone ignores turnover");
                Box::new(o)
            }
            SNEAKY_GIT_CAN_MOVE_AFTER_FOUL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Sneaky Git has to end action after fouling")
                    .set_message_true("Sneaky Git may move after fouling");
                Box::new(o)
            }
            BOMB_USES_MB => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Bombs do not use MB")
                    .set_message_true("Bombs use MB");
                Box::new(o)
            }
            CATCH_WORKS_FOR_BOMBS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Catch and Monstrous Mouth can not be used for bombs")
                    .set_message_true("Catch and Monstrous Mouth can be used for bombs");
                Box::new(o)
            }
            ONLY_ONE_BRIBE_PER_SEND_OFF => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_false("Multiple Bribes can be used")
                    .set_message_true("Only one Bribe can be used");
                Box::new(o)
            }
            CHAINSAW_TURNOVER_ON_AV_BREAK => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false);
                Box::new(o)
            }
            OVERTIME_GOLDEN_GOAL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Overtime ends after first touchdown")
                    .set_message_false("Overtime lasts a whole half");
                Box::new(o)
            }
            OVERTIME_KICK_OFF_RESULTS => {
                let mut o = GameOptionString::new(name);
                o.set_default(OVERTIME_KICK_OFF_ALL)
                    .set_message("Kick off events in overtime: $1")
                    .add_value_message(OVERTIME_KICK_OFF_ALL, "all events")
                    .add_value_message(OVERTIME_KICK_OFF_BLITZ, "Blitz")
                    .add_value_message(OVERTIME_KICK_OFF_SOLID_DEFENCE, "Solid Defence")
                    .add_value_message(OVERTIME_KICK_OFF_BLITZ_OR_SOLID_DEFENCE, "Choice of Blitz or Solid Defence")
                    .add_value_message(
                        OVERTIME_KICK_OFF_RANDOM_BLITZ_OR_SOLID_DEFENCE,
                        "Blitz or Solid Defence, chosen randomly",
                    );
                Box::new(o)
            }
            INDUCEMENTS_ALLOW_OVERDOG_SPENDING => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_false("Overdog can not spend treasury")
                    .set_message_true("Overdog can spend treasury");
                Box::new(o)
            }
            ENABLE_TACKLEZONE_OVERLAYS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Tacklezone overlays are enabled.")
                    .set_message_false("Tacklezone overlays are disabled.");
                Box::new(o)
            }
            MB_STACKS_AGAINST_CHAINSAW => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Mighty Blow can be used against Chainsaw players")
                    .set_message_false("Mighty Blow can not be used against Chainsaw players");
                Box::new(o)
            }
            INDUCEMENT_DUMMY => {
                let mut o = GameOptionInt::new(name);
                o.set_default(0);
                Box::new(o)
            }
            BOMB_BOUNCES_ON_EMPTY_SQUARES => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_true("Bombs bounce on empty squares")
                    .set_message_false("Bombs explode on empty squares");
                Box::new(o)
            }
            ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_true("Animal Savagery lash out ends activation")
                    .set_message_false("Animal Savagery lash out does not end activation");
                Box::new(o)
            }
            ALLOW_SPECIAL_ACTIONS_FROM_PRONE => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Special actions can be declared when prone")
                    .set_message_false("Special actions can only be declared when standing");
                Box::new(o)
            }
            ALLOW_BRAWLER_ON_BOTH_BLOCKS => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Brawler can be used twice on Frenzy or Multi Block")
                    .set_message_false("Brawler can only be used once per activation");
                Box::new(o)
            }
            ASK_FOR_KICK_AFTER_ROLL => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(false)
                    .set_message_true("Kick use is decided after the roll")
                    .set_message_false("Kick use has to be decided before the roll");
                Box::new(o)
            }
            BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER => {
                let mut o = GameOptionBoolean::new(name);
                o.set_default(true)
                    .set_message_true("Bombs knocking down team-mates cause turnovers")
                    .set_message_false("Bombs knocking down team-mates do not cause turnovers");
                Box::new(o)
            }
        };
        Some(opt)
    }
}

impl Default for GameOptionFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_boolean_option_has_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::ALLOW_CONCESSIONS).unwrap();
        assert_eq!(opt.get_value_as_string(), "true");
        assert_eq!(opt.get_id(), "allowConcessions");
    }

    #[test]
    fn create_int_option_has_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::TURNTIME).unwrap();
        assert_eq!(opt.get_value_as_string(), "240");
    }

    #[test]
    fn create_string_option_with_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::RULESVERSION).unwrap();
        assert_eq!(opt.get_value_as_string(), "BB2016");
    }

    #[test]
    fn create_string_option_without_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::PITCH_URL).unwrap();
        assert_eq!(opt.get_value_as_string(), "");
    }

    #[test]
    fn create_max_players_on_field_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::MAX_PLAYERS_ON_FIELD).unwrap();
        assert_eq!(opt.get_value_as_string(), "11");
    }

    #[test]
    fn every_variant_produces_an_option() {
        let factory = GameOptionFactory::new();
        for id in GameOptionId::values() {
            let opt = factory.create_game_option(*id);
            assert!(opt.is_some(), "expected Some for {:?}", id);
            assert_eq!(opt.unwrap().get_id(), id.get_name());
        }
    }

    #[test]
    fn chainsaw_turnover_uses_kickback_default() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::CHAINSAW_TURNOVER).unwrap();
        assert_eq!(opt.get_value_as_string(), CHAINSAW_TURNOVER_KICKBACK);
    }

    #[test]
    fn boolean_option_default_false() {
        let factory = GameOptionFactory::new();
        let opt = factory.create_game_option(GameOptionId::TEST_MODE).unwrap();
        assert_eq!(opt.get_value_as_string(), "false");
    }
}
