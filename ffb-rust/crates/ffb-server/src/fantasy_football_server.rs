/// 1:1 translation of com.fumbbl.ffb.server.FantasyFootballServer.
///
/// Top-level orchestrator: owns `GameCache`, `SessionManager`, and
/// `ServerCommunication`; binds the axum HTTP/WebSocket listener.
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use axum::Router;
use ffb_engine::util::rng::fortuna::Fortuna;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::net::command_servlet::CommandServlet;
use crate::net::command_socket::AppState;
use crate::net::file_servlet::FileServlet;
use crate::net::server_communication::ServerCommunication;
use crate::net::server_db_keep_alive_task::ServerDbKeepAliveTask;
use crate::net::server_game_time_task::ServerGameTimeTask;
use crate::net::server_network_entropy_task::ServerNetworkEntropyTask;
use crate::net::session_manager::SessionManager;
use crate::net::session_timeout_task::SessionTimeoutTask;

/// Java: `FantasyFootballServer`
pub struct FantasyFootballServer {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    /// Java: `dbConnectionManager` local in `start()`, promoted here so handlers
    /// constructed by `run()` can share the same pool.
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    /// Java: `fFortuna`, created in `start()` and exposed via `getFortuna()`.
    fortuna: Arc<Mutex<Fortuna>>,
}

impl FantasyFootballServer {
    /// Java: `new FantasyFootballServer()`
    pub fn new() -> Self {
        Self {
            game_cache: Arc::new(Mutex::new(GameCache::new())),
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
            db_connection_manager: Arc::new(Mutex::new(Self::build_db_connection_manager())),
            fortuna: Arc::new(Mutex::new(Fortuna::new())),
        }
    }

    /// Java:
    /// ```java
    /// DbConnectionManager dbConnectionManager = new DbConnectionManager(this);
    /// dbConnectionManager.setDbUrl(getProperty(IServerProperty.DB_URL));
    /// dbConnectionManager.setDbUser(getProperty(IServerProperty.DB_USER));
    /// dbConnectionManager.setDbPassword(getProperty(IServerProperty.DB_PASSWORD));
    /// dbConnectionManager.setDbType(getProperty(IServerProperty.DB_TYPE));
    /// ```
    /// Java reads these from a server properties file (`IServerProperty.DB_URL` etc); the
    /// Rust server has no properties-file layer yet, so the equivalent config is read from
    /// environment variables (`FFB_DB_URL`/`FFB_DB_USER`/`FFB_DB_PASSWORD`/`FFB_DB_TYPE`).
    /// When `FFB_DB_URL` is unset (e.g. in tests or a DB-less dev run), the pool is left
    /// uninitialized — mirroring how a misconfigured Java deployment would fail DB calls
    /// lazily rather than at startup.
    fn build_db_connection_manager() -> DbConnectionManager {
        let mut manager = DbConnectionManager::new();
        if let Ok(url) = std::env::var("FFB_DB_URL") {
            manager.set_db_url(url);
            manager.set_db_user(std::env::var("FFB_DB_USER").unwrap_or_default());
            manager.set_db_password(std::env::var("FFB_DB_PASSWORD").unwrap_or_default());
            manager.set_db_type(std::env::var("FFB_DB_TYPE").unwrap_or_else(|_| "mysql".to_string()));
            if let Err(e) = manager.init_pool() {
                log::error!("failed to initialize DB pool: {e}");
            }
        } else {
            log::warn!("FFB_DB_URL not set — DB layer disabled for this run");
        }
        manager
    }

    /// Java: `getDbConnectionManager()` — shared handle used by handlers that need DB access.
    pub fn db_connection_manager(&self) -> Arc<Mutex<DbConnectionManager>> {
        Arc::clone(&self.db_connection_manager)
    }

    /// Java: `getFortuna()`.
    pub fn fortuna(&self) -> Arc<Mutex<Fortuna>> {
        Arc::clone(&self.fortuna)
    }

    /// Java: `start()` — binds the HTTP/WebSocket listener, wires the servlets,
    /// schedules the `TimerTask`s, and runs the event loop.
    ///
    /// The Rust server has no properties-file layer (see `build_db_connection_manager`'s
    /// doc comment), so the servlet-mapping/timer-scheduling gates Java reads from
    /// `IServerProperty` are read from environment variables instead:
    ///   - `FFB_SERVER_COMMAND_COMPRESSION` → `IServerProperty.SERVER_COMMAND_COMPRESSION`
    ///   - `FFB_HTTP_DIR`                   → `IServerProperty.SERVER_BASE_DIR` (`httpDir`)
    ///   - `FFB_TIMER_DB_KEEP_ALIVE_MS`     → `IServerProperty.TIMER_DB_KEEP_ALIVE`
    ///   - `FFB_TIMER_NETWORK_ENTROPY_MS`   → `IServerProperty.TIMER_NETWORK_ENTROPY`
    ///   - `FFB_TIMER_SESSION_TIMEOUT_ENABLED` → `IServerProperty.TIMER_SESSION_TIMEOUT_ENABLED`
    ///   - `FFB_TIMER_SESSION_TIMEOUT_SCHEDULE_MS` → `IServerProperty.TIMER_SESSION_TIMEOUT_SCHEDULE`
    ///   - `FFB_SESSION_TIMEOUT_VALUE_MS`   → `IServerProperty.SESSION_TIMEOUT_VALUE`
    pub async fn run(self, addr: SocketAddr) {
        let server_comms = Arc::new(ServerCommunication::new(
            Arc::clone(&self.game_cache),
            Arc::clone(&self.session_manager),
            Arc::clone(&self.db_connection_manager),
        ));

        let state = AppState {
            game_cache: Arc::clone(&self.game_cache),
            session_manager: Arc::clone(&self.session_manager),
            dispatch_tx: server_comms.sender(),
        };

        // Java: `context.addServlet(new ServletHolder(new CommandServlet(this)), "/command/*")`.
        let command_compression = std::env::var("FFB_SERVER_COMMAND_COMPRESSION")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let command_servlet = CommandServlet::new(command_compression);
        let mut app: Router = command_servlet.configure(Router::new()).with_state(state);

        // Java:
        // ```java
        // ServletHolder fileServletHolder = new ServletHolder(new FileServlet(this));
        // fileServletHolder.setInitParameter("resourceBase", httpDir.getAbsolutePath());
        // context.addServlet(fileServletHolder, "/*");
        // ```
        // gated (in Java) on both `SERVER_PORT` and `SERVER_BASE_DIR` being provided;
        // `SERVER_PORT` is already required to reach `run()` at all here, so only
        // `FFB_HTTP_DIR` is checked.
        if let Ok(http_dir) = std::env::var("FFB_HTTP_DIR") {
            let root_dir = std::path::PathBuf::from(http_dir);
            let file_router: Router<std::path::PathBuf> = Router::new()
                .route("/*path", axum::routing::get(file_get_handler));
            app = app.merge(file_router.with_state(root_dir));
        }

        // Java: `if (dbKeepAlivePeriod > 0) { ...schedule(new ServerDbKeepAliveTask(...), 0, dbKeepAlivePeriod); }`.
        if let Some(period_ms) = env_u64("FFB_TIMER_DB_KEEP_ALIVE_MS") {
            let task = ServerDbKeepAliveTask::new(Arc::clone(&self.db_connection_manager));
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(period_ms.max(1)));
                loop {
                    interval.tick().await;
                    task.run().await;
                }
            });
        }

        // Java: `if (networkEntropyPeriod > 0) { ...schedule(new ServerNetworkEntropyTask(this), 0, networkEntropyPeriod); }`.
        if let Some(period_ms) = env_u64("FFB_TIMER_NETWORK_ENTROPY_MS") {
            let task = ServerNetworkEntropyTask::new();
            let fortuna = Arc::clone(&self.fortuna);
            spawn_interval(period_ms, move || task.run(&fortuna));
        }

        // Java: `fServerGameTimeTimer.scheduleAtFixedRate(new ServerGameTimeTask(this), 0, 1000);`
        // — always scheduled, not gated by a property.
        {
            let task = ServerGameTimeTask::new(Arc::clone(&self.game_cache), Arc::clone(&self.session_manager));
            spawn_interval(1000, move || task.run());
        }

        // Java:
        // ```java
        // if (Boolean.parseBoolean(timerEnabledProperty)) {
        //     sessionTimeoutTimer.scheduleAtFixedRate(
        //         new SessionTimeoutTask(fSessionManager, replaySessionManager, fCommunication, sessionTimeout),
        //         0, timerSchedule);
        // }
        // ```
        let session_timeout_enabled = std::env::var("FFB_TIMER_SESSION_TIMEOUT_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if session_timeout_enabled {
            if let (Some(schedule_ms), Some(timeout_ms)) = (
                env_u64("FFB_TIMER_SESSION_TIMEOUT_SCHEDULE_MS"),
                env_u64("FFB_SESSION_TIMEOUT_VALUE_MS"),
            ) {
                let task = SessionTimeoutTask::new(
                    Arc::clone(&self.session_manager),
                    server_comms.replay_session_manager(),
                    Arc::clone(&server_comms),
                    timeout_ms as i64,
                );
                spawn_interval(schedule_ms, move || task.run());
            } else {
                log::warn!(
                    "FFB_TIMER_SESSION_TIMEOUT_ENABLED set but FFB_TIMER_SESSION_TIMEOUT_SCHEDULE_MS / \
                     FFB_SESSION_TIMEOUT_VALUE_MS missing — session timeout task not scheduled"
                );
            }
        }

        log::info!("FFB server listening on ws://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await
            .unwrap_or_else(|e| panic!("failed to bind {}: {}", addr, e));
        axum::serve(listener, app).await
            .unwrap_or_else(|e| panic!("server error: {}", e));
    }
}

/// Java: `Timer.schedule(task, 0, period)` / `scheduleAtFixedRate(task, 0, period)`
/// for a synchronous `TimerTask::run`. Not a Java method itself — the
/// `tokio::time::interval` loop that stands in for Jetty's `java.util.Timer`.
fn spawn_interval<F>(period_ms: u64, mut tick: F)
where
    F: FnMut() + Send + 'static,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(period_ms.max(1)));
        loop {
            interval.tick().await;
            tick();
        }
    });
}

/// Parses a millisecond period from an environment variable, treating a
/// missing var, non-numeric value, or `0` the same as Java's `period > 0` gate
/// (i.e. "timer not scheduled").
fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok()).filter(|&v| v > 0)
}

/// Java: `FileServlet.doGet` reached via the `/*` servlet mapping.
async fn file_get_handler(
    axum::extract::State(root_dir): axum::extract::State<std::path::PathBuf>,
    uri: axum::http::Uri,
) -> axum::response::Response {
    FileServlet::new().do_get(&root_dir, uri.path()).await
}

impl Default for FantasyFootballServer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Environment variables are process-global, so these tests run serially via a mutex
    // to avoid interference from cargo's parallel test threads within this file.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn new_without_db_url_leaves_pool_uninitialized() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FFB_DB_URL");
        let server = FantasyFootballServer::new();
        let mgr = server.db_connection_manager();
        assert_eq!(mgr.lock().unwrap().get_db_url(), "");
    }

    #[test]
    fn new_with_db_url_configures_manager() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FFB_DB_URL", "jdbc:mysql://127.0.0.1:3306/ffblive");
        std::env::set_var("FFB_DB_USER", "root");
        std::env::set_var("FFB_DB_PASSWORD", "secret");
        std::env::set_var("FFB_DB_TYPE", "mysql");

        let server = FantasyFootballServer::new();
        let mgr = server.db_connection_manager();
        {
            let mgr = mgr.lock().unwrap();
            assert_eq!(mgr.get_db_url(), "jdbc:mysql://127.0.0.1:3306/ffblive");
            assert_eq!(mgr.get_db_user(), "root");
            assert!(mgr.use_mysql_dialect());
        }

        std::env::remove_var("FFB_DB_URL");
        std::env::remove_var("FFB_DB_USER");
        std::env::remove_var("FFB_DB_PASSWORD");
        std::env::remove_var("FFB_DB_TYPE");
    }

    #[test]
    fn db_connection_manager_shares_same_arc() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FFB_DB_URL");
        let server = FantasyFootballServer::new();
        let a = server.db_connection_manager();
        let b = server.db_connection_manager();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn fortuna_accessor_shares_same_arc() {
        let server = FantasyFootballServer::new();
        let a = server.fortuna();
        let b = server.fortuna();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn env_u64_missing_var_is_none() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FFB_TEST_ENV_U64_MISSING");
        assert_eq!(env_u64("FFB_TEST_ENV_U64_MISSING"), None);
    }

    #[test]
    fn env_u64_zero_is_none() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FFB_TEST_ENV_U64_ZERO", "0");
        assert_eq!(env_u64("FFB_TEST_ENV_U64_ZERO"), None);
        std::env::remove_var("FFB_TEST_ENV_U64_ZERO");
    }

    #[test]
    fn env_u64_positive_value_parses() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FFB_TEST_ENV_U64_POSITIVE", "5000");
        assert_eq!(env_u64("FFB_TEST_ENV_U64_POSITIVE"), Some(5000));
        std::env::remove_var("FFB_TEST_ENV_U64_POSITIVE");
    }

    #[test]
    fn env_u64_non_numeric_is_none() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FFB_TEST_ENV_U64_JUNK", "not-a-number");
        assert_eq!(env_u64("FFB_TEST_ENV_U64_JUNK"), None);
        std::env::remove_var("FFB_TEST_ENV_U64_JUNK");
    }
}
