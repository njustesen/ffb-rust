/// 1:1 translation of com.fumbbl.ffb.model.property.NamedProperties.
///
/// All constants are `&'static str` using the Java field name (camelCase) as
/// the string value.  Callers use these with `Player::has_skill_property`.
pub struct NamedProperties;

impl NamedProperties {
    pub const ADD_BONUS_FOR_ACCURATE_PASS: &'static str = "addBonusForAccuratePass";
    pub const ADD_STRENGTH_ON_BLITZ: &'static str = "addStrengthOnBlitz";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_BLOCK: &'static str = "affectsEitherArmourOrInjuryOnBlock";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_DODGE: &'static str = "affectsEitherArmourOrInjuryOnDodge";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_FOUL: &'static str = "affectsEitherArmourOrInjuryOnFoul";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_JUMP: &'static str = "affectsEitherArmourOrInjuryOnJump";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_TTM: &'static str = "affectsEitherArmourOrInjuryOnTtm";
    pub const AFFECTS_EITHER_ARMOUR_OR_INJURY_WITH_PARTNER: &'static str = "affectsEitherArmourOrInjuryWithPartner";
    pub const ALLOWS_ADDITIONAL_FOUL: &'static str = "allowsAdditionalFoul";
    pub const ALLOWS_RAISING_LINEMAN: &'static str = "allowsRaisingLineman";
    pub const ALLOW_STAND_UP_ASSISTS: &'static str = "allowStandUpAssists";
    pub const APPLIES_CONFUSION: &'static str = "appliesConfusion";
    pub const APPLIES_POISON_ON_BADLY_HURT: &'static str = "appliesPoisonOnBadlyHurt";
    pub const ASSISTS_BLOCKS_IN_TACKLEZONES: &'static str = "assistsBlocksInTacklezones";
    pub const ASSISTS_FOULS_IN_TACKLEZONES: &'static str = "assistsFoulsInTacklezones";
    pub const BECOMES_IMMOVABLE: &'static str = "becomesImmovable";
    pub const BLOCKS_DURING_MOVE: &'static str = "blocksDuringMove";
    pub const BLOCKS_LIKE_CHAINSAW: &'static str = "blocksLikeChainsaw";
    pub const CAN_ADD_BLOCK_DIE: &'static str = "canAddBlockDie";
    pub const CAN_ADD_STRENGTH_TO_DODGE: &'static str = "canAddStrengthToDodge";
    pub const CAN_ADD_STRENGTH_TO_PASS: &'static str = "canAddStrengthToPass";
    pub const CAN_ALWAYS_ASSIST_FOULS: &'static str = "canAlwaysAssistFouls";
    pub const CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH: &'static str = "canAttackOpponentForBallAfterCatch";
    pub const CAN_ATTEMPT_CATCH_IN_ADJACENT_SQUARES: &'static str = "canAttemptCatchInAdjacentSquares";
    pub const CAN_ATTEMPT_TO_TACKLE_DODGING_PLAYER: &'static str = "canAttemptToTackleDodgingPlayer";
    pub const CAN_ATTEMPT_TO_TACKLE_JUMPING_PLAYER: &'static str = "canAttemptToTackleJumpingPlayer";
    pub const CAN_AVOID_DODGING: &'static str = "canAvoidDodging";
    pub const CAN_AVOID_FALLING_DOWN: &'static str = "canAvoidFallingDown";
    pub const CAN_BE_GAINED_BY_GETTING_EVEN: &'static str = "canBeGainedByGettingEven";
    pub const CAN_BE_KICKED: &'static str = "canBeKicked";
    pub const CAN_BE_THROWN: &'static str = "canBeThrown";
    pub const CAN_BE_THROWN_IF_STRENGTH_IS_3_OR_LESS: &'static str = "canBeThrownIfStrengthIs3orLess";
    pub const CAN_BITE_OPPONENTS: &'static str = "canBiteOpponents";
    pub const CAN_BLAST_REMOTE_PLAYER: &'static str = "canBlastRemotePlayer";
    pub const CAN_BLOCK_MORE_THAN_ONCE: &'static str = "canBlockMoreThanOnce";
    pub const CAN_BLOCK_TWO_AT_ONCE: &'static str = "canBlockTwoAtOnce";
    pub const CAN_BLOCK_OVER_DISTANCE: &'static str = "canBlockOverDistance";
    pub const CAN_BLOCK_SAME_TEAM_PLAYER: &'static str = "canBlockSameTeamPlayer";
    pub const CAN_CANCEL_INTERCEPTIONS: &'static str = "canCancelInterceptions";
    pub const CAN_CHOOSE_TO_IGNORE_DODGE_MODIFIER_AFTER_ROLL: &'static str = "canChooseToIgnoreDodgeModifierAfterRoll";
    pub const CAN_CHOOSE_TO_IGNORE_JUMP_MODIFIER_AFTER_ROLL: &'static str = "canChooseToIgnoreJumpModifierAfterRoll";
    pub const CAN_CHOOSE_TO_IGNORE_RUSH_MODIFIER_AFTER_ROLL: &'static str = "canChooseToIgnoreRushModifierAfterRoll";
    pub const CAN_CHOOSE_OWN_PUSHED_BACK_SQUARE: &'static str = "canChooseOwnPushedBackSquare";
    pub const CAN_CONVERT_BOTH_DOWN_TO_PUSH: &'static str = "canConvertBothDownToPush";
    pub const CAN_DOUBLE_STRENGTH_AFTER_DAUNTLESS: &'static str = "canDoubleStrengthAfterDauntless";
    pub const CAN_DROP_BALL: &'static str = "canDropBall";
    pub const CAN_FOLLOW_PLAYER_LEAVING_TACKLEZONES: &'static str = "canFollowPlayerLeavingTacklezones";
    pub const CAN_FORCE_BOMB_EXPLOSION: &'static str = "canForceBombExplosion";
    pub const CAN_FORCE_INTERCEPTION_REROLL_OF_LONG_PASSES: &'static str = "canForceInterceptionRerollOfLongPasses";
    pub const CAN_FOUL_AFTER_BLOCK: &'static str = "canFoulAfterBlock";
    pub const CAN_GAIN_CLAWS_FOR_BLITZ: &'static str = "canGainClawsForBlitz";
    pub const CAN_GAIN_FRENZY_FOR_BLITZ: &'static str = "canGainFrenzyForBlitz";
    pub const CAN_GAIN_GAZE: &'static str = "canGainGaze";
    pub const CAN_GAIN_HAIL_MARY: &'static str = "canGainHailMary";
    pub const CAN_GAZE_AUTOMATICALLY: &'static str = "canGazeAutomatically";
    pub const CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY: &'static str = "canGazeAutomaticallyThreeSquaresAway";
    pub const CAN_GAZE_DURING_MOVE: &'static str = "canGazeDuringMove";
    pub const CAN_GET_BALL_ON_GROUND: &'static str = "canGetBallOnGround";
    pub const CAN_GRANT_RE_ROLL_AFTER_TOUCHDOWN: &'static str = "canGrantReRollAfterTouchdown";
    pub const CAN_GRANT_SKILLS_TO_TEAM_MATES: &'static str = "canGrantSkillsToTeamMates";
    pub const CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES: &'static str = "canHoldPlayersLeavingTacklezones";
    pub const CAN_INTERCEPT_EASILY: &'static str = "canInterceptEasily";
    pub const CAN_JOIN_TEAM_IF_LESS_THAN_ELEVEN: &'static str = "canJoinTeamIfLessThanEleven";
    pub const CAN_KICK_TEAM_MATES: &'static str = "canKickTeamMates";
    pub const CAN_MOVE_BEFORE_BEING_BLOCKED: &'static str = "canMoveBeforeBeingBlocked";
    pub const CAN_THROW_KEG: &'static str = "canThrowKeg";
    pub const CAN_LASH_OUT_AGAINST_OPPONENTS: &'static str = "canLashOutAgainstOpponents";
    pub const CAN_LEAP: &'static str = "canLeap";
    pub const CAN_MAKE_AN_EXTRA_GFI: &'static str = "canMakeAnExtraGfi";
    pub const CAN_MAKE_AN_EXTRA_GFI_ONCE: &'static str = "canMakeAnExtraGfiOnce";
    pub const CAN_MAKE_OPPONENT_MISS_TURN: &'static str = "canMakeOpponentMissTurn";
    pub const CAN_IGNORE_JUMP_MODIFIERS: &'static str = "canIgnoreJumpModifiers";
    pub const CAN_PIN_PLAYERS: &'static str = "canPinPlayers";
    pub const CAN_MOVE_AFTER_BLOCK: &'static str = "canMoveAfterBlock";
    pub const CAN_MOVE_AFTER_FOUL: &'static str = "canMoveAfterFoul";
    pub const CAN_MOVE_AFTER_QUICK_PASS: &'static str = "canMoveAfterQuickPass";
    pub const CAN_MOVE_AFTER_HAND_OFF: &'static str = "canMoveAfterHandOff";
    pub const CAN_MOVE_DURING_KICK_OFF_SCATTER: &'static str = "canMoveDuringKickOffScatter";
    pub const CAN_MOVE_OPEN_TEAM_MATE: &'static str = "canMoveOpenTeamMate";
    pub const CAN_MOVE_WHEN_OPPONENT_PASSES: &'static str = "canMoveWhenOpponentPasses";
    pub const CAN_PASS_TO_ANY_SQUARE: &'static str = "canPassToAnySquare";
    pub const CAN_PASS_TO_PARTNER_WITH_NO_MODIFIERS: &'static str = "canPassToPartnerWithNoModifiers";
    pub const CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK: &'static str = "canPerformArmourRollInsteadOfBlock";
    pub const CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL: &'static str = "canPerformArmourRollInsteadOfBlockThatMightFail";
    pub const CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER: &'static str = "canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover";
    pub const CAN_PERFORM_SECOND_CHAINSAW_ATTACK: &'static str = "canPerformSecondChainsawAttack";
    pub const CAN_PERFORM_TWO_BLOCKS_AFTER_FAILED_FURY: &'static str = "canPerformTwoBlocksAfterFailedFury";
    pub const CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE: &'static str = "canPlaceBallWhenKnockedDownOrPlacedProne";
    pub const CAN_PUSH_BACK_TO_ANY_SQUARE: &'static str = "canPushBackToAnySquare";
    pub const CAN_PUNT: &'static str = "canPunt";
    pub const CAN_PILE_ON_OPPONENT: &'static str = "canPileOnOpponent";
    pub const CAN_REDUCE_KICK_DISTANCE: &'static str = "canReduceKickDistance";
    pub const CAN_REFUSE_TO_BE_PUSHED: &'static str = "canRefuseToBePushed";
    pub const CAN_REMOVE_OPPONENT_ASSISTS: &'static str = "canRemoveOpponentAssists";
    pub const CAN_RE_ROLL_ANY_NUMBER_OF_BLOCK_DICE: &'static str = "canReRollAnyNumberOfBlockDice";
    pub const CAN_REROLL_SINGLE_BOTH_DOWN: &'static str = "canRerollSingleBothDown";
    pub const CAN_REROLL_DODGE: &'static str = "canRerollDodge";
    pub const CAN_RE_ROLL_HMP_SCATTER: &'static str = "canReRollHmpScatter";
    pub const CAN_RE_ROLL_ONES_ON_KO_RECOVERY: &'static str = "canReRollOnesOnKORecovery";
    pub const CAN_REROLL_ONCE_PER_TURN: &'static str = "canRerollOncePerTurn";
    pub const CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD: &'static str = "canRerollSingleDieOncePerPeriod";
    pub const CAN_REROLL_SINGLE_BLOCK_DIE_ONCE_PER_PERIOD: &'static str = "canRerollSingleBlockDieOncePerPeriod";
    pub const CAN_REROLL_SINGLE_BLOCK_DIE_DURING_BLITZ: &'static str = "canRerollSingleBlockDieDuringBlitz";
    pub const CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_PARTNER_IS_MARKING: &'static str = "canRerollSingleBlockDieWhenPartnerIsMarking";
    pub const CAN_REROLL_SINGLE_BLOCK_DIE_WHEN_WOULD_BE_KNOCKED_DOWN: &'static str = "canRerollSingleBlockDieWhenWouldBeKnockedDown";
    pub const CAN_REROLL_SINGLE_SKULL: &'static str = "canRerollSingleSkull";
    pub const CAN_ROLL_TO_MATCH_OPPONENTS_STRENGTH: &'static str = "canRollToMatchOpponentsStrength";
    pub const CAN_ROLL_TO_SAVE_FROM_INJURY: &'static str = "canRollToSaveFromInjury";
    pub const CAN_SABOTAGE_BLOCKER_ON_KNOCKDOWN: &'static str = "canSabotageBlockerOnKnockdown";
    pub const CAN_SAVE_RE_ROLLS: &'static str = "canSaveReRolls";
    pub const CAN_SKIP_TTM_SCATTER_ON_SUPERB_THROW: &'static str = "canSkipTtmScatterOnSuperbThrow";
    pub const CAN_SNEAK_EXTRA_PLAYERS_ONTO_PITCH: &'static str = "canSneakExtraPlayersOntoPitch";
    pub const CAN_STAB_AND_MOVE_AFTERWARDS: &'static str = "canStabAndMoveAfterwards";
    pub const CAN_STAB_TEAM_MATE_FOR_BALL: &'static str = "canStabTeamMateForBall";
    pub const CAN_STAND_UP_FOR_FREE: &'static str = "canStandUpForFree";
    pub const CAN_STAND_UP_TEAM_MATES: &'static str = "canStandUpTeamMates";
    pub const CAN_STEAL_BALL_FROM_OPPONENT: &'static str = "canStealBallFromOpponent";
    pub const CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN: &'static str = "canTakeDownPlayersWithHimOnBothDown";
    pub const CAN_TELEPORT_BEFORE_AND_AFTER_AV_ROLL_ATTACK: &'static str = "canTeleportBeforeAndAfterAvRollAttack";
    pub const CAN_THROW_TEAM_MATES: &'static str = "canThrowTeamMates";
    pub const CAN_USE_CHAINSAW_ON_DOWNED_OPPONENTS: &'static str = "canUseChainsawOnDownedOpponents";
    pub const CAN_USE_THROW_BOMB_ACTION_TWICE: &'static str = "canUseThrowBombActionTwice";
    pub const CAN_USE_VOMIT_AFTER_BLOCK: &'static str = "canUseVomitAfterBlock";
    pub const CONVERT_KO_TO_STUN_ON_8: &'static str = "convertKOToStunOn8";
    pub const CONVERT_STUN_TO_KO: &'static str = "convertStunToKO";
    pub const DONT_DROP_FUMBLES: &'static str = "dontDropFumbles";
    pub const DROPPED_BALL_CAUSES_ARMOUR_ROLL: &'static str = "droppedBallCausesArmourRoll";
    pub const ENABLE_STAND_UP_AND_END_BLITZ_ACTION: &'static str = "enableStandUpAndEndBlitzAction";
    pub const ENABLE_THROW_BOMB_ACTION: &'static str = "enableThrowBombAction";
    pub const FLIP_SAME_TEAM_OPPONENT_TO_OTHER_TEAM: &'static str = "flipSameTeamOpponentToOtherTeam";
    pub const FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK: &'static str = "forceOpponentToDropBallOnPushback";
    pub const FORCE_OPPONENT_TO_FOLLOW_UP: &'static str = "forceOpponentToFollowUp";
    pub const FORCE_FOLLOWUP: &'static str = "forceFollowup";
    pub const FORCE_FULL_MOVEMENT: &'static str = "forceFullMovement";
    pub const FORCE_ROLL_BEFORE_BEING_BLOCKED: &'static str = "forceRollBeforeBeingBlocked";
    pub const FORCE_SECOND_BLOCK: &'static str = "forceSecondBlock";
    pub const FOUL_BREAKS_ARMOUR_WITHOUT_ROLL: &'static str = "foulBreaksArmourWithoutRoll";
    pub const FUMBLED_PLAYER_LANDS_SAFELY: &'static str = "fumbledPlayerLandsSafely";
    pub const GETS_SENT_OFF_AT_END_OF_DRIVE: &'static str = "getsSentOffAtEndOfDrive";
    pub const GO_FOR_IT_AFTER_BLOCK: &'static str = "goForItAfterBlock";
    pub const GRAB_OUTSIDE_BLOCK: &'static str = "grabOutsideBlock";
    pub const GRANTS_CATCH_BONUS_TO_RECEIVER: &'static str = "grantsCatchBonusToReceiver";
    pub const GRANTS_SPP_WHEN_HITTING_OPPONENT_ON_TTM: &'static str = "grantsSppWhenHittingOpponentOnTtm";
    pub const GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS: &'static str = "grantsSppFromSpecialActionsCas";
    pub const GRANTS_TEAM_RE_ROLL_WHEN_CAUSING_BLOCK_CAS: &'static str = "grantsTeamReRollWhenCausingBlockCas";
    pub const GRANTS_TEAM_RE_ROLL_WHEN_CAUSING_CAS: &'static str = "grantsTeamReRollWhenCausingCas";
    pub const GRANTS_TEAM_RE_ROLL_WHEN_ON_PITCH: &'static str = "grantsTeamReRollWhenOnPitch";
    pub const GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH: &'static str = "grantsSingleUseTeamRerollWhenOnPitch";
    pub const IGNORES_DEFENDER_STUMBLES_RESULT_FOR_FIRST_BLOCK: &'static str = "ignoresDefenderStumblesResultForFirstBlock";
    pub const HAS_TO_MISS_TURN: &'static str = "hasToMissTurn";
    pub const HAS_NO_TACKLEZONE_FOR_DODGING: &'static str = "hasNoTacklezoneForDodging";
    pub const HAS_TO_ROLL_TO_PASS_BALL_ON: &'static str = "hasToRollToPassBallOn";
    pub const HAS_TO_ROLL_TO_USE_TEAM_REROLL: &'static str = "hasToRollToUseTeamReroll";
    pub const IGNORES_ARMOUR_MODIFIERS_FROM_FOULS: &'static str = "ignoresArmourModifiersFromFouls";
    pub const IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS: &'static str = "ignoresArmourModifiersFromSkills";
    pub const IGNORES_ARMOUR_MODIFIERS_FROM_SPECIAL_EFFECTS: &'static str = "ignoresArmourModifiersFromSpecialEffects";
    pub const IGNORE_BLOCK_ASSISTS: &'static str = "ignoreBlockAssists";
    pub const IGNORE_DEFENDER_STUMBLES_RESULT: &'static str = "ignoreDefenderStumblesResult";
    pub const IGNORE_FIRST_ARMOUR_BREAK: &'static str = "ignoreFirstArmourBreak";
    pub const IGNORE_FIRST_SECRET_WEAPON_SENT_OFF: &'static str = "ignoreFirstSecretWeaponSentOff";
    pub const IGNORE_TACKLE_WHEN_BLOCKED: &'static str = "ignoreTackleWhenBlocked";
    pub const IGNORE_TACKLEZONES_WHEN_CATCHING: &'static str = "ignoreTacklezonesWhenCatching";
    pub const IGNORE_TACKLEZONES_WHEN_DODGING: &'static str = "ignoreTacklezonesWhenDodging";
    pub const IGNORE_TACKLEZONES_WHEN_JUMPING: &'static str = "ignoreTacklezonesWhenJumping";
    pub const IGNORE_TACKLEZONES_WHEN_MOVING: &'static str = "ignoreTacklezonesWhenMoving";
    pub const IGNORE_TACKLEZONES_WHEN_PASSING: &'static str = "ignoreTacklezonesWhenPassing";
    pub const IGNORE_TACKLEZONES_WHEN_PICKING_UP: &'static str = "ignoreTacklezonesWhenPickingUp";
    pub const IGNORE_WEATHER_WHEN_PICKING_UP: &'static str = "ignoreWeatherWhenPickingUp";
    pub const INCREASES_TEAMS_FAME: &'static str = "increasesTeamsFame";
    pub const INFLICTS_CONFUSION: &'static str = "inflictsConfusion";
    pub const INFLICTS_DISTURBING_PRESENCE: &'static str = "inflictsDisturbingPresence";
    pub const IS_HURT_MORE_EASILY: &'static str = "isHurtMoreEasily";
    pub const MAKES_DODGING_HARDER: &'static str = "makesDodgingHarder";
    pub const MAKES_JUMPING_HARDER: &'static str = "makesJumpingHarder";
    pub const MAKES_STRENGTH_TEST_OBSOLETE: &'static str = "makesStrengthTestObsolete";
    pub const MIGHT_EAT_PLAYER_TO_THROW: &'static str = "mightEatPlayerToThrow";
    pub const MOVES_RANDOMLY: &'static str = "movesRandomly";
    pub const NEEDS_NO_DICE_DECORATIONS: &'static str = "needsNoDiceDecorations";
    pub const NEEDS_TO_BE_SET_UP: &'static str = "needsToBeSetUp";
    pub const NEEDS_TO_ROLL_FOR_ACTION_BUT_KEEPS_TACKLEZONE: &'static str = "needsToRollForActionButKeepsTacklezone";
    pub const NEEDS_TO_ROLL_FOR_ACTION_BLOCKING_IS_EASIER: &'static str = "needsToRollForActionBlockingIsEasier";
    pub const NEEDS_TO_ROLL_HIGH_TO_AVOID_CONFUSION: &'static str = "needsToRollHighToAvoidConfusion";
    pub const FAILED_RUSH_FOR_JUMP_ALWAYS_LANDS_IN_TARGET_SQUARE: &'static str = "failedRushForJumpAlwaysLandsInTargetSquare";
    pub const PASSES_ARE_INTERCEPTED_EASIER: &'static str = "passesAreInterceptedEasier";
    pub const PASSES_ARE_NOT_INTERCEPTED: &'static str = "passesAreNotIntercepted";
    pub const PLACED_PRONE_CAUSES_INJURY_ROLL: &'static str = "placedProneCausesInjuryRoll";
    pub const PREVENT_ARMOUR_MODIFICATIONS: &'static str = "preventArmourModifications";
    pub const PREVENT_AUTO_MOVE: &'static str = "preventAutoMove";
    pub const PREVENT_BEING_FOULED: &'static str = "preventBeingFouled";
    pub const PREVENT_CARD_RABBITS_FOOT: &'static str = "preventCardRabbitsFoot";
    pub const PREVENT_CATCH: &'static str = "preventCatch";
    pub const PREVENT_DAMAGING_INJURY_MODIFICATIONS: &'static str = "preventDamagingInjuryModifications";
    pub const PREVENT_FALL_ON_BOTH_DOWN: &'static str = "preventFallOnBothDown";
    pub const PREVENT_HOLD_BALL: &'static str = "preventHoldBall";
    pub const PREVENT_KICK_TEAM_MATE_ACTION: &'static str = "preventKickTeamMateAction";
    pub const PREVENT_OPPONENT_FOLLOWING_UP: &'static str = "preventOpponentFollowingUp";
    pub const PREVENT_PICKUP: &'static str = "preventPickup";
    pub const PREVENT_RAISE_FROM_DEAD: &'static str = "preventRaiseFromDead";
    pub const PREVENT_RECOVER_FROM_CONFUSION_ACTION: &'static str = "preventRecoverFromConcusionAction";
    pub const PREVENT_RECOVER_FROM_GAZE_ACTION: &'static str = "preventRecoverFromGazeAction";
    pub const PREVENT_REGULAR_BLITZ_ACTION: &'static str = "preventRegularBlitzAction";
    pub const PREVENT_REGULAR_BLOCK_ACTION: &'static str = "preventRegularBlockAction";
    pub const PREVENT_REGULAR_FOUL_ACTION: &'static str = "preventRegularFoulAction";
    pub const PREVENT_REGULAR_HAND_OVER_ACTION: &'static str = "preventRegularHandOverAction";
    pub const PREVENT_REGULAR_PASS_ACTION: &'static str = "preventRegularPassAction";
    pub const PREVENT_PUNT_ACTION: &'static str = "preventPuntAction";
    pub const PREVENT_SECURE_THE_BALL_ACTION: &'static str = "preventSecureTheBallAction";
    pub const PREVENT_STAND_UP_ACTION: &'static str = "preventStandUpAction";
    pub const PREVENT_STUNTY_DODGE_MODIFIER: &'static str = "preventStuntyDodgeModifier";
    pub const PREVENT_THROW_TEAM_MATE_ACTION: &'static str = "preventThrowTeamMateAction";
    pub const PROVIDES_BLOCK_ALTERNATIVE: &'static str = "providesBlockAlternative";
    pub const PROVIDES_BLOCK_ALTERNATIVE_DURING_BLITZ: &'static str = "providesBlockAlternativeDuringBlitz";
    pub const PROVIDES_FOULING_ALTERNATIVE: &'static str = "providesFoulingAlternative";
    pub const PROVIDES_CHAINSAW_BLOCK_ALTERNATIVE: &'static str = "providesChainsawBlockAlternative";
    pub const PROVIDES_CHAINSAW_FOULING_ALTERNATIVE: &'static str = "providesChainsawFoulingAlternative";
    pub const PROVIDES_MULTIPLE_BLOCK_ALTERNATIVE: &'static str = "providesMultipleBlockAlternative";
    pub const PROVIDES_STAB_BLOCK_ALTERNATIVE: &'static str = "providesStabBlockAlternative";
    pub const REDUCES_ARMOUR_TO_FIXED_VALUE: &'static str = "reducesArmourToFixedValue";
    pub const REDUCES_LONER_ROLL_IF_PARTNER_IS_HURT: &'static str = "reducesLonerRollIfPartnerIsHurt";
    pub const REQUIRES_SECOND_CASUALTY_ROLL: &'static str = "requiresSecondCasualtyRoll";
    pub const SMALL_ICON: &'static str = "smallIcon";
    pub const SET_GFI_ROLL_TO_FIVE: &'static str = "setGfiRollToFive";
    pub const TTM_SCATTERS_IN_SINGLE_DIRECTION: &'static str = "ttmScattersInSingleDirection";
    pub const WEAKEN_OPPOSING_BLITZER: &'static str = "weakenOpposingBlitzer";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_avoid_falling_down_is_camel_case() {
        assert_eq!(NamedProperties::CAN_AVOID_FALLING_DOWN, "canAvoidFallingDown");
    }

    #[test]
    fn becomes_immovable_is_camel_case() {
        assert_eq!(NamedProperties::BECOMES_IMMOVABLE, "becomesImmovable");
    }

    #[test]
    fn can_make_an_extra_gfi_is_camel_case() {
        assert_eq!(NamedProperties::CAN_MAKE_AN_EXTRA_GFI, "canMakeAnExtraGfi");
    }

    #[test]
    fn inflicts_confusion_matches_java_field_name() {
        assert_eq!(NamedProperties::INFLICTS_CONFUSION, "inflictsConfusion");
    }

    #[test]
    fn can_follow_player_leaving_tacklezones_matches() {
        assert_eq!(NamedProperties::CAN_FOLLOW_PLAYER_LEAVING_TACKLEZONES, "canFollowPlayerLeavingTacklezones");
    }

    #[test]
    fn all_constants_are_non_empty() {
        let all = [
            NamedProperties::ADD_BONUS_FOR_ACCURATE_PASS,
            NamedProperties::CAN_LEAP,
            NamedProperties::MAKES_DODGING_HARDER,
            NamedProperties::PREVENT_FALL_ON_BOTH_DOWN,
            NamedProperties::GO_FOR_IT_AFTER_BLOCK,
        ];
        for c in all {
            assert!(!c.is_empty(), "constant should not be empty: {}", c);
        }
    }
}
