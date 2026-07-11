/// 1:1 translation of com.fumbbl.ffb.server.net.ServerDbKeepAliveTask.
///
/// Java: a `TimerTask` scheduled at a fixed period (`FantasyFootballServer.start()`,
/// `TIMER_DB_KEEP_ALIVE` property) that pings the DB to keep the connection alive,
/// and calls `System.exit(99)` on any exception. Rust: `run()` is async (the DB
/// call is async via `mysql_async`), meant to be driven from a `tokio::time::interval`
/// loop (wired in `fantasy_football_server.rs::run()`).
use std::sync::{Arc, Mutex};
use crate::db::db_connection_manager::DbConnectionManager;

pub struct ServerDbKeepAliveTask {
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
}

impl ServerDbKeepAliveTask {
    /// Java: `ServerDbKeepAliveTask(FantasyFootballServer server, DbConnectionManager dbConnectionManager)`.
    /// `fServer` is dropped here — Rust's error path (`log::error!` instead of
    /// `getServer().getDebugLog()`) doesn't need the server handle.
    pub fn new(db_connection_manager: Arc<Mutex<DbConnectionManager>>) -> Self {
        Self { db_connection_manager }
    }

    pub fn get_db_connection_manager(&self) -> Arc<Mutex<DbConnectionManager>> {
        Arc::clone(&self.db_connection_manager)
    }

    /// Java:
    /// ```java
    /// public void run() {
    ///     try {
    ///         getDbConnectionManager().doKeepAlivePing();
    ///     } catch (Exception anyException) {
    ///         getServer().getDebugLog().logWithOutGameId(anyException);
    ///         System.exit(99);
    ///     }
    /// }
    /// ```
    pub async fn run(&self) {
        // Clone the manager out from behind the std::sync::Mutex before awaiting
        // (see DbConnectionManager's Clone doc comment) — mirrors Java calling
        // getDbConnectionManager() once and invoking doKeepAlivePing() on it.
        let manager = { self.db_connection_manager.lock().unwrap().clone() };
        if !manager.pool_ready() {
            // Java only schedules this task when TIMER_DB_KEEP_ALIVE is configured
            // and a DB is expected to be present, so doKeepAlivePing() would always
            // have a live connection to ping. The Rust server has no properties-file
            // gate, so this task may run in a DB-less dev/test configuration; treat
            // an unconfigured pool as "nothing to ping" rather than a fatal error.
            log::warn!("ServerDbKeepAliveTask: DB pool not initialized, skipping keep-alive ping");
            return;
        }
        if let Err(any_exception) = manager.do_keep_alive_ping().await {
            log::error!("ServerDbKeepAliveTask: keep-alive ping failed: {any_exception}");
            std::process::exit(99);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager_without_pool() -> Arc<Mutex<DbConnectionManager>> {
        Arc::new(Mutex::new(DbConnectionManager::new()))
    }

    #[test]
    fn construct() {
        let _ = ServerDbKeepAliveTask::new(manager_without_pool());
    }

    #[test]
    fn get_db_connection_manager_shares_same_arc() {
        let mgr = manager_without_pool();
        let task = ServerDbKeepAliveTask::new(Arc::clone(&mgr));
        assert!(Arc::ptr_eq(&mgr, &task.get_db_connection_manager()));
    }

    #[tokio::test]
    async fn run_without_pool_does_not_panic_or_exit() {
        // A DB-less configuration (no FFB_DB_URL) must not call doKeepAlivePing()
        // (which would panic on `pool.as_ref().expect(...)`) and must not exit
        // the test process.
        let task = ServerDbKeepAliveTask::new(manager_without_pool());
        task.run().await;
    }
}
