/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerTimer.
///
/// Static utility methods for game clock management:
/// - startTurnTimer: begins tracking turn time if not already started
/// - stopTurnTimer: saves elapsed turn time, resets start marker
/// - syncTime: updates game_time and turn_time from system clock; sets timeout_possible flag
///
/// no-op: all methods are confirmed intentional no-ops — headless engine has no turn timer
/// and GameState (server-only class) is not ported. Method signatures are declared so callers
/// compile; bodies do nothing.
pub struct UtilServerTimer;

impl UtilServerTimer {
    pub fn new() -> Self { Self }

    /// Java: startTurnTimer(GameState, long currentTimeMillis).
    /// no-op: headless engine has no turn timer; GameState not ported.
    pub fn start_turn_timer(_current_time_millis: i64) {
        // no-op: if gameState.turn_time_started == 0 && game.is_turn_time_enabled()
        //   then gameState.turn_time_started = current_time_millis - game.turn_time
    }

    /// Java: stopTurnTimer(GameState, long currentTimeMillis).
    /// no-op: headless engine has no turn timer; GameState not ported.
    pub fn stop_turn_timer(_current_time_millis: i64) {
        // no-op: if gameState.turn_time_started > 0 && game.is_turn_time_enabled()
        //   then game.turn_time = current_time_millis - gameState.turn_time_started
        //   gameState.turn_time_started = 0
    }

    /// Java: syncTime(GameState, long currentTimeMillis).
    /// no-op: headless engine has no turn timer or game clock; GameState not ported.
    pub fn sync_time(_current_time_millis: i64) {
        // no-op: update game.game_time and game.turn_time from clock — GameState not ported
        // set game.timeout_possible if configured turntime exceeded
    }
}

impl Default for UtilServerTimer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_can_be_created() {
        let _ = UtilServerTimer::new();
    }

    #[test]
    fn start_turn_timer_does_not_panic() {
        UtilServerTimer::start_turn_timer(1_000_000);
    }

    #[test]
    fn stop_turn_timer_does_not_panic() {
        UtilServerTimer::stop_turn_timer(1_000_000);
    }

    #[test]
    fn sync_time_does_not_panic() {
        UtilServerTimer::sync_time(1_000_000);
    }
}
