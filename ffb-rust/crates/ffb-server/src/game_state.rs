/// Server-side game state wrapper.
///
/// Analogous to com.fumbbl.ffb.server.GameState but adapted for the Rust
/// engine API.  In Java, GameState owns the StepStack and drives step
/// execution.  In Rust, that role is played by `DriverGameState` (the engine
/// crate's game loop).  This struct wraps it and adds server-specific state:
/// the game log, command counter, and outgoing model-change list.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::Rules;
use ffb_model::events::GameEvent;
use ffb_engine::step::driver::DriverGameState;
use ffb_engine::action::Action;
use ffb_engine::legal_actions::TeamSide;

/// Server-side wrapper around `DriverGameState`.
pub struct GameState {
    /// Java: `fGame` — the live game model (accessed via driver)
    pub driver: Option<DriverGameState>,
    /// Java: GameState id (derived from Game.id)
    game_id: i64,
    /// Java: `fCommandNrGenerator`
    command_nr: i64,
    /// Java: `fGameLog` — JSON strings of each ServerCommand sent (for replay)
    pub game_log: Vec<String>,
}

impl GameState {
    /// Java: `new GameState(server)` — creates an empty slot awaiting teams.
    pub fn new(game_id: i64) -> Self {
        Self {
            driver: None,
            game_id,
            command_nr: 0,
            game_log: Vec::new(),
        }
    }

    /// Java: `getId()`
    pub fn get_id(&self) -> i64 { self.game_id }

    /// Java: `getGame()`
    pub fn get_game(&self) -> Option<&Game> {
        self.driver.as_ref().map(|d| &d.game)
    }

    /// Java: `getGame()` — mutable
    pub fn get_game_mut(&mut self) -> Option<&mut Game> {
        self.driver.as_mut().map(|d| &mut d.game)
    }

    /// Whether both teams have joined and the engine is initialized.
    pub fn is_started(&self) -> bool { self.driver.is_some() }

    /// Initialize the engine once both teams are present.
    ///
    /// Java equivalent: `GameCache.addTeamToGame()` + `UtilServerStartGame.startGame()`
    pub fn start_game(&mut self, home: Team, away: Team, rules: Rules, seed: u64) {
        let driver = DriverGameState::new(home, away, rules, seed);
        self.driver = Some(driver);
    }

    /// Java: `generateCommandNr()`
    pub fn generate_command_nr(&mut self) -> i64 {
        self.command_nr += 1;
        self.command_nr
    }

    /// Apply an action to the engine and return the resulting events.
    ///
    /// Java: `gameState.handleCommand(receivedCommand)` → step dispatch
    pub fn handle_action(&mut self, side: TeamSide, action: Action) -> Result<Vec<GameEvent>, String> {
        match &mut self.driver {
            Some(driver) => driver.apply(side, action),
            None => Err("game not started".to_string()),
        }
    }

    /// Drain events accumulated since the last call.
    ///
    /// Java: events are sent immediately inside step execution; here we
    /// accumulate them and drain after each command.
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        match &mut self.driver {
            Some(driver) => driver.take_events(),
            None => Vec::new(),
        }
    }

    /// Whether the game is finished.
    pub fn is_finished(&self) -> bool {
        self.driver.as_ref().map(|d| d.is_finished()).unwrap_or(false)
    }

    /// The current side that should act (home or away).
    pub fn active_side(&self) -> Option<TeamSide> {
        self.driver.as_ref().map(|d| d.active_side())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_not_started() {
        let gs = GameState::new(1);
        assert_eq!(gs.get_id(), 1);
        assert!(!gs.is_started());
        assert!(gs.get_game().is_none());
    }

    #[test]
    fn generate_command_nr_increments() {
        let mut gs = GameState::new(1);
        assert_eq!(gs.generate_command_nr(), 1);
        assert_eq!(gs.generate_command_nr(), 2);
    }
}
