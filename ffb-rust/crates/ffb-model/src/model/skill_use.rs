use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.SkillUse.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillUse {
    WOULD_NOT_HELP, NO_TEAM_MATE_IN_RANGE, STOP_OPPONENT, PUSH_BACK_OPPONENT,
    BRING_DOWN_OPPONENT, AVOID_PUSH, CANCEL_FEND, CANCEL_STAND_FIRM,
    STAY_AWAY_FROM_OPPONENT, CATCH_BALL, STEAL_BALL, CANCEL_STRIP_BALL,
    HALVE_KICKOFF_SCATTER, CANCEL_DODGE, CANCEL_WATCH_OUT, AVOID_FALLING,
    CANCEL_TACKLE, INCREASE_STRENGTH_BY_1, CANCEL_DIVING_CATCH, PLACE_BALL,
    RE_ROLL_SINGLE_ARMOUR_DIE, ADD_ARMOUR_MODIFIER, INCREASE_CHAINSAW_DAMAGE,
    ADD_INJURY_MODIFIER, RE_ROLL_INJURY, RE_ROLL_ARMOUR, FUMBLED_PLAYER_LANDS_SAFELY,
    GAIN_FRENZY_FOR_BLITZ, GAIN_GAZE, GAIN_HAIL_MARY, TREACHEROUS,
    RUSH_ADDITIONAL_SQUARE_ONCE, ADD_STRENGTH_TO_ROLL, GAIN_GRANTED_SKILL,
    IGNORE_SENT_OFF, MOVE_OPEN_TEAM_MATE, MOVE_SQUARE, ADD_BLOCK_DIE,
    PERFORM_SECOND_CHAINSAW_ATTACK, PERFORM_SECOND_TWO_BLOCKS, FORCE_BOMB_EXPLOSION,
    RE_ROLL_DIRECTION, GRANT_CATCH_BONUS, RE_ROLL_CATCH, LOOK_INTO_MY_EYES,
    MAKE_OPPONENT_MISS_TURN, LASH_OUT_AGAINST_OPPONENT, EASY_INTERCEPT,
    PERFORM_ADDITIONAL_ATTACK, CANCEL_WRESTLE, REMOVE_TACKLEZONE, GET_BALL_ON_GROUND,
    PASS_DODGE_WITHOUT_MODIFIERS, PASS_JUMP_WITHOUT_MODIFIERS, PASS_RUSH_WITHOUT_MODIFIERS,
    QUICK_BITE, STEADY_FOOTING, NO_TACKLEZONE, FORCE_FOLLOW_UP, EYE_GOUGED,
    DISTRACT_OPPONENT, BULLSEYE, SAVED_FUMBLE_BALL, SAVED_FUMBLE_BOMB,
    GAIN_CLAWS_FOR_BLITZ, RE_ROLL_PUNT_DIRECTION, RE_ROLL_PUNT_DISTANCE,
    AVOID_DODGING, GRANT_SKILL_TO_TEAM_MATE,
}

impl SkillUse {
    pub fn get_name(self) -> &'static str {
        match self {
            SkillUse::WOULD_NOT_HELP => "wouldNotHelp",
            SkillUse::NO_TEAM_MATE_IN_RANGE => "noTeamMateInRange",
            SkillUse::STOP_OPPONENT => "stopOpponent",
            SkillUse::PUSH_BACK_OPPONENT => "pushBackOpponent",
            SkillUse::BRING_DOWN_OPPONENT => "bringDownOppponent",
            SkillUse::AVOID_PUSH => "avoidPush",
            SkillUse::CANCEL_FEND => "cancelFend",
            SkillUse::CANCEL_STAND_FIRM => "cancelStandFirm",
            SkillUse::STAY_AWAY_FROM_OPPONENT => "stayAwayFromOpponent",
            SkillUse::CATCH_BALL => "catchBall",
            SkillUse::STEAL_BALL => "stealBall",
            SkillUse::CANCEL_STRIP_BALL => "cancelStripBall",
            SkillUse::HALVE_KICKOFF_SCATTER => "halveKickoffScatter",
            SkillUse::CANCEL_DODGE => "cancelDodge",
            SkillUse::CANCEL_WATCH_OUT => "cancelWatchOut",
            SkillUse::AVOID_FALLING => "avoidFalling",
            SkillUse::CANCEL_TACKLE => "cancelTackle",
            SkillUse::INCREASE_STRENGTH_BY_1 => "increaseStrengthBy1",
            SkillUse::CANCEL_DIVING_CATCH => "cancelDivingCatch",
            SkillUse::PLACE_BALL => "placeBall",
            SkillUse::RE_ROLL_SINGLE_ARMOUR_DIE => "reRollSingleArmourDie",
            SkillUse::ADD_ARMOUR_MODIFIER => "addArmourModifier",
            SkillUse::INCREASE_CHAINSAW_DAMAGE => "increaseChainsawDamage",
            SkillUse::ADD_INJURY_MODIFIER => "addInjuryModifier",
            SkillUse::RE_ROLL_INJURY => "reRollInjury",
            SkillUse::RE_ROLL_ARMOUR => "reRollArmour",
            SkillUse::FUMBLED_PLAYER_LANDS_SAFELY => "fumbledPlayerLandsSafely",
            SkillUse::GAIN_FRENZY_FOR_BLITZ => "gainFrenzy",
            SkillUse::GAIN_GAZE => "gainFrenzy",
            SkillUse::GAIN_HAIL_MARY => "gainHailMary",
            SkillUse::TREACHEROUS => "treacherous",
            SkillUse::RUSH_ADDITIONAL_SQUARE_ONCE => "rushAdditionalSquareOnce",
            SkillUse::ADD_STRENGTH_TO_ROLL => "addStrengthToRoll",
            SkillUse::GAIN_GRANTED_SKILL => "gainGrantedSkill",
            SkillUse::IGNORE_SENT_OFF => "ignoreSentOff",
            SkillUse::MOVE_OPEN_TEAM_MATE => "moveOpenTeamMate",
            SkillUse::MOVE_SQUARE => "moveSquare",
            SkillUse::ADD_BLOCK_DIE => "addBlockDie",
            SkillUse::PERFORM_SECOND_CHAINSAW_ATTACK => "performSecondChainsawAttack",
            SkillUse::PERFORM_SECOND_TWO_BLOCKS => "performSecondTwoBlocks",
            SkillUse::FORCE_BOMB_EXPLOSION => "forceBombExplosion",
            SkillUse::RE_ROLL_DIRECTION => "reRollDirection",
            SkillUse::GRANT_CATCH_BONUS => "grantCatchBonus",
            SkillUse::RE_ROLL_CATCH => "reRollCatch",
            SkillUse::LOOK_INTO_MY_EYES => "lookIntoMyEyes",
            SkillUse::MAKE_OPPONENT_MISS_TURN => "makeOpponentMissTurn",
            SkillUse::LASH_OUT_AGAINST_OPPONENT => "lashOutAgainstOpponent",
            SkillUse::EASY_INTERCEPT => "easyIntercept",
            SkillUse::PERFORM_ADDITIONAL_ATTACK => "performAdditionalAttack",
            SkillUse::CANCEL_WRESTLE => "cancelWrestle",
            SkillUse::REMOVE_TACKLEZONE => "removeTacklezone",
            SkillUse::GET_BALL_ON_GROUND => "getBallFromGround",
            SkillUse::PASS_DODGE_WITHOUT_MODIFIERS => "passDodgeWithoutModifiers",
            SkillUse::PASS_JUMP_WITHOUT_MODIFIERS => "passJumpWithoutModifiers",
            SkillUse::PASS_RUSH_WITHOUT_MODIFIERS => "passRushWithoutModifiers",
            SkillUse::QUICK_BITE => "quickBite",
            SkillUse::STEADY_FOOTING => "steadyFooting",
            SkillUse::NO_TACKLEZONE => "noTackleZone",
            SkillUse::FORCE_FOLLOW_UP => "forceFollowUp",
            SkillUse::EYE_GOUGED => "eyeGouged",
            SkillUse::DISTRACT_OPPONENT => "distractOpponent",
            SkillUse::BULLSEYE => "bullseye",
            SkillUse::SAVED_FUMBLE_BALL => "savedFumbleBall",
            SkillUse::SAVED_FUMBLE_BOMB => "savedFumbleBomb",
            SkillUse::GAIN_CLAWS_FOR_BLITZ => "gainClaws",
            SkillUse::RE_ROLL_PUNT_DIRECTION => "reRollPuntDirection",
            SkillUse::RE_ROLL_PUNT_DISTANCE => "reRollPuntDistance",
            SkillUse::AVOID_DODGING => "avoidDodging",
            SkillUse::GRANT_SKILL_TO_TEAM_MATE => "grantSkillToTeamMate",
        }
    }

    pub fn get_description(self) -> &'static str {
        match self {
            SkillUse::WOULD_NOT_HELP => "because it would not help",
            SkillUse::NO_TEAM_MATE_IN_RANGE => "because no team-mate is in range",
            SkillUse::STOP_OPPONENT => "to stop opponent",
            SkillUse::PUSH_BACK_OPPONENT => "to push opponent back",
            SkillUse::BRING_DOWN_OPPONENT => "to bring opponent down",
            SkillUse::AVOID_PUSH => "to avoid being pushed",
            SkillUse::CANCEL_FEND => "to cancel opponent's Fend skill",
            SkillUse::CANCEL_STAND_FIRM => "to cancel opponent's Stand Firm skill",
            SkillUse::STAY_AWAY_FROM_OPPONENT => "to stay away from opponent",
            SkillUse::CATCH_BALL => "to catch the ball",
            SkillUse::STEAL_BALL => "to steal the ball",
            SkillUse::CANCEL_STRIP_BALL => "to cancel opponent's Strip Ball skill",
            SkillUse::HALVE_KICKOFF_SCATTER => "to halve the kickoff scatter",
            SkillUse::CANCEL_DODGE => "to cancel opponent's Dodge skill",
            SkillUse::CANCEL_WATCH_OUT => "to cancel opponent's Watch Out! skill",
            SkillUse::AVOID_FALLING => "to avoid falling",
            SkillUse::CANCEL_TACKLE => "to cancel opponent's Tackle skill",
            SkillUse::INCREASE_STRENGTH_BY_1 => "to increase strength by 1",
            SkillUse::CANCEL_DIVING_CATCH => "because players from both teams hinder each other",
            SkillUse::PLACE_BALL => "to place ball in an empty adjacent square",
            SkillUse::RE_ROLL_SINGLE_ARMOUR_DIE => "to re-roll a single armour die",
            SkillUse::ADD_ARMOUR_MODIFIER => "to add +1 to the armour roll",
            SkillUse::INCREASE_CHAINSAW_DAMAGE => "to add +4 instead of +3 to armour roll",
            SkillUse::ADD_INJURY_MODIFIER => "to add +1 to injury roll",
            SkillUse::RE_ROLL_INJURY => "to re-roll the injury roll",
            SkillUse::RE_ROLL_ARMOUR => "to re-roll the armour roll",
            SkillUse::FUMBLED_PLAYER_LANDS_SAFELY => "to let the fumbled player land safely",
            SkillUse::GAIN_FRENZY_FOR_BLITZ => "to gain the Frenzy skill for this Blitz action",
            SkillUse::GAIN_GAZE => "to gain the Hypnotic Gaze skill",
            SkillUse::GAIN_HAIL_MARY => "to gain Hail Mary Pass skill",
            SkillUse::TREACHEROUS => "to steal the ball from team mate",
            SkillUse::RUSH_ADDITIONAL_SQUARE_ONCE => "to rush an additional square",
            SkillUse::ADD_STRENGTH_TO_ROLL => "to add strength to the roll",
            SkillUse::GAIN_GRANTED_SKILL => "to gain a skill for this turn",
            SkillUse::IGNORE_SENT_OFF => "to not be ejected",
            SkillUse::MOVE_OPEN_TEAM_MATE => "to move a team-mate",
            SkillUse::MOVE_SQUARE => "to move a square",
            SkillUse::ADD_BLOCK_DIE => "to add a block die",
            SkillUse::PERFORM_SECOND_CHAINSAW_ATTACK => "to perform a second chainsaw attack",
            SkillUse::PERFORM_SECOND_TWO_BLOCKS => "to perform two block actions",
            SkillUse::FORCE_BOMB_EXPLOSION => "to force the bomb to explode",
            SkillUse::RE_ROLL_DIRECTION => "to re-roll the direction roll",
            SkillUse::GRANT_CATCH_BONUS => "to grant team-mate a catch bonus",
            SkillUse::RE_ROLL_CATCH => "to re-roll the catch roll",
            SkillUse::LOOK_INTO_MY_EYES => "to steal the ball from opponent",
            SkillUse::MAKE_OPPONENT_MISS_TURN => "to make an opponent player miss a turn",
            SkillUse::LASH_OUT_AGAINST_OPPONENT => "to lash out against an opponent player instead",
            SkillUse::EASY_INTERCEPT => "to try an easy interception",
            SkillUse::PERFORM_ADDITIONAL_ATTACK => "to perform an additional attack",
            SkillUse::CANCEL_WRESTLE => "to cancel wrestle",
            SkillUse::REMOVE_TACKLEZONE => "to remove opponents tacklezone",
            SkillUse::GET_BALL_ON_GROUND => "to try getting the ball on the ground",
            SkillUse::PASS_DODGE_WITHOUT_MODIFIERS => "to pass the dodge roll ignoring modifiers",
            SkillUse::PASS_JUMP_WITHOUT_MODIFIERS => "to pass the jump roll ignoring modifiers",
            SkillUse::PASS_RUSH_WITHOUT_MODIFIERS => "to pass the rush roll ignoring modifiers",
            SkillUse::QUICK_BITE => "to try to get the ball",
            SkillUse::STEADY_FOOTING => "to keep standing",
            SkillUse::NO_TACKLEZONE => "because they have no tacklezones",
            SkillUse::FORCE_FOLLOW_UP => "to force opponent to follow up",
            SkillUse::EYE_GOUGED => "to prevent the pushed player from assisting blocks",
            SkillUse::DISTRACT_OPPONENT => "to distract opponent",
            SkillUse::BULLSEYE => "to land the thrown player in the target square",
            SkillUse::SAVED_FUMBLE_BALL => "to hold on to the ball",
            SkillUse::SAVED_FUMBLE_BOMB => "to hold on to the bomb and put out the fuse",
            SkillUse::GAIN_CLAWS_FOR_BLITZ => "to gain the Claws skill for this Blitz action",
            SkillUse::RE_ROLL_PUNT_DIRECTION => "to re-roll the punt direction",
            SkillUse::RE_ROLL_PUNT_DISTANCE => "to re-roll the punt distance",
            SkillUse::AVOID_DODGING => "to avoid dodge rolls",
            SkillUse::GRANT_SKILL_TO_TEAM_MATE => "to grant a team-mate a skill for this turn",
        }
    }
    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [Self] {
        &[
            Self::WOULD_NOT_HELP, Self::NO_TEAM_MATE_IN_RANGE, Self::STOP_OPPONENT,
            Self::PUSH_BACK_OPPONENT, Self::BRING_DOWN_OPPONENT, Self::AVOID_PUSH,
            Self::CANCEL_FEND, Self::CANCEL_STAND_FIRM, Self::STAY_AWAY_FROM_OPPONENT,
            Self::CATCH_BALL, Self::STEAL_BALL, Self::CANCEL_STRIP_BALL,
            Self::HALVE_KICKOFF_SCATTER, Self::CANCEL_DODGE, Self::CANCEL_WATCH_OUT,
            Self::AVOID_FALLING, Self::CANCEL_TACKLE, Self::INCREASE_STRENGTH_BY_1,
            Self::CANCEL_DIVING_CATCH, Self::PLACE_BALL, Self::RE_ROLL_SINGLE_ARMOUR_DIE,
            Self::ADD_ARMOUR_MODIFIER, Self::INCREASE_CHAINSAW_DAMAGE, Self::ADD_INJURY_MODIFIER,
            Self::RE_ROLL_INJURY, Self::RE_ROLL_ARMOUR, Self::FUMBLED_PLAYER_LANDS_SAFELY,
            Self::GAIN_FRENZY_FOR_BLITZ, Self::GAIN_GAZE, Self::GAIN_HAIL_MARY, Self::TREACHEROUS,
            Self::RUSH_ADDITIONAL_SQUARE_ONCE, Self::ADD_STRENGTH_TO_ROLL, Self::GAIN_GRANTED_SKILL,
            Self::IGNORE_SENT_OFF, Self::MOVE_OPEN_TEAM_MATE, Self::MOVE_SQUARE, Self::ADD_BLOCK_DIE,
            Self::PERFORM_SECOND_CHAINSAW_ATTACK, Self::PERFORM_SECOND_TWO_BLOCKS,
            Self::FORCE_BOMB_EXPLOSION, Self::RE_ROLL_DIRECTION, Self::GRANT_CATCH_BONUS,
            Self::RE_ROLL_CATCH, Self::LOOK_INTO_MY_EYES, Self::MAKE_OPPONENT_MISS_TURN,
            Self::LASH_OUT_AGAINST_OPPONENT, Self::EASY_INTERCEPT, Self::PERFORM_ADDITIONAL_ATTACK,
            Self::CANCEL_WRESTLE, Self::REMOVE_TACKLEZONE, Self::GET_BALL_ON_GROUND,
            Self::PASS_DODGE_WITHOUT_MODIFIERS, Self::PASS_JUMP_WITHOUT_MODIFIERS,
            Self::PASS_RUSH_WITHOUT_MODIFIERS, Self::QUICK_BITE, Self::STEADY_FOOTING,
            Self::NO_TACKLEZONE, Self::FORCE_FOLLOW_UP, Self::EYE_GOUGED, Self::DISTRACT_OPPONENT,
            Self::BULLSEYE, Self::SAVED_FUMBLE_BALL, Self::SAVED_FUMBLE_BOMB,
            Self::GAIN_CLAWS_FOR_BLITZ, Self::RE_ROLL_PUNT_DIRECTION, Self::RE_ROLL_PUNT_DISTANCE,
            Self::AVOID_DODGING, Self::GRANT_SKILL_TO_TEAM_MATE,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_returns_camel_case() {
        assert_eq!(SkillUse::WOULD_NOT_HELP.get_name(), "wouldNotHelp");
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(SkillUse::for_name("wouldNotHelp"), Some(SkillUse::WOULD_NOT_HELP));
        assert_eq!(SkillUse::for_name("WOULDNOTHELP"), Some(SkillUse::WOULD_NOT_HELP));
        assert_eq!(SkillUse::for_name("invalid"), None);
    }

    #[test]
    fn get_description_returns_non_empty() {
        assert!(!SkillUse::WOULD_NOT_HELP.get_description().is_empty());
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        assert_eq!(SkillUse::for_name("nonexistentSkillUse"), None);
    }

    #[test]
    fn all_variants_have_non_empty_description() {
        let variants = [
            SkillUse::STOP_OPPONENT, SkillUse::CATCH_BALL, SkillUse::STEAL_BALL,
            SkillUse::RE_ROLL_ARMOUR, SkillUse::ADD_BLOCK_DIE, SkillUse::BULLSEYE,
            SkillUse::GRANT_SKILL_TO_TEAM_MATE, SkillUse::AVOID_DODGING,
            SkillUse::GAIN_CLAWS_FOR_BLITZ, SkillUse::SAVED_FUMBLE_BOMB,
        ];
        for variant in variants {
            assert!(!variant.get_description().is_empty(),
                "{:?} has an empty description", variant);
        }
    }
}
