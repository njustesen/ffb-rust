/// Server-side game state wrapper.
///
/// Analogous to com.fumbbl.ffb.server.GameState but adapted for the Rust
/// engine API.  In Java, GameState owns the StepStack and drives step
/// execution.  In Rust, that role is played by `DriverGameState` (the engine
/// crate's game loop).  This struct wraps it and adds server-specific state:
/// the game log, command counter, and outgoing model-change list.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::{GameStatus, Rules};
use ffb_model::events::GameEvent;
use ffb_engine::step::driver::DriverGameState;
use ffb_engine::action::Action;
use ffb_engine::legal_actions::TeamSide;
use ffb_engine::game_log::GameLog;

/// Server-side wrapper around `DriverGameState`.
pub struct GameState {
    /// Java: `fGame` — the live game model (accessed via driver)
    pub driver: Option<DriverGameState>,
    /// Java: GameState id (derived from Game.id)
    game_id: i64,
    /// Java: `fCommandNrGenerator`
    command_nr: i64,
    /// Java: `fGameLog` — the replayable server command log for this game.
    pub game_log: GameLog,
    /// Java: `fStatus` — the server-side `GameState`'s own lifecycle status
    /// (distinct from `Game.status`; Java's field has no explicit
    /// constructor initializer, so it defaults to `null` — modeled here as
    /// `None`).
    status: Option<GameStatus>,
}

impl GameState {
    /// Java: `new GameState(server)` — creates an empty slot awaiting teams.
    pub fn new(game_id: i64) -> Self {
        Self {
            driver: None,
            game_id,
            command_nr: 0,
            game_log: GameLog::new(),
            status: None,
        }
    }

    /// Java: `getId()`
    pub fn get_id(&self) -> i64 { self.game_id }

    /// Java: `getStatus()`.
    pub fn get_status(&self) -> Option<GameStatus> { self.status }

    /// Java: `setStatus(GameStatus)`.
    pub fn set_status(&mut self, status: GameStatus) { self.status = Some(status); }

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

    /// Java: `gameState.getStepStack().clear()`.
    pub fn clear_step_stack(&mut self) {
        if let Some(driver) = &mut self.driver {
            driver.clear_step_stack();
        }
    }

    /// Java: `((EndGame) factory.forName("EndGame")).pushSequence(new EndGame.SequenceParams(gameState, adminMode))`.
    pub fn push_end_game_sequence(&mut self, admin_mode: bool) {
        if let Some(driver) = &mut self.driver {
            driver.push_end_game_sequence(admin_mode);
        }
    }

    /// Java: `gameState.startNextStep()`.
    pub fn start_next_step(&mut self) {
        if let Some(driver) = &mut self.driver {
            driver.run_until_prompt();
        }
    }

    /// Java: `GameState.initFrom(IFactorySource emptySource, JsonValue jsonValue)` — rehydrates
    /// this `GameState` from a JSON backup blob (used by `ServerRequestLoadReplay.process()`
    /// when the backup/replay service returns a saved game).
    ///
    /// Java's real `initFrom` reconstructs `fCurrentStep`, `fStepStack`, `fGameLog`,
    /// `passState`, `blitzTurnState`, `prayerState`, and `activeEffects` from a custom
    /// `IServerJsonOption`-keyed wire format (minimal-json `JsonObject`s), on top of
    /// `Game.initFrom` for the nested game model. This crate has no `IServerJsonOption`
    /// equivalent decoder (no step-stack/game-log/pass-state/blitz-turn-state/prayer-state/
    /// active-effects JSON schema has been ported), so this instead deserializes `json`
    /// directly into the already-`Serialize`/`Deserialize`-derived `Game` struct (this
    /// crate's own serde shape, not Java's wire format) via `DriverGameState::from_game`,
    /// and resets the command counter / game log exactly like Java resets `fCurrentStep`/
    /// `fStepStack`/`fGameLog` before repopulating them. This closes the same *control-flow*
    /// shape Java's method has (deserialize -> rehydrate -> reset transient step/log state)
    /// without claiming wire-format parity with real FUMBBL backup JSON -- matching this
    /// backup JSON to Java's exact `IServerJsonOption` schema is a separate, much larger
    /// follow-up (a full step-stack/game-log/pass-state/blitz-turn-state/prayer-state/
    /// active-effects JSON decoder).
    pub fn init_from(&mut self, json: &str) -> Result<(), String> {
        let game: Game = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.driver = Some(DriverGameState::from_game(game, 0));
        self.command_nr = 0;
        self.game_log = GameLog::new();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_has_no_status() {
        let gs = GameState::new(1);
        assert_eq!(gs.get_status(), None);
    }

    #[test]
    fn set_status_updates_status() {
        let mut gs = GameState::new(1);
        gs.set_status(GameStatus::Replaying);
        assert_eq!(gs.get_status(), Some(GameStatus::Replaying));
    }

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

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn started_game_state() -> GameState {
        let mut gs = GameState::new(1);
        gs.start_game(empty_team("home"), empty_team("away"), Rules::Bb2025, 42);
        gs
    }

    #[test]
    fn clear_step_stack_and_push_end_game_sequence_drives_game_to_finished() {
        let mut gs = started_game_state();
        assert!(!gs.is_finished());
        gs.clear_step_stack();
        gs.push_end_game_sequence(true);
        gs.start_next_step();
        assert!(gs.is_finished());
    }

    #[test]
    fn bridge_methods_are_no_ops_before_game_started() {
        let mut gs = GameState::new(1);
        gs.clear_step_stack();
        gs.push_end_game_sequence(true);
        gs.start_next_step();
        assert!(!gs.is_started());
    }

    #[test]
    fn init_from_round_trips_a_serialized_game() {
        let started = started_game_state();
        let json = serde_json::to_string(started.get_game().unwrap()).unwrap();

        let mut restored = GameState::new(1);
        restored.init_from(&json).unwrap();

        assert!(restored.is_started());
        assert_eq!(restored.get_game().unwrap().team_home.id, started.get_game().unwrap().team_home.id);
    }

    #[test]
    fn init_from_resets_command_nr_and_game_log() {
        let mut gs = started_game_state();
        gs.generate_command_nr();
        gs.generate_command_nr();
        let json = serde_json::to_string(gs.get_game().unwrap()).unwrap();

        gs.init_from(&json).unwrap();

        assert_eq!(gs.generate_command_nr(), 1);
    }

    #[test]
    fn init_from_invalid_json_returns_err() {
        let mut gs = GameState::new(1);
        assert!(gs.init_from("not json").is_err());
        assert!(!gs.is_started());
    }
}
