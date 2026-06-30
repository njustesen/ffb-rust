/// 1:1 translation of com.fumbbl.ffb.FactoryType (annotation + inner enums).
/// In Java this is an annotation — in Rust we just expose the inner enums.

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactoryContext {
    APPLICATION,
    GAME,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Factory {
    NET_COMMAND_ID,
    CLIENT_MODE,
    GAME_OPTION_ID,
    SERVER_STATUS,
    GAME_STATUS,
    ANIMATION_TYPE,
    APOTHECARY_MODE,
    APOTHECARY_STATUS,
    ARMOUR_MODIFIER,
    BLOCK_RESULT,
    CARD,
    CARD_EFFECT,
    CARD_HANDLER,
    CARD_TYPE,
    CASUALTY_MODIFIER,
    CATCH_MODIFIER,
    CATCH_SCATTER_THROWIN_MODE,
    CLIENT_STATE_ID,
    CONCEDE_GAME_STATUS,
    DEFERRED_COMMAND,
    DEFERRED_COMMAND_ID,
    DIALOG_ID,
    DIRECTION,
    DODGE_MODIFIER,
    GAZE_MODIFIER,
    GO_FOR_IT_MODIFIER,
    INDUCEMENT_PHASE,
    INDUCEMENT_TYPE,
    INJURY_MODIFIER,
    INJURY_TYPE,
    INJURY_TYPE_SERVER,
    INTERCEPTION_MODIFIER,
    KICKOFF_RESULT,
    LEADER_STATE,
    JUMP_MODIFIER,
    JUMP_UP_MODIFIER,
    LOGIC_PLUGIN,
    MECHANIC,
    MODEL_CHANGE_DATA_TYPE,
    MODEL_CHANGE_ID,
    PASSING_DISTANCE,
    PASS_MODIFIER,
    PASS_RESULT,
    PICKUP_MODIFIER,
    PLAYER_ACTION,
    PLAYER_CHOICE_MODE,
    PLAYER_GENDER,
    PLAYER_TYPE,
    PUSHBACK_MODE,
    PRAYER,
    PRAYER_HANDLER,
    REPORT,
    REPORT_ID,
    RE_ROLLED_ACTION,
    RE_ROLL_SOURCE,
    RE_ROLL_PROPERTY,
    RIGHT_STUFF_MODIFIER,
    OBSERVERS,
    SEND_TO_BOX_REASON,
    SEQUENCE_GENERATOR,
    SERIOUS_INJURY,
    SKILL,
    SKILL_CATEGORY,
    SKILL_PROPERTIES,
    SKILL_USE,
    SOUND_ID,
    SPECIAL_EFFECT,
    STEP_ACTION,
    STEP_ID,
    TEAM_STATUS,
    TEMPORARY_STAT_MODIFIER,
    TURN_MODE,
    WEATHER,
}

impl Factory {
    pub fn get_context(self) -> FactoryContext {
        match self {
            Factory::NET_COMMAND_ID
            | Factory::CLIENT_MODE
            | Factory::GAME_OPTION_ID
            | Factory::SERVER_STATUS
            | Factory::GAME_STATUS => FactoryContext::APPLICATION,
            _ => FactoryContext::GAME,
        }
    }
}
