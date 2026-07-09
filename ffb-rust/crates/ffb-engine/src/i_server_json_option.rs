/// JSON key constants for server-side serialization — 1:1 translation of Java IServerJsonOption.
pub struct IServerJsonOption;

impl IServerJsonOption {
    pub const ACTIVE_EFFECTS: &'static str = "activeEffects";
    pub const ADD_BLOCK_DIE_HANDLED: &'static str = "addBlockDieHandled";
    pub const ADMIN_MODE: &'static str = "adminMode";
    pub const ALREADY_REPORTED: &'static str = "alreadyReported";
    pub const BLOCK_DEFENDER_ID: &'static str = "blockDefenderId";
    pub const BLITZ_TURN_STATE: &'static str = "blitzTurnState";
    pub const CATCH_SCATTER_THROW_IN_MODE: &'static str = "catchScatterThrowInMode";
    pub const CONFIRMED: &'static str = "confirmed";
    pub const COORDINATE_TO: &'static str = "coordinateTo";
    pub const CURRENT_STEP: &'static str = "currentStep";
    pub const DEFERRED_COMMAND_ID: &'static str = "deferredCommandId";
    pub const DEFERRED_COMMANDS: &'static str = "deferredCommands";
    pub const GAME: &'static str = "game";
    pub const GAME_LOG: &'static str = "gameLog";
    pub const GAME_STATUS: &'static str = "gameStatus";
    pub const PASS_STATE: &'static str = "passState";
    pub const PLAYER_IDS: &'static str = "playerIds";
    pub const PRAYER_STATE: &'static str = "prayerState";
    pub const STEP_STACK: &'static str = "stepStack";
    pub const SWARMING_PLAYER_ACTUAL: &'static str = "swarmingPlayerActual";
    pub const TURN_TIME_STARTED: &'static str = "turnTimeStarted";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_key() {
        assert_eq!(IServerJsonOption::GAME, "game");
    }

    #[test]
    fn test_step_stack_key() {
        assert_eq!(IServerJsonOption::STEP_STACK, "stepStack");
    }
}
