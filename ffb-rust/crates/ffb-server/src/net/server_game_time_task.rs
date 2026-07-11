/// 1:1 translation of com.fumbbl.ffb.server.net.ServerGameTimeTask.
///
/// Java: a `TimerTask` run every second (`FantasyFootballServer.start()`:
/// `scheduleAtFixedRate(new ServerGameTimeTask(this), 0, 1000)`) that syncs
/// each game's clock, broadcasts the current time, and re-syncs the game
/// model if the timeout-possible flag flipped.
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use ffb_engine::util::util_server_timer::UtilServerTimer;
use crate::game_cache::GameCache;
use crate::net::session_manager::SessionManager;

pub struct ServerGameTimeTask {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerGameTimeTask {
    /// Java: `ServerGameTimeTask(FantasyFootballServer server)`.
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java:
    /// ```java
    /// public void run() {
    ///     try {
    ///         long currentTimeMillis = System.currentTimeMillis();
    ///         for (GameState gameState : fServer.getGameCache().allGameStates()) {
    ///             Game game = gameState.getGame();
    ///             boolean timeoutPossible = game.isTimeoutPossible();
    ///             UtilServerTimer.syncTime(gameState, currentTimeMillis);
    ///             fServer.getCommunication().sendGameTime(gameState);
    ///             if (timeoutPossible != game.isTimeoutPossible()) {
    ///                 UtilServerGame.syncGameModel(gameState, null, null, SoundId.WHISTLE);
    ///             }
    ///         }
    ///     } catch (Exception anyException) {
    ///         getServer().getDebugLog().logWithOutGameId(anyException);
    ///         System.exit(99);
    ///     }
    /// }
    /// ```
    pub fn run(&self) {
        let now = current_time_millis();
        Self::tick(&self.game_cache, &self.session_manager, now);
    }

    /// Per-tick logic extracted out of `run()` so it's testable without a
    /// `tokio::time::interval` loop, per project convention for TimerTask ports.
    fn tick(game_cache: &Arc<Mutex<GameCache>>, session_manager: &Arc<Mutex<SessionManager>>, now: i64) {
        let game_ids = { game_cache.lock().unwrap().all_game_ids() };
        for game_id in game_ids {
            let timeout_possible_before = {
                let gc = game_cache.lock().unwrap();
                gc.get_game_state_by_id(game_id)
                    .and_then(|gs| gs.get_game())
                    .map(|g| g.timeout_possible)
            };

            // Java: `UtilServerTimer.syncTime(gameState, currentTimeMillis)`.
            // The ported `UtilServerTimer::sync_time` is an intentional no-op
            // (headless engine has no turn timer, per its own doc comment),
            // so `timeout_possible` cannot actually change here yet.
            UtilServerTimer::sync_time(now);

            // Java: `fServer.getCommunication().sendGameTime(gameState)` — broadcasts
            // `ServerCommandGameTime(game.getGameTime(), game.getTurnTime())` to every
            // session in the game. `Game` (ffb-model) has no `game_time`/`turn_time`
            // fields yet (matching `UtilServerTimer`'s "no game clock" stance), so the
            // broadcast below carries only the command id until those fields exist.
            let message = serde_json::json!({ "netCommandId": "serverGameTime" }).to_string();
            session_manager.lock().unwrap().send_all(game_id, &message);

            let timeout_possible_after = {
                let gc = game_cache.lock().unwrap();
                gc.get_game_state_by_id(game_id)
                    .and_then(|gs| gs.get_game())
                    .map(|g| g.timeout_possible)
            };

            if timeout_possible_before != timeout_possible_after {
                // Java: `UtilServerGame.syncGameModel(gameState, null, null, SoundId.WHISTLE)`.
                // `UtilServerGame::sync_game_model` is not ported (it touches
                // IStep/server/WebSocket layers per that file's header comment), and
                // this branch is unreachable today since `sync_time` above never
                // changes the flag — kept for structural fidelity with the Java
                // source and to pick up automatically once both land.
                log::debug!(
                    "game {} timeout_possible changed but UtilServerGame::sync_game_model is not yet ported",
                    game_id
                );
            }
        }
    }
}

fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

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

    #[test]
    fn construct() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let _ = ServerGameTimeTask::new(gc, sm);
    }

    #[test]
    fn run_with_no_games_does_not_panic() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let task = ServerGameTimeTask::new(gc, sm);
        task.run();
    }

    #[test]
    fn tick_broadcasts_game_time_to_all_sessions_in_game() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));

        let game_id = {
            let mut cache = gc.lock().unwrap();
            let id = cache.create_game_state();
            let gs = cache.get_game_state_by_id_mut(id).unwrap();
            gs.start_game(empty_team("home"), empty_team("away"), Rules::Bb2025, 1);
            id
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(
            1,
            game_id,
            "Coach".into(),
            ffb_model::model::ClientMode::PLAYER,
            true,
            vec![],
            tx,
        );

        ServerGameTimeTask::tick(&gc, &sm, 1_000);

        let msg = rx.try_recv().expect("expected a serverGameTime broadcast");
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["netCommandId"], "serverGameTime");
    }

    #[test]
    fn tick_with_unstarted_game_does_not_panic() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        // create_game_state() alone leaves `driver` as None (game not started).
        gc.lock().unwrap().create_game_state();
        // Should not panic even though `get_game()` returns None for this slot.
        ServerGameTimeTask::tick(&gc, &sm, 1_000);
    }
}
