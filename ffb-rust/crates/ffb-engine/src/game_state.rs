use crate::dice_roller::DiceRoller;
use crate::game_log::GameLog;

/// Server-side game state container — 1:1 translation of Java GameState (server package).
///
/// Wraps the active Game, its step stack, dice roller, game log, and transient
/// replay/change tracking. Distinct from `step::GameState` (the step-machine state).
pub struct GameState {
    pub game_log: GameLog,
    pub dice_roller: DiceRoller,
    status: Option<String>,
    zapped_player_ids: std::collections::HashSet<String>,
    kicking_swarmers: i32,
    turn_time_started: i64,
    spectator_cooldown_time: std::collections::HashMap<String, i64>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            game_log: GameLog::new(),
            dice_roller: DiceRoller::new(),
            status: None,
            zapped_player_ids: std::collections::HashSet::new(),
            kicking_swarmers: 0,
            turn_time_started: 0,
            spectator_cooldown_time: std::collections::HashMap::new(),
        }
    }

    pub fn get_status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = Some(status.into());
    }

    pub fn get_turn_time_started(&self) -> i64 {
        self.turn_time_started
    }

    pub fn set_turn_time_started(&mut self, ts: i64) {
        self.turn_time_started = ts;
    }

    pub fn get_spectator_cooldown_time(&self, coach: &str) -> i64 {
        self.spectator_cooldown_time.get(coach).copied().unwrap_or(0)
    }

    pub fn put_spectator_cooldown_time(&mut self, coach: impl Into<String>, timestamp: i64) {
        self.spectator_cooldown_time.insert(coach.into(), timestamp);
    }

    pub fn add_zapped_player(&mut self, player_id: impl Into<String>) {
        self.zapped_player_ids.insert(player_id.into());
    }

    pub fn remove_zapped_player(&mut self, player_id: &str) {
        self.zapped_player_ids.remove(player_id);
    }

    pub fn is_zapped(&self, player_id: &str) -> bool {
        self.zapped_player_ids.contains(player_id)
    }

    pub fn get_kicking_swarmers(&self) -> i32 {
        self.kicking_swarmers
    }

    pub fn set_kicking_swarmers(&mut self, count: i32) {
        self.kicking_swarmers = count;
    }

    pub fn generate_command_nr(&mut self) -> i32 {
        todo!("Phase ZU: IdGenerator integration")
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state_defaults() {
        let gs = GameState::new();
        assert!(gs.get_status().is_none());
        assert_eq!(gs.get_kicking_swarmers(), 0);
        assert_eq!(gs.get_turn_time_started(), 0);
    }

    #[test]
    fn test_zapped_player_tracking() {
        let mut gs = GameState::new();
        gs.add_zapped_player("player1");
        assert!(gs.is_zapped("player1"));
        gs.remove_zapped_player("player1");
        assert!(!gs.is_zapped("player1"));
    }
}
